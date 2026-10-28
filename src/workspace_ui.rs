//! Workspace snapshots are restored disconnected, without replaying commands.
use super::*;
use nexuterm::workspaces::{Layout, Store, Tab, Target, Workspace};

fn capture_layout(state: &State, widget: &gtk::Widget) -> Option<Layout> {
    if let Some(tracker) = state
        .recoveries
        .borrow()
        .iter()
        .filter_map(Weak::upgrade)
        .find(|tracker| {
            tracker
                .root
                .upgrade()
                .as_ref()
                .map(|root| root.upcast_ref::<gtk::Widget>())
                == Some(widget)
        })
    {
        return Some(Layout::Document {
            id: tracker.id.clone(),
            source: tracker.source(),
            had_draft: tracker.modified(),
        });
    }
    if let Some((_, layout)) = state
        .workspace_documents
        .borrow()
        .iter()
        .find(|(root, _)| root.upgrade().as_ref() == Some(widget))
    {
        return Some(layout.clone());
    }

    if let Some(paned) = widget.downcast_ref::<gtk::Paned>()
        && paned.has_css_class("nexuterm-terminal-split")
    {
        let vertical = paned.orientation() == gtk::Orientation::Vertical;
        let size = if vertical {
            paned.height()
        } else {
            paned.width()
        };
        return Some(Layout::Split {
            vertical,
            ratio: if size > 0 {
                (f64::from(paned.position()) / f64::from(size)).clamp(0.05, 0.95)
            } else {
                0.5
            },
            first: Box::new(capture_layout(state, &paned.start_child()?)?),
            second: Box::new(capture_layout(state, &paned.end_child()?)?),
        });
    }
    if let Some((_, target)) = state
        .workspace_placeholders
        .borrow()
        .iter()
        .find(|(weak, _)| weak.upgrade().as_ref() == Some(widget))
    {
        return Some(Layout::Terminal {
            target: target.clone(),
        });
    }
    if let Some(term) = widget.downcast_ref::<vte4::Terminal>() {
        let session = state
            .sessions
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .find(|s| s.terminal.upgrade().as_ref() == Some(term))?;
        let target = if let Some(connection) = &session.connection {
            Target::Connection {
                profile_id: connection.profile.id.clone(),
                directory: state
                    .remote_browsers
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .find_map(|panel| panel.workspace_directory(connection)),
            }
        } else if std::path::Path::new(&session.spec.program)
            .file_name()
            .is_some_and(|name| name == "ssh" || name == "telnet")
        {
            state
                .config
                .borrow()
                .profiles
                .iter()
                .find(|p| connection::command(p).ok().as_ref() == Some(&session.spec))
                .map(|p| Target::Connection {
                    profile_id: p.id.clone(),
                    directory: None,
                })
                .unwrap_or(Target::Unavailable {
                    description: "Conexión sin perfil recuperable".into(),
                })
        } else {
            Target::Local {
                directory: session
                    .pid
                    .get()
                    .and_then(|pid| std::fs::read_link(format!("/proc/{pid}/cwd")).ok())
                    .or_else(|| {
                        term.current_directory_uri()
                            .and_then(|uri| gio::File::for_uri(&uri).path())
                    })
                    .filter(|p| p.is_absolute()),
            }
        };
        return Some(Layout::Terminal { target });
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(layout) = capture_layout(state, &current) {
            return Some(layout);
        }
        child = current.next_sibling();
    }
    None
}

pub(super) fn capture(state: &State, id: String, name: String, notes: String) -> Workspace {
    let tabs = (0..state.notebook.n_pages())
        .map(|i| {
            let page = state.notebook.nth_page(i);
            Tab {
                title: page.title().to_string(),
                layout: capture_layout(state, &page.child()).unwrap_or(Layout::Terminal {
                    target: Target::Unavailable {
                        description: "Este tipo de pestaña todavía no admite recuperación".into(),
                    },
                }),
            }
        })
        .collect();
    Workspace {
        id,
        name,
        notes,
        tabs,
        selected: state
            .notebook
            .selected_page()
            .map(|p| state.notebook.page_position(&p).max(0) as usize)
            .unwrap_or(0),
        directory: state.local_files.borrow().as_ref().map(|p| p.directory()),
        sidebar_visible: state.sidebar_area.is_visible(),
        chats: state
            .ai_workspace
            .borrow()
            .as_ref()
            .map(|p| (p.capture)())
            .unwrap_or_default(),
    }
}

type Connections =
    Rc<RefCell<std::collections::HashMap<String, std::sync::Arc<nexuterm::ssh::SshConnection>>>>;

fn restore_layout(state: &Rc<State>, layout: &Layout, connections: &Connections) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_hexpand(true);
    root.set_vexpand(true);
    match layout {
        Layout::Document {
            id,
            source,
            had_draft,
        } => {
            state
                .workspace_documents
                .borrow_mut()
                .retain(|(root, _)| root.upgrade().is_some());
            state
                .workspace_documents
                .borrow_mut()
                .push((root.upcast_ref::<gtk::Widget>().downgrade(), layout.clone()));
            restore_document(state, &root, id, source, *had_draft);
        }
        Layout::Split {
            vertical,
            ratio,
            first,
            second,
        } => {
            let orientation = if *vertical {
                gtk::Orientation::Vertical
            } else {
                gtk::Orientation::Horizontal
            };
            let paned = gtk::Paned::new(orientation);
            paned.add_css_class("nexuterm-terminal-split");
            paned.set_hexpand(true);
            paned.set_vexpand(true);
            paned.set_start_child(Some(&restore_layout(state, first, connections)));
            paned.set_end_child(Some(&restore_layout(state, second, connections)));
            let ratio = *ratio;
            paned.add_tick_callback(move |paned, _| {
                let size = if orientation == gtk::Orientation::Vertical {
                    paned.height()
                } else {
                    paned.width()
                };
                if size > 0 {
                    paned.set_position((f64::from(size) * ratio) as i32);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
            root.append(&paned);
        }
        Layout::Terminal { target } => {
            let label = match target {
                Target::Local { directory } => format!(
                    "Terminal local · {}",
                    directory
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
                Target::Connection { profile_id, .. } => state
                    .config
                    .borrow()
                    .profiles
                    .iter()
                    .find(|p| &p.id == profile_id)
                    .map(|p| format!("{} · desconectado", p.name))
                    .unwrap_or_else(|| format!("Perfil no disponible: {profile_id}")),
                Target::Unavailable { description } => description.clone(),
            };
            let message = gtk::Label::new(Some(&label));
            message.set_wrap(true);
            root.append(&message);
            let start = gtk::Button::with_label(match target {
                Target::Local { .. } => "Abrir terminal",
                _ => "Conectar",
            });
            start.set_halign(gtk::Align::Center);
            start.set_sensitive(match target {
                Target::Unavailable { .. } => false,
                Target::Connection { profile_id, .. } => state
                    .config
                    .borrow()
                    .profiles
                    .iter()
                    .any(|p| &p.id == profile_id),
                _ => true,
            });
            root.append(&start);
            state
                .workspace_placeholders
                .borrow_mut()
                .retain(|(widget, _)| widget.upgrade().is_some());
            state
                .workspace_placeholders
                .borrow_mut()
                .push((root.upcast_ref::<gtk::Widget>().downgrade(), target.clone()));
            let s = Rc::downgrade(state);
            let r = root.downgrade();
            let m = message.downgrade();
            let target = target.clone();
            let connections = connections.clone();
            start.connect_clicked(move |_| {
                let (Some(state), Some(root), Some(message)) = (s.upgrade(), r.upgrade(), m.upgrade()) else { return; };
                let result = match &target {
                    Target::Local { directory } => {
                        if directory.as_ref().is_some_and(|path| !path.is_dir() || path.to_str().is_none()) {
                            message.set_text("La carpeta guardada no está disponible; no se ha iniciado la terminal.");
                            return;
                        }
                        let shell = std::env::var("SHELL").ok().filter(|s| std::path::Path::new(s).is_absolute()).unwrap_or_else(||"/bin/sh".into());
                        Ok(("Terminal local".to_string(), connection::CommandSpec { program: shell, args: vec!["-l".into()] }, None, directory.clone()))
                    }
                    Target::Connection { profile_id, .. } => {
                        let profile = state.config.borrow().profiles.iter().find(|p| &p.id == profile_id).cloned();
                        let Some(profile) = profile else { message.set_text("El perfil ya no existe."); return; };
                        if profile.protocol != Protocol::Ssh {
                            if let Some(window) = state.window.upgrade() { connect_profile(&state, &window, &profile); }
                            return;
                        }
                        let existing = connections.borrow().get(profile_id).cloned().filter(|c| !c.closed.load(std::sync::atomic::Ordering::Relaxed));
                        let connection = existing.map(Ok).unwrap_or_else(||nexuterm::ssh::SshConnection::new(profile.clone()));
                        connection.and_then(|connection| {
                            let spec = connection.terminal_command()?;
                            connections.borrow_mut().insert(profile_id.clone(), connection.clone());
                            Ok((profile.name, spec, Some(connection), None))
                        })
                    }
                    Target::Unavailable { .. } => return,
                };
                match result {
                    Ok((title, spec, connection, directory)) => {
                        state.workspace_placeholders.borrow_mut().retain(|(w, _)| w.upgrade().as_ref() != Some(root.upcast_ref()));
                        while let Some(child) = root.first_child() { root.remove(&child); }
                        if let (Some(connection), Some(window)) = (&connection, state.window.upgrade()) {
                            let directory = match &target { Target::Connection { directory, .. } => directory.clone().unwrap_or_else(||PathBuf::from(".")), _ => PathBuf::from(".") };
                            let panel = crate::remote_panel::Panel::new_in_directory(&state, &window, connection.clone(), directory);
                            state.sidebar_stack.add_child(&panel.widget);
                            let page = (0..state.notebook.n_pages()).map(|i|state.notebook.nth_page(i).child()).find(|page|root.upcast_ref::<gtk::Widget>() == page || root.is_ancestor(page));
                            if let Some(page) = page { state.remote_panels.borrow_mut().push((page.downgrade(), panel.widget.downgrade())); }
                            state.workspace_panels.borrow_mut().push((root.upcast_ref::<gtk::Widget>().downgrade(), panel));
                            update_sidebar(&state);
                        }
                        terminal_in_directory(&state, &title, spec, Some(&root), connection, directory);
                    }
                    Err(error) => message.set_text(&error),
                }
            });
        }
    }
    root
}

fn page_for(state: &State, root: &gtk::Box) -> Option<adw::TabPage> {
    (0..state.notebook.n_pages())
        .map(|i| state.notebook.nth_page(i))
        .find(|page| page.child() == *root.upcast_ref::<gtk::Widget>())
}

fn reuse_document(
    state: &State,
    placeholder: &gtk::Box,
    id: &str,
    source: &nexuterm::recovery::Source,
) -> Result<bool, String> {
    if let Some((editor, origin)) = crate::files::existing_editor(state, id) {
        if !source.same_document(&origin) {
            return Err(
                "El identificador del documento corresponde a otro origen; no se ha sustituido."
                    .into(),
            );
        }
        let selected = state.notebook.selected_page();
        if let Some(old) = page_for(state, placeholder) {
            state.notebook.close_page(&old);
            if selected.as_ref() == Some(&old) {
                state
                    .notebook
                    .set_selected_page(&state.notebook.page(&editor));
            } else if let Some(selected) = selected
                && (0..state.notebook.n_pages()).any(|i| state.notebook.nth_page(i) == selected)
            {
                state.notebook.set_selected_page(&selected);
            }
        }
        return Ok(true);
    }
    Ok(false)
}

fn replace_document(
    state: &State,
    placeholder: &gtk::Box,
    editor: &gtk::Box,
    selected: Option<adw::TabPage>,
) {
    let (Some(old), Some(new)) = (page_for(state, placeholder), page_for(state, editor)) else {
        return;
    };
    let position = state.notebook.page_position(&old);
    state.notebook.reorder_page(&new, position);
    state.notebook.close_page(&old);
    if selected.as_ref() == Some(&old) {
        state.notebook.set_selected_page(&new);
    } else if let Some(selected) = selected
        && (0..state.notebook.n_pages()).any(|i| state.notebook.nth_page(i) == selected)
    {
        state.notebook.set_selected_page(&selected);
    }
}

fn restore_document(
    state: &Rc<State>,
    root: &gtk::Box,
    id: &str,
    source: &nexuterm::recovery::Source,
    had_draft: bool,
) {
    use nexuterm::recovery::{self, Source};
    let label = gtk::Label::new(Some(&format!("Recuperando documento: {}", source.title())));
    label.set_wrap(true);
    root.append(&label);
    let open = gtk::Button::with_label("Abrir original");
    open.set_halign(gtk::Align::Center);
    open.set_sensitive(false);
    root.append(&open);
    let s = Rc::downgrade(state);
    let r = root.downgrade();
    let message = label.downgrade();
    let origin = source.clone();
    let identifier = id.to_string();
    // A corrupt or mismatched record must never be reused or deleted by opening its original.
    let fallback_id = Rc::new(RefCell::new(uuid::Uuid::new_v4().to_string()));
    let original_id = fallback_id.clone();
    open.connect_clicked(move |button| {
        let (Some(state), Some(root), Some(message)) = (s.upgrade(), r.upgrade(), message.upgrade()) else { return; };
        if page_for(&state, &root).is_none() { return; }
        match reuse_document(&state, &root, &identifier, &origin) {
            Ok(true) => return, Err(error) => { message.set_text(&error); return; }, _ => {},
        }
        let Some(window) = state.window.upgrade() else { return; };
        let selected = state.notebook.selected_page();
        let editor_id = original_id.borrow().clone();
        match &origin {
            Source::Local { path, .. } => {
                let editor = crate::files::editor_document(&state, &window, gio::File::for_path(path), None, None, Some(editor_id));
                replace_document(&state, &root, &editor, selected);
            }
            Source::Remote { profile_id, destination, path, .. } => {
                let Some(connection) = state.recovery_connection(profile_id, destination) else {
                    message.set_text("Conecta primero una terminal SSH al mismo perfil y destino. No se ha iniciado una conexión."); return;
                };
                button.set_sensitive(false);
                let (path, identifier, origin, root, message, button) = (path.clone(), identifier.clone(), origin.clone(), root.downgrade(), message.downgrade(), button.downgrade());
                glib::spawn_future_local(async move {
                    let result = nexuterm::runtime::runtime().spawn(async move { nexuterm::remote_edit::RemoteDocument::open(connection, path).await }).await.map_err(|e|e.to_string()).and_then(|r|r);
                    let (Some(root), Some(message)) = (root.upgrade(), message.upgrade()) else { return; };
                    if page_for(&state, &root).is_none() { return; }
                    let result = match reuse_document(&state, &root, &identifier, &origin) {
                        Ok(true) => return, Err(error) => Err(error), Ok(false) => result,
                    };
                    match result {
                        Ok(remote) => {
                            let selected = state.notebook.selected_page();
                            let editor = crate::files::editor_document(&state, &window, gio::File::for_path(remote.local_path()), Some(remote), None, Some(editor_id));
                            replace_document(&state, &root, &editor, selected);
                        }
                        Err(error) => message.set_text(&format!("No se pudo abrir el original SSH: {error}")),
                    }
                    if let Some(button) = button.upgrade() { button.set_sensitive(true); }
                });
            }
            Source::Detached { .. } => message.set_text("No hay un original disponible para este documento; conserva su borrador para recuperarlo."),
        }
    });
    let (state, root, id, source) = (
        state.clone(),
        root.downgrade(),
        id.to_string(),
        source.clone(),
    );
    glib::spawn_future_local(async move {
        let Some(widget) = root.upgrade() else {
            return;
        };
        match reuse_document(&state, &widget, &id, &source) {
            Ok(true) => return,
            Err(error) => {
                label.set_text(&error);
                return;
            }
            _ => {}
        }
        drop(widget);
        let (identifier, expected) = (id.clone(), source.clone());
        let result = nexuterm::runtime::runtime().spawn_blocking(move || {
            let path = recovery::directory()?.join(format!("{identifier}.json"));
            match std::fs::symlink_metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    Ok(None)
                }
                Err(error) => Err(error.to_string()),
                Ok(_) => {
                    let draft = recovery::read(&path)?;
                    if !expected.same_document(&draft.source) { return Err("El borrador pertenece a otro origen; se ha conservado sin abrirlo.".into()); }
                    Ok(Some(draft))
                }
            }
        }).await.map_err(|e|e.to_string()).and_then(|r|r);
        let Some(root) = root.upgrade() else {
            return;
        };
        if page_for(&state, &root).is_none() {
            return;
        }
        let Some(window) = state.window.upgrade() else {
            return;
        };
        match reuse_document(&state, &root, &id, &source) {
            Ok(true) => return,
            Err(error) => {
                label.set_text(&error);
                return;
            }
            _ => {}
        }
        match result {
            Ok(Some(draft)) => {
                let file = match &draft.source {
                    Source::Local { path, .. } => gio::File::for_path(path),
                    _ => gio::File::for_path(draft.source.basename()),
                };
                let selected = state.notebook.selected_page();
                let editor =
                    crate::files::editor_document(&state, &window, file, None, Some(draft), None);
                replace_document(&state, &root, &editor, selected);
            }
            Ok(None) => {
                *fallback_id.borrow_mut() = id.clone();
                if had_draft {
                    label.set_text("El borrador pendiente ya no está disponible. Puedes abrir el original expresamente; no se han recuperado esos cambios.");
                    open.set_sensitive(!matches!(source, Source::Detached { .. }));
                    return;
                }
                match &source {
                    Source::Local { path, .. } => {
                        let selected = state.notebook.selected_page();
                        let editor = crate::files::editor_document(
                            &state,
                            &window,
                            gio::File::for_path(path),
                            None,
                            None,
                            Some(id),
                        );
                        replace_document(&state, &root, &editor, selected);
                    }
                    Source::Remote { .. } => {
                        label.set_text(&format!("Documento SSH pendiente: {}. Conecta el perfil y abre el original expresamente.", source.title()));
                        open.set_sensitive(true);
                    }
                    Source::Detached { .. } => label.set_text(
                        "No se conserva un borrador ni un original accesible para este documento.",
                    ),
                }
            }
            Err(error) => {
                label.set_text(&format!("No se pudo recuperar el documento: {error}"));
                open.set_sensitive(!matches!(source, Source::Detached { .. }));
            }
        }
    });
}

pub(super) fn restore(state: &Rc<State>, workspace: &Workspace) -> bool {
    if let Some(panel) = state.ai_workspace.borrow().as_ref() {
        if let Err(error) = (panel.restore)(&workspace.chats) {
            state.status.set_text(&error);
            return false;
        }
    }
    let offset = state.notebook.n_pages();
    for tab in &workspace.tabs {
        let connections = Rc::new(RefCell::new(std::collections::HashMap::new()));
        let root = restore_layout(state, &tab.layout, &connections);
        let s = Rc::downgrade(state);
        let r = root.downgrade();
        append_tab(&state.notebook, &root, &tab.title, move || {
            if let (Some(state), Some(root)) = (s.upgrade(), r.upgrade()) {
                state
                    .workspace_documents
                    .borrow_mut()
                    .retain(|(widget, _)| widget.upgrade().as_ref() != Some(root.upcast_ref()));
                for session in state.sessions.borrow().iter().filter_map(Weak::upgrade) {
                    if session
                        .terminal
                        .upgrade()
                        .is_some_and(|t| t.is_ancestor(&root))
                    {
                        session.stop();
                        if let Some(connection) = &session.connection {
                            connection
                                .closed
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                state
                    .workspace_panels
                    .borrow_mut()
                    .retain(|(widget, panel)| {
                        if widget.upgrade().is_none_or(|widget| {
                            widget == *root.upcast_ref::<gtk::Widget>() || widget.is_ancestor(&root)
                        }) {
                            panel.stop();
                            false
                        } else {
                            true
                        }
                    });
                state.remote_panels.borrow_mut().retain(|(page, panel)| {
                    if page.upgrade().as_ref() == Some(root.upcast_ref()) {
                        if let Some(panel) = panel.upgrade() {
                            state.sidebar_stack.remove(&panel);
                        }
                        false
                    } else {
                        true
                    }
                });
            }
        });
    }
    if !workspace.tabs.is_empty() {
        state
            .notebook
            .set_selected_page(&state.notebook.nth_page(offset + workspace.selected as i32));
    }
    if let Some(directory) = &workspace.directory
        && directory.is_dir()
        && let Some(panel) = state.local_files.borrow().as_ref()
    {
        panel.navigate(directory.clone());
    }
    state.sidebar_area.set_visible(workspace.sidebar_visible);
    state.status.set_text(
        "Espacio abierto. Las conexiones esperan tu acción; las pestañas anteriores se conservan.",
    );
    true
}

pub(super) fn show(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    if let Some(dialog) = state.workspace_manager.borrow().upgrade() {
        dialog.present();
        return;
    }
    let loaded = Store::path().and_then(|path| Store::load(&path).map(|store| (path, store)));
    let (path, store) = match loaded {
        Ok(value) => value,
        Err(error) => {
            state
                .status
                .set_text(&format!("No se pueden abrir los espacios: {error}"));
            return;
        }
    };
    let store = Rc::new(RefCell::new(store));
    let dialog = gtk::Window::builder()
        .title("Espacios de trabajo")
        .transient_for(window)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(760)
        .default_height(560)
        .build();
    *state.workspace_manager.borrow_mut() = dialog.downgrade();
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_margin_top(16);
    body.set_margin_bottom(16);
    body.set_margin_start(16);
    body.set_margin_end(16);
    let info = gtk::Label::new(Some(
        "Guarda terminales, divisiones, documentos, borradores, rutas y notas. Incluye mensajes visibles y borradores de chats, sin conectarlos automáticamente. Abrir conserva el trabajo actual y reutiliza los documentos ya abiertos.",
    ));
    info.set_wrap(true);
    body.append(&info);
    let names: Vec<_> = store
        .borrow()
        .workspaces
        .iter()
        .map(|w| w.name.clone())
        .collect();
    let model = gtk::StringList::new(&names.iter().map(String::as_str).collect::<Vec<_>>());
    let selector = gtk::DropDown::new(Some(model.clone()), gtk::Expression::NONE);
    selector.set_selected(gtk::INVALID_LIST_POSITION);
    body.append(&selector);
    let name = gtk::Entry::builder()
        .placeholder_text("Nombre del espacio")
        .max_length(120)
        .build();
    body.append(&name);
    let notes = gtk::TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .vexpand(true)
        .build();
    notes.set_tooltip_text(Some(
        "Notas del proyecto o cliente; no introduzcas credenciales",
    ));
    let scroll = gtk::ScrolledWindow::builder()
        .min_content_height(160)
        .child(&notes)
        .vexpand(true)
        .build();
    body.append(&scroll);
    let status = gtk::Label::new(None);
    status.set_widget_name("workspace-save-status");
    status.set_wrap(true);
    body.append(&status);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let new = gtk::Button::with_label("Nuevo");
    let save = gtk::Button::with_label("Guardar pestañas actuales");
    let edit = gtk::Button::with_label("Guardar nombre y notas");
    let open = gtk::Button::with_label("Abrir");
    let delete = gtk::Button::with_label("Eliminar");
    for button in [&new, &save, &edit, &open, &delete] {
        buttons.append(button);
    }
    body.append(&buttons);
    dialog.set_child(Some(&body));
    let data = store.clone();
    let entry = name.clone();
    let buffer = notes.buffer();
    selector.connect_selected_notify(move |selector| {
        if let Some(workspace) = data.borrow().workspaces.get(selector.selected() as usize) {
            entry.set_text(&workspace.name);
            buffer.set_text(&workspace.notes);
        }
    });
    let select = selector.clone();
    let entry = name.clone();
    let buffer = notes.buffer();
    new.connect_clicked(move |_| {
        select.set_selected(gtk::INVALID_LIST_POSITION);
        entry.set_text("");
        buffer.set_text("");
    });
    let saving = Rc::new(Cell::new(false));
    let close_busy = saving.clone();
    let close_status = status.clone();
    dialog.connect_close_request(move |_| {
        if close_busy.get() {
            close_status.set_text("Espera a que termine el guardado del espacio y sus borradores.");
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    for (button, capture_tabs) in [(&save, true), (&edit, false)] {
        let (data, select, entry, buffer, path, model, status, s, saving, body) = (
            store.clone(),
            selector.clone(),
            name.clone(),
            notes.buffer(),
            path.clone(),
            model.clone(),
            status.clone(),
            Rc::downgrade(state),
            saving.clone(),
            body.downgrade(),
        );
        button.connect_clicked(move |_| {
            if saving.get() { return; }
            let Some(state) = s.upgrade() else { return; };
            if capture_tabs && state.writes.borrow().iter().filter_map(Weak::upgrade).any(|busy|busy.get()) {
                status.set_text("Espera a que termine el guardado o la transferencia actual antes de guardar el espacio."); return;
            }
            let mut next = Store { version: 1, workspaces: data.borrow().workspaces.clone() };
            let index = select.selected() as usize;
            if !capture_tabs && index >= next.workspaces.len() { status.set_text("Selecciona un espacio para editar sus notas."); return; }
            let id = next.workspaces.get(index).map(|w|w.id.clone()).unwrap_or_else(||uuid::Uuid::new_v4().to_string());
            let name = entry.text().trim().to_string();
            let notes = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true).to_string();
            let trackers: Vec<_> = if capture_tabs { state.recoveries.borrow().iter().filter_map(Weak::upgrade).collect() } else { Vec::new() };
            saving.set(true);
            if let Some(body) = body.upgrade() { body.set_sensitive(false); }
            status.set_text("Conservando los borradores y el espacio…");
            let (data, select, path, model, status, saving, body) = (data.clone(), select.clone(), path.clone(), model.clone(), status.clone(), saving.clone(), body.clone());
            glib::spawn_future_local(async move {
                let result = async {
                    for tracker in trackers { tracker.flush().await.map_err(|e|format!("No se pudo conservar un borrador: {e}"))?; }
                    let workspace = if capture_tabs { capture(&state, id, name, notes) }
                        else { let mut workspace = next.workspaces[index].clone(); workspace.name = name; workspace.notes = notes; workspace };
                    let index = if index < next.workspaces.len() { next.workspaces[index] = workspace; index }
                        else { next.workspaces.push(workspace); next.workspaces.len() - 1 };
                    nexuterm::runtime::runtime().spawn_blocking(move || next.save(&path).map(|_|(next,index))).await.map_err(|e|e.to_string())?
                }.await;
                saving.set(false);
                if let Some(body) = body.upgrade() { body.set_sensitive(true); }
                match result {
                    Ok((next,index)) => {
                        let names: Vec<_> = next.workspaces.iter().map(|w|w.name.clone()).collect();
                        *data.borrow_mut() = next;
                        model.splice(0, model.n_items(), &names.iter().map(String::as_str).collect::<Vec<_>>());
                        select.set_selected(index as u32);
                        status.set_text("Espacio guardado; los borradores de los editores abiertos están conservados.");
                    }
                    Err(error) => status.set_text(&format!("No se guardó el espacio: {error}")),
                }
            });
        });
    }
    let data = store.clone();
    let select = selector.clone();
    let s = Rc::downgrade(state);
    let d = dialog.downgrade();
    open.connect_clicked(move |_| {
        if let (Some(state), Some(workspace)) = (
            s.upgrade(),
            data.borrow().workspaces.get(select.selected() as usize),
        ) {
            restore(&state, workspace);
            if let Some(dialog) = d.upgrade() {
                dialog.close();
            }
        }
    });
    #[cfg(debug_assertions)]
    if let Ok(mode) = std::env::var("NEXUTERM_SMOKE_WORKSPACES") {
        let (name, notes, save, edit, open, selector, path) = (
            name.clone(),
            notes.clone(),
            save.clone(),
            edit.clone(),
            open.clone(),
            selector.clone(),
            path.clone(),
        );
        let state = state.clone();
        let saving = saving.clone();
        glib::spawn_future_local(async move {
            glib::timeout_future(std::time::Duration::from_millis(200)).await;
            if mode == "save" {
                name.set_text("Cliente de prueba");
                notes.buffer().set_text("Notas persistentes con acentos");
                save.emit_clicked();
                while saving.get() {
                    glib::timeout_future(std::time::Duration::from_millis(20)).await;
                }
                let saved = Store::load(&path).unwrap();
                assert_eq!(saved.workspaces.len(), 1);
                assert!(matches!(
                    saved.workspaces[0].tabs[0].layout,
                    Layout::Split { .. }
                ));
                name.set_text("Cliente renombrado");
                edit.emit_clicked();
                while saving.get() {
                    glib::timeout_future(std::time::Duration::from_millis(20)).await;
                }
                let saved = Store::load(&path).unwrap();
                assert_eq!(saved.workspaces[0].name, "Cliente renombrado");
                assert_eq!(saved.workspaces[0].notes, "Notas persistentes con acentos");
                assert_eq!(saved.workspaces[0].tabs.len(), 1);
                eprintln!("NEXUTERM_SMOKE_WORKSPACES_SAVE_OK");
            } else {
                let before = state
                    .sessions
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .count();
                let count = state.notebook.n_pages();
                selector.set_selected(0);
                open.emit_clicked();
                assert_eq!(state.notebook.n_pages(), count + 1);
                assert_eq!(
                    state
                        .sessions
                        .borrow()
                        .iter()
                        .filter_map(Weak::upgrade)
                        .count(),
                    before,
                    "Restoring must not spawn processes"
                );
                let root = state.notebook.nth_page(count).child();
                assert!(matches!(
                    capture_layout(&state, &root),
                    Some(Layout::Split { .. })
                ));
                assert_eq!(state.workspace_placeholders.borrow().len(), 3);
                if mode == "missing-profile" {
                    let target =
                        find_button(&root, "Conectar").expect("Missing profile placeholder");
                    assert!(
                        !target.is_sensitive(),
                        "A missing profile must not be substituted"
                    );
                    eprintln!("NEXUTERM_SMOKE_WORKSPACES_MISSING_PROFILE_OK");
                    return;
                }
                let button = find_start(&root).unwrap();
                button.emit_clicked();
                if mode == "missing-folder" {
                    assert_eq!(
                        state
                            .sessions
                            .borrow()
                            .iter()
                            .filter_map(Weak::upgrade)
                            .count(),
                        before
                    );
                    assert!(
                        find_start(&root).is_some(),
                        "Keep placeholder for an unavailable directory"
                    );
                    eprintln!("NEXUTERM_SMOKE_WORKSPACES_MISSING_FOLDER_OK");
                    return;
                }
                let state = state.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                    let session = state
                        .sessions
                        .borrow()
                        .iter()
                        .filter_map(Weak::upgrade)
                        .last()
                        .unwrap();
                    let pid = session
                        .pid
                        .get()
                        .expect("Explicitly opened terminal must start");
                    let cwd = std::fs::read_link(format!("/proc/{pid}/cwd")).unwrap();
                    let saved = Store::load(&path).unwrap();
                    fn first_directory(layout: &Layout) -> &Option<PathBuf> {
                        match layout {
                            Layout::Split { first, .. } => first_directory(first),
                            Layout::Terminal {
                                target: Target::Local { directory },
                            } => directory,
                            _ => panic!("Expected local terminal"),
                        }
                    }
                    assert_eq!(
                        Some(cwd),
                        *first_directory(&saved.workspaces[0].tabs[0].layout)
                    );
                    eprintln!("NEXUTERM_SMOKE_WORKSPACES_RESTORE_OK");
                });
            }
        });
    }
    let data = store;
    let select = selector;
    let status = status.clone();
    let d = dialog.downgrade();
    delete.connect_clicked(move |_| {
        let Some(workspace) = data
            .borrow()
            .workspaces
            .get(select.selected() as usize)
            .cloned()
        else {
            return;
        };
        let confirm = gtk::AlertDialog::builder()
            .modal(true)
            .message(format!("¿Eliminar el espacio «{}»?", workspace.name))
            .detail("No se cerrarán las pestañas ni se borrarán archivos del proyecto.")
            .buttons(["Cancelar", "Eliminar"])
            .cancel_button(0)
            .default_button(0)
            .build();
        let (data, select, status, path, model, d) = (
            data.clone(),
            select.clone(),
            status.clone(),
            path.clone(),
            model.clone(),
            d.clone(),
        );
        glib::spawn_future_local(async move {
            let Some(dialog) = d.upgrade() else {
                return;
            };
            if confirm.choose_future(Some(&dialog)).await != Ok(1) {
                return;
            }
            let mut next = Store {
                version: 1,
                workspaces: data.borrow().workspaces.clone(),
            };
            next.workspaces.retain(|w| w.id != workspace.id);
            match next.save(&path) {
                Ok(()) => {
                    let names: Vec<_> = next.workspaces.iter().map(|w| w.name.clone()).collect();
                    *data.borrow_mut() = next;
                    model.splice(
                        0,
                        model.n_items(),
                        &names.iter().map(String::as_str).collect::<Vec<_>>(),
                    );
                    select.set_selected(gtk::INVALID_LIST_POSITION);
                    status.set_text("Espacio eliminado.");
                }
                Err(error) => status.set_text(&error),
            }
        });
    });
    dialog.present();
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os("NEXUTERM_WORKSPACE_SCREENSHOT") {
        let dialog = dialog.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_secs(1), move || {
            if let Some(dialog) = dialog.upgrade() {
                let paintable = gtk::WidgetPaintable::new(Some(&dialog));
                let snapshot = gtk::Snapshot::new();
                paintable.snapshot(&snapshot, dialog.width() as f64, dialog.height() as f64);
                if let (Some(node), Some(renderer)) = (snapshot.to_node(), dialog.renderer()) {
                    renderer
                        .render_texture(&node, None)
                        .save_to_png(path)
                        .unwrap();
                }
            }
        });
    }
}

#[cfg(debug_assertions)]
fn find_start(widget: &gtk::Widget) -> Option<gtk::Button> {
    find_button(widget, "Abrir terminal")
}

#[cfg(debug_assertions)]
fn find_button(widget: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    if let Some(button) = widget.downcast_ref::<gtk::Button>()
        && button.label().as_deref() == Some(label)
    {
        return Some(button.clone());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(button) = find_button(&current, label) {
            return Some(button);
        }
        child = current.next_sibling();
    }
    None
}

#[cfg(debug_assertions)]
pub(super) fn smoke(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    let state = state.clone();
    let window = window.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
        if std::env::var("NEXUTERM_SMOKE_WORKSPACES").as_deref() == Ok("save") {
            gio::prelude::ActionGroupExt::activate_action(&window, "split-horizontal", None);
            gio::prelude::ActionGroupExt::activate_action(&window, "split-vertical", None);
        }
        show(&state, &window);
    });
}

#[cfg(debug_assertions)]
fn smoke_widget(widget: &gtk::Widget, kind: glib::Type) -> Option<gtk::Widget> {
    if widget.type_() == kind {
        return Some(widget.clone());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(found) = smoke_widget(&current, kind) {
            return Some(found);
        }
        child = current.next_sibling();
    }
    None
}

#[cfg(debug_assertions)]
pub(super) fn smoke_documents(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    use nexuterm::recovery::{self, Source};
    let mode = std::env::var("NEXUTERM_SMOKE_WORKSPACE_DOCUMENTS").unwrap();
    let base = PathBuf::from(std::env::var_os("NEXUTERM_DOCUMENT_FIXTURE").unwrap());
    if mode == "save" {
        crate::files::editor(state, window, gio::File::for_path(base.join("clean.txt")));
        crate::files::editor(state, window, gio::File::for_path(base.join("dirty.txt")));
        let seed = base.join("remote-seed.json");
        let value: recovery::Draft = serde_json::from_slice(&std::fs::read(seed).unwrap()).unwrap();
        value.validate().unwrap();
        crate::files::recovered_editor(state, window, value);
    } else if mode == "save-failure" {
        crate::files::editor(state, window, gio::File::for_path(base.join("dirty.txt")));
    }
    let (state, window) = (state.clone(), window.clone());
    glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        let path = Store::path().unwrap();
        if mode == "save" || mode == "save-failure" {
            let dirty = state.recoveries.borrow().iter().filter_map(Weak::upgrade).find(|t|matches!(t.source(), Source::Local { path, .. } if path == base.join("dirty.txt"))).unwrap();
            let root = dirty.root.upgrade().unwrap();
            let view = smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap();
            assert!(view.is_editable());
            view.buffer().set_text("Cambios pendientes áé\n");
            state
                .notebook
                .set_selected_page(&state.notebook.page(&root));
            let previous = std::fs::read(&path).ok();
            show(&state, &window);
            let manager = state.workspace_manager.borrow().upgrade().unwrap();
            smoke_widget(manager.upcast_ref(), gtk::Entry::static_type())
                .unwrap()
                .downcast::<gtk::Entry>()
                .unwrap()
                .set_text("Documentos cliente");
            find_button(manager.upcast_ref(), "Guardar pestañas actuales")
                .unwrap()
                .emit_clicked();
            for _ in 0..60 {
                glib::timeout_future(std::time::Duration::from_millis(30)).await;
                if find_button(manager.upcast_ref(), "Guardar pestañas actuales")
                    .unwrap()
                    .is_sensitive()
                {
                    break;
                }
            }
            if mode == "save-failure" {
                assert_eq!(
                    std::fs::read(&path).ok(),
                    previous,
                    "A failed draft flush must preserve the old workspace"
                );
                assert!(view.buffer().is_modified());
                eprintln!("NEXUTERM_SMOKE_DOCUMENTS_SAVE_FAILURE_OK");
                return;
            }
            let saved = Store::load(&path).unwrap();
            let workspace = &saved.workspaces[0];
            assert_eq!(workspace.tabs.len(), 4);
            assert_eq!(workspace.selected, 2);
            for tab in &workspace.tabs[1..] {
                let Layout::Document {
                    id,
                    source,
                    had_draft,
                } = &tab.layout
                else {
                    panic!("Editor must be recorded as a document");
                };
                let draft_path = recovery::directory().unwrap().join(format!("{id}.json"));
                if *had_draft {
                    assert!(
                        recovery::read(&draft_path)
                            .unwrap()
                            .source
                            .same_document(source)
                    );
                } else {
                    assert!(!draft_path.exists());
                }
            }
            assert!(
                !String::from_utf8(std::fs::read(&path).unwrap())
                    .unwrap()
                    .contains("Cambios pendientes")
            );
            eprintln!("NEXUTERM_DOCUMENTS_PID={}", std::process::id());
            eprintln!("NEXUTERM_SMOKE_DOCUMENTS_SAVE_OK");
            return;
        }
        let saved = Store::load(&path).unwrap();
        let workspace = &saved.workspaces[0];
        if mode == "remote-clean" {
            let Layout::Document { id, .. } = &workspace.tabs[0].layout else {
                unreachable!()
            };
            let before = state.notebook.n_pages();
            let sessions = state
                .sessions
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .count();
            show(&state, &window);
            let manager = state.workspace_manager.borrow().upgrade().unwrap();
            smoke_widget(manager.upcast_ref(), gtk::DropDown::static_type())
                .unwrap()
                .downcast::<gtk::DropDown>()
                .unwrap()
                .set_selected(0);
            find_button(manager.upcast_ref(), "Abrir")
                .unwrap()
                .emit_clicked();
            glib::timeout_future(std::time::Duration::from_millis(100)).await;
            assert!(
                crate::files::existing_editor(&state, id).is_none(),
                "A clean SSH document must wait for an explicit open"
            );
            let placeholder = state.notebook.nth_page(before).child();
            let open = find_button(&placeholder, "Abrir original").unwrap();
            assert!(open.is_sensitive());
            open.emit_clicked();
            for _ in 0..60 {
                glib::timeout_future(std::time::Duration::from_millis(30)).await;
                if crate::files::existing_editor(&state, id).is_some_and(|(root, _)| {
                    smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                        .unwrap()
                        .downcast::<gtk::TextView>()
                        .unwrap()
                        .is_editable()
                }) {
                    break;
                }
            }
            let (root, _) = crate::files::existing_editor(&state, id).unwrap();
            let buffer = smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap()
                .buffer();
            assert_eq!(
                crate::recovery_ui::snapshot(&buffer).unwrap(),
                "Documento SSH actualizado\n"
            );
            assert!(!buffer.is_modified());
            assert_eq!(state.notebook.n_pages(), before + 1);
            assert_eq!(
                state.notebook.page_position(&state.notebook.page(&root)),
                before
            );
            assert_eq!(
                state
                    .sessions
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .count(),
                sessions
            );
            restore(&state, workspace);
            glib::timeout_future(std::time::Duration::from_millis(100)).await;
            assert_eq!(state.notebook.n_pages(), before + 1);
            assert_eq!(state.notebook.selected_page(), page_for(&state, &root));
            eprintln!("NEXUTERM_SMOKE_DOCUMENTS_REMOTE_CLEAN_OK");
            return;
        }
        let before_sessions = state
            .sessions
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .count();
        show(&state, &window);
        let manager = state.workspace_manager.borrow().upgrade().unwrap();
        smoke_widget(manager.upcast_ref(), gtk::DropDown::static_type())
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap()
            .set_selected(0);
        find_button(manager.upcast_ref(), "Abrir")
            .unwrap()
            .emit_clicked();
        for _ in 0..60 {
            glib::timeout_future(std::time::Duration::from_millis(30)).await;
            let ready = workspace.tabs[1..].iter().all(|tab| {
                let Layout::Document { id, source, .. } = &tab.layout else { return false; };
                if matches!(mode.as_str(), "missing-draft" | "corrupt-local") && matches!(source, Source::Local { path, .. } if path == &base.join("dirty.txt")) { return true; }
            if mode == "corrupt-draft" && matches!(source, Source::Remote { .. }) { return true; }
                crate::files::existing_editor(&state, id).is_some_and(|(root,_)|smoke_widget(root.upcast_ref(), gtk::TextView::static_type()).unwrap().downcast::<gtk::TextView>().unwrap().is_editable())
            });
            if ready {
                break;
            }
        }
        assert_eq!(state.notebook.n_pages(), 5);
        assert_eq!(
            state
                .sessions
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .count(),
            before_sessions,
            "Opening documents must not start SSH or terminal processes"
        );
        for (index, tab) in workspace.tabs.iter().enumerate().skip(1) {
            let Layout::Document { id, source, .. } = &tab.layout else {
                panic!("Expected document");
            };
            let placeholder = state
                .notebook
                .nth_page(index as i32 + 1)
                .child()
                .downcast::<gtk::Box>()
                .unwrap();
            if mode == "missing-draft"
                && matches!(source, Source::Local { path, .. } if path == &base.join("dirty.txt"))
            {
                assert!(crate::files::existing_editor(&state, id).is_none());
                assert!(
                    find_button(placeholder.upcast_ref(), "Abrir original")
                        .unwrap()
                        .is_sensitive()
                );
                find_button(placeholder.upcast_ref(), "Abrir original")
                    .unwrap()
                    .emit_clicked();
                glib::timeout_future(std::time::Duration::from_millis(100)).await;
                let (root, _) = crate::files::existing_editor(&state, id).unwrap();
                let buffer = smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                    .unwrap()
                    .downcast::<gtk::TextView>()
                    .unwrap()
                    .buffer();
                assert_eq!(
                    crate::recovery_ui::snapshot(&buffer).unwrap(),
                    "Cambio externo\n"
                );
                assert!(!buffer.is_modified());
                continue;
            }
            if mode == "corrupt-local"
                && matches!(source, Source::Local { path, .. } if path == &base.join("dirty.txt"))
            {
                assert!(crate::files::existing_editor(&state, id).is_none());
                find_button(placeholder.upcast_ref(), "Abrir original")
                    .unwrap()
                    .emit_clicked();
                glib::timeout_future(std::time::Duration::from_millis(100)).await;
                assert!(
                    crate::files::existing_editor(&state, id).is_none(),
                    "Opening the original must use another identity for a corrupt draft"
                );
                let fresh = state
                    .recoveries
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .find(|t| t.id != *id && t.source().same_document(source))
                    .unwrap();
                let root = fresh.root.upgrade().unwrap();
                let buffer = smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                    .unwrap()
                    .downcast::<gtk::TextView>()
                    .unwrap()
                    .buffer();
                assert_eq!(
                    crate::recovery_ui::snapshot(&buffer).unwrap(),
                    "Cambio externo\n"
                );
                assert!(!buffer.is_modified());
                continue;
            }
            if mode == "corrupt-draft" && matches!(source, Source::Remote { .. }) {
                assert!(crate::files::existing_editor(&state, id).is_none());
                let data = std::fs::read(recovery::directory().unwrap().join(format!("{id}.json")))
                    .unwrap();
                assert_eq!(data, b"corrupt");
                continue;
            }
            let (root, _) = crate::files::existing_editor(&state, id).unwrap();
            assert_eq!(
                state.notebook.page_position(&state.notebook.page(&root)),
                index as i32 + 1
            );
            let buffer = smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap()
                .buffer();
            match source {
                Source::Local { path, .. } if path == &base.join("clean.txt") => {
                    assert_eq!(
                        crate::recovery_ui::snapshot(&buffer).unwrap(),
                        "Original limpio actualizado\n"
                    );
                    assert!(!buffer.is_modified());
                }
                Source::Local { .. } => {
                    assert_eq!(
                        crate::recovery_ui::snapshot(&buffer).unwrap(),
                        "Cambios pendientes áé\n"
                    );
                    assert!(buffer.is_modified());
                    assert!(
                        !find_button(root.upcast_ref(), "Guardar")
                            .unwrap()
                            .is_sensitive()
                    );
                    // The handler must reject a save without a version even if invoked directly.
                    find_button(root.upcast_ref(), "Guardar")
                        .unwrap()
                        .emit_clicked();
                }
                Source::Remote { .. } => {
                    assert_eq!(
                        crate::recovery_ui::snapshot(&buffer).unwrap(),
                        "Borrador SSH sin conexión\n"
                    );
                    assert!(buffer.is_modified());
                    assert!(
                        !find_button(root.upcast_ref(), "Guardar")
                            .unwrap()
                            .is_sensitive()
                    );
                    // The handler must reject a save without a version even if invoked directly.
                    find_button(root.upcast_ref(), "Guardar")
                        .unwrap()
                        .emit_clicked();
                }
                _ => panic!("Unexpected fixture document"),
            }
        }
        if mode == "corrupt-local" {
            let Layout::Document { id, source, .. } = &workspace.tabs[2].layout else {
                unreachable!()
            };
            let fresh = state
                .recoveries
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .find(|t| t.id != *id && t.source().same_document(source))
                .unwrap();
            let root = fresh.root.upgrade().unwrap();
            state.notebook.close_page(&state.notebook.page(&root));
            glib::timeout_future(std::time::Duration::from_millis(100)).await;
            assert_eq!(
                std::fs::read(recovery::directory().unwrap().join(format!("{id}.json"))).unwrap(),
                b"corrupt"
            );
        }
        if mode == "restore" {
            let Layout::Document { id, .. } = &workspace.tabs[2].layout else {
                unreachable!()
            };
            let (root, _) = crate::files::existing_editor(&state, id).unwrap();
            assert_eq!(state.notebook.selected_page(), page_for(&state, &root));
            let buffer = smoke_widget(root.upcast_ref(), gtk::TextView::static_type())
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap()
                .buffer();
            buffer.insert_at_cursor("otra edición\n");
            // The general recovery list must also reuse the live editor.
            let draft =
                recovery::read(&recovery::directory().unwrap().join(format!("{id}.json"))).unwrap();
            crate::files::recovered_editor(&state, &window, draft);
            assert_eq!(state.notebook.n_pages(), 5);
            restore(&state, workspace);
            glib::timeout_future(std::time::Duration::from_millis(200)).await;
            assert_eq!(
                state.notebook.n_pages(),
                6,
                "A second open adds only its terminal, reusing the three editors"
            );
            assert_eq!(
                state
                    .recoveries
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .filter(|t| t.root.upgrade().is_some())
                    .count(),
                3
            );
            assert_eq!(
                crate::recovery_ui::snapshot(&buffer).unwrap(),
                "Cambios pendientes áé\notra edición\n"
            );
            assert_eq!(state.notebook.selected_page(), page_for(&state, &root));
            assert_eq!(state.notebook.page_position(&state.notebook.page(&root)), 3);
            for (index, tab) in workspace.tabs.iter().enumerate().skip(1) {
                let Layout::Document { id, .. } = &tab.layout else {
                    unreachable!()
                };
                let (root, _) = crate::files::existing_editor(&state, id).unwrap();
                assert_eq!(
                    state.notebook.page_position(&state.notebook.page(&root)),
                    index as i32 + 1,
                    "Reopening must preserve the position of existing editors"
                );
            }
        }
        eprintln!("NEXUTERM_SMOKE_DOCUMENTS_RESTORE_OK");
    });
}

#[cfg(debug_assertions)]
pub(super) fn smoke_chats(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    fn named(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
        if root.widget_name() == name {
            return Some(root.clone());
        }
        let mut child = root.first_child();
        while let Some(w) = child {
            if let Some(found) = named(&w, name) {
                return Some(found);
            }
            child = w.next_sibling();
        }
        None
    }
    let state = state.clone();
    let window = window.clone();
    glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(200)).await;
        let mode = std::env::var("NEXUTERM_SMOKE_WORKSPACE_CHATS").unwrap();
        let stored = Store::load(&Store::path().unwrap()).unwrap();
        let expected = stored.workspaces[0].chats.clone();
        show(&state, &window);
        let manager = state.workspace_manager.borrow().upgrade().unwrap();
        let selector = smoke_widget(manager.upcast_ref(), gtk::DropDown::static_type())
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap();
        selector.set_selected(0);
        find_button(manager.upcast_ref(), "Abrir")
            .unwrap()
            .emit_clicked();
        drop(selector);
        drop(manager);
        assert_eq!(
            (state.ai_workspace.borrow().as_ref().unwrap().capture)(),
            expected,
            "offline restore must preserve every field"
        );
        assert!(!state.ai_process.borrow().as_ref().unwrap().busy());
        let notebook = named(window.upcast_ref(), "ai-conversations")
            .unwrap()
            .downcast::<gtk::Notebook>()
            .unwrap();
        assert_eq!(notebook.n_pages(), expected.tabs.len() as u32);
        assert_eq!(notebook.current_page(), Some(expected.selected as u32));
        let page = notebook.nth_page(notebook.current_page()).unwrap();
        let input = named(&page, "codex-composer")
            .unwrap()
            .downcast::<gtk::TextView>()
            .unwrap();
        input
            .buffer()
            .set_text("Borrador editado después de recuperar áé\n");
        restore(&state, &stored.workspaces[0]);
        assert_eq!(notebook.n_pages(), expected.tabs.len() as u32);
        assert_eq!(
            input.buffer().text(
                &input.buffer().start_iter(),
                &input.buffer().end_iter(),
                false
            ),
            "Borrador editado después de recuperar áé\n"
        );
        let before = state.notebook.n_pages();
        let chats_before = (state.ai_workspace.borrow().as_ref().unwrap().capture)();
        restore(&state, &stored.workspaces[1]);
        assert_eq!(
            state.notebook.n_pages(),
            before,
            "capacity rejection must not partially restore terminal tabs"
        );
        assert_eq!(
            (state.ai_workspace.borrow().as_ref().unwrap().capture)(),
            chats_before
        );
        assert!(state.status.text().contains("máximo de ocho"));
        if mode == "save" {
            show(&state, &window);
            let manager = state.workspace_manager.borrow().upgrade().unwrap();
            smoke_widget(manager.upcast_ref(), gtk::DropDown::static_type())
                .unwrap()
                .downcast::<gtk::DropDown>()
                .unwrap()
                .set_selected(0);
            smoke_widget(manager.upcast_ref(), gtk::Entry::static_type())
                .unwrap()
                .downcast::<gtk::Entry>()
                .unwrap()
                .set_text(&stored.workspaces[0].name);
            find_button(manager.upcast_ref(), "Guardar pestañas actuales")
                .unwrap()
                .emit_clicked();
            let status = named(manager.upcast_ref(), "workspace-save-status")
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap();
            for _ in 0..60 {
                glib::timeout_future(std::time::Duration::from_millis(30)).await;
                if status.text().starts_with("Espacio guardado")
                    || status.text().starts_with("No se guardó")
                {
                    break;
                }
            }
            assert!(
                status.text().starts_with("Espacio guardado"),
                "{}",
                status.text()
            );
            let saved = Store::load(&Store::path().unwrap()).unwrap();
            assert_eq!(saved.workspaces[0].chats, chats_before);
            eprintln!("NEXUTERM_SMOKE_CHATS_SAVE_OK");
        } else if mode == "resume" || mode == "resume-failure" {
            notebook.set_current_page(Some(0));
            let page = notebook.nth_page(Some(0)).unwrap();
            let resume = named(&page, "workspace-resume-chat")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap();
            let resume_draft = "Borrador editado antes de reanudar áé\n";
            named(&page, "codex-composer")
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap()
                .buffer()
                .set_text(resume_draft);
            resume.emit_clicked();
            for _ in 0..70 {
                glib::timeout_future(std::time::Duration::from_millis(35)).await;
                let snap = (state.ai_workspace.borrow().as_ref().unwrap().capture)();
                if mode == "resume"
                    && snap.tabs[0]
                        .messages
                        .iter()
                        .any(|m| m.text == "Respuesta anterior")
                {
                    break;
                }
            }
            let snap = (state.ai_workspace.borrow().as_ref().unwrap().capture)();
            assert_eq!(snap.tabs[0].codex_thread.as_deref(), Some("fixture-thread"));
            assert_eq!(snap.tabs[0].draft, resume_draft);
            if mode == "resume" {
                assert!(
                    snap.tabs[0]
                        .messages
                        .iter()
                        .any(|m| m.text == "Respuesta anterior")
                );
            } else {
                assert_eq!(snap.tabs[0].messages, expected.tabs[0].messages);
                let send = named(&page, "codex-send")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap();
                assert!(!send.is_sensitive());
                send.emit_clicked();
                let after = (state.ai_workspace.borrow().as_ref().unwrap().capture)();
                assert_eq!(after.tabs[0].draft, resume_draft);
                assert_eq!(after.tabs[0].messages, snap.tabs[0].messages);
            }
            if mode=="resume" {assert!(named(&page,"ai-status").unwrap().downcast::<gtk::Label>().unwrap().text().is_empty());}
            eprintln!("NEXUTERM_SMOKE_CHATS_RESUME_OK");
        } else {
            // Closing even the last restored chat leaves a usable empty palette.
            for _ in 0..expected.tabs.len() {
                let page = notebook.nth_page(Some(0)).unwrap();
                named(&notebook.tab_label(&page).unwrap(), "workspace-close-chat")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .emit_clicked();
            }
            assert_eq!(notebook.n_pages(), 1);
            assert!(
                (state.ai_workspace.borrow().as_ref().unwrap().capture)()
                    .tabs
                    .is_empty()
            );
            eprintln!("NEXUTERM_SMOKE_CHATS_RESTORE_OK");
        }
    });
}
