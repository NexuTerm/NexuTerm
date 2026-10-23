use crate::ui::{State, icon_button};
use gtk::{glib, prelude::*};
use nexuterm::{
    ssh::SshConnection,
    transfer::{Client, Entry, Event, Request},
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

pub struct Panel {
    pub widget: gtk::Box,
    auth_area: RefCell<Option<glib::WeakRef<gtk::Widget>>>,
    local: RefCell<Option<std::rc::Weak<crate::file_browser::LocalPane>>>,
    #[cfg(debug_assertions)]
    smoke_step: Cell<u8>,
    #[cfg(debug_assertions)]
    notebook: glib::WeakRef<adw::TabView>,
    busy: Cell<bool>,
    writing: Rc<Cell<bool>>,
    client: RefCell<Client>,
    current: RefCell<PathBuf>,
    entries: RefCell<Vec<Entry>>,
    list: gtk::ListBox,
    path: gtk::Entry,
    status: gtk::Label,
    progress: gtk::ProgressBar,
    tools: gtk::Box,
    next: gtk::Button,
    cancel_button: gtk::Button,
    transfer: Cell<bool>,
    window: glib::WeakRef<gtk::ApplicationWindow>,
}
impl Panel {
    pub fn hide_auth_when_ready(&self, widget: &gtk::Widget) {
        *self.auth_area.borrow_mut() = Some(widget.downgrade());
    }
    pub fn bind_local(&self, local: &Rc<crate::file_browser::LocalPane>) {
        *self.local.borrow_mut() = Some(Rc::downgrade(local));
    }
    pub fn upload_file(&self, local: PathBuf) {
        self.request(
            Request::Upload {
                local,
                directory: self.current.borrow().clone(),
            },
            true,
        );
    }
    pub fn new(
        state: &Rc<State>,
        window: &gtk::ApplicationWindow,
        connection: Arc<SshConnection>,
    ) -> Rc<Self> {
        Self::with_client(state, window, Client::start(connection))
    }
    pub fn with_client(
        state: &Rc<State>,
        window: &gtk::ApplicationWindow,
        client: Client,
    ) -> Rc<Self> {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("nexuterm-section");
        widget.add_css_class("nexuterm-panel");
        widget.set_width_request(240);
        for set in [
            gtk::Widget::set_margin_top,
            gtk::Widget::set_margin_bottom,
            gtk::Widget::set_margin_start,
            gtk::Widget::set_margin_end,
        ] {
            set(widget.upcast_ref(), 8);
        }
        let heading = gtk::Label::new(Some("ARCHIVOS REMOTOS"));
        heading.set_xalign(0.0);
        heading.add_css_class("heading");
        widget.append(&heading);
        let tools = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let up = icon_button("go-up-symbolic", "Carpeta superior");
        let home = icon_button("user-home-symbolic", "Carpeta de inicio remota");
        let refresh = icon_button("view-refresh-symbolic", "Actualizar o reintentar conexión");
        let upload = icon_button("document-send-symbolic", "Subir archivo…");
        let download = icon_button(
            "folder-download-symbolic",
            "Descargar archivo seleccionado…",
        );
        for b in [&up, &home, &refresh, &upload, &download] {
            tools.append(b);
        }
        widget.append(&tools);
        let path = gtk::Entry::builder()
            .placeholder_text("Ruta remota · Enter para abrir")
            .build();
        path.set_tooltip_text(Some("Ruta remota; se abre al pulsar Enter"));
        widget.append(&path);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.add_css_class("navigation-sidebar");
        widget.append(
            &gtk::ScrolledWindow::builder()
                .child(&list)
                .vexpand(true)
                .hscrollbar_policy(gtk::PolicyType::Never)
                .build(),
        );
        let next = gtk::Button::with_label("Página siguiente");
        next.set_sensitive(false);
        widget.append(&next);
        let progress = gtk::ProgressBar::new();
        progress.set_show_text(true);
        progress.set_visible(false);
        widget.append(&progress);
        let status = gtk::Label::new(Some("Completa la autenticación SSH en la terminal…"));
        status.set_xalign(0.0);
        status.set_wrap(true);
        status.add_css_class("dim-label");
        widget.append(&status);
        let cancel = icon_button(
            "process-stop-symbolic",
            "Cancelar transferencia; puede quedar un archivo parcial",
        );
        cancel.set_sensitive(false);
        widget.append(&cancel);
        let writing = Rc::new(Cell::new(false));
        state.writes.borrow_mut().retain(|w| w.strong_count() > 0);
        state.writes.borrow_mut().push(Rc::downgrade(&writing));
        let panel = Rc::new(Self {
            widget,
            auth_area: RefCell::new(None),
            local: RefCell::new(None),
            #[cfg(debug_assertions)]
            smoke_step: Cell::new(0),
            #[cfg(debug_assertions)]
            notebook: state.notebook.downgrade(),
            busy: Cell::new(false),
            writing,
            client: RefCell::new(client),
            current: RefCell::new(PathBuf::from(".")),
            entries: RefCell::new(Vec::new()),
            list,
            path,
            status,
            progress,
            tools,
            next,
            cancel_button: cancel.clone(),
            transfer: Cell::new(false),
            window: window.downgrade(),
        });
        let p = Rc::downgrade(&panel);
        up.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                let target = p
                    .current
                    .borrow()
                    .parent()
                    .unwrap_or(std::path::Path::new("/"))
                    .into();
                p.navigate(target);
            }
        });
        let p = Rc::downgrade(&panel);
        home.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.navigate(".".into());
            }
        });
        let p = Rc::downgrade(&panel);
        refresh.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                let path = p.current.borrow().clone();
                p.navigate(path);
            }
        });
        let p = Rc::downgrade(&panel);
        panel.path.connect_activate(move |entry| {
            if let Some(p) = p.upgrade() {
                p.navigate(PathBuf::from(entry.text().as_str()));
            }
        });
        let p = Rc::downgrade(&panel);
        panel.next.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.request(Request::Next, false);
            }
        });
        let p = Rc::downgrade(&panel);
        upload.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.upload();
            }
        });
        let p = Rc::downgrade(&panel);
        download.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.download();
            }
        });
        let p = Rc::downgrade(&panel);
        cancel.connect_clicked(move |_| {if let Some(p) = p.upgrade() {p.client.borrow().cancel.store(true, Ordering::Relaxed);p.status.set_text("Cancelando: se espera la operación ya enviada; puede quedar un archivo parcial.");}});
        let p = Rc::downgrade(&panel);
        panel.list.connect_row_activated(move |_, row| {
            if let Some(p) = p.upgrade() {
                let entry = p.entries.borrow().get(row.index() as usize).cloned();
                if let Some(e) = entry {
                    if e.directory {
                        let target = p.current.borrow().join(&e.name);
                        p.navigate(target);
                    } else {
                        p.download();
                    }
                }
            }
        });
        let p = Rc::downgrade(&panel);
        glib::timeout_add_local(Duration::from_millis(100), move || {
            let Some(p) = p.upgrade() else {
                return glib::ControlFlow::Break;
            };
            for _ in 0..4 {
                let event = p.client.borrow_mut().events.try_recv();
                match event {
                    Ok(event) => p.event(event),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(_) => {
                        p.status.set_text("Navegador desconectado.");
                        p.set_busy(false);
                        p.writing.set(false);
                        p.cancel_button.set_sensitive(false);
                        return glib::ControlFlow::Break;
                    }
                }
            }
            if p.transfer.get() {
                let progress = p.client.borrow().progress.borrow().clone();
                p.progress.set_fraction(
                    progress
                        .total
                        .filter(|n| *n > 0)
                        .map_or(0.0, |n| progress.bytes as f64 / n as f64)
                        .clamp(0.0, 1.0),
                );
                p.progress
                    .set_text(Some(&format!("{} bytes transferidos", progress.bytes)));
            }
            glib::ControlFlow::Continue
        });
        panel.navigate(".".into());
        panel
    }
    pub fn stop(&self) {
        self.client.borrow().stop();
    }
    fn set_busy(&self, busy: bool) {
        self.busy.set(busy);
        self.tools.set_sensitive(!busy);
        self.path.set_sensitive(!busy);
        self.list.set_sensitive(!busy);
        self.next.set_sensitive(!busy);
    }
    fn request(&self, request: Request, transfer: bool) {
        if self.busy.get() {
            return;
        }
        self.client.borrow().cancel.store(false, Ordering::Relaxed);
        match self.client.borrow().requests.try_send(request) {
            Ok(()) => {
                self.set_busy(true);
                self.transfer.set(transfer);
                self.cancel_button.set_sensitive(transfer);
                self.writing.set(transfer);
                self.progress.set_visible(transfer);
                if transfer {
                    self.status.set_text("Transfiriendo…");
                } else {
                    self.status.set_text("Leyendo carpeta remota…");
                }
            }
            Err(e) => self
                .status
                .set_text(&format!("No se pudo enviar la operación: {e}")),
        }
    }
    fn navigate(&self, directory: PathBuf) {
        self.request(Request::Navigate(directory), false);
    }
    fn event(self: &Rc<Self>, event: Event) {
        self.set_busy(false);
        self.writing.set(false);
        self.cancel_button.set_sensitive(false);
        self.transfer.set(false);
        match event {
            Event::Error(e) => {
                self.next.set_sensitive(false);
                self.status.set_text(&e);
            }
            Event::Finished(result) => match result {
                Ok(()) => {
                    self.progress.set_fraction(1.0);
                    self.status.set_text("Transferencia completada.");
                    if let Some(local) = self
                        .local
                        .borrow()
                        .as_ref()
                        .and_then(std::rc::Weak::upgrade)
                    {
                        local.refresh();
                    }
                    let path = self.current.borrow().clone();
                    self.navigate(path);
                }
                Err(e) => {
                    self.status.set_text(&e);
                    #[cfg(debug_assertions)]
                    if std::env::var_os("NEXUTERM_SMOKE_FILE_BROWSER").is_some() {
                        assert_eq!(self.smoke_step.get(), 4, "Unexpected transfer failure: {e}");
                        self.smoke_step.set(5);
                        self.navigate(self.current.borrow().clone());
                    }
                }
            },
            Event::Page {
                directory,
                entries,
                more,
            } => {
                if let Some(area) = self
                    .auth_area
                    .borrow_mut()
                    .take()
                    .and_then(|area| area.upgrade())
                {
                    area.set_visible(false);
                    #[cfg(debug_assertions)]
                    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                        && std::env::var_os("NEXUTERM_SMOKE_SFTP").is_some()
                    {
                        assert!(!area.is_visible());
                        eprintln!("NEXUTERM_SMOKE_SFTP_AUTH_HIDDEN");
                    }
                }
                if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                    && std::env::var_os("NEXUTERM_SMOKE_SSH_LAB").is_some()
                {
                    eprintln!("NEXUTERM_SMOKE_REMOTE_READY entries={}", entries.len());
                }
                self.path.set_text(&directory.to_string_lossy());
                *self.current.borrow_mut() = directory;
                while let Some(child) = self.list.first_child() {
                    self.list.remove(&child);
                }
                for entry in &entries {
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    let icon = if entry.directory {
                        "folder-symbolic"
                    } else if entry.regular {
                        "text-x-generic-symbolic"
                    } else {
                        "emblem-symbolic-link-symbolic"
                    };
                    row.append(&gtk::Image::from_icon_name(icon));
                    let name = gtk::Label::new(Some(&entry.name.to_string_lossy()));
                    name.set_xalign(0.0);
                    name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                    name.set_hexpand(true);
                    name.set_tooltip_text(Some(&entry.name.to_string_lossy()));
                    row.append(&name);
                    if let Some(size) = entry.size.filter(|_| entry.regular) {
                        let label = gtk::Label::new(Some(&glib::format_size(size)));
                        label.add_css_class("dim-label");
                        row.append(&label);
                    }
                    self.list.append(&row);
                }
                self.status.set_text(&format!(
                    "{} entradas · {}",
                    entries.len(),
                    if more {
                        "hay más páginas"
                    } else {
                        "fin de carpeta"
                    }
                ));
                self.next.set_sensitive(more);
                *self.entries.borrow_mut() = entries;
                #[cfg(debug_assertions)]
                if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                    && std::env::var_os("NEXUTERM_SMOKE_FILE_BROWSER").is_some()
                {
                    self.smoke_page();
                }
            }
        }
    }
    #[cfg(debug_assertions)]
    fn smoke_page(self: &Rc<Self>) {
        let local = self
            .local
            .borrow()
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
            .expect("Local pane");
        let row_for = |name: &str| {
            let index = self
                .entries
                .borrow()
                .iter()
                .position(|entry| entry.name == std::path::Path::new(name))
                .expect(name);
            self.list.row_at_index(index as i32).unwrap()
        };
        let upload_name =
            std::env::var("NEXUTERM_SMOKE_UPLOAD_NAME").unwrap_or_else(|_| "upload.txt".into());
        match self.smoke_step.get() {
            0 => {
                self.smoke_step.set(1);
                let row = row_for("fixtures");
                self.list.emit_by_name::<()>("row-activated", &[&row]);
            }
            1 => {
                self.smoke_step.set(2);
                self.list.select_row(Some(&row_for("unicode.txt")));
                self.download();
            }
            2 => {
                assert_eq!(
                    std::fs::read(local.directory().join("unicode.txt")).unwrap(),
                    "Texto de prueba UTF-8: áéíóú\n".as_bytes()
                );
                self.smoke_step.set(3);
                local.smoke_upload(&upload_name);
            }
            3 => {
                assert!(
                    self.entries
                        .borrow()
                        .iter()
                        .any(|entry| entry.name == std::path::Path::new(&upload_name))
                );
                self.smoke_step.set(4);
                local.smoke_upload(&upload_name);
            }
            5 => {
                self.smoke_step.set(6);
                let notebook = self.notebook.clone();
                let remote = self.clone();
                glib::timeout_add_local_once(Duration::from_millis(500), move || {
                    let notebook = notebook.upgrade().unwrap();
                    let page = notebook.selected_page().unwrap();
                    assert!(remote.widget.is_ancestor(&page.child()));
                    fn terminals(widget: &gtk::Widget) -> Vec<vte4::Terminal> {
                        if let Some(terminal) = widget.downcast_ref::<vte4::Terminal>() {
                            return vec![terminal.clone()];
                        }
                        let mut result = Vec::new();
                        let mut child = widget.first_child();
                        while let Some(widget) = child {
                            result.extend(terminals(&widget));
                            child = widget.next_sibling();
                        }
                        result
                    }
                    assert!(
                        terminals(&page.child())
                            .iter()
                            .all(|terminal| !terminal.is_mapped()),
                        "File explorer must not show a console"
                    );
                    let window = remote.window.upgrade().unwrap();
                    if let Some(path) = std::env::var_os("NEXUTERM_FILE_BROWSER_SCREENSHOT") {
                        let paintable = gtk::WidgetPaintable::new(Some(&window));
                        let snapshot = gtk::Snapshot::new();
                        paintable.snapshot(
                            &snapshot,
                            window.width() as f64,
                            window.height() as f64,
                        );
                        if let (Some(node), Some(renderer)) =
                            (snapshot.to_node(), window.renderer())
                        {
                            renderer
                                .render_texture(&node, None)
                                .save_to_png(path)
                                .unwrap();
                        }
                    }
                    notebook.close_page(&page);
                    assert!(remote.client.borrow().cancel.load(Ordering::Relaxed));
                    eprintln!(
                        "NEXUTERM_SMOKE_FILE_BROWSER_OK navigation=1 download=1 upload=1 overwrite_blocked=1 closed=1"
                    );
                    let app = window.application().unwrap();
                    glib::timeout_add_local_once(Duration::from_secs(2), move || {
                        app.quit();
                    });
                });
            }
            _ => {}
        }
    }
    fn upload(self: &Rc<Self>) {
        if let Some(local) = self
            .local
            .borrow()
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
        {
            if let Some(path) = local.selected_file() {
                self.upload_file(path);
            } else {
                self.status
                    .set_text("Selecciona un archivo en el panel local para subir.");
            }
            return;
        }
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let p = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            match gtk::FileDialog::builder()
                .title("Subir archivo al servidor")
                .build()
                .open_future(Some(&window))
                .await
            {
                Ok(file) => {
                    if let (Some(p), Some(local)) = (p.upgrade(), file.path()) {
                        let directory = p.current.borrow().clone();
                        p.request(Request::Upload { local, directory }, true);
                    }
                }
                Err(e)
                    if !e.matches(gtk::DialogError::Dismissed)
                        && !e.matches(gtk::DialogError::Cancelled) =>
                {
                    if let Some(p) = p.upgrade() {
                        p.status.set_text(&e.to_string());
                    }
                }
                _ => {}
            }
        });
    }
    fn download(self: &Rc<Self>) {
        let Some(row) = self.list.selected_row() else {
            self.status
                .set_text("Selecciona un archivo para descargar.");
            return;
        };
        let Some(entry) = self.entries.borrow().get(row.index() as usize).cloned() else {
            return;
        };
        if !entry.regular {
            self.status
                .set_text("Selecciona un archivo regular; las carpetas se abren con doble clic.");
            return;
        }
        let remote = self.current.borrow().join(&entry.name);
        if let Some(local) = self
            .local
            .borrow()
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
        {
            self.request(
                Request::Download {
                    remote,
                    local: local.directory().join(&entry.name),
                },
                true,
            );
            return;
        }
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let p = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let dialog = gtk::FileDialog::builder()
                .title("Descargar archivo del servidor")
                .initial_name(entry.name.to_string_lossy())
                .build();
            match dialog.save_future(Some(&window)).await {
                Ok(file) => {
                    if let (Some(p), Some(local)) = (p.upgrade(), file.path()) {
                        p.request(Request::Download { remote, local }, true);
                    }
                }
                Err(e)
                    if !e.matches(gtk::DialogError::Dismissed)
                        && !e.matches(gtk::DialogError::Cancelled) =>
                {
                    if let Some(p) = p.upgrade() {
                        p.status.set_text(&e.to_string());
                    }
                }
                _ => {}
            }
        });
    }
}
