//! Static import and explicitly activated ControlMaster forwardings.
use super::*;
use nexuterm::ssh::SshConnection;
use nexuterm::{
    ssh_import,
    tunnels::{Handle, Kind, Spec, Status},
};
use std::sync::Arc;
pub(super) struct Controller {
    window: glib::WeakRef<gtk::Window>,
    jobs: RefCell<Vec<Arc<Handle>>>,
    closing: Cell<bool>,
}
fn controller(state: &Rc<State>) -> Rc<Controller> {
    if let Some(c) = state.ssh_tools.borrow().as_ref() {
        return c.clone();
    }
    let c = Rc::new(Controller {
        window: glib::WeakRef::new(),
        jobs: RefCell::new(Vec::new()),
        closing: Cell::new(false),
    });
    *state.ssh_tools.borrow_mut() = Some(c.clone());
    c
}
pub(super) fn closing(state: &State) -> bool {
    state
        .ssh_tools
        .borrow()
        .as_ref()
        .is_some_and(|c| c.closing.get())
}
pub(super) fn busy(state: &State) -> bool {
    state
        .ssh_tools
        .borrow()
        .as_ref()
        .is_some_and(|c| c.closing.get() || c.jobs.borrow().iter().any(|h| h.busy()))
}
pub(super) async fn stop_all(state: &Rc<State>) {
    let c = state.ssh_tools.borrow().as_ref().cloned();
    if let Some(c) = c {
        c.closing.set(true);
        for h in c.jobs.borrow().iter() {
            h.stop();
        }
        while c.jobs.borrow().iter().any(|h| h.busy()) {
            glib::timeout_future(std::time::Duration::from_millis(25)).await;
        }
        c.closing.set(false);
    }
}
fn save(state: &Rc<State>, config: nexuterm::config::Config) -> Result<(), String> {
    if !state.config_writable {
        return Err("Configuración de solo lectura".into());
    }
    config::save(&state.path, &config)?;
    *state.config.borrow_mut() = config;
    if let Some(window) = state.window() {
        refresh_sidebar(state, &window);
    }
    sessions::refresh_open(state);
    Ok(())
}
pub(super) fn import(state: &Rc<State>, parent: &gtk::ApplicationWindow) {
    if !state.config_writable {
        state.status.set_text("Configuración de solo lectura");
        return;
    }
    let state = Rc::downgrade(state);
    let parent = parent.downgrade();
    glib::spawn_future_local(async move {
        let (Some(state), Some(parent)) = (state.upgrade(), parent.upgrade()) else {
            return;
        };
        let file = gtk::FileDialog::builder()
            .title("Seleccionar config de OpenSSH (solo lectura)")
            .build();
        if let Ok(file) = file.open_future(Some(&parent)).await {
            if let Some(path) = file.path() {
                let home = std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("/"));
                match nexuterm::runtime::runtime()
                    .spawn_blocking(move || ssh_import::read(&path, &home))
                    .await
                {
                    Ok(Ok(preview)) => preview_import(&state, &parent, preview),
                    result => state.status.set_text(&format!(
                        "No se pudo leer: {}",
                        match result {
                            Ok(Err(e)) => e,
                            Err(e) => e.to_string(),
                            _ => unreachable!(),
                        }
                    )),
                }
            }
        }
    });
}
fn preview_import(
    state: &Rc<State>,
    parent: &gtk::ApplicationWindow,
    preview: ssh_import::Preview,
) {
    let window = gtk::Window::builder()
        .title("Previsualizar perfiles OpenSSH")
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(760)
        .default_height(600)
        .build();
    window.set_widget_name("ssh-import-preview");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for set in [&body] {
        set.set_margin_top(12);
        set.set_margin_bottom(12);
        set.set_margin_start(12);
        set.set_margin_end(12);
    }
    let label = gtk::Label::new(Some(
        "Solo Host explícitos, HostName, User, Port y primer IdentityFile. No se leen Include ni se evalúa Match. Se importan valores parciales revisados; los nuevos perfiles usarán -F /dev/null. No se conecta nada.",
    ));
    label.set_wrap(true);
    label.set_xalign(0.0);
    body.append(&label);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let mut choices = Vec::new();
    for p in preview.profiles {
        let duplicate = state
            .config
            .borrow()
            .profiles
            .iter()
            .any(|old| old.name.eq_ignore_ascii_case(&p.name));
        let check = gtk::CheckButton::with_label(&format!(
            "{} → {}@{}:{}\nClave: {}{}",
            p.name,
            p.username,
            p.host,
            p.port,
            if p.identity.is_empty() {
                "agente/predeterminadas"
            } else {
                &p.identity
            },
            if duplicate {
                " · ya existe; no se sobrescribe"
            } else {
                ""
            }
        ));
        check.set_sensitive(!duplicate);
        check.set_active(!duplicate);
        list.append(&check);
        choices.push((p, check.downgrade()));
    }
    if choices.is_empty() {
        list.append(&gtk::Label::new(Some(
            "No hay alias compatibles para importar.",
        )));
    }
    body.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&list)
            .build(),
    );
    let warnings = gtk::TextView::builder()
        .editable(false)
        .wrap_mode(gtk::WrapMode::WordChar)
        .build();
    warnings.buffer().set_text(&if preview.warnings.is_empty() {
        "Sin directivas omitidas.".into()
    } else {
        preview.warnings.join("\n")
    });
    body.append(
        &gtk::ScrolledWindow::builder()
            .min_content_height(120)
            .max_content_height(180)
            .child(&warnings)
            .build(),
    );
    let notice = gtk::Label::new(None);
    notice.set_wrap(true);
    body.append(&notice);
    let accept = gtk::Button::with_label("Importar seleccionados sin conectar");
    accept.set_widget_name("ssh-import-confirm");
    body.append(&accept);
    window.set_child(Some(&body));
    let s = Rc::downgrade(state);
    let w = window.downgrade();
    accept.connect_clicked(move |_| {
        let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) else {
            return;
        };
        let mut config = s.config.borrow().clone();
        let mut count = 0;
        for (p, check) in &choices {
            if check.upgrade().is_some_and(|c| c.is_active())
                && !config
                    .profiles
                    .iter()
                    .any(|old| old.name.eq_ignore_ascii_case(&p.name))
            {
                config.profiles.push(p.clone());
                count += 1;
            }
        }
        if count == 0 {
            notice.set_text("Selecciona algún perfil nuevo.");
            return;
        }
        match save(&s, config) {
            Ok(()) => {
                s.status
                    .set_text(&format!("{count} perfiles importados sin conectar"));
                w.close();
            }
            Err(e) => notice.set_text(&e),
        }
    });
    window.present();
}
fn entry(grid: &gtk::Grid, row: i32, label: &str, value: &str, name: &str) -> gtk::Entry {
    let e = gtk::Entry::builder().text(value).hexpand(true).build();
    e.set_widget_name(name);
    e.set_max_length(255);
    grid.attach(&gtk::Label::new(Some(label)), 0, row, 1, 1);
    grid.attach(&e, 1, row, 1, 1);
    e
}
pub(super) fn show(state: &Rc<State>, parent: &gtk::ApplicationWindow) {
    let c = controller(state);
    if let Some(w) = c.window.upgrade() {
        w.present();
        return;
    }
    let w = gtk::Window::builder()
        .title("Túneles SSH supervisados")
        .transient_for(parent)
        .destroy_with_parent(true)
        .default_width(850)
        .default_height(700)
        .build();
    w.set_widget_name("ssh-tunnels");
    c.window.set(Some(&w));
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_top(12);
    body.set_margin_bottom(12);
    body.set_margin_start(12);
    body.set_margin_end(12);
    let grid = gtk::Grid::builder()
        .column_spacing(8)
        .row_spacing(6)
        .build();
    let name = entry(&grid, 0, "Nombre", "Túnel SSH", "tunnel-name");
    let choices = gtk::DropDown::from_strings(&["Sin sesión SSH abierta"]);
    choices.set_widget_name("tunnel-target");
    grid.attach(&choices, 1, 1, 1, 1);
    grid.attach(&gtk::Label::new(Some("Perfil / sesión SSH")), 0, 1, 1, 1);
    let kind = gtk::DropDown::from_strings(&["Local (-L)", "Remoto (-R)", "SOCKS local (-D)"]);
    kind.set_widget_name("tunnel-kind");
    grid.attach(&kind, 1, 2, 1, 1);
    grid.attach(&gtk::Label::new(Some("Tipo")), 0, 2, 1, 1);
    let bind = entry(&grid, 3, "IP de escucha", "127.0.0.1", "tunnel-bind");
    let port = entry(&grid, 4, "Puerto de escucha", "8080", "tunnel-port");
    let host = entry(&grid, 5, "Host de destino", "127.0.0.1", "tunnel-host");
    let dest = entry(
        &grid,
        6,
        "Puerto de destino",
        "80",
        "tunnel-destination-port",
    );
    body.append(&grid);
    let help = gtk::Label::new(Some(
        "Local: escucha aquí y llega al destino desde el servidor SSH. Remoto: escucha en el servidor y llega al destino desde este equipo. SOCKS: proxy local. La política GatewayPorts del servidor puede alterar la escucha remota. Ningún túnel se activa al guardar o restaurar.",
    ));
    help.set_wrap(true);
    help.set_xalign(0.0);
    body.append(&help);
    let saved = gtk::DropDown::from_strings(&["Definición nueva"]);
    saved.set_widget_name("tunnel-saved");
    body.append(&saved);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let refresh = gtk::Button::with_label("Actualizar sesiones y definiciones");
    let store = gtk::Button::with_label("Guardar nueva definición");
    let remove = gtk::Button::with_label("Borrar definición…");
    let start = gtk::Button::with_label("Revisar y activar…");
    start.set_widget_name("tunnel-start");
    store.set_widget_name("tunnel-save");
    start.add_css_class("suggested-action");
    for b in [&refresh, &store, &remove, &start] {
        buttons.append(b);
    }
    body.append(&buttons);
    let notice = gtk::Label::new(None);
    notice.set_wrap(true);
    notice.set_xalign(0.0);
    notice.set_widget_name("tunnel-notice");
    body.append(&notice);
    let jobs = gtk::ListBox::new();
    jobs.set_selection_mode(gtk::SelectionMode::Single);
    jobs.set_widget_name("tunnel-jobs");
    let displayed = Rc::new(RefCell::new(Vec::<Arc<Handle>>::new()));
    body.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&jobs)
            .build(),
    );
    let stop = gtk::Button::with_label("Detener túnel seleccionado");
    stop.set_widget_name("tunnel-stop");
    body.append(&stop);
    w.set_child(Some(&body));
    let targets = Rc::new(RefCell::new(
        Vec::<(Profile, Option<Arc<SshConnection>>)>::new(),
    ));
    let definitions = Rc::new(RefCell::new(Vec::<nexuterm::config::Tunnel>::new()));
    let update: Rc<dyn Fn()> = {
        let s = Rc::downgrade(state);
        let target = choices.downgrade();
        let saved = saved.downgrade();
        let targets = targets.clone();
        let definitions = definitions.clone();
        Rc::new(move || {
            if let (Some(s), Some(target), Some(saved)) =
                (s.upgrade(), target.upgrade(), saved.upgrade())
            {
                let mut sessions = s
                    .sessions
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .filter(|s| s.pid.get().is_some())
                    .filter_map(|s| {
                        s.connection
                            .as_ref()
                            .filter(|c| !c.closed.load(std::sync::atomic::Ordering::Relaxed))
                            .map(|c| (c.profile.clone(), Some(c.clone())))
                    })
                    .collect::<Vec<_>>();
                for profile in &s.config.borrow().profiles {
                    if profile.protocol == Protocol::Ssh
                        && !sessions.iter().any(|(p, _)| p.id == profile.id)
                    {
                        sessions.push((profile.clone(), None));
                    }
                }
                let labels = sessions
                    .iter()
                    .map(|(p, c)| {
                        format!(
                            "{} · {}@{}:{} · {}",
                            p.name,
                            p.username,
                            p.host,
                            p.port,
                            if c.is_some() {
                                "sesión abierta"
                            } else {
                                "sin conexión"
                            }
                        )
                    })
                    .collect::<Vec<_>>();
                let names = if labels.is_empty() {
                    vec!["Sin sesión SSH abierta"]
                } else {
                    labels.iter().map(String::as_str).collect()
                };
                *targets.borrow_mut() = sessions;
                target.set_model(Some(&gtk::StringList::new(&names)));
                target.set_selected(0);
                *definitions.borrow_mut() = s.config.borrow().tunnels.clone();
                let defs = definitions.borrow();
                let mut names = vec!["Definición nueva"];
                names.extend(defs.iter().map(|d| d.name.as_str()));
                saved.set_model(Some(&gtk::StringList::new(&names)));
                saved.set_selected(0);
            }
        })
    };
    update();
    let u = update.clone();
    refresh.connect_clicked(move |_| u());
    let read: Rc<dyn Fn() -> Result<(Spec, Profile, Option<Arc<SshConnection>>), String>> = {
        let targets = targets.clone();
        let stored = definitions.clone();
        let saved_choice = saved.downgrade();
        let choice = choices.downgrade();
        let kind = kind.downgrade();
        let bind = bind.downgrade();
        let port = port.downgrade();
        let host = host.downgrade();
        let dest = dest.downgrade();
        Rc::new(move || {
            let (Some(choice), Some(kind), Some(bind), Some(port), Some(host), Some(dest)) = (
                choice.upgrade(),
                kind.upgrade(),
                bind.upgrade(),
                port.upgrade(),
                host.upgrade(),
                dest.upgrade(),
            ) else {
                return Err("Ventana cerrada".into());
            };
            let (profile, connection) = targets
                .borrow()
                .get(choice.selected() as usize)
                .cloned()
                .ok_or("Selecciona un perfil SSH y actualiza el listado")?;
            if let Some(saved) = saved_choice.upgrade() {
                if saved.selected() != 0 {
                    let stored = stored.borrow();
                    let definition = saved
                        .selected()
                        .checked_sub(1)
                        .and_then(|i| stored.get(i as usize))
                        .ok_or("Selecciona una definición disponible")?;
                    Spec::from_definition(definition)?;
                    if definition.profile_id != profile.id {
                        return Err("La definición pertenece a otro perfil sin sesión abierta. No se sustituye el destino; elige Definición nueva para otro perfil.".into());
                    }
                }
            }
            let spec = Spec {
                kind: match kind.selected() {
                    0 => Kind::Local,
                    1 => Kind::Remote,
                    _ => Kind::Socks,
                },
                bind: bind.text().parse().map_err(|_| "IP de escucha inválida")?,
                port: port
                    .text()
                    .parse()
                    .map_err(|_| "Puerto de escucha inválido")?,
                host: host.text().into(),
                destination_port: if kind.selected() == 2 {
                    0
                } else {
                    dest.text()
                        .parse()
                        .map_err(|_| "Puerto de destino inválido")?
                },
            };
            spec.validate()?;
            Ok((spec, profile, connection))
        })
    };
    let h = host.downgrade();
    let d = dest.downgrade();
    kind.connect_selected_notify(move |k| {
        if let (Some(h), Some(d)) = (h.upgrade(), d.upgrade()) {
            h.set_sensitive(k.selected() != 2);
            d.set_sensitive(k.selected() != 2);
        }
    });
    let defs = definitions.clone();
    let fields = (
        name.downgrade(),
        kind.downgrade(),
        bind.downgrade(),
        port.downgrade(),
        host.downgrade(),
        dest.downgrade(),
        choices.downgrade(),
    );
    let ts = targets.clone();
    let msg = notice.downgrade();
    saved.connect_selected_notify(move|selected|{let defs=defs.borrow();let Some(d)=selected.selected().checked_sub(1).and_then(|i|defs.get(i as usize))else{return;};let(Some(name),Some(kind),Some(bind),Some(port),Some(host),Some(dest),Some(choice))=(fields.0.upgrade(),fields.1.upgrade(),fields.2.upgrade(),fields.3.upgrade(),fields.4.upgrade(),fields.5.upgrade(),fields.6.upgrade())else{return;};
 if let Err(error)=Spec::from_definition(d){choice.set_selected(gtk::INVALID_LIST_POSITION);if let Some(msg)=msg.upgrade(){msg.set_text(&error);}return;}
 name.set_text(&d.name);kind.set_selected(match d.kind.as_str(){"local"=>0,"remote"=>1,_=>2});bind.set_text(&d.bind_host);port.set_text(&d.bind_port.to_string());host.set_text(&d.destination_host);dest.set_text(&d.destination_port.to_string());
 if let Some(index)=ts.borrow().iter().position(|(p,_)|p.id==d.profile_id){choice.set_selected(index as u32);}else{choice.set_selected(gtk::INVALID_LIST_POSITION);if let Some(msg)=msg.upgrade(){msg.set_text("El perfil de esta definición no tiene una sesión autenticada abierta. No se cambia de destino automáticamente.");}}
 });
    let s = Rc::downgrade(state);
    let get = read.clone();
    let n = name.downgrade();
    let msg = notice.downgrade();
    let u = update.clone();
    store.set_sensitive(state.config_writable);
    store.connect_clicked(move |_| {
        let (Some(s), Some(n), Some(msg)) = (s.upgrade(), n.upgrade(), msg.upgrade()) else {
            return;
        };
        let result = (|| {
            let (spec, profile, _) = get()?;
            if n.text().trim().is_empty()
                || n.text().len() > 120
                || n.text().chars().any(char::is_control)
            {
                return Err("Nombre entre 1 y 120 bytes".into());
            }
            let mut config = s.config.borrow().clone();
            if config.tunnels.len() >= 64 {
                return Err("Máximo 64 definiciones".into());
            }
            config
                .tunnels
                .push(spec.definition(profile.id.clone(), n.text().into()));
            save(&s, config)
        })();
        match result {
            Ok(()) => {
                u();
                msg.set_text("Definición guardada sin activar");
            }
            Err(e) => msg.set_text(&e),
        }
    });
    let s = Rc::downgrade(state);
    let defs = definitions.clone();
    let choice = saved.downgrade();
    let msg = notice.downgrade();
    let parent = parent.downgrade();
    let u = update.clone();
    remove.set_sensitive(state.config_writable);
    remove.connect_clicked(move |_| {
        let (Some(s), Some(choice), Some(msg), Some(parent)) = (
            s.upgrade(),
            choice.upgrade(),
            msg.upgrade(),
            parent.upgrade(),
        ) else {
            return;
        };
        let Some(d) = choice
            .selected()
            .checked_sub(1)
            .and_then(|i| defs.borrow().get(i as usize).cloned())
        else {
            msg.set_text("Selecciona una definición guardada");
            return;
        };
        let u = u.clone();
        glib::spawn_future_local(async move {
            if gtk::AlertDialog::builder()
                .message("¿Borrar definición?")
                .detail("No detiene los túneles activos; solo elimina la configuración guardada.")
                .buttons(["Cancelar", "Borrar"])
                .cancel_button(0)
                .default_button(0)
                .build()
                .choose_future(Some(&parent))
                .await
                == Ok(1)
            {
                let mut config = s.config.borrow().clone();
                config.tunnels.retain(|t| t.id != d.id);
                match save(&s, config) {
                    Ok(()) => u(),
                    Err(e) => msg.set_text(&e),
                }
            }
        });
    });
    let s = Rc::downgrade(state);
    let controller = Rc::downgrade(&c);
    let parent = state.window.clone();
    let msg = notice.downgrade();
    start.connect_clicked(move |_|{let(Some(s),Some(c),Some(parent),Some(msg))=(s.upgrade(),controller.upgrade(),parent.upgrade(),msg.upgrade())else{return;};let(spec,_,connection)=match read(){Ok(v)=>v,Err(e)=>{msg.set_text(&e);return;}};let Some(connection)=connection else{msg.set_text("Abre primero una sesión SSH autenticada para este perfil y actualiza el listado. Guardar no conecta.");return;};
 let detail=format!("Sesión: {} · {}@{}:{}\n{}\n\n{}\nSe solicita el reenvío al ControlMaster existente. No se reconecta ni se reintenta automáticamente. Los streams ya abiertos pueden continuar tras cancelar la escucha.",connection.profile.name,connection.profile.username,connection.profile.host,connection.profile.port,spec.description(),if spec.bind.is_loopback(){"Escucha solicitada en loopback; en remoto la política GatewayPorts del servidor puede modificarla."}else{"ESCUCHA FUERA DE LOOPBACK: puede dar acceso a otros equipos."});
 glib::spawn_future_local(async move{if gtk::AlertDialog::builder().message("¿Activar túnel SSH?").detail(detail).buttons(["Cancelar","Activar"]).cancel_button(0).default_button(0).build().choose_future(Some(&parent)).await!=Ok(1){return;}
 if c.closing.get(){msg.set_text("Se está cerrando la ventana; no se activa");return;}
 let mut jobs=c.jobs.borrow_mut();if jobs.iter().any(|h|Arc::ptr_eq(&h.connection,&connection)&&h.spec==spec&&!matches!(h.snapshot().status,Status::Stopped|Status::Failed)){msg.set_text("Ya existe este túnel o su estado no está confirmado; compruébalo antes de repetir.");return;}
 while jobs.len()>=16{if let Some(i)=jobs.iter().position(|h|matches!(h.snapshot().status,Status::Stopped|Status::Failed)){jobs.remove(i);}else{msg.set_text("Máximo 16 resultados activos/no confirmados");return;}}
 match Handle::start(spec,connection){Ok(h)=>{jobs.push(h);msg.set_text("Activación solicitada. Consulta el estado; no acredita que el servicio destino esté disponible.");},Err(e)=>msg.set_text(&e)}drop(jobs);let _=s;
 });});
    let list = jobs.downgrade();
    let shown = displayed.clone();
    stop.connect_clicked(move |_| {
        if let Some(list) = list.upgrade() {
            if let Some(row) = list.selected_row() {
                if let Some(h) = shown.borrow().get(row.index() as usize) {
                    h.stop();
                }
            }
        }
    });
    let weak = Rc::downgrade(&c);
    let list = jobs.downgrade();
    let window = w.downgrade();
    let last = RefCell::new(Vec::<String>::new());
    glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
        let (Some(c), Some(list), Some(_)) = (weak.upgrade(), list.upgrade(), window.upgrade())
        else {
            return glib::ControlFlow::Break;
        };
        let handles = c.jobs.borrow().clone();
        let rows = handles
            .iter()
            .map(|h| {
                let snap = h.snapshot();
                format!(
                    "{} · {}\n{}: {}",
                    h.connection.profile.name,
                    h.spec.description(),
                    phase(snap.status),
                    snap.detail
                )
            })
            .collect::<Vec<_>>();
        if *last.borrow() != rows {
            let selected = list.selected_row().map(|r| r.index());
            while let Some(row) = list.first_child() {
                list.remove(&row);
            }
            for text in &rows {
                let label = gtk::Label::new(Some(text));
                label.set_wrap(true);
                label.set_xalign(0.0);
                list.append(&label);
            }
            if let Some(row) = list.row_at_index(selected.unwrap_or(0)) {
                list.select_row(Some(&row));
            }
            *displayed.borrow_mut() = handles;
            *last.borrow_mut() = rows;
        }
        glib::ControlFlow::Continue
    });
    w.connect_close_request(|w| {
        w.set_visible(false);
        glib::Propagation::Stop
    });
    w.present();
}

#[cfg(debug_assertions)]
#[path = "ssh_tools_smoke.rs"]
mod smoke;
#[cfg(debug_assertions)]
pub(super) fn smoke(state: &Rc<State>) {
    smoke::run(state);
}

fn phase(status: Status) -> &'static str {
    match status {
        Status::Starting => "Preparando",
        Status::Active => "Activo",
        Status::Stopping => "Deteniendo",
        Status::Stopped => "Detenido",
        Status::Failed => "No iniciado",
        Status::Unknown => "Resultado desconocido",
    }
}
