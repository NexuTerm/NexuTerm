use crate::ui::State;
use adw::prelude::*;
use gtk::{gio, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

const PRIORITY: glib::Priority = glib::Priority::DEFAULT;
const EDIT_LIMIT: usize = 8 * 1024 * 1024;

pub fn choose_editor(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    let dialog = gtk::FileDialog::builder()
        .title("Abrir archivo de texto")
        .build();
    let weak = Rc::downgrade(state);
    let win = window.clone();
    glib::spawn_future_local(async move {
        if let Ok(file) = dialog.open_future(Some(&win)).await
            && let Some(s) = weak.upgrade()
        {
            editor(&s, &win, file);
        }
    });
}

pub fn editor(state: &Rc<State>, window: &gtk::ApplicationWindow, file: gio::File) {
    editor_document(state, window, file, None, None, None);
}
pub fn remote_editor(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    remote: std::sync::Arc<nexuterm::remote_edit::RemoteDocument>,
) {
    editor_document(
        state,
        window,
        gio::File::for_path(remote.local_path()),
        Some(remote),
        None,
        None,
    );
}
pub(crate) fn existing_editor(state: &State, id: &str) -> Option<(gtk::Box, nexuterm::recovery::Source)> {
    state.recoveries.borrow().iter().filter_map(std::rc::Weak::upgrade).filter(|tracker|tracker.id == id)
        .find_map(|tracker| {
            let root = tracker.root.upgrade()?;
            (0..state.notebook.n_pages()).any(|i|state.notebook.nth_page(i).child() == root)
                .then(||(root, tracker.source()))
        })
}
pub fn recovered_editor(state: &Rc<State>, window: &gtk::ApplicationWindow, draft: nexuterm::recovery::Draft) {
    if let Some((root, source)) = existing_editor(state, &draft.id) {
        if source.same_document(&draft.source) { state.notebook.set_selected_page(&state.notebook.page(&root)); }
        else { state.status.set_text("El identificador del borrador corresponde a otro origen; no se ha abierto otra copia."); }
        return;
    }
    let file = match &draft.source {
        nexuterm::recovery::Source::Local { path, .. } => gio::File::for_path(path),
        _ => gio::File::for_path(draft.source.basename()),
    };
    editor_document(state, window, file, None, Some(draft), None);
}
pub(crate) fn editor_document(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    file: gio::File,
    remote: Option<std::sync::Arc<nexuterm::remote_edit::RemoteDocument>>,
    recovered: Option<nexuterm::recovery::Draft>,
    document_id: Option<String>,
) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.add_css_class("nexuterm-section");
    root.add_css_class("modern-editor");
    let save = gtk::Button::with_label("Guardar");
    save.set_sensitive(false);
    let status = gtk::Label::new(Some("Cargando texto UTF-8 (máximo 8 MiB)…"));
    status.set_xalign(0.0);
    status.set_wrap(true);
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    toolbar.add_css_class("editor-toolbar");
    let badge = gtk::Label::new(Some("EDITOR / UTF-8"));
    badge.set_hexpand(true);
    badge.set_xalign(0.0);
    badge.add_css_class("heading");
    if let Some(remote) = &remote {
        badge.set_text(&format!("SSH / {}", remote.host()));
        badge.set_tooltip_text(Some(&format!(
            "{} · copia temporal: {}",
            remote.remote_path().display(),
            remote.local_path().display()
        )));
    }
    toolbar.append(&badge);
    toolbar.append(&save);
    let save_copy = gtk::Button::with_label("Guardar copia…");
    save_copy.set_sensitive(false);
    toolbar.append(&save_copy);
    let source = recovered.as_ref().map(|draft|draft.source.clone()).unwrap_or_else(|| {
        remote.as_ref().map(|r|r.recovery_source()).unwrap_or_else(|| file.path().map(|path|nexuterm::recovery::Source::Local { path, etag: None })
            .unwrap_or(nexuterm::recovery::Source::Detached { title: "Documento sin ruta local".into() }))
    });
    if recovered.is_some() {
        badge.set_text("BORRADOR RECUPERADO");
        badge.set_tooltip_text(Some(&source.title()));
        let compare = gtk::Button::with_label("Comparar original");
        toolbar.append(&compare);
        let s = Rc::downgrade(state); let w = window.downgrade(); let origin = source.clone();
        let b = root.downgrade();
        compare.connect_clicked(move |_| {
            if let (Some(state), Some(window), Some(root)) = (s.upgrade(), w.upgrade(), b.upgrade()) {
                crate::recovery_ui::compare(&state, &window, &origin, &root);
            }
        });
    }
    root.append(&toolbar);
    let bind_remote = if recovered.is_some() && matches!(&source, nexuterm::recovery::Source::Remote { .. }) {
        let button = gtk::Button::with_label("Vincular a SSH");
        button.set_halign(gtk::Align::Start);
        if !matches!(&source, nexuterm::recovery::Source::Remote { fingerprint: Some(_), .. }) {
            button.set_sensitive(false);
            button.set_tooltip_text(Some("Este borrador anterior no contiene la huella original. Compara o guarda una copia local."));
        }
        root.append(&button);
        Some(button)
    } else { None };
    root.append(&status);
    let text = gtk::TextView::builder()
        .monospace(true)
        .editable(false)
        .left_margin(12)
        .right_margin(12)
        .top_margin(12)
        .build();
    let buffer = text.buffer();
    let recovery = crate::recovery_ui::Tracker::attach(state, &root, &buffer, &status, source.clone(), recovered.as_ref().map(|d|d.id.clone()).or(document_id));
    recovery.set_base(recovered.as_ref().and_then(|d|d.base_text.clone()));
    crate::recovery_ui::bind_copy(&save_copy, window, &buffer, &status, source.basename());
    crate::syntax::bind(&buffer, file.basename().unwrap_or_default());
    state
        .documents
        .borrow_mut()
        .retain(|b| b.upgrade().is_some());
    state.documents.borrow_mut().push(buffer.downgrade());
    let scroll = gtk::ScrolledWindow::builder()
        .child(&text)
        .vexpand(true)
        .hexpand(true)
        .build();
    let gutter = gtk::DrawingArea::new();
    gutter.set_content_width(64);
    gutter.add_css_class("editor-gutter");
    gutter.set_visible(state.config.borrow().preferences.editor_line_numbers);
    let view = text.downgrade();
    gutter.set_draw_func(move |area, cr, width, height| {
        let Some(view) = view.upgrade() else {
            return;
        };
        let visible = view.visible_rect();
        let (mut iter, _) = view.line_at_y(visible.y());
        let color = area.color();
        cr.set_source_rgba(
            color.red().into(),
            color.green().into(),
            color.blue().into(),
            0.6,
        );
        cr.select_font_face(
            "monospace",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Normal,
        );
        cr.set_font_size(13.0);
        loop {
            let rect = view.iter_location(&iter);
            let y = rect.y() - visible.y();
            if y > height {
                break;
            }
            let number = (iter.line() + 1).to_string();
            if let Ok(extent) = cr.text_extents(&number) {
                cr.move_to(
                    f64::from(width) - extent.width() - 12.0,
                    f64::from(y) + 13.0,
                );
                let _ = cr.show_text(&number);
            }
            let previous_line = iter.line();
            iter.forward_line();
            if iter.line() == previous_line {
                break;
            }
        }
    });
    let g = gutter.downgrade();
    buffer.connect_changed(move |_| {
        if let Some(g) = g.upgrade() {
            g.queue_draw();
        }
    });
    let g = gutter.downgrade();
    scroll.vadjustment().connect_value_changed(move |_| {
        if let Some(g) = g.upgrade() {
            g.queue_draw();
        }
    });
    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.append(&gutter);
    body.append(&scroll);
    root.append(&body);
    let etag = Rc::new(RefCell::new(None::<String>));
    let busy = Rc::new(Cell::new(false));
    let loaded = Rc::new(Cell::new(false));
    state.writes.borrow_mut().retain(|b| b.strong_count() > 0);
    state.writes.borrow_mut().push(Rc::downgrade(&busy));
    let title = file
        .basename()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Editor".into());
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let label = gtk::Label::new(Some(&title));
    header.append(&label);
    let close = gtk::Button::from_icon_name("window-close-symbolic");
    header.append(&close);
    let page = state.notebook.append(&root);
    page.set_title(&if let Some(remote) = &remote {
        format!("{title} @ {}", remote.host())
    } else {
        title.clone()
    });
    if let Some(remote) = &remote {
        page.set_tooltip(&remote.remote_path().to_string_lossy());
    }
    if recovered.is_some() { page.set_title(&format!("{title} · recuperado")); page.set_tooltip(&source.title()); }
    state.notebook.set_selected_page(&page);
    let target = page.downgrade();
    let current_file = Rc::new(RefCell::new(file.clone()));
    let current_remote = Rc::new(RefCell::new(remote));
    if let Some(bind) = bind_remote {
        let s = Rc::downgrade(state);
        let remote = current_remote.clone(); let file = current_file.clone(); let tag = etag.clone();
        let b = busy.clone(); let l = loaded.clone(); let st = status.downgrade(); let save = save.downgrade();
        let recovery = recovery.clone(); let badge = badge.downgrade();
        bind.connect_clicked(move |button| {
            if b.get() || !l.get() { return; }
            let Some(state) = s.upgrade() else { return; };
            let origin = recovery.source();
            let nexuterm::recovery::Source::Remote { profile_id, destination, .. } = &origin else { return; };
            let Some(connection) = state.recovery_connection(profile_id, destination) else {
                if let Some(status) = st.upgrade() { status.set_text("Conecta primero una terminal SSH al mismo perfil y destino. No se ha iniciado otra conexión."); }
                return;
            };
            b.set(true); button.set_sensitive(false);
            if let Some(save) = save.upgrade() { save.set_sensitive(false); }
            if let Some(status) = st.upgrade() { status.set_text("Comprobando la versión original en la conexión SSH…"); }
            let base=recovery.base();
            let (origin, remote, file, tag, busy, status, save, recovery, badge, button) = (origin.clone(), remote.clone(), file.clone(), tag.clone(), b.clone(), st.clone(), save.clone(), recovery.clone(), badge.clone(), button.downgrade());
            glib::spawn_future_local(async move {
                let result = async {
                    let document = nexuterm::runtime::runtime().spawn(async move { nexuterm::remote_edit::RemoteDocument::recovery_preview(connection, &origin,base).await }).await.map_err(|e|e.to_string())??;
                    let local = gio::File::for_path(document.local_path());
                    let info = local.query_info_future("etag::value", gio::FileQueryInfoFlags::NONE, PRIORITY).await.map_err(|e|e.to_string())?;
                    let etag = info.etag().map(String::from).ok_or("La copia local no ofrece una versión segura de guardado")?;
                    recovery.set_source(document.recovery_source());
                    recovery.flush().await?;
                    Ok::<_,String>((document, local, etag))
                }.await;
                match result {
                    Ok((document, local, etag)) => {
                        if let Some(badge) = badge.upgrade() { badge.set_text(&format!("SSH / {} · recuperado", document.host())); }
                        let review=document.requires_review();
                        *remote.borrow_mut() = Some(document); *file.borrow_mut() = local; *tag.borrow_mut() = Some(etag);
                        if let Some(save) = save.upgrade() { save.set_sensitive(!review); }
                        if let Some(status) = status.upgrade() { status.set_text(if review{"Conflicto: el original remoto cambió. Pulsa Comparar versiones y revisa el resultado antes de guardar."}else{"Original vinculado y comprobado. Guardado atómico con permisos POSIX; ACL y atributos extendidos no garantizados."}); }
                        if let Some(button) = button.upgrade() { button.set_label("Revalidar SSH"); button.set_sensitive(true); }
                    }
                    Err(error) => {
                        if let Some(status) = status.upgrade() { status.set_text(&format!("No se vinculó el borrador: {error}")); }
                        if let Some(save) = save.upgrade() { save.set_sensitive(remote.borrow().as_ref().is_some_and(|d|!d.requires_review()) && tag.borrow().is_some()); }
                        if let Some(button) = button.upgrade() { button.set_sensitive(true); }
                    }
                }
                busy.set(false);
            });
        });
    }
    crate::editor_compare::install(&toolbar,window,&buffer,&status,&save,loaded.clone(),busy.clone(),etag.clone(),current_file.clone(),current_remote.clone(),recovery.clone());
    let close_button = close.clone();
    let close_handler = Rc::new(RefCell::new(None));
    *close_handler.borrow_mut() = Some(state.notebook.connect_close_page(move |_, page| {
        if target.upgrade().as_ref() == Some(page) {
            close_button.emit_clicked();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    }));
    let tab = root.downgrade();
    let nb = state.notebook.downgrade();
    let buf = buffer.downgrade();
    let w = window.downgrade();
    let b = busy.clone();
    let st = status.downgrade();
    let close_recovery = recovery.clone();
    let handler = close_handler.clone();
    close.connect_clicked(move |_| {
        if b.get() {
            if let Some(s) = st.upgrade() {
                s.set_text("Espera a que finalice la escritura antes de cerrar.");
            }
            return;
        }
        if let Some(buffer) = buf.upgrade()
            && buffer.is_modified()
        {
            let Some(window) = w.upgrade() else {
                return;
            };
            let dialog = gtk::AlertDialog::builder()
                .modal(true)
                .message("¿Descartar los cambios sin guardar?")
                .buttons(["Cancelar", "Descartar"])
                .cancel_button(0)
                .default_button(0)
                .build();
            let nb = nb.clone();
            let tab = tab.clone(); let recovery = close_recovery.clone(); let handler = handler.clone(); let status = st.clone();
            glib::spawn_future_local(async move {
                if dialog.choose_future(Some(&window)).await == Ok(1) {
                    match recovery.discard().await {
                        Ok(()) => remove_page(&nb, &tab, &handler),
                        Err(error) => if let Some(status) = status.upgrade() { status.set_text(&format!("No se pudo descartar el borrador: {error}")); },
                    }
                }
            });
            return;
        }
        let (nb, tab, recovery, handler, status) = (nb.clone(), tab.clone(), close_recovery.clone(), handler.clone(), st.clone());
        glib::spawn_future_local(async move {
            match recovery.discard().await {
                Ok(()) => remove_page(&nb, &tab, &handler),
                Err(error) => if let Some(status) = status.upgrade() { status.set_text(&format!("No se pudo cerrar el documento: {error}")); },
            }
        });
    });
    let file_save = current_file.clone();
    let remote_save = current_remote.clone();
    let buf = buffer.downgrade();
    let tag = etag.clone();
    let b = busy.clone();
    let l = loaded.clone();
    let st = status.downgrade();
    let save_recovery = recovery.clone();
    save.connect_clicked(move |button| {
        if b.get() || !l.get() || tag.borrow().is_none() || remote_save.borrow().as_ref().is_some_and(|d|d.requires_review()) {
            return;
        }
        let Some(buffer) = buf.upgrade() else {
            return;
        };
        let contents = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
        if contents.len() > EDIT_LIMIT {
            if let Some(status) = st.upgrade() {
                status.set_text("El documento supera el límite de 8 MiB");
            }
            return;
        }
        b.set(true);
        button.set_sensitive(false);
        let file = file_save.borrow().clone();
        let expected = tag.borrow().clone();
        let tag = tag.clone();
        let busy = b.clone();
        let status = st.clone();
        let button = button.downgrade();
        let buffer = buffer.downgrade();
        let remote = remote_save.borrow().clone();
        let recovery = save_recovery.clone();
        if let Some(status) = status.upgrade() {
            status.set_text(if remote.is_some() {
                "Guardando mediante temporal SFTP y reemplazo atómico…"
            } else {
                "Guardando…"
            });
        }
        glib::spawn_future_local(async move {
            let result = async {
                let (_, new_tag) = file
                    .replace_contents_future(
                        contents.clone().into_bytes(),
                        expected.as_deref(),
                        false,
                        gio::FileCreateFlags::NONE,
                    )
                    .await
                    .map_err(|(_, e)| e.to_string())?;
                *tag.borrow_mut() = new_tag.map(String::from);
                if let Some(remote) = remote.as_ref() {
                    let document = remote.clone();
                    let text = contents.clone();
                    nexuterm::runtime::runtime()
                        .spawn(async move { document.save(text).await })
                        .await
                        .map_err(|e| e.to_string())??;
                }
                Ok::<(), String>(())
            }
            .await;
            match result {
                Ok(()) => {
                    if let Some(buffer) = buffer.upgrade()
                        && buffer
                            .text(&buffer.start_iter(), &buffer.end_iter(), true)
                            .as_str()
                            == contents
                    {
                        buffer.set_modified(false);
                    }
                    recovery.set_base(Some(contents.clone()));
                    if let Some(remote) = &remote { recovery.set_source(remote.recovery_source()); }
                    let recovery_result = recovery.saved(tag.borrow().clone()).await;
                    if let Some(status) = status.upgrade() {
                        status.set_text(if remote.is_some() {
                            "Guardado atómico SFTP y verificado en el servidor."
                        } else {
                            "Guardado con comprobación de versión."
                        });
                    }
                    if let Err(error) = recovery_result {
                        if let Some(status) = status.upgrade() { status.set_text(&format!("Original guardado; no se pudo retirar el borrador: {error}")); }
                    }
                    #[cfg(debug_assertions)]
                    if remote.is_some()
                        && std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR").is_some()
                    {
                        eprintln!("NEXUTERM_SMOKE_REMOTE_SAVED_OK");
                    }
                }
                Err(error) => {
                    if let Some(status) = status.upgrade() {
                        let copy = remote
                            .as_ref()
                            .map(|r| format!(" Copia local: {}", r.local_path().display()))
                            .unwrap_or_default();
                        status.set_text(&format!("No se confirmó el guardado: {error}.{copy}"));
                    }
                }
            }
            busy.set(false);
            if let Some(button) = button.upgrade() {
                button.set_sensitive(tag.borrow().is_some() && !remote.as_ref().is_some_and(|d|d.requires_review()));
            }
        });
    });
    let keys = gtk::EventControllerKey::new();
    let save_key = save.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if (key == gtk::gdk::Key::s || key == gtk::gdk::Key::S)
            && modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            && !modifiers
                .intersects(gtk::gdk::ModifierType::SHIFT_MASK | gtk::gdk::ModifierType::ALT_MASK)
        {
            if let Some(save) = save_key.upgrade()
                && save.is_sensitive()
            {
                save.emit_clicked();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    #[cfg(debug_assertions)]
    let save_keys = keys.downgrade();
    root.add_controller(keys);
    let buf = buffer.downgrade();
    let view = text.downgrade();
    let save = save.downgrade();
    let status = status.downgrade();
    let copy = save_copy.downgrade();
    let recovery_loaded = recovery.clone();
    let is_remote_origin = matches!(&source, nexuterm::recovery::Source::Remote { .. });
    #[cfg(debug_assertions)]
    let numbers = gutter.downgrade();
    #[cfg(debug_assertions)]
    let smoke_window = window.downgrade();
    glib::spawn_future_local(async move {
        let mut notice = None;
        let is_recovered = recovered.is_some();
        let result = if let Some(draft) = recovered {
            let tag = match &draft.source {
                nexuterm::recovery::Source::Local { etag: Some(expected), .. } => {
                    match file.query_info_future("etag::value,standard::type", gio::FileQueryInfoFlags::NONE, PRIORITY).await {
                        Ok(info) if info.file_type() == gio::FileType::Regular && info.etag().as_deref() == Some(expected.as_str()) => Some(expected.clone()),
                        _ => { notice = Some("El original cambió o no está disponible. Compara las versiones o guarda una copia con otro nombre."); None }
                    }
                }
                _ => { notice = Some("Borrador recuperado sin conexión al original. Puedes compararlo o guardar una copia local; no se subirá automáticamente."); None }
            };
            Ok((draft.text, tag))
        } else { read_text(&file).await };
        if let (Some(buffer), Some(view), Some(save), Some(status)) = (
            buf.upgrade(),
            view.upgrade(),
            save.upgrade(),
            status.upgrade(),
        ) {
            match result {
                Ok((contents, tag)) => {
                    if !is_recovered {recovery_loaded.set_base(Some(contents.clone()));}
                    buffer.set_text(&contents);
                    buffer.set_modified(is_recovered);
                    recovery_loaded.loaded(tag.clone());
                    if let Some(copy) = copy.upgrade() { copy.set_sensitive(true); }
                    #[cfg(debug_assertions)]
                    if std::env::var_os("NEXUTERM_SMOKE_LOCAL_EXPLORER").is_some() {
                        assert_eq!(contents, "primera línea\nsegunda línea\n");
                        assert_eq!(buffer.line_count(), 3);
                        assert_eq!(
                            numbers.upgrade().unwrap().is_visible(),
                            std::env::var_os("NEXUTERM_SMOKE_NO_NUMBERS").is_none()
                        );
                        eprintln!("NEXUTERM_SMOKE_LOCAL_EDITOR_OK");
                    }
                    view.set_editable(true);
                    save.set_sensitive(tag.is_some());
                    *etag.borrow_mut() = tag;
                    loaded.set(true);
                    status.set_text(if is_remote_origin && !is_recovered {
                        "UTF-8 · SFTP atómico con propietario/grupo/permisos POSIX; ACL y atributos extendidos no garantizados."
                    } else if etag.borrow().is_some() { "UTF-8 · guardado con comprobación de versión" }
                    else { "El servidor no aporta versión/etag: edición disponible, guardado deshabilitado para evitar sobrescrituras." });
                    if let Some(notice) = notice { status.set_text(notice); }
                    #[cfg(debug_assertions)]
                    if let Some(window) = smoke_window.upgrade() {
                        crate::recovery_ui::smoke_loaded(&recovery_loaded, &buffer, &save, &window);
                    }
                    #[cfg(debug_assertions)]
                    if std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR").is_some() && std::env::var_os("NEXUTERM_SMOKE_EDITOR_COMPARE").is_none() {
                        assert_eq!(contents, "def value():\n    return 1\n");
                        buffer.set_text("def value():\n    return 42  # editado desde NexuTerm\n");
                        let keys = save_keys.upgrade().unwrap();
                        assert!(keys.emit_by_name::<bool>(
                            "key-pressed",
                            &[
                                &gtk::gdk::Key::s,
                                &0_u32,
                                &gtk::gdk::ModifierType::CONTROL_MASK
                            ]
                        ));
                    }
                }
                Err(e) => status.set_text(&format!("No se pudo abrir: {e}")),
            }
        }
    });
    root
}

fn remove_page(nb: &glib::WeakRef<adw::TabView>, tab: &glib::WeakRef<gtk::Box>, handler: &RefCell<Option<glib::SignalHandlerId>>) {
    if let (Some(n), Some(t)) = (nb.upgrade(), tab.upgrade())
        && let Some(i) = n
            .pages()
            .iter::<adw::TabPage>()
            .filter_map(Result::ok)
            .find(|p| p.child() == t)
    {
        if let Some(handler) = handler.borrow_mut().take() { n.disconnect(handler); }
        n.close_page_finish(&i, true);
    }
}

pub(crate) async fn read_text(file: &gio::File) -> Result<(String, Option<String>), String> {
    let before = file
        .query_info_future("etag::value", gio::FileQueryInfoFlags::NONE, PRIORITY)
        .await
        .map_err(|e| e.to_string())?;
    let stream = file
        .read_future(PRIORITY)
        .await
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    let read_result = async {
        loop {
            let chunk = stream
                .read_bytes_future(65536, PRIORITY)
                .await
                .map_err(|e| e.to_string())?;
            if chunk.is_empty() {
                break;
            }
            if bytes.len() + chunk.len() > EDIT_LIMIT {
                return Err("Archivo mayor de 8 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok::<(), String>(())
    }
    .await;
    let closed = stream.close_future(PRIORITY).await;
    read_result?;
    closed.map_err(|e| e.to_string())?;
    let after = file
        .query_info_future("etag::value", gio::FileQueryInfoFlags::NONE, PRIORITY)
        .await
        .map_err(|e| e.to_string())?;
    let before = before.etag();
    let after = after.etag();
    if before != after {
        return Err("El archivo cambió durante la lectura; vuelve a abrirlo".into());
    }
    if bytes.contains(&0) {
        return Err("Archivo binario; no se abre como texto".into());
    }
    let contents = String::from_utf8(bytes)
        .map_err(|_| "Codificación no UTF-8: conviértela antes de editar")?;
    Ok((contents, after.map(|s| s.into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_utf8_and_conflict_protection() {
        let dir = std::env::temp_dir().join(format!("nexuterm-editor-test-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("document.txt");
        std::fs::write(&path, "áé Linux\r\n").unwrap();
        let file = gio::File::for_path(&path);
        let context = glib::MainContext::new();
        context.block_on(async {
            let (text, etag) = read_text(&file).await.unwrap();
            assert_eq!(text, "áé Linux\r\n");
            assert!(etag.is_some());
            std::thread::sleep(std::time::Duration::from_millis(5));
            std::fs::write(&path, "contenido cambiado fuera del editor").unwrap();
            let result = file
                .replace_contents_future(
                    b"no sobrescribir".to_vec(),
                    etag.as_deref(),
                    false,
                    gio::FileCreateFlags::NONE,
                )
                .await;
            assert!(result.is_err());
            assert_eq!(
                std::fs::read_to_string(&path).unwrap(),
                "contenido cambiado fuera del editor"
            );
            std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
            assert!(read_text(&file).await.is_err());
            std::fs::write(&path, vec![b'a'; EDIT_LIMIT + 1]).unwrap();
            assert!(read_text(&file).await.unwrap_err().contains("8 MiB"));
        });
        std::fs::remove_dir_all(dir).unwrap();
    }
}
