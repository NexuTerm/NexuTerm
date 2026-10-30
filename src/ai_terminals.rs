//! GTK side of the terminal MCP bridge. Operations follow explicit, remembered or per-chat full-access approval.
use super::*;
use nexuterm::terminal_mcp::{Broker, Request, result};
use serde_json::json;
type Grants = Rc<RefCell<std::collections::HashMap<String, Rc<RefCell<nexuterm::approval_grants::Grants>>>>>;
fn sessions(state: &State) -> Vec<Rc<Session>> {
    state
        .sessions
        .borrow()
        .iter()
        .filter_map(Weak::upgrade)
        .filter(|s| s.pid.get().is_some() && s.terminal.upgrade().is_some())
        .collect()
}
fn visible_text(term: &vte4::Terminal) -> String {
    use glib::translate::{ToGlibPtr, from_glib_full};
    let text: Option<glib::GString> = unsafe {
        from_glib_full(vte4::ffi::vte_terminal_get_text(
            term.to_glib_none().0,
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ))
    };
    text.unwrap_or_default()
        .chars()
        .rev()
        .take(16384)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
pub(crate) fn install(state: &Rc<State>) -> Option<Rc<Broker>> {
    let (broker, mut requests) = match Broker::start() {
        Ok(value) => value,
        Err(error) => {
            state.status.set_text(&error);
            return None;
        }
    };
    let access = broker.full_access.clone();
    let s = Rc::downgrade(state);
    let grants: Grants = Rc::new(RefCell::new(std::collections::HashMap::new()));
    glib::timeout_add_local(std::time::Duration::from_millis(40), move || {
        let Some(state) = s.upgrade() else {
            return glib::ControlFlow::Break;
        };
        for _ in 0..8 {
            let request = match requests.try_recv() {
                Ok(request) => request,
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => return glib::ControlFlow::Break,
            };
            handle(&state, request, &grants, access.load(std::sync::atomic::Ordering::Relaxed));
        }
        glib::ControlFlow::Continue
    });
    let broker = Rc::new(broker);
    #[cfg(debug_assertions)]
    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("terminals")
        && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
    {
        smoke(state, &broker);
    }
    Some(broker)
}
fn handle(state: &Rc<State>, request: Request, policies: &Grants, full_access: bool) {
    if matches!(request.name.as_str(),"supervised_targets"|"supervised_execute"){crate::supervised_ui::handle_mcp_with_access(state,request,full_access);return;}
    let targets = sessions(state);
    let active = state.focused_terminal.borrow().upgrade();
    if request.name == "terminal_list" {
        let list=targets.iter().map(|s|json!({"terminal_id":s.id,"name":s.title,"host":s.connection.as_ref().map(|c|c.profile.host.as_str()),"remote":s.connection.is_some()||std::path::Path::new(&s.spec.program).file_name().is_some_and(|p|p=="ssh"||p=="telnet"),"active":s.terminal.upgrade().as_ref()==active.as_ref()})).collect::<Vec<_>>();
        let _ = request
            .response
            .send(result(json!({"terminals":list}).to_string(), false));
        return;
    }
    if !matches!(request.name.as_str(), "terminal_read" | "terminal_execute") {
        let _ = request
            .response
            .send(result("Herramienta no admitida".into(), true));
        return;
    }
    let id = request.arguments["terminal_id"]
        .as_str()
        .unwrap_or("active");
    let target = targets.into_iter().find(|s| {
        if id == "active" {
            s.terminal.upgrade().as_ref() == active.as_ref()
        } else {
            s.id == id
        }
    });
    let Some(target) = target else {
        let _ = request.response.send(result(
            "La terminal solicitada no está abierta o no tiene un proceso activo".into(),
            true,
        ));
        return;
    };
    let command = request.arguments["command"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    let execute = request.name == "terminal_execute";
    if execute
        && (command.trim().is_empty()
            || command.len() > 8192
            || command
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\t'))
    {
        let _ = request
            .response
            .send(result("Comando inválido o demasiado grande".into(), true));
        return;
    }
    let Some(parent) = state.window.upgrade() else {
        return;
    };
    let destination = target
        .connection
        .as_ref()
        .map(|c| {
            format!(
                "{}@{}:{}",
                c.profile.username, c.profile.host, c.profile.port
            )
        })
        .unwrap_or_else(|| target.title.clone());
    let detail = if execute {
        format!(
            "Destino: {}\nTerminal: {}\n\n{}\n\nSe enviará a la sesión existente, que puede estar ejecutando otro programa. La confirmación solo indica el envío; no garantiza que termine correctamente.",
            destination, target.title, command
        )
    } else {
        format!(
            "Destino: {destination}\nTerminal: {}\n\nSe compartirá el texto visible de esta terminal. Puede contener datos sensibles.",
            target.title
        )
    };
    let policy = {
        let config = state.config.borrow();
        let mut agents = config.ai_agents.iter().filter(|agent| agent.name == request.agent);
        let agent = agents.next().filter(|_| agents.next().is_none());
        agent.map(|agent| policies.borrow_mut().entry(agent.id.clone()).or_insert_with(||
            Rc::new(RefCell::new(nexuterm::approval_grants::Grants::new(nexuterm::agent_profile::root(agent))))).clone())
    };
    let operation = json!({"tool":request.name,"command":command});
    let terminal = target.terminal.upgrade();
    let cwd = terminal.as_ref().and_then(|term|term.current_directory_uri()).filter(|uri| {
        target.connection.as_ref().is_none_or(|connection| url::Url::parse(uri).ok().and_then(|uri|uri.host_str().map(String::from)).is_some_and(|host|host == connection.profile.host))
    }).map(|uri|uri.to_string())
        .or_else(|| if target.connection.is_none() { target.pid.get().and_then(|pid|std::fs::read_link(format!("/proc/{pid}/cwd")).ok()).map(|path|path.to_string_lossy().into_owned()) } else { None });
    let session_key = nexuterm::approval_grants::fingerprint(&json!({"operation":operation,"terminal":target.id,"cwd":cwd}));
    let destination_key = target.connection.as_ref().map(|connection|json!({"id":connection.profile.id,"host":connection.profile.host,"port":connection.profile.port,"username":connection.profile.username,"jump":connection.profile.jump_host})).unwrap_or_else(||json!({"local":true,"program":target.spec.program,"args":target.spec.args}));
    // Do not remember command execution across restarts without a known directory.
    let persistent_key = (!execute || cwd.is_some()).then(|| nexuterm::approval_grants::fingerprint(&json!({"operation":operation,"destination":destination_key,"cwd":cwd})));
    let remembered = full_access || policy.as_ref().is_some_and(|policy| {
        let policy = policy.borrow();
        policy.allows(&session_key) || persistent_key.as_ref().is_some_and(|key|policy.allows(key))
    });
    let mut buttons = vec!["Rechazar", "Permitir una vez"];
    if policy.is_some() {
        buttons.push("Permitir en esta sesión");
        if persistent_key.is_some() { buttons.push("Permitir siempre"); }
    }
    let dialog = gtk::AlertDialog::builder()
        .message(format!(
            "{} solicita {}",
            request.agent,
            if execute {
                "ejecutar un comando"
            } else {
                "leer una terminal"
            }
        ))
        .detail(format!("{detail}\n\nLos permisos recordados se limitan a esta operación y destino. Ejecutar siempre requiere conocer la carpeta de trabajo."))
        .buttons(buttons)
        .cancel_button(0)
        .default_button(0)
        .build();
    #[cfg(debug_assertions)]
    if request.agent == "NexuTerm smoke"
        && std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("terminals")
        && target.title == "Remoto de prueba" && !remembered
    {
        let selected_label = if execute {"Permitir una vez"} else {"Permitir en esta sesión"};
        glib::timeout_add_local_once(std::time::Duration::from_millis(120), move || {
            fn click(widget: &gtk::Widget, selected_label: &str) -> bool {
                if let Some(button) = widget.downcast_ref::<gtk::Button>()
                    && button.label().is_some_and(|s| s == selected_label) {
                        button.emit_clicked();
                        return true;
                    }
                let mut child = widget.first_child();
                while let Some(widget) = child {
                    child = widget.next_sibling();
                    if click(&widget, selected_label) {
                        return true;
                    }
                }
                false
            }
            assert!(
                gtk::Window::list_toplevels().iter().any(|window|click(window,selected_label)),
                "Terminal approval dialog was not shown"
            );
        });
    }
    #[cfg(debug_assertions)]
    if remembered && std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("terminals") { eprintln!("NEXUTERM_SMOKE_TERMINAL_GRANT_USED"); }
    let target = Rc::downgrade(&target);
    glib::spawn_future_local(async move {
        let choice = if remembered { Ok(1) } else { dialog.choose_future(Some(&parent)).await };
        if request.response.is_closed() {
            return;
        }
        if !matches!(choice, Ok(1..=3)) {
            let _ = request
                .response
                .send(result("El usuario rechazó la operación".into(), true));
            return;
        }
        if let Some(policy) = &policy {
            let grant = match choice {
                Ok(2) => Some((session_key, false)),
                Ok(3) => persistent_key.map(|key|(key,true)),
                _ => None,
            };
            if let Some((key, always)) = grant
                && let Err(error) = policy.borrow_mut().remember(key, always) {
                    let _ = request.response.send(result(error, true));
                    return;
                }
        }
        let Some(target) = target.upgrade().filter(|t| t.pid.get().is_some()) else {
            let _ = request.response.send(result(
                "La terminal se cerró antes de la aprobación".into(),
                true,
            ));
            return;
        };
        let Some(term) = target.terminal.upgrade() else {
            return;
        };
        let answer = if execute {
            term.feed_child(format!("{command}\n").as_bytes());
            format!(
                "Comando enviado a {} ({}). Lee la terminal para comprobar el resultado; todavía no se conoce su estado de salida.",
                target.title, target.id
            )
        } else {
            visible_text(&term)
        };
        let _ = request.response.send(result(answer, false));
    });
}

#[cfg(debug_assertions)]
fn smoke(state: &Rc<State>, broker: &Rc<Broker>) {
    let s = Rc::downgrade(state);
    let socket = broker.path().to_string_lossy().into_owned();
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let Some(state) = s.upgrade() else {
            return;
        };
        let spec = connection::CommandSpec {
            program: "/bin/sh".into(),
            args: vec![
                "-c".into(),
                r#"while IFS= read -r line; do printf 'RECIBIDO: %s\n' "$line"; done"#.into(),
            ],
        };
        terminal_in(&state, "Local de prueba", spec.clone(), None, None);
        let remote = nexuterm::ssh::SshConnection::new(Profile {
            id: "remote-fixture".into(),
            name: "Remoto de prueba".into(),
            host: "remote.example.invalid".into(),
            port: 22,
            username: "fixture".into(),
            ..Default::default()
        })
        .unwrap();
        terminal_in(&state, "Remoto de prueba", spec, None, Some(remote));
        let context_state = Rc::downgrade(&state);
        let context_attempts = Cell::new(0);
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            context_attempts.set(context_attempts.get()+1);
            assert!(context_attempts.get()<20, "Remote terminal context did not become available");
            let state = context_state.upgrade().unwrap();
            if let Some(workspace) = super::agent_workspace(&state)
                && workspace.context["remote"] == true {
                    assert!(workspace.local_directory.is_none());
                    assert_eq!(workspace.context["host"], "remote.example.invalid");
                    eprintln!("NEXUTERM_SMOKE_REMOTE_WORKSPACE_OK");
                    return glib::ControlFlow::Break;
                }
            glib::ControlFlow::Continue
        });
        let mut worker = nexuterm::runtime::runtime().spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let list =
                nexuterm::terminal_mcp::call(&socket, "NexuTerm smoke", "terminal_list", json!({}))
                    .await
                    .unwrap();
            let list: serde_json::Value =
                serde_json::from_str(list["content"][0]["text"].as_str().unwrap()).unwrap();
            let terminals = list["terminals"].as_array().unwrap();
            assert!(
                terminals
                    .iter()
                    .any(|t| t["name"] == "Local de prueba" && t["remote"] == false)
            );
            let id = terminals
                .iter()
                .find(|t| t["host"] == "remote.example.invalid")
                .unwrap()["terminal_id"]
                .clone();
            let sent = nexuterm::terminal_mcp::call(
                &socket,
                "NexuTerm smoke",
                "terminal_execute",
                json!({"terminal_id":id,"command":"printf NEXUTERM_TERMINAL_TEST"}),
            )
            .await
            .unwrap();
            assert_eq!(sent["isError"], false);
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            let read = nexuterm::terminal_mcp::call(
                &socket,
                "NexuTerm smoke",
                "terminal_read",
                json!({"terminal_id":id}),
            )
            .await
            .unwrap();
            assert_eq!(read["isError"], false);
            assert!(
                read["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains("RECIBIDO: printf NEXUTERM_TERMINAL_TEST")
            );
            let repeated = nexuterm::terminal_mcp::call(&socket, "NexuTerm smoke", "terminal_read", json!({"terminal_id":id})).await.unwrap();
            assert_eq!(repeated["isError"], false);
        });
        glib::spawn_future_local(async move {
            while !worker.is_finished() {
                glib::timeout_future(std::time::Duration::from_millis(50)).await;
            }
            (&mut worker).await.unwrap();
            eprintln!("NEXUTERM_SMOKE_TERMINALS_OK");
        });
    });
}
