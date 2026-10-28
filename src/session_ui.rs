//! Automatic, non-executing workspace recovery. All disk I/O runs off the GTK thread.
use super::*;
use nexuterm::session_recovery::{self, Journal};
use nexuterm::workspaces::Workspace;
use std::sync::Arc;

pub(super) struct Controller {
    journal: RefCell<Option<Arc<Journal>>>,
    running: Cell<bool>,
    closing: Cell<bool>,
    finished: Cell<bool>,
    last: RefCell<Option<Workspace>>,
    error: RefCell<Option<String>>,
    scanning: Cell<bool>,
    manager: RefCell<glib::WeakRef<gtk::Window>>,
}
impl Controller {
    pub(super) async fn flush(&self, state: &Rc<State>, force: bool) -> Result<(), String> {
        while self.running.get() {
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        self.running.set(true);
        let result = async {
            if self.journal.borrow().is_none() {
                let journal = nexuterm::runtime::runtime()
                    .spawn_blocking(|| Journal::create(&session_recovery::directory()?))
                    .await
                    .map_err(|e| e.to_string())??;
                *self.journal.borrow_mut() = Some(journal);
            }
            let trackers: Vec<_> = state
                .recoveries
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .collect();
            for tracker in trackers {
                tracker.flush().await?;
            }
            let journal = self.journal.borrow().as_ref().unwrap().clone();
            let snapshot = workspace_ui::capture(
                state,
                journal.id.clone(),
                "Sesión automática".into(),
                String::new(),
            );
            if !force && self.last.borrow().as_ref() == Some(&snapshot) {
                return Ok(());
            }
            let saved = nexuterm::runtime::runtime()
                .spawn_blocking(move || journal.save(snapshot.clone()).map(|_| snapshot))
                .await
                .map_err(|e| e.to_string())??;
            *self.last.borrow_mut() = Some(saved);
            Ok(())
        }
        .await;
        self.running.set(false);
        result
    }
    fn report(&self, state: &State, error: String) {
        if self.error.borrow().as_ref() != Some(&error) {
            state.status.set_text(&format!("No se pudo actualizar la recuperación automática; se conserva la última copia válida. {error}"));
        }
        *self.error.borrow_mut() = Some(error);
    }
}

pub(super) fn start(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    let controller = Rc::new(Controller {
        journal: RefCell::new(None),
        running: Cell::new(false),
        closing: Cell::new(false),
        finished: Cell::new(false),
        last: RefCell::new(None),
        error: RefCell::new(None),
        scanning: Cell::new(false),
        manager: RefCell::new(glib::WeakRef::new()),
    });
    *state.session_recovery.borrow_mut() = Some(controller.clone());
    show(state, window, true);
    let state = Rc::downgrade(state);
    glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        let Some(state) = state.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if controller.finished.get() || state.window.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        if !controller.running.get()
            && !controller.closing.get()
            && !state
                .writes
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .any(|v| v.get())
        {
            let controller = controller.clone();
            glib::spawn_future_local(async move {
                match controller.flush(&state, false).await {
                    Ok(()) => {
                        if controller.error.borrow_mut().take().is_some() {
                            state
                                .status
                                .set_text("La recuperación automática vuelve a estar disponible.");
                        }
                    }
                    Err(error) => controller.report(&state, error),
                }
            });
        }
        glib::ControlFlow::Continue
    });
}

pub(super) fn prepare_close(state: &Rc<State>, window: &gtk::ApplicationWindow) -> bool {
    let Some(controller) = state.session_recovery.borrow().clone() else {
        return false;
    };
    if controller.finished.get() {
        return false;
    }
    if controller.closing.replace(true) {
        return true;
    }
    window.set_sensitive(false);
    let state = state.clone();
    let window = window.downgrade();
    glib::spawn_future_local(async move {
        let result = controller.flush(&state, true).await;
        let Some(window) = window.upgrade() else {
            return;
        };
        window.set_sensitive(true);
        controller.closing.set(false);
        match result {
            Ok(()) => {
                controller.finished.set(true);
                controller.journal.borrow_mut().take();
                window.close();
            }
            Err(error) => {
                controller.report(&state, error);
                let dialog=gtk::AlertDialog::builder().modal(true).message("No se pudo conservar la última sesión")
                    .detail("La última copia válida y los borradores existentes se conservan. Puedes cancelar para corregir el problema o cerrar sin actualizar la copia de sesión.")
                    .buttons(["Cancelar","Cerrar sin actualizar la copia"]).cancel_button(0).default_button(0).build();
                if dialog.choose_future(Some(&window)).await == Ok(1) {
                    controller.finished.set(true);
                    controller.journal.borrow_mut().take();
                    window.close();
                }
            }
        }
    });
    true
}

pub(super) fn show(state: &Rc<State>, window: &gtk::ApplicationWindow, only_if_found: bool) {
    let Some(controller) = state.session_recovery.borrow().clone() else {
        return;
    };
    if let Some(manager) = controller.manager.borrow().upgrade() {
        manager.present();
        return;
    }
    if controller.scanning.replace(true) {
        return;
    }
    let state = Rc::downgrade(state);
    let window = window.downgrade();
    glib::spawn_future_local(async move {
        let result = nexuterm::runtime::runtime()
            .spawn_blocking(|| session_recovery::list(&session_recovery::directory()?))
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r);
        controller.scanning.set(false);
        let (Some(state), Some(window)) = (state.upgrade(), window.upgrade()) else {
            return;
        };
        let (entries, invalid) = match result {
            Ok(v) => v,
            Err(error) => {
                controller.report(&state, error);
                return;
            }
        };
        if only_if_found && entries.is_empty() && invalid == 0 {
            crate::recovery_ui::offer(&state, &window);
            return;
        }
        let dialog = gtk::Window::builder()
            .title("Recuperar sesión anterior")
            .transient_for(&window)
            .destroy_with_parent(true)
            .default_width(740)
            .default_height(420)
            .build();
        dialog.set_widget_name("session-recovery-manager");
        *controller.manager.borrow_mut() = dialog.downgrade();
        let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
        body.set_margin_start(16);
        body.set_margin_end(16);
        body.set_margin_top(16);
        body.set_margin_bottom(16);
        let explanation = gtk::Label::new(Some(
            "Las copias automáticas conservan pestañas, divisiones, documentos y chats. Recuperar añade el trabajo sin iniciar conexiones ni ejecutar comandos. Las copias de ventanas activas no se muestran.",
        ));
        explanation.set_wrap(true);
        explanation.set_xalign(0.0);
        body.append(&explanation);
        let rows = gtk::Box::new(gtk::Orientation::Vertical, 12);
        let scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(180)
            .child(&rows)
            .build();
        body.append(&scroll);
        let status = gtk::Label::new(Some(&format!(
            "{} copias disponibles. {invalid} copias dañadas o desconocidas se conservan sin modificar.",
            entries.len()
        )));
        status.set_wrap(true);
        status.set_xalign(0.0);
        status.set_widget_name("session-recovery-status");
        body.append(&status);
        for entry in entries {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let date = glib::DateTime::from_unix_local(entry.saved_at as i64)
                .ok()
                .and_then(|d| d.format("%d/%m/%Y %H:%M:%S").ok())
                .map(|v| v.to_string())
                .unwrap_or_else(|| entry.journal.id[..8].to_string());
            let title = gtk::Label::new(Some(&format!(
                "{date} · {} pestañas · {} chats",
                entry.workspace.tabs.len(),
                entry.workspace.chats.tabs.len()
            )));
            title.set_wrap(true);
            title.set_xalign(0.0);
            title.set_hexpand(true);
            row.append(&title);
            let recover = gtk::Button::with_label("Recuperar");
            recover.set_widget_name("session-recover");
            row.append(&recover);
            let discard = gtk::Button::with_label("Descartar copia");
            discard.set_widget_name("session-discard");
            row.append(&discard);
            rows.append(&row);
            let loaded = Rc::new(Cell::new(false));
            let s = Rc::downgrade(&state);
            let c = controller.clone();
            let j = entry.journal.clone();
            let workspace = entry.workspace;
            let r = row.downgrade();
            let notice = status.downgrade();
            let busy = Rc::new(Cell::new(false));
            let busy_recover = busy.clone();
            recover.connect_clicked(move |_| {
                if busy_recover.replace(true){return;}
                let Some(state)=s.upgrade() else {busy_recover.set(false);return;};
                if !loaded.get(){if !workspace_ui::restore(&state,&workspace){busy_recover.set(false);return;}loaded.set(true);}
                if let Some(row)=r.upgrade(){row.set_sensitive(false);}
                let (controller,journal,row,notice,busy)=(c.clone(),j.clone(),r.clone(),notice.clone(),busy_recover.clone());
                glib::spawn_future_local(async move {
                    let result=async {controller.flush(&state,true).await?;nexuterm::runtime::runtime().spawn_blocking(move ||journal.remove()).await.map_err(|e|e.to_string())?}.await;
                    busy.set(false);
                    if let Some(row)=row.upgrade(){match result {
                        Ok(())=>{row.set_sensitive(false);if let Some(notice)=notice.upgrade(){notice.set_text("Sesión recuperada y conservada en la copia actual. Las conexiones esperan tu acción.");}},
                        Err(error)=>{row.set_sensitive(true);if let Some(notice)=notice.upgrade(){notice.set_text(&format!("La sesión sigue abierta; no se pudo completar la conservación y retirada de la copia anterior: {error}"));}},
                    }}
                });
            });
            let j = entry.journal;
            let w = dialog.downgrade();
            let r = row.downgrade();
            let notice = status.downgrade();
            discard.connect_clicked(move |_| {
                if busy.replace(true){return;}
                let (j,w,row,notice,busy)=(j.clone(),w.clone(),r.clone(),notice.clone(),busy.clone());
                glib::spawn_future_local(async move {
                    if let Some(window)=w.upgrade(){
                        let alert=gtk::AlertDialog::builder().modal(true).message("¿Descartar esta copia de sesión?")
                            .detail("Se eliminará la copia automática de estructura y chats. Los documentos originales y sus borradores no se borran.")
                            .buttons(["Cancelar","Descartar copia"]).cancel_button(0).default_button(0).build();
                        if alert.choose_future(Some(&window)).await==Ok(1){
                            let result=nexuterm::runtime::runtime().spawn_blocking(move ||j.remove()).await.map_err(|e|e.to_string()).and_then(|r|r);
                            match result{Ok(())=>{if let Some(row)=row.upgrade(){row.set_sensitive(false);}},Err(error)=>{if let Some(notice)=notice.upgrade(){notice.set_text(&error);}}}
                        }
                    }
                    busy.set(false);
                });
            });
        }
        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let documents = gtk::Button::with_label("Recuperar documentos…");
        let s = Rc::downgrade(&state);
        let w = window.downgrade();
        documents.connect_clicked(move |_| {
            if let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) {
                crate::recovery_ui::show(&s, &w);
            }
        });
        buttons.append(&documents);
        let later = gtk::Button::with_label("Cerrar");
        let d = dialog.downgrade();
        later.connect_clicked(move |_| {
            if let Some(d) = d.upgrade() {
                d.close();
            }
        });
        buttons.append(&later);
        body.append(&buttons);
        dialog.set_child(Some(&body));
        dialog.present();
    });
}

#[cfg(debug_assertions)]
pub(super) fn smoke(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    fn named(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
        if root.widget_name() == name {
            return Some(root.clone());
        }
        let mut child = root.first_child();
        while let Some(w) = child {
            if let Some(v) = named(&w, name) {
                return Some(v);
            }
            child = w.next_sibling();
        }
        None
    }
    fn button(root: &gtk::Widget, text: &str) -> Option<gtk::Button> {
        if let Some(b) = root.downcast_ref::<gtk::Button>()
            && b.label().as_deref() == Some(text)
        {
            return Some(b.clone());
        }
        let mut child = root.first_child();
        while let Some(w) = child {
            if let Some(b) = button(&w, text) {
                return Some(b);
            }
            child = w.next_sibling();
        }
        None
    }
    async fn confirm(text: &str) {
        for _ in 0..60 {
            for w in gtk::Window::list_toplevels() {
                if w.widget_name() == "session-recovery-manager" {
                    continue;
                }
                if let Some(b) = button(&w, text) {
                    b.emit_clicked();
                    return;
                }
            }
            glib::timeout_future(std::time::Duration::from_millis(30)).await;
        }
        panic!("Missing confirmation: {text}");
    }
    let mode = std::env::var("NEXUTERM_SMOKE_SESSION").unwrap();
    let base = PathBuf::from(std::env::var_os("HOME").unwrap());
    if matches!(mode.as_str(), "capture" | "close" | "failure") {
        window
            .lookup_action("split-horizontal")
            .unwrap()
            .activate(None);
        crate::files::editor(
            state,
            window,
            gio::File::for_path(base.join("document.txt")),
        );
        let chats: nexuterm::workspaces::Chats =
            serde_json::from_slice(&std::fs::read(base.join("chat-seed.json")).unwrap()).unwrap();
        (state.ai_workspace.borrow().as_ref().unwrap().restore)(&chats).unwrap();
    }
    let state = state.clone();
    let window = window.clone();
    glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        let controller = state.session_recovery.borrow().clone().unwrap();
        if matches!(mode.as_str(), "capture" | "close" | "failure") {
            let tracker = state
                .recoveries
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .next()
                .unwrap();
            let root = tracker.root.upgrade().unwrap();
            fn text_view(w: &gtk::Widget) -> Option<gtk::TextView> {
                if let Some(v) = w.downcast_ref::<gtk::TextView>() {
                    return Some(v.clone());
                }
                let mut child = w.first_child();
                while let Some(c) = child {
                    if let Some(v) = text_view(&c) {
                        return Some(v);
                    }
                    child = c.next_sibling();
                }
                None
            }
            let buffer = text_view(root.upcast_ref()).unwrap().buffer();
            buffer.set_text("Primera edición automática áé\n");
            if mode == "close" {
                window.close();
                confirm("Cerrar").await;
                eprintln!("NEXUTERM_SMOKE_SESSION_CLOSE_REQUESTED");
                return;
            }
            for _ in 0..120 {
                glib::timeout_future(std::time::Duration::from_millis(30)).await;
                if controller.last.borrow().is_some() {
                    break;
                }
            }
            assert!(
                controller.last.borrow().is_some(),
                "periodic checkpoint did not run"
            );
            let first = controller.last.borrow().clone().unwrap();
            let journal = controller.journal.borrow().clone().unwrap();
            assert_eq!(journal.read().unwrap(), first);
            let editor = named(window.upcast_ref(), "codex-composer")
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap();
            buffer.set_text("Última edición automática áé\n");
            editor
                .buffer()
                .set_text("Último borrador de chat sin enviar áé\n");
            if mode == "failure" {
                let directory = session_recovery::directory().unwrap();
                let backup = base.join("preserved-sessions");
                std::fs::rename(&directory, &backup).unwrap();
                std::fs::write(&directory, b"storage blocked").unwrap();
                assert!(controller.flush(&state, true).await.is_err());
                assert_eq!(
                    nexuterm::workspaces::Store::load(
                        &backup.join(journal.path.file_name().unwrap())
                    )
                    .unwrap()
                    .workspaces[0],
                    first
                );
                window.close();
                confirm("Cerrar").await;
                confirm("Cancelar").await;
                assert!(window.is_visible() && buffer.is_modified());
                assert!(!controller.finished.get());
                assert_eq!(
                    editor.buffer().text(
                        &editor.buffer().start_iter(),
                        &editor.buffer().end_iter(),
                        false
                    ),
                    "Último borrador de chat sin enviar áé\n"
                );
                std::fs::remove_file(&directory).unwrap();
                std::fs::rename(&backup, &directory).unwrap();
                controller.flush(&state, true).await.unwrap();
                assert_eq!(
                    journal.read().unwrap().chats.tabs[0].draft,
                    "Último borrador de chat sin enviar áé\n"
                );
                eprintln!("NEXUTERM_SMOKE_SESSION_FAILURE_OK");
                return;
            }
            for _ in 0..120 {
                glib::timeout_future(std::time::Duration::from_millis(30)).await;
                if controller.last.borrow().as_ref().is_some_and(|s| {
                    s.chats.tabs[0].draft == "Último borrador de chat sin enviar áé\n"
                }) {
                    break;
                }
            }
            assert_eq!(
                journal.read().unwrap().chats.tabs[0].draft,
                "Último borrador de chat sin enviar áé\n"
            );
            assert_eq!(
                nexuterm::recovery::read(
                    &nexuterm::recovery::directory()
                        .unwrap()
                        .join(format!("{}.json", tracker.id))
                )
                .unwrap()
                .text,
                "Última edición automática áé\n"
            );
            eprintln!("NEXUTERM_SESSION_PID={}", std::process::id());
            eprintln!("NEXUTERM_SMOKE_SESSION_CAPTURE_OK");
        } else {
            let manager = controller
                .manager
                .borrow()
                .upgrade()
                .expect("startup recovery not offered");
            if mode == "invalid" {
                let status = named(manager.upcast_ref(), "session-recovery-status")
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();
                assert!(status.text().contains("1 copias dañadas"));
                controller.flush(&state, true).await.unwrap();
                eprintln!("NEXUTERM_SMOKE_SESSION_INVALID_OK");
                return;
            }
            if mode == "discard" {
                let discard = named(manager.upcast_ref(), "session-discard")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap();
                discard.emit_clicked();
                confirm("Descartar copia").await;
                for _ in 0..60 {
                    glib::timeout_future(std::time::Duration::from_millis(30)).await;
                    if !discard.is_sensitive() {
                        break;
                    }
                }
                assert!(!discard.is_sensitive());
                eprintln!("NEXUTERM_SMOKE_SESSION_DISCARD_OK");
                return;
            }
            let before = state.notebook.n_pages();
            let sessions = state
                .sessions
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .count();
            let recover = named(manager.upcast_ref(), "session-recover")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap();
            let directory = session_recovery::directory().unwrap();
            let backup = base.join("handoff-preserved");
            if mode == "restore-failure" {
                std::fs::rename(&directory, &backup).unwrap();
                std::fs::write(&directory, b"storage blocked").unwrap();
            }
            recover.emit_clicked();
            let status = named(manager.upcast_ref(), "session-recovery-status")
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap();
            if mode == "restore-failure" {
                for _ in 0..100 {
                    glib::timeout_future(std::time::Duration::from_millis(30)).await;
                    if status.text().starts_with("La sesión sigue abierta") {
                        break;
                    }
                }
                assert!(
                    status.text().starts_with("La sesión sigue abierta"),
                    "{}",
                    status.text()
                );
                assert_eq!(
                    std::fs::read_dir(&backup)
                        .unwrap()
                        .filter_map(Result::ok)
                        .filter(|e| e.path().extension().is_some_and(|v| v == "json"))
                        .count(),
                    1
                );
                assert_eq!(state.notebook.n_pages(), before + 2);
                std::fs::remove_file(&directory).unwrap();
                std::fs::rename(&backup, &directory).unwrap();
                recover.emit_clicked();
            }
            for _ in 0..100 {
                glib::timeout_future(std::time::Duration::from_millis(30)).await;
                if status.text().starts_with("Sesión recuperada y conservada") {
                    break;
                }
            }
            assert!(
                status.text().starts_with("Sesión recuperada y conservada"),
                "{}",
                status.text()
            );
            assert_eq!(state.notebook.n_pages(), before + 2);
            assert_eq!(
                state
                    .sessions
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .count(),
                sessions
            );
            let chats = (state.ai_workspace.borrow().as_ref().unwrap().capture)();
            assert_eq!(
                chats.tabs[0].draft,
                "Último borrador de chat sin enviar áé\n"
            );
            assert!(!state.ai_process.borrow().as_ref().unwrap().busy());
            assert!(
                !named(window.upcast_ref(), "codex-welcome")
                    .unwrap()
                    .is_visible()
            );
            let tracker = state
                .recoveries
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .next()
                .unwrap();
            assert!(tracker.modified());
            let snapshot = controller
                .journal
                .borrow()
                .as_ref()
                .unwrap()
                .read()
                .unwrap();
            assert_eq!(snapshot.tabs.len(), 3);
            assert!(matches!(
                snapshot.tabs[1].layout,
                nexuterm::workspaces::Layout::Split { .. }
            ));
            assert_eq!(
                state
                    .notebook
                    .selected_page()
                    .map(|p| state.notebook.page_position(&p)),
                Some(2)
            );
            if mode == "restore-failure" {
                eprintln!("NEXUTERM_SMOKE_SESSION_RESTORE_FAILURE_OK");
            } else {
                eprintln!("NEXUTERM_SMOKE_SESSION_RESTORE_OK");
            }
        }
    });
}
