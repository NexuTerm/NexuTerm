use super::*;
fn named(w: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if w.widget_name() == name {
        return Some(w.clone());
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(v) = named(&c, name) {
            return Some(v);
        }
        child = c.next_sibling();
    }
    None
}
fn win(name: &str) -> Option<gtk::Window> {
    gtk::Window::list_toplevels()
        .into_iter()
        .find(|w| w.widget_name() == name)
        .and_then(|w| w.downcast().ok())
}
async fn until(mut f: impl FnMut() -> bool) {
    for _ in 0..240 {
        if f() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(25)).await;
    }
    panic!("Supervised UI timed out");
}
fn click(w: &gtk::Window, name: &str) {
    named(w.upcast_ref(), name)
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}
fn command(w: &gtk::Window, s: &str) {
    named(w.upcast_ref(), "supervised-command")
        .unwrap()
        .downcast::<gtk::TextView>()
        .unwrap()
        .buffer()
        .set_text(s);
}
async fn confirm(w: &gtk::Window) {
    click(w, "supervised-run");
    until(|| win("supervised-approval").is_some()).await;
    click(&win("supervised-approval").unwrap(), "supervised-approve");
}
pub(super) fn run(state: &Rc<State>) {
    let state = state.clone();
    glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(200)).await;
        let parent = state.window().unwrap();
        gtk::prelude::WidgetExt::activate_action(&parent, "win.supervised", None).unwrap();
        let w = win("supervised-execution").unwrap();
        let c = controller(&state);
        match std::env::var("NEXUTERM_SMOKE_SUPERVISED").as_deref() {
            Ok("recipes") => {
                recipes(&state, &parent, &w, &c).await;
                return;
            }
            Ok("close") => {
                let folder = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
                named(w.upcast_ref(), "supervised-directory")
                    .unwrap()
                    .downcast::<gtk::Entry>()
                    .unwrap()
                    .set_text(&folder.to_string_lossy());
                command(&w, "printf checkpoint; sleep 20 & wait");
                confirm(&w).await;
                until(|| {
                    c.selected
                        .borrow()
                        .as_ref()
                        .is_some_and(|h| h.snapshot().stdout == b"checkpoint")
                })
                .await;
                eprintln!("NEXUTERM_SUPERVISED_CLOSE_REQUESTED");
                parent.close();
                until(|| {
                    gtk::Window::list_toplevels()
                        .iter()
                        .any(|w| label_button(w, "Cerrar").is_some())
                })
                .await;
                for top in gtk::Window::list_toplevels() {
                    if let Some(button) = label_button(&top, "Cerrar") {
                        button.emit_clicked();
                        break;
                    }
                }
                glib::timeout_future(std::time::Duration::from_secs(10)).await;
                panic!("Normal close did not finish");
            }
            Ok("capture") => {
                let folder = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
                named(w.upcast_ref(), "supervised-directory")
                    .unwrap()
                    .downcast::<gtk::Entry>()
                    .unwrap()
                    .set_text(&folder.to_string_lossy());
                command(&w, "printf checkpoint; sleep 3");
                confirm(&w).await;
                until(|| {
                    c.selected
                        .borrow()
                        .as_ref()
                        .is_some_and(|h| h.snapshot().status == Status::Running)
                })
                .await;
                let handle = c.selected.borrow().as_ref().unwrap().clone();
                let id = handle.id();
                for _ in 0..100 {
                    glib::timeout_future(std::time::Duration::from_millis(30)).await;
                    let id = id.clone();
                    let record = nexuterm::runtime::runtime()
                        .spawn_blocking(move || {
                            nexuterm::execution_history::read(
                                &nexuterm::execution_history::directory().unwrap(),
                                &id,
                            )
                        })
                        .await
                        .unwrap();
                    if record.is_ok_and(|r| r.stdout == b"checkpoint") {
                        eprintln!("NEXUTERM_EXECUTION_PID={}", std::process::id());
                        glib::timeout_future(std::time::Duration::from_secs(10)).await;
                        panic!("Fixture did not kill the application");
                    }
                }
                panic!("No durable running checkpoint");
            }
            Ok("recover") => {
                let entries = nexuterm::runtime::runtime()
                    .spawn_blocking(|| {
                        nexuterm::execution_history::list(
                            &nexuterm::execution_history::directory().unwrap(),
                        )
                    })
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(entries.len(), 1);
                let id = entries[0].id.clone();
                let record = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        nexuterm::execution_history::read(
                            &nexuterm::execution_history::directory().unwrap(),
                            &id,
                        )
                    })
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(record.status, Status::Unknown);
                assert_eq!(record.exit_code, None);
                assert_eq!(record.stdout, b"checkpoint");
                assert!(c.jobs.borrow().is_empty());
                click(&w, "supervised-history");
                until(|| win("supervised-history-window").is_some()).await;
                assert!(c.jobs.borrow().is_empty());
                eprintln!("NEXUTERM_SUPERVISED_RECOVERY_OK");
                parent.application().unwrap().quit();
                return;
            }
            _ => (),
        }

        let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
        let dir = root.join("space ' folder");
        std::fs::create_dir(&dir).unwrap();
        let marker = dir.join("executed");
        named(w.upcast_ref(), "supervised-directory")
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_text(&dir.to_string_lossy());
        command(
            &w,
            "printf 'salida á'; printf 'error ñ' >&2; touch executed; exit 7",
        );
        click(&w, "supervised-run");
        until(|| win("supervised-approval").is_some()).await;
        assert!(!marker.exists());
        click(&win("supervised-approval").unwrap(), "supervised-reject");
        assert!(!marker.exists());
        assert!(c.jobs.borrow().is_empty());
        confirm(&w).await;
        until(|| c.selected.borrow().as_ref().is_some_and(|h| h.finished())).await;
        let first = c.selected.borrow().as_ref().unwrap().clone();
        let r = first.snapshot();
        assert_eq!(r.stdout, "salida á".as_bytes());
        assert_eq!(r.stderr, "error ñ".as_bytes());
        assert_eq!(r.exit_code, Some(7));
        assert!(marker.exists());
        let id = r.id.clone();
        let stored = nexuterm::runtime::runtime()
            .spawn_blocking(move || {
                nexuterm::execution_history::read(
                    &nexuterm::execution_history::directory().unwrap(),
                    &id,
                )
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.stdout, r.stdout);
        assert_eq!(stored.status, Status::Completed);
        let keep = find_check(w.upcast_ref()).unwrap();
        keep.set_active(false);
        command(&w, "sleep 20 & wait");
        named(w.upcast_ref(), "supervised-timeout")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap()
            .set_value(1.0);
        confirm(&w).await;
        until(|| c.selected.borrow().as_ref().is_some_and(|h| h.finished())).await;
        assert_eq!(
            c.selected.borrow().as_ref().unwrap().snapshot().status,
            Status::TimedOut
        );
        named(w.upcast_ref(), "supervised-timeout")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap()
            .set_value(60.0);
        confirm(&w).await;
        until(|| {
            c.selected
                .borrow()
                .as_ref()
                .is_some_and(|h| h.snapshot().status == Status::Running)
        })
        .await;
        click(&w, "supervised-cancel");
        until(|| c.selected.borrow().as_ref().is_some_and(|h| h.finished())).await;
        assert_eq!(
            c.selected.borrow().as_ref().unwrap().snapshot().status,
            Status::Cancelled
        );
        let before = c.jobs.borrow().len();
        let (tx, rx) = tokio::sync::oneshot::channel();
        drop(rx);
        handle_mcp(
            &state,
            nexuterm::terminal_mcp::Request {
                agent: "Fixture".into(),
                name: "supervised_execute".into(),
                arguments: serde_json::json!({"target_id":"local","directory":dir,"command":"touch expired","keep_history":false}),
                response: tx,
            },
        );
        click(&win("supervised-approval").unwrap(), "supervised-approve");
        assert_eq!(c.jobs.borrow().len(), before);
        assert!(!dir.join("expired").exists());
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        handle_mcp(
            &state,
            nexuterm::terminal_mcp::Request {
                agent: "Fixture".into(),
                name: "supervised_execute".into(),
                arguments: serde_json::json!({"target_id":"local","directory":dir,"command":"printf MCP","keep_history":false}),
                response: tx,
            },
        );
        click(&win("supervised-approval").unwrap(), "supervised-approve");
        let mut reply = None;
        until(|| {
            if let Ok(value) = rx.try_recv() {
                reply = Some(value);
            }
            reply.is_some()
        })
        .await;
        let reply = reply.unwrap();
        assert_eq!(reply["isError"], false);
        let result: serde_json::Value =
            serde_json::from_str(reply["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(result["exit_code"], 0);
        assert_eq!(result["stdout"], "MCP");
        let (tx, rx) = tokio::sync::oneshot::channel();
        handle_mcp(
            &state,
            nexuterm::terminal_mcp::Request {
                agent: "Fixture".into(),
                name: "supervised_execute".into(),
                arguments: serde_json::json!({"target_id":"local","directory":dir,"command":"sleep 20 & wait","keep_history":false}),
                response: tx,
            },
        );
        click(&win("supervised-approval").unwrap(), "supervised-approve");
        until(|| {
            c.selected
                .borrow()
                .as_ref()
                .is_some_and(|h| h.snapshot().status == Status::Running)
        })
        .await;
        drop(rx);
        until(|| c.selected.borrow().as_ref().is_some_and(|h| h.finished())).await;
        assert_eq!(
            c.selected.borrow().as_ref().unwrap().snapshot().status,
            Status::Cancelled
        );
        let broker = state.execution_broker().unwrap();
        let socket = broker.path().to_string_lossy().into_owned();
        let directory = dir.clone();
        let call=nexuterm::runtime::runtime().spawn(async move{nexuterm::terminal_mcp::call(&socket,"Fixture","supervised_execute",serde_json::json!({"target_id":"local","directory":directory,"command":"printf WIRE","keep_history":false})).await});
        until(|| win("supervised-approval").is_some()).await;
        click(&win("supervised-approval").unwrap(), "supervised-approve");
        until(|| call.is_finished()).await;
        let reply = call.await.unwrap().unwrap();
        assert_eq!(reply["isError"], false);
        let body: serde_json::Value =
            serde_json::from_str(reply["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(body["stdout"], "WIRE");
        assert_eq!(body["exit_code"], 0);
        // Each chat owns a private broker: granting one cannot authorize another.
        let authorized = crate::ui::ai_terminals::install(&state).unwrap();
        assert_ne!(authorized.path(), broker.path());
        authorized.full_access.store(true, std::sync::atomic::Ordering::Relaxed);
        let socket = authorized.path().to_string_lossy().into_owned();
        let directory = dir.clone();
        let call = nexuterm::runtime::runtime().spawn(async move {
            nexuterm::terminal_mcp::call(&socket, "Fixture", "supervised_execute",
                serde_json::json!({"target_id":"local","directory":directory,"command":"printf FULL","keep_history":false})).await
        });
        until(|| call.is_finished()).await;
        assert!(win("supervised-approval").is_none());
        let reply = call.await.unwrap().unwrap();
        assert_eq!(reply["isError"], false);
        let body: serde_json::Value = serde_json::from_str(reply["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(body["stdout"], "FULL");
        assert!(!broker.full_access.load(std::sync::atomic::Ordering::Relaxed));
        // Revocation restores the explicit approval dialog on that same socket.
        authorized.full_access.store(false, std::sync::atomic::Ordering::Relaxed);
        let socket = authorized.path().to_string_lossy().into_owned();
        let directory = dir.clone();
        let call = nexuterm::runtime::runtime().spawn(async move {
            nexuterm::terminal_mcp::call(&socket, "Fixture", "supervised_execute",
                serde_json::json!({"target_id":"local","directory":directory,"command":"touch revoked-never","keep_history":false})).await
        });
        until(|| win("supervised-approval").is_some()).await;
        click(&win("supervised-approval").unwrap(), "supervised-reject");
        until(|| call.is_finished()).await;
        assert_eq!(call.await.unwrap().unwrap()["isError"], true);
        assert!(!dir.join("revoked-never").exists());
        eprintln!("NEXUTERM_SMOKE_BROKER_FULL_ACCESS_OK");
        *c.selected.borrow_mut() = Some(first);
        click(&w, "supervised-history");
        until(|| win("supervised-history-window").is_some()).await;
        let history = win("supervised-history-window").unwrap();
        glib::timeout_future(std::time::Duration::from_millis(250)).await;
        history.close();
        glib::timeout_future(std::time::Duration::from_millis(200)).await;
        if let Some(path) = std::env::var_os("NEXUTERM_SUPERVISED_SCREENSHOT") {
            let p = gtk::WidgetPaintable::new(Some(&w));
            let s = gtk::Snapshot::new();
            p.snapshot(&s, w.width() as f64, w.height() as f64);
            if let (Some(node), Some(renderer)) = (s.to_node(), w.renderer()) {
                renderer
                    .render_texture(&node, None)
                    .save_to_png(path)
                    .unwrap();
            }
        }
        let quota = nexuterm::execution_history::directory()
            .unwrap()
            .join(format!("{}.json", uuid::Uuid::new_v4()));
        let file = std::fs::File::create(&quota).unwrap();
        file.set_len(nexuterm::execution_history::LIMIT).unwrap();
        drop(file);
        keep.set_active(true);
        command(&w, "touch quota-never");
        confirm(&w).await;
        until(|| c.selected.borrow().as_ref().is_some_and(|h| h.finished())).await;
        let failed = c.selected.borrow().as_ref().unwrap().clone();
        assert_eq!(failed.snapshot().status, Status::Failed);
        assert!(!dir.join("quota-never").exists());
        assert!(!failed.snapshot().recording_error.is_empty());
        assert!(finish_before_close(&state).await.is_err());
        std::fs::remove_file(quota).unwrap();
        glib::timeout_future(std::time::Duration::from_millis(160)).await;
        click(&w, "supervised-forget");
        until(|| {
            gtk::Window::list_toplevels()
                .iter()
                .any(|w| label_button(w, "Retirar").is_some())
        })
        .await;
        for top in gtk::Window::list_toplevels() {
            if let Some(button) = label_button(&top, "Retirar") {
                button.emit_clicked();
                break;
            }
        }
        until(|| !c.jobs.borrow().iter().any(|h| h.id() == failed.id())).await;
        assert!(finish_before_close(&state).await.is_ok());
        eprintln!("NEXUTERM_SMOKE_SUPERVISED_OK");
        parent.application().unwrap().quit();
    });
}
fn find_check(w: &gtk::Widget) -> Option<gtk::CheckButton> {
    if let Ok(c) = w.clone().downcast::<gtk::CheckButton>() {
        return Some(c);
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(v) = find_check(&c) {
            return Some(v);
        }
        child = c.next_sibling();
    }
    None
}

fn label_button(w: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    if let Ok(button) = w.clone().downcast::<gtk::Button>() {
        if button.label().as_deref() == Some(label) {
            return Some(button);
        }
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(button) = label_button(&c, label) {
            return Some(button);
        }
        child = c.next_sibling();
    }
    None
}

async fn recipes(state: &Rc<State>, parent: &gtk::ApplicationWindow, w: &gtk::Window, c: &Rc<Controller>) {
    let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    let dir = root.join("space ' $(touch injected)");
    std::fs::create_dir(&dir).unwrap();
    let folder = named(w.upcast_ref(), "supervised-directory").unwrap().downcast::<gtk::Entry>().unwrap();
    folder.set_text(&dir.to_string_lossy());
    let selector = named(w.upcast_ref(), "recipe-choice").unwrap().downcast::<gtk::DropDown>().unwrap();
    let parameter = named(w.upcast_ref(), "recipe-parameter").unwrap().downcast::<gtk::Entry>().unwrap();
    let preview = named(w.upcast_ref(), "supervised-command").unwrap().downcast::<gtk::TextView>().unwrap();
    command(w, "printf manual-draft");
    selector.set_selected(1);
    assert!(!preview.is_editable());
    parameter.set_text(&dir.to_string_lossy());
    named(w.upcast_ref(), "recipe-threshold").unwrap().downcast::<gtk::SpinButton>().unwrap().set_value(100.0);
    click(w, "supervised-run");
    until(|| win("supervised-approval").is_some()).await;
    assert!(c.jobs.borrow().is_empty());
    click(&win("supervised-approval").unwrap(), "supervised-reject");
    assert!(c.jobs.borrow().is_empty());
    confirm(w).await;
    until(|| c.selected.borrow().as_ref().is_some_and(|h|h.finished())).await;
    let record = c.selected.borrow().as_ref().unwrap().snapshot();
    assert!(matches!(record.exit_code, Some(0 | 2)));
    assert!(String::from_utf8_lossy(&record.stdout).contains("VERIFICACION:"));
    assert!(!dir.join("injected").exists());
    selector.set_selected(2);
    parameter.set_text("bad;touch injected.service");
    assert_eq!(preview.buffer().char_count(), 0);
    click(w, "supervised-run");
    assert!(win("supervised-approval").is_none());
    assert_eq!(c.jobs.borrow().len(), 1);
    parameter.set_text("demo.service");
    confirm(w).await;
    until(|| c.selected.borrow().as_ref().is_some_and(|h|h.finished())).await;
    let record = c.selected.borrow().as_ref().unwrap().snapshot();
    assert_eq!(record.exit_code, Some(0));
    assert!(String::from_utf8_lossy(&record.stdout).contains("servicio activo"));
    std::fs::write(root.join("service-inactive"), "").unwrap();
    confirm(w).await;
    until(|| c.selected.borrow().as_ref().is_some_and(|h|h.finished())).await;
    assert_eq!(c.selected.borrow().as_ref().unwrap().snapshot().exit_code, Some(3));
    selector.set_selected(3);
    assert_eq!(parameter.text(), "demo.service");
    named(w.upcast_ref(), "recipe-lines").unwrap().downcast::<gtk::SpinButton>().unwrap().set_value(12.0);
    named(w.upcast_ref(), "recipe-minutes").unwrap().downcast::<gtk::SpinButton>().unwrap().set_value(5.0);
    confirm(w).await;
    until(|| c.selected.borrow().as_ref().is_some_and(|h|h.finished())).await;
    let record = c.selected.borrow().as_ref().unwrap().snapshot();
    assert_eq!(record.exit_code, Some(0));
    assert!(String::from_utf8_lossy(&record.stdout).contains("fixture journal"));
    assert!(record.spec.command.contains("--lines=12 --since='-5 minutes'"));
    assert_eq!(nexuterm::execution_history::list(&nexuterm::execution_history::directory().unwrap()).unwrap().len(), 4);
    // Capture the populated recipe before returning to the preserved manual draft.
    glib::timeout_future(std::time::Duration::from_millis(200)).await;
    if let Some(path) = std::env::var_os("NEXUTERM_SUPERVISED_SCREENSHOT") {
        let paintable = gtk::WidgetPaintable::new(Some(w));
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, w.width() as f64, w.height() as f64);
        if let (Some(node), Some(renderer)) = (snapshot.to_node(), w.renderer()) {
            renderer.render_texture(&node, None).save_to_png(path).unwrap();
        }
    }
    selector.set_selected(0);
    assert!(preview.is_editable());
    let b=preview.buffer();
    assert_eq!(b.text(&b.start_iter(),&b.end_iter(),false), "printf manual-draft");
    assert!(!dir.join("injected").exists());
    assert!(!state.supervised.borrow().as_ref().unwrap().closing.get());
    eprintln!("NEXUTERM_SMOKE_RECIPES_OK");
    parent.application().unwrap().quit();
}
