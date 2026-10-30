//! Non-blocking draft capture and explicit recovery/discard controls.
use crate::ui::State;
use adw::prelude::*;
use gtk::{gio, glib};
use nexuterm::recovery::{self, Draft, Journal, Source};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

/// Capture at most the editor limit, even if a large paste exceeds it.
pub fn snapshot(buffer: &gtk::TextBuffer) -> Result<String, String> {
    if buffer.char_count() > recovery::TEXT_LIMIT as i32 {
        return Err("El documento supera el límite de recuperación de 8 MiB".into());
    }
    let mut text = String::new();
    let mut start = buffer.start_iter();
    while !start.is_end() {
        let mut end = start;
        end.forward_chars(16 * 1024);
        let chunk = buffer.text(&start, &end, true);
        if text.len() + chunk.len() > recovery::TEXT_LIMIT {
            return Err("El documento supera el límite de recuperación de 8 MiB".into());
        }
        text.push_str(&chunk);
        start = end;
    }
    Ok(text)
}

pub struct Tracker {
    pub id: String,
    pub root: glib::WeakRef<gtk::Box>,
    buffer: glib::WeakRef<gtk::TextBuffer>,
    pending_view: Rc<Cell<Option<nexuterm::workspaces::EditorView>>>,
    status: glib::WeakRef<gtk::Label>,
    source: RefCell<Source>,
    base: RefCell<Option<String>>,
    journal: Result<Arc<Journal>, String>,
    revision: Cell<u64>,
    persisted: Cell<u64>,
    enabled: Cell<bool>,
    running: Cell<bool>,
    retired: Cell<bool>,
}
impl Tracker {
    pub fn attach(
        state: &Rc<State>,
        root: &gtk::Box,
        buffer: &gtk::TextBuffer,
        status: &gtk::Label,
        source: Source,
        id: Option<String>,
    ) -> Rc<Self> {
        let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let journal = recovery::directory()
            .and_then(|directory| Journal::new(&directory, &id))
            .map(Arc::new);
        let tracker = Rc::new(Self {
            id,
            root: root.downgrade(),
            buffer: buffer.downgrade(),
            pending_view: Rc::new(Cell::new(None)),
            status: status.downgrade(),
            source: RefCell::new(source),
            base: RefCell::new(None),
            journal,
            revision: Cell::new(0),
            persisted: Cell::new(0),
            enabled: Cell::new(false),
            running: Cell::new(false),
            retired: Cell::new(false),
        });
        state
            .recoveries
            .borrow_mut()
            .retain(|weak| weak.strong_count() > 0);
        state.recoveries.borrow_mut().push(Rc::downgrade(&tracker));
        let t = Rc::downgrade(&tracker);
        buffer.connect_changed(move |_| {
            if let Some(t) = t.upgrade() {
                t.advance();
            }
        });
        let t = Rc::downgrade(&tracker);
        glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
            let Some(t) = t.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if t.retired.get() || t.root.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            if t.needs_save() && !t.running.get() {
                glib::spawn_future_local(async move {
                    if let Err(error) = t.persist_once().await {
                        t.error(&error);
                    }
                });
            }
            glib::ControlFlow::Continue
        });
        tracker
    }
    fn advance(&self) -> u64 {
        let revision = self.revision.get() + 1;
        self.revision.set(revision);
        revision
    }
    pub fn workspace_view(&self) -> Option<nexuterm::workspaces::EditorView> {
        self.pending_view.get().or_else(|| {
            self.root
                .upgrade()
                .and_then(|root| crate::widget_state::editor(root.upcast_ref()))
        })
    }
    pub fn restore_workspace_view(&self, view: nexuterm::workspaces::EditorView) {
        self.pending_view.set(Some(view));
        if self.enabled.get() {
            if let Some(root) = self.root.upgrade() {
                crate::widget_state::restore_editor(root.upcast_ref(), self.pending_view.clone());
            }
        }
    }
    pub fn loaded(&self, etag: Option<String>) {
        if let Source::Local { etag: current, .. } = &mut *self.source.borrow_mut() {
            *current = etag;
        }
        self.enabled.set(true);
        if let Some(root)=self.root.upgrade() {crate::widget_state::restore_editor(root.upcast_ref(),self.pending_view.clone());}
        self.advance();
    }
    fn needs_save(&self) -> bool {
        self.enabled.get()
            && !self.retired.get()
            && self.persisted.get() != self.revision.get()
            && self
                .buffer
                .upgrade()
                .is_some_and(|buffer| buffer.is_modified())
    }
    fn error(&self, message: &str) {
        if let Some(status) = self.status.upgrade() {
            status.set_text(&format!(
                "No se pudo conservar el borrador de recuperación: {message}"
            ));
        }
    }
    async fn persist_once(&self) -> Result<(), String> {
        if !self.needs_save() || self.running.get() {
            return Ok(());
        }
        let journal = self.journal.as_ref().map_err(Clone::clone)?.clone();
        let Some(buffer) = self.buffer.upgrade() else {
            return Ok(());
        };
        let revision = self.revision.get();
        let draft = Draft {
            version: 1,
            id: self.id.clone(),
            source: self.source.borrow().clone(),
            text: snapshot(&buffer)?,
            base_text:self.base.borrow().clone(),
        };
        draft.validate()?;
        // New keystrokes may arrive during I/O: publish this bounded snapshot,
        // then capture the newer revision next time. Save/discard invalidate it.
        journal.advance(revision);
        self.running.set(true);
        let result = nexuterm::runtime::runtime()
            .spawn_blocking(move || journal.write(revision, &draft))
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r);
        self.running.set(false);
        if result? {
            self.persisted.set(revision);
        }
        Ok(())
    }
    pub async fn flush(&self) -> Result<(), String> {
        while self.running.get() {
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        while self.needs_save() {
            self.persist_once().await?;
        }
        Ok(())
    }
    pub fn modified(&self) -> bool {
        self.buffer
            .upgrade()
            .is_some_and(|buffer| buffer.is_modified())
    }
    pub fn base(&self)->Option<String>{self.base.borrow().clone()}
    pub fn set_base(&self,base:Option<String>){*self.base.borrow_mut()=base;self.advance();}
    pub fn source(&self) -> Source {
        self.source.borrow().clone()
    }
    pub fn set_source(&self, source: Source) {
        *self.source.borrow_mut() = source;
        self.advance();
    }
    pub async fn saved(&self, etag: Option<String>) -> Result<(), String> {
        if let Source::Local { etag: current, .. } = &mut *self.source.borrow_mut() {
            *current = etag;
        }
        let revision = self.advance();
        if let Ok(journal) = &self.journal {
            journal.advance(revision);
        }
        if self.buffer.upgrade().is_some_and(|b| b.is_modified()) {
            return Ok(());
        }
        let journal = self.journal.as_ref().map_err(Clone::clone)?.clone();
        nexuterm::runtime::runtime()
            .spawn_blocking(move || journal.clear(revision))
            .await
            .map_err(|e| e.to_string())??;
        self.persisted.set(revision);
        Ok(())
    }
    pub async fn discard(&self) -> Result<(), String> {
        self.retired.set(true);
        if let Ok(journal) = &self.journal {
            journal.retire();
            let journal = journal.clone();
            let revision = self.advance();
            journal.advance(revision);
            let result = nexuterm::runtime::runtime()
                .spawn_blocking(move || journal.clear(revision))
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r);
            if let Err(error) = result {
                if let Ok(journal) = &self.journal {
                    journal.resume();
                }
                self.retired.set(false);
                return Err(error);
            }
        }
        Ok(())
    }
}

/// Called only once a user has confirmed closing the application.
pub fn prepare_close(state: &Rc<State>, window: &gtk::ApplicationWindow) -> bool {
    if state.recovery_closing.get() {
        return true;
    }
    let trackers: Vec<_> = state
        .recoveries
        .borrow()
        .iter()
        .filter_map(std::rc::Weak::upgrade)
        .filter(|t| t.needs_save() || t.running.get())
        .collect();
    if trackers.is_empty() {
        return false;
    }
    state.recovery_closing.set(true);
    state
        .status
        .set_text("Conservando documentos antes de cerrar…");
    let state = state.clone();
    let window = window.downgrade();
    glib::spawn_future_local(async move {
        let mut result = Ok(());
        for tracker in trackers {
            if let Err(error) = tracker.flush().await {
                result = Err(error);
                break;
            }
        }
        state.recovery_closing.set(false);
        match result {
            Ok(()) => if let Some(window) = window.upgrade() { window.close(); },
            Err(error) => state.status.set_text(&format!("No se ha cerrado: no se pudo conservar un borrador. Guarda una copia o descarta el documento expresamente. {error}")),
        }
    });
    true
}

pub fn offer(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    request(state, window, true);
}
fn request(state: &Rc<State>, window: &gtk::ApplicationWindow, only_if_found: bool) {
    if let Some(manager) = state.recovery_manager.borrow().upgrade() {
        manager.present();
        return;
    }
    if state.recovery_scanning.replace(true) {
        return;
    }
    let state = Rc::downgrade(state);
    let window = window.downgrade();
    glib::spawn_future_local(async move {
        let result = scan().await;
        if let Some(state) = state.upgrade() {
            state.recovery_scanning.set(false);
            let Some(window) = window.upgrade() else {
                return;
            };
            match result {
                Ok((entries, invalid)) if !only_if_found || !entries.is_empty() || invalid > 0 => {
                    show_entries(&state, &window, entries, invalid)
                }
                Err(error) => state
                    .status
                    .set_text(&format!("No se pudieron consultar los borradores: {error}")),
                _ => {}
            }
        }
    });
}
async fn scan() -> Result<(Vec<recovery::Entry>, usize), String> {
    nexuterm::runtime::runtime()
        .spawn_blocking(|| recovery::directory().and_then(|dir| recovery::list(&dir)))
        .await
        .map_err(|e| e.to_string())?
}

pub fn show(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    request(state, window, false);
}

fn show_entries(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    entries: Vec<recovery::Entry>,
    invalid: usize,
) {
    if let Some(manager) = state.recovery_manager.borrow().upgrade() {
        manager.present();
        return;
    }
    let dialog = gtk::Window::builder()
        .title("Recuperar documentos")
        .transient_for(window)
        .destroy_with_parent(true)
        .default_width(720)
        .default_height(400)
        .build();
    *state.recovery_manager.borrow_mut() = dialog.downgrade();
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_margin_start(16);
    body.set_margin_end(16);
    body.set_margin_top(16);
    body.set_margin_bottom(16);
    let info = gtk::Label::new(Some(
        "Los borradores permanecen hasta guardar el original o descartarlos. Recuperar no modifica el original ni inicia conexiones.",
    ));
    info.set_wrap(true);
    body.append(&info);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&list)
        .build();
    body.append(&scroll);
    let status = gtk::Label::new(None);
    status.set_wrap(true);
    body.append(&status);
    if invalid > 0 {
        status.set_text(&format!("{invalid} borradores dañados o con formato desconocido se han conservado sin modificar."));
    }
    if entries.is_empty() {
        list.append(&gtk::Label::new(Some("No hay borradores recuperables.")));
    }
    for (index, entry) in entries.into_iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let title = gtk::Label::new(Some(&entry.source.title()));
        title.set_wrap(true);
        title.set_xalign(0.0);
        title.set_hexpand(true);
        row.append(&title);
        let recover = gtk::Button::with_label("Recuperar");
        row.append(&recover);
        let discard = gtk::Button::with_label("Descartar");
        row.append(&discard);
        list.append(&row);
        discard.set_sensitive(
            !state
                .recoveries
                .borrow()
                .iter()
                .filter_map(std::rc::Weak::upgrade)
                .any(|t| t.id == entry.id),
        );
        #[cfg(debug_assertions)]
        if index == 0
            && let Ok(mode) = std::env::var("NEXUTERM_SMOKE_RECOVERY")
        {
            if mode.starts_with("restore-") {
                let recover = recover.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
                    recover.emit_clicked()
                });
            } else if mode == "discard" {
                let discard = discard.clone();
                let dialog = dialog.clone();
                glib::spawn_future_local(async move {
                    glib::timeout_future(std::time::Duration::from_millis(200)).await;
                    discard.emit_clicked();
                    glib::timeout_future(std::time::Duration::from_millis(100)).await;
                    confirm_button("Descartar", Some(&dialog));
                    glib::timeout_future(std::time::Duration::from_millis(200)).await;
                    assert!(scan().await.unwrap().0.is_empty());
                    eprintln!("NEXUTERM_SMOKE_RECOVERY_DISCARD_OK");
                });
            }
        }
        #[cfg(not(debug_assertions))]
        let _ = index;
        let s = Rc::downgrade(state);
        let win = window.downgrade();
        let message = status.downgrade();
        let r = row.downgrade();
        let id = entry.id.clone();
        recover.connect_clicked(move |button| {
            let Some(state) = s.upgrade() else {
                return;
            };
            if let Some(tracker) = state
                .recoveries
                .borrow()
                .iter()
                .filter_map(std::rc::Weak::upgrade)
                .find(|t| t.id == id)
                && let Some(root) = tracker.root.upgrade()
            {
                let page = state.notebook.page(&root);
                state.notebook.set_selected_page(&page);
                return;
            }
            button.set_sensitive(false);
            let (win, message, row, button, id) = (
                win.clone(),
                message.clone(),
                r.clone(),
                button.downgrade(),
                id.clone(),
            );
            glib::spawn_future_local(async move {
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        recovery::directory()
                            .and_then(|dir| recovery::read(&dir.join(format!("{id}.json"))))
                    })
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r);
                match result {
                    Ok(draft) => {
                        if let Some(window) = win.upgrade() {
                            crate::files::recovered_editor(&state, &window, draft);
                            if let Some(row) = row.upgrade() {
                                row.set_visible(false);
                            }
                        }
                    }
                    Err(error) => {
                        if let Some(status) = message.upgrade() {
                            status.set_text(&error);
                        }
                    }
                }
                if let Some(button) = button.upgrade() {
                    button.set_sensitive(true);
                }
            });
        });
        let id = entry.id;
        let message = status.downgrade();
        let r = row.downgrade();
        let d = dialog.downgrade();
        discard.connect_clicked(move |_| {
            let confirm = gtk::AlertDialog::builder()
                .modal(true)
                .message("¿Descartar definitivamente este borrador?")
                .buttons(["Cancelar", "Descartar"])
                .cancel_button(0)
                .default_button(0)
                .build();
            let (id, status, row, dialog) = (id.clone(), message.clone(), r.clone(), d.clone());
            glib::spawn_future_local(async move {
                let Some(dialog) = dialog.upgrade() else {
                    return;
                };
                if confirm.choose_future(Some(&dialog)).await != Ok(1) {
                    return;
                }
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        let journal = Journal::new(&recovery::directory()?, &id)?;
                        journal.retire();
                        journal.clear(0)
                    })
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r);
                match result {
                    Ok(()) => {
                        if let Some(row) = row.upgrade() {
                            row.set_visible(false);
                        }
                    }
                    Err(error) => {
                        if let Some(status) = status.upgrade() {
                            status.set_text(&error);
                        }
                    }
                }
            });
        });
    }
    dialog.set_child(Some(&body));
    dialog.present();
}

pub fn bind_copy(
    button: &gtk::Button,
    window: &gtk::ApplicationWindow,
    buffer: &gtk::TextBuffer,
    status: &gtk::Label,
    basename: std::path::PathBuf,
) {
    let window = window.downgrade();
    let buffer = buffer.downgrade();
    let status = status.downgrade();
    button.connect_clicked(move |button| {
        let (Some(window), Some(buffer)) = (window.upgrade(), buffer.upgrade()) else { return; };
        let dialog = gtk::FileDialog::builder().title("Guardar copia local con un nombre nuevo").initial_name(basename.to_string_lossy()).build();
        let status = status.clone(); let button = button.downgrade();
        if let Some(button) = button.upgrade() { button.set_sensitive(false); }
        glib::spawn_future_local(async move {
            if let Ok(file) = dialog.save_future(Some(&window)).await {
                let result = async {
                    let text = snapshot(&buffer)?;
                    let path = file.path().ok_or("Elige un destino local para la copia de recuperación")?;
                    nexuterm::runtime::runtime().spawn_blocking(move ||recovery::export_copy(&path, &text)).await.map_err(|e|e.to_string())?
                }.await;
                if let Some(status) = status.upgrade() {
                    match result {
                        Ok(()) => status.set_text("Copia local guardada. El original no se ha modificado; el borrador sigue disponible hasta guardarlo o descartarlo."),
                        Err(error) => status.set_text(&format!("No se guardó la copia. Elige un nombre que no exista. {error}")),
                    }
                }
            }
            if let Some(button) = button.upgrade() { button.set_sensitive(true); }
        });
    });
}

fn text_view(widget: &gtk::Widget) -> Option<gtk::TextView> {
    if let Some(view) = widget.downcast_ref::<gtk::TextView>() {
        return Some(view.clone());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(view) = text_view(&current) {
            return Some(view);
        }
        child = current.next_sibling();
    }
    None
}
pub fn compare(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    source: &Source,
    root: &gtk::Box,
) {
    let Some(view) = text_view(root.upcast_ref()) else {
        return;
    };
    let buffer = view.buffer();
    let draft = match snapshot(&buffer) {
        Ok(text) => text,
        Err(error) => {
            state.status.set_text(&error);
            return;
        }
    };
    let source = source.clone();
    let window = window.downgrade();
    let state = state.clone();
    glib::spawn_future_local(async move {
        let result = match &source {
            Source::Local { path, .. } => crate::files::read_text(&gio::File::for_path(path))
                .await
                .map(|v| v.0),
            Source::Remote {
                profile_id,
                destination,
                path,
                ..
            } => {
                if let Some(connection) = state.recovery_connection(profile_id, destination) {
                    let path = path.clone();
                    match nexuterm::runtime::runtime()
                        .spawn(async move {
                            nexuterm::remote_edit::RemoteDocument::open(connection, path).await
                        })
                        .await
                    {
                        Ok(Ok(document)) => {
                            crate::files::read_text(&gio::File::for_path(document.local_path()))
                                .await
                                .map(|v| v.0)
                        }
                        Ok(Err(error)) => Err(error),
                        Err(error) => Err(error.to_string()),
                    }
                } else {
                    Err("Conecta primero el mismo perfil SSH al destino original; recuperar no inicia una conexión.".into())
                }
            }
            Source::Detached { .. } => {
                Err("Este borrador no tiene un original local o SSH disponible.".into())
            }
        };
        let Some(window) = window.upgrade() else {
            return;
        };
        match result {
            Ok(original) => {
                let dialog = gtk::Window::builder()
                    .title("Comparar original y borrador")
                    .transient_for(&window)
                    .destroy_with_parent(true)
                    .default_width(1000)
                    .default_height(650)
                    .build();
                let split = gtk::Paned::new(gtk::Orientation::Horizontal);
                for (label, text, first) in [
                    ("Original actual (solo lectura)", original, true),
                    ("Borrador (solo lectura)", draft, false),
                ] {
                    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
                    column.append(&gtk::Label::new(Some(label)));
                    let view = gtk::TextView::builder()
                        .monospace(true)
                        .editable(false)
                        .cursor_visible(false)
                        .build();
                    view.buffer().set_text(&text);
                    column.append(
                        &gtk::ScrolledWindow::builder()
                            .vexpand(true)
                            .hexpand(true)
                            .child(&view)
                            .build(),
                    );
                    if first {
                        split.set_start_child(Some(&column));
                    } else {
                        split.set_end_child(Some(&column));
                    }
                }
                split.set_position(500);
                dialog.set_child(Some(&split));
                dialog.present();
            }
            Err(error) => {
                gtk::AlertDialog::builder()
                    .message("No se pudo comparar el original")
                    .detail(error)
                    .build()
                    .show(Some(&window));
            }
        }
    });
}

#[cfg(debug_assertions)]
pub fn smoke_start(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    if matches!(
        std::env::var("NEXUTERM_SMOKE_RECOVERY").as_deref(),
        Ok("capture" | "close" | "fail-close")
    ) && let Some(path) = std::env::var_os("NEXUTERM_RECOVERY_FILE")
    {
        crate::files::editor(state, window, gio::File::for_path(path));
    }
}

#[cfg(debug_assertions)]
fn button_named(widget: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    if let Some(button) = widget.downcast_ref::<gtk::Button>()
        && button.label().as_deref() == Some(label)
    {
        return Some(button.clone());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(button) = button_named(&current, label) {
            return Some(button);
        }
        child = current.next_sibling();
    }
    None
}
#[cfg(debug_assertions)]
fn confirm_button(label: &str, exclude: Option<&gtk::Window>) {
    for widget in gtk::Window::list_toplevels() {
        if exclude.is_some_and(|w| w.upcast_ref::<gtk::Widget>() == &widget) {
            continue;
        }
        if let Some(button) = button_named(&widget, label) {
            button.emit_clicked();
            return;
        }
    }
    panic!("Missing confirmation button");
}

#[cfg(debug_assertions)]
pub fn smoke_loaded(
    tracker: &Rc<Tracker>,
    buffer: &gtk::TextBuffer,
    save: &gtk::Button,
    window: &gtk::ApplicationWindow,
) {
    let Ok(mode) = std::env::var("NEXUTERM_SMOKE_RECOVERY") else {
        return;
    };
    let (tracker, buffer, save, window) = (
        tracker.clone(),
        buffer.clone(),
        save.clone(),
        window.clone(),
    );
    glib::spawn_future_local(async move {
        const CONTENTS: &str = "Borrador recuperable con acentos áé\nÚltima edición sin guardar\n";
        let path = recovery::directory()
            .unwrap()
            .join(format!("{}.json", tracker.id));
        match mode.as_str() {
            "capture" => {
                buffer.set_text(CONTENTS);
                for _ in 0..40 {
                    glib::timeout_future(std::time::Duration::from_millis(50)).await;
                    if tracker.persisted.get() == tracker.revision.get() {
                        break;
                    }
                }
                assert_eq!(recovery::read(&path).unwrap().text, CONTENTS);
                eprintln!("NEXUTERM_RECOVERY_PID={}", std::process::id());
                eprintln!("NEXUTERM_SMOKE_RECOVERY_CAPTURE_OK");
            }
            "close" | "fail-close" => {
                if mode == "fail-close" {
                    let directory = recovery::directory().unwrap();
                    std::fs::create_dir_all(directory.parent().unwrap()).unwrap();
                    std::fs::write(&directory, "not a directory").unwrap();
                }
                buffer.set_text(CONTENTS);
                // Close before the periodic autosave has run: exercise close flushing.
                window.close();
                glib::timeout_future(std::time::Duration::from_millis(100)).await;
                confirm_button("Cerrar", None);
                if mode == "fail-close" {
                    glib::timeout_future(std::time::Duration::from_millis(300)).await;
                    assert!(window.is_visible());
                    assert!(buffer.is_modified());
                    eprintln!("NEXUTERM_SMOKE_RECOVERY_FAIL_CLOSE_OK");
                }
            }
            "restore-ssh-save" | "restore-ssh-conflict" => {
                const CHANGES: &str = "def value():\n    return 84  # recuperado\n";
                assert_eq!(snapshot(&buffer).unwrap(), CHANGES);
                assert!(!save.is_sensitive());
                let root = tracker.root.upgrade().unwrap();
                let bind = button_named(root.upcast_ref(), "Vincular a SSH").unwrap();
                assert!(bind.is_sensitive());
                bind.emit_clicked();
                for _ in 0..90 {
                    glib::timeout_future(std::time::Duration::from_millis(50)).await;
                    if bind.is_sensitive() {
                        break;
                    }
                }
                if mode == "restore-ssh-conflict" {
                    assert!(!save.is_sensitive());
                    assert_eq!(snapshot(&buffer).unwrap(), CHANGES);
                    assert!(buffer.is_modified());
                    let status = tracker.status.upgrade().unwrap();
                    assert!(
                        status.text().contains("Conflicto"),
                        "Expected baseline conflict, got {}",
                        status.text()
                    );
                    eprintln!("NEXUTERM_SMOKE_RECOVERY_REBIND_CONFLICT_OK");
                    crate::editor_compare::smoke_recovered(root.upcast_ref(),&buffer,&tracker).await;
                } else {
                    assert!(
                        save.is_sensitive(),
                        "Binding failed: {}",
                        tracker.status.upgrade().unwrap().text()
                    );
                    assert_eq!(snapshot(&buffer).unwrap(), CHANGES);
                    save.emit_clicked();
                    for _ in 0..50 {
                        glib::timeout_future(std::time::Duration::from_millis(50)).await;
                        if !buffer.is_modified() && !path.exists() {
                            break;
                        }
                    }
                    assert!(!buffer.is_modified());
                    assert!(!path.exists());
                    // Binding again must use the baseline updated after that save.
                    bind.emit_clicked();
                    for _ in 0..50 {
                        glib::timeout_future(std::time::Duration::from_millis(50)).await;
                        if bind.is_sensitive() {
                            break;
                        }
                    }
                    assert!(save.is_sensitive());
                    let original = match tracker.source() {
                        Source::Remote { fingerprint, .. } => fingerprint.unwrap(),
                        _ => panic!("Expected remote origin"),
                    };
                    buffer.insert_at_cursor("# siguiente edición\n");
                    tracker.flush().await.unwrap();
                    let stored = recovery::read(&path).unwrap();
                    assert!(
                        matches!(stored.source, Source::Remote { fingerprint: Some(value), .. } if value == original)
                    );
                    eprintln!("NEXUTERM_SMOKE_RECOVERY_REBIND_SAVED_OK");
                }
            }
            "restore-save" => {
                assert_eq!(
                    buffer
                        .text(&buffer.start_iter(), &buffer.end_iter(), true)
                        .as_str(),
                    CONTENTS
                );
                assert!(buffer.is_modified());
                assert!(save.is_sensitive());
                save.emit_clicked();
                for _ in 0..30 {
                    glib::timeout_future(std::time::Duration::from_millis(50)).await;
                    if !path.exists() && !buffer.is_modified() {
                        break;
                    }
                }
                assert!(!buffer.is_modified());
                assert!(!path.exists());
                eprintln!("NEXUTERM_SMOKE_RECOVERY_SAVE_OK");
            }
            "restore-conflict" | "restore-missing" | "restore-remote" => {
                assert_eq!(
                    buffer
                        .text(&buffer.start_iter(), &buffer.end_iter(), true)
                        .as_str(),
                    CONTENTS
                );
                assert!(buffer.is_modified());
                assert!(!save.is_sensitive());
                let copy =
                    std::path::PathBuf::from(std::env::var_os("NEXUTERM_RECOVERY_COPY").unwrap());
                let contents = CONTENTS.to_string();
                nexuterm::runtime::runtime()
                    .spawn_blocking(move || recovery::export_copy(&copy, &contents))
                    .await
                    .unwrap()
                    .unwrap();
                if mode == "restore-conflict" {
                    button_named(
                        tracker.root.upgrade().unwrap().upcast_ref(),
                        "Comparar original",
                    )
                    .unwrap()
                    .emit_clicked();
                    glib::timeout_future(std::time::Duration::from_millis(200)).await;
                    assert!(
                        gtk::Window::list_toplevels()
                            .iter()
                            .filter_map(|w| w.downcast_ref::<gtk::Window>())
                            .any(|w| w.title().as_deref() == Some("Comparar original y borrador"))
                    );
                }
                eprintln!("NEXUTERM_SMOKE_RECOVERY_RESTORE_OK");
            }
            _ => {}
        }
    });
}
