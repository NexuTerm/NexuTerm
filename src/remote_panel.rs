use crate::ui::{State, icon_button};
use gtk::{glib, prelude::*};
use nexuterm::{
    ssh::SshConnection,
    transfer::{Client, Entry, Event, Request},
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::{Rc,Weak},
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

pub struct Panel {
    pub widget: gtk::Box,
    state: std::rc::Weak<State>,
    connection: RefCell<Option<Arc<SshConnection>>>,
    auth_area: RefCell<Option<glib::WeakRef<gtk::Widget>>>,
    local: RefCell<Option<std::rc::Weak<crate::file_browser::LocalPane>>>,
    #[cfg(debug_assertions)]
    smoke_step: Cell<u8>,
    #[cfg(debug_assertions)]
    notebook: glib::WeakRef<adw::TabView>,
    clipboard_id: String,
    queue: RefCell<Option<Rc<crate::transfer_queue_ui::Controller>>>,
    queued_job: RefCell<Option<String>>,
    closed:Cell<bool>,
    busy: Cell<bool>,
    writing: Rc<Cell<bool>>,
    client: RefCell<Client>,
    current: RefCell<PathBuf>,
    pending_view: Rc<RefCell<Option<nexuterm::workspaces::BrowserView>>>,
    view_epoch: Rc<Cell<u64>>,
    entries: RefCell<Vec<Entry>>,
    list: gtk::ListBox,
    path: gtk::Entry,
    status: gtk::Label,
    progress: gtk::ProgressBar,
    tools: gtk::Box,
    next: gtk::Button,
    cancel_button: gtk::Button,
    transfer: Cell<bool>,
    hidden: gtk::ToggleButton,
    window: glib::WeakRef<gtk::ApplicationWindow>,
}
impl Panel {
    pub(crate) fn bind_queue(self: &Rc<Self>, profile: &nexuterm::config::Profile) {
        if self.queue.borrow().is_some() {
            return;
        }
        if let Some(state) = self.state.upgrade() {
            let queue = crate::transfer_queue_ui::Controller::attach(&state, profile, self);
            *self.queue.borrow_mut() = Some(queue);
        }
    }
    pub(crate) fn queue_busy(&self) -> bool {
        self.busy.get()
    }
    pub(crate) fn queue_closed(&self) -> bool {
        self.closed.get()
    }
    pub(crate) fn cancel_queued(&self) {
        self.client.borrow().cancel.store(true, Ordering::Relaxed);
    }
    pub(crate) fn begin_queued(
        self: &Rc<Self>,
        job: &nexuterm::transfer_queue::Job,
    ) -> Result<(), String> {
        if self.closed.get() || self.busy.get() {
            return Err("La conexión no está disponible".into());
        }
        self.client.borrow().cancel.store(false, Ordering::Relaxed);
        self.client
            .borrow()
            .requests
            .try_send(job.request())
            .map_err(|e| e.to_string())?;
        *self.queued_job.borrow_mut() = Some(job.id.clone());
        self.set_busy(true);
        self.writing.set(true);
        self.transfer.set(true);
        self.cancel_button.set_sensitive(true);
        self.progress.set_visible(true);
        self.progress.set_fraction(0.0);
        self.status.set_text(if job.tree {
            "Preparando carpeta…"
        } else {
            "Transfiriendo…"
        });
        Ok(())
    }
    pub fn upload_files(self: &Rc<Self>, paths: Vec<PathBuf>) {
        if paths.is_empty() || paths.len() > 128 {
            self.status.set_text("Selecciona entre 1 y 128 elementos");
            return;
        }
        let directory = self.current.borrow().clone();
        if !directory.is_absolute() {
            self.status
                .set_text("Espera a que cargue la carpeta remota");
            return;
        }
        if let Some(queue) = self.queue.borrow().clone() {
            let panel = Rc::downgrade(self);
            glib::spawn_future_local(async move {
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        let mut jobs = Vec::new();
                        for local in paths {
                            let metadata =
                                std::fs::symlink_metadata(&local).map_err(|e| e.to_string())?;
                            if !metadata.is_file() && !metadata.is_dir() {
                                return Err(
                                    "No se siguen enlaces ni se transfieren archivos especiales"
                                        .to_string(),
                                );
                            }
                            let remote = nexuterm::ssh::join_remote(
                                &directory,
                                std::path::Path::new(
                                    local.file_name().ok_or("Elemento sin nombre")?,
                                ),
                            )?;
                            jobs.push(nexuterm::transfer_queue::Job::new(
                                true,
                                metadata.is_dir(),
                                local,
                                remote,
                            ));
                        }
                        Ok(jobs)
                    })
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r);
                if let Some(panel) = panel.upgrade() {
                    match result {
                        Ok(jobs) => queue.enqueue(&panel, jobs),
                        Err(e) => panel.status.set_text(&e),
                    }
                }
            });
        } else if paths.len() == 1 {
            self.request(
                Request::Upload {
                    local: paths[0].clone(),
                    directory,
                },
                true,
            );
        } else {
            self.status.set_text("La cola no está disponible");
        }
    }
    fn download_paths(self: &Rc<Self>, paths: Vec<(PathBuf, bool)>, directory: PathBuf) {
        if paths.is_empty() || paths.len() > 128 {
            self.status.set_text("Selecciona entre 1 y 128 elementos");
            return;
        }
        if let Some(queue) = self.queue.borrow().clone() {
            let jobs = paths
                .into_iter()
                .map(|(remote, tree)| {
                    let local = directory.join(remote.file_name().unwrap());
                    nexuterm::transfer_queue::Job::new(false, tree, local, remote)
                })
                .collect();
            queue.enqueue(self, jobs);
        } else if paths.len() == 1 {
            let remote = paths[0].0.clone();
            let local = directory.join(remote.file_name().unwrap());
            self.request(Request::Download { remote, local }, true);
        } else {
            self.status.set_text("La cola no está disponible");
        }
    }
    #[cfg(debug_assertions)]
    pub fn smoke_has_entry(&self, name: &str) -> bool {
        self.entries
            .borrow()
            .iter()
            .any(|e| e.name == std::path::Path::new(name))
    }
    pub fn workspace_view(&self) -> nexuterm::workspaces::BrowserView {
        if let Some(view) = self.pending_view.borrow().clone() {
            return view;
        }
        let selected = self
            .list
            .selected_rows().into_iter().next()
            .and_then(|r| {
                self.entries
                    .borrow()
                    .get(r.index() as usize)
                    .map(|e| e.name.clone())
            })
            .filter(|p| p.to_str().is_some());
        crate::widget_state::browser(self.widget.upcast_ref(), self.hidden.is_active(), selected)
    }
    pub fn restore_workspace_view(&self, view: nexuterm::workspaces::BrowserView) {
        self.view_epoch.set(self.view_epoch.get().wrapping_add(1));
        self.hidden.set_active(view.hidden);
        *self.pending_view.borrow_mut() = Some(view);
    }
    fn apply_pending_view(&self) {
        let Some(mut view) = self.pending_view.borrow().clone() else {
            return;
        };
        self.hidden.set_active(view.hidden);
        self.list.unselect_all();
        if let Some(index) = view
            .selected
            .as_ref()
            .and_then(|name| self.entries.borrow().iter().position(|e| &e.name == name))
        {
            if let Some(row) = self.list.row_at_index(index as i32) {
                self.list.select_row(Some(&row));
            }
        }
        view.selected = self.list.selected_rows().into_iter().next().and_then(|r| {
            self.entries
                .borrow()
                .get(r.index() as usize)
                .map(|e| e.name.clone())
        });
        *self.pending_view.borrow_mut() = Some(view);
        crate::widget_state::restore_browser_scroll(
            self.widget.upcast_ref(),
            self.pending_view.clone(),
            self.view_epoch.clone(),
        );
    }
    pub fn apply_preferences(&self, preferences: &nexuterm::config::Preferences) {
        crate::file_browser::apply_columns(self.widget.upcast_ref(), preferences);
        self.hidden.set_active(preferences.browser_show_hidden);
    }
    pub fn hide_auth_when_ready(&self, widget: &gtk::Widget) {
        *self.auth_area.borrow_mut() = Some(widget.downgrade());
    }
    pub fn bind_local(&self, local: &Rc<crate::file_browser::LocalPane>) {
        *self.local.borrow_mut() = Some(Rc::downgrade(local));
    }
    pub fn upload_file(self: &Rc<Self>, local: PathBuf) {
        self.upload_files(vec![local]);
    }
    pub fn paste_download(self: &Rc<Self>, text: &str, directory: PathBuf) -> bool {
        let parsed = serde_json::from_str::<(String, Vec<PathBuf>)>(text).or_else(|_| {
            serde_json::from_str::<(String, PathBuf)>(text).map(|(id, path)| (id, vec![path]))
        });
        let Ok((id, paths)) = parsed else {
            return false;
        };
        if id != self.clipboard_id || paths.is_empty() || paths.len() > 128 {
            return false;
        }
        let mut validated = Vec::new();
        for path in paths {
            let entries = self.entries.borrow();
            let Some(entry) = entries.iter().find(|entry| {
                (entry.regular || entry.directory)
                    && self.current.borrow().join(&entry.name) == path
            }) else {
                return false;
            };
            validated.push((path, entry.directory));
        }
        self.download_paths(validated, directory);
        true
    }
    pub(crate) fn current_directory(&self) -> Option<PathBuf> {
        Some(self.current.borrow().clone()).filter(|p|p.is_absolute())
    }
    pub fn new(
        state: &Rc<State>,
        window: &gtk::ApplicationWindow,
        connection: Arc<SshConnection>,
    ) -> Rc<Self> {
        Self::new_in_directory(state, window, connection, PathBuf::from("."))
    }
    pub fn new_in_directory(
        state: &Rc<State>,
        window: &gtk::ApplicationWindow,
        connection: Arc<SshConnection>,
        directory: PathBuf,
    ) -> Rc<Self> {
        let panel = Self::with_client_directory(state, window, Client::start(connection.clone()), directory);
        panel.bind_queue(&connection.profile);
        *panel.connection.borrow_mut() = Some(connection);
        panel
    }
    pub(crate) fn with_client_directory(
        state: &Rc<State>,
        window: &gtk::ApplicationWindow,
        client: Client,
        directory: PathBuf,
    ) -> Rc<Self> {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("nexuterm-section");
        widget.add_css_class("nexuterm-panel");
        widget.set_width_request(240);
        widget.set_margin_top(0);
        widget.set_margin_bottom(0);
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
        list.set_selection_mode(gtk::SelectionMode::Multiple);
        list.set_activate_on_single_click(false);
        list.add_css_class("file-details");
        list.set_overflow(gtk::Overflow::Hidden);
        let (hidden, _) = crate::file_browser::hidden_controls(
            &widget,
            &list,
            state.config.borrow().preferences.browser_show_hidden,
        );
        tools.append(&hidden);
        let details = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let columns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        for (caption, width, expand, class) in [
            ("Nombre", 144, true, "browser-name"),
            ("Tipo", 70, false, "browser-type"),
            ("Tamaño", 76, false, "browser-size"),
            ("Permisos", 96, false, "browser-permissions"),
        ] {
            let label = gtk::Label::new(Some(caption));
            label.set_xalign(0.0);
            label.set_hexpand(expand);
            label.set_width_request(width);
            label.add_css_class(class);
            columns.append(&label);
        }
        columns.add_css_class("detail-columns");
        details.append(&columns);
        details.append(&list);
        widget.append(
            &gtk::ScrolledWindow::builder()
                .child(&details)
                .overlay_scrolling(false)
                .vexpand(true)
                .hscrollbar_policy(gtk::PolicyType::Automatic)
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
            hidden,
            state: Rc::downgrade(state),
            connection: RefCell::new(None),
            auth_area: RefCell::new(None),
            local: RefCell::new(None),
            #[cfg(debug_assertions)]
            smoke_step: Cell::new(0),
            #[cfg(debug_assertions)]
            notebook: state.notebook.downgrade(),
            clipboard_id: uuid::Uuid::new_v4().to_string(),
            queue:RefCell::new(None),
            queued_job:RefCell::new(None),
            closed:Cell::new(false),
            busy: Cell::new(false),
            writing,
            client: RefCell::new(client),
            current: RefCell::new(directory.clone()),
            pending_view: Rc::new(RefCell::new(None)),
            view_epoch: Rc::new(Cell::new(0)),
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
        state
            .remote_browsers
            .borrow_mut()
            .retain(|p| p.strong_count() > 0);
        state
            .remote_browsers
            .borrow_mut()
            .push(Rc::downgrade(&panel));
        panel.apply_preferences(&state.config.borrow().preferences);
        let weak = Rc::downgrade(&panel);
        let source = move || {
            let panel=weak.upgrade()?;let entries=panel.entries.borrow();
            let mut paths=Vec::new();
            for row in panel.list.selected_rows(){
                let entry=entries.get(row.index() as usize)?;
                if !entry.regular && !entry.directory{return None;}
                paths.push(panel.current.borrow().join(&entry.name));
            }
            if paths.is_empty() || paths.len()>128{return None;}serde_json::to_string(&(&panel.clipboard_id,paths)).ok()
        };
        let weak=Rc::downgrade(&panel);
        crate::file_browser::transfer_controls(&panel.widget,&panel.list,source,move |text| {
            let Some(panel)=weak.upgrade() else{return false;};
            let lines:Vec<_>=text.lines().filter(|line|!line.is_empty() && !line.starts_with('#')).collect();
            if lines.is_empty() || lines.len()>128{return false;}
            let paths:Option<Vec<_>>=lines.iter().map(|line|url::Url::parse(line).ok()?.to_file_path().ok()).collect();
            let Some(paths)=paths else{return false;};panel.upload_files(paths);true
        });

        for (icon, label, folder) in [
            ("folder-new-symbolic", "Nueva carpeta", true),
            ("document-new-symbolic", "Nuevo archivo", false),
        ] {
            let button = icon_button(icon, label);
            let weak = Rc::downgrade(&panel);
            button.connect_clicked(move |_| {
                if let Some(panel) = weak.upgrade()
                    && let Some(window) = panel.window.upgrade()
                {
                    let directory = panel.current.borrow().clone();
                    let weak = Rc::downgrade(&panel);
                    crate::file_browser::create_dialog(&window, folder, move |name| {
                        if let Some(panel) = weak.upgrade() {
                            panel.request(
                                Request::Create {
                                    directory: directory.clone(),
                                    name,
                                    folder,
                                },
                                true,
                            );
                        }
                    });
                }
            });
            panel.tools.append(&button);
        }
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
                    } else if e.regular && p.connection.borrow().is_some() {
                        p.edit(p.current.borrow().join(&e.name));
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
        #[cfg(debug_assertions)]
        if let Some(path) = std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR") {
            panel.navigate(PathBuf::from(path).parent().unwrap().to_path_buf());
            return panel;
        }
        panel.navigate(directory);
        panel
    }
    pub fn stop(&self) {
        if self.closed.replace(true){return;}
        self.queued_job.borrow_mut().take();
        if let Some(queue)=self.queue.borrow().as_ref(){queue.detach(self);}
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
        if matches!(request,Request::Navigate(_)) {self.view_epoch.set(self.view_epoch.get().wrapping_add(1));self.pending_view.borrow_mut().take();}
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
        if let Some(id)=self.queued_job.borrow_mut().take() {
            if let Some(queue)=self.queue.borrow().clone() {
                match &event {
                    Event::Planned(_) if self.client.borrow().cancel.load(Ordering::Relaxed)=>queue.finished(id,Err("Preparación de carpeta cancelada".into()),true),
                    Event::Planned(plan)=>queue.planned(id,plan.clone()),
                    Event::Finished(result)=>queue.finished(id,result.clone(),self.client.borrow().cancel.load(Ordering::Relaxed)),
                    Event::Error(error)=>queue.finished(id,Err(error.clone()),self.client.borrow().cancel.load(Ordering::Relaxed)),
                    Event::Page {..}=>queue.finished(id,Err("Respuesta de transferencia inesperada".into()),false),
                }
            }
        }
        self.set_busy(false);
        self.writing.set(false);
        self.cancel_button.set_sensitive(false);
        self.transfer.set(false);
        match event {
            Event::Planned(_)=>self.status.set_text(if self.client.borrow().cancel.load(Ordering::Relaxed){"Preparación de carpeta cancelada"}else{"Carpeta preparada en la cola"}),
            Event::Error(e) => {
                self.next.set_sensitive(false);
                self.status.set_text(&e);
            }
            Event::Finished(result) => match result {
                Ok(()) => {
                    self.progress.set_fraction(1.0);
                    self.status.set_text("Operación completada.");
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
                while let Some(child) = self.list.row_at_index(0) {
                    self.list.remove(&child);
                }
                for (index, entry) in entries.iter().enumerate() {
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    let icon = if entry.directory {
                        "folder-symbolic"
                    } else if entry.regular {
                        "text-x-generic-symbolic"
                    } else {
                        "emblem-symbolic-link-symbolic"
                    };
                    let name_cell = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    name_cell.set_width_request(144);
                    name_cell.set_hexpand(true);
                    name_cell.append(&gtk::Image::from_icon_name(icon));
                    let name = gtk::Label::new(Some(&entry.name.to_string_lossy()));
                    name.set_xalign(0.0);
                    name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                    name.set_hexpand(true);
                    name.set_tooltip_text(Some(&entry.name.to_string_lossy()));
                    name_cell.append(&name);
                    row.append(&name_cell);
                    let kind = gtk::Label::new(Some(if entry.directory {
                        "Carpeta"
                    } else if entry.regular {
                        "Archivo"
                    } else {
                        "Enlace / otro"
                    }));
                    kind.set_width_request(70);
                    kind.set_xalign(0.0);
                    kind.add_css_class("browser-type");
                    row.append(&kind);
                    let size = gtk::Label::new(
                        entry
                            .size
                            .filter(|_| entry.regular)
                            .map(glib::format_size)
                            .as_deref(),
                    );
                    size.set_width_request(76);
                    size.set_xalign(1.0);
                    size.add_css_class("browser-size");
                    row.append(&size);
                    let permissions = gtk::Label::new(Some(&nexuterm::transfer::permission_text(
                        entry.permissions,
                    )));
                    permissions.set_width_request(96);
                    permissions.add_css_class("browser-permissions");
                    permissions.add_css_class("monospace");
                    row.append(&permissions);
                    row.add_css_class(if index % 2 == 0 {
                        "file-even"
                    } else {
                        "file-odd"
                    });
                    let item = gtk::ListBoxRow::new();
                    item.set_child(Some(&row));
                    if entry.name.as_os_str().as_encoded_bytes().starts_with(b".") {
                        item.add_css_class("hidden-file");
                    }
                    self.list.append(&item);
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
                if let Some(state) = self.state.upgrade() {
                    crate::file_browser::apply_columns(
                        self.widget.upcast_ref(),
                        &state.config.borrow().preferences,
                    );
                }
                self.list.invalidate_filter();
                self.apply_pending_view();
                #[cfg(debug_assertions)]
                if let Some(path) = std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR")
                    && self.smoke_step.replace(1) == 0
                {
                    let path = PathBuf::from(path);
                    let index = self
                        .entries
                        .borrow()
                        .iter()
                        .position(|e| Some(e.name.as_os_str()) == path.file_name())
                        .unwrap();
                    let row = self.list.row_at_index(index as i32).unwrap();
                    self.list.select_row(Some(&row));
                    self.list.emit_by_name::<()>("row-activated", &[&row]);
                }
                #[cfg(debug_assertions)]
                if std::env::var_os("NEXUTERM_SMOKE_TRANSFER_QUEUE").is_some(){self.smoke_queue();}
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
    fn smoke_queue(self: &Rc<Self>) {
        use nexuterm::transfer_queue::Status;
        if self.smoke_step.get() == 0 {
            self.smoke_step.set(1);
            self.navigate(std::env::var_os("NEXUTERM_QUEUE_REMOTE").unwrap().into());
            return;
        }
        if self.smoke_step.replace(2) != 1 {
            return;
        }
        let panel = self.clone();
        glib::spawn_future_local(async move {
            let queue = panel.queue.borrow().as_ref().unwrap().clone();
            let local = panel
                .local
                .borrow()
                .as_ref()
                .and_then(Weak::upgrade)
                .unwrap();
            async fn idle(
                q: &Rc<crate::transfer_queue_ui::Controller>,
                panel: &Rc<Panel>,
                minimum: usize,
            ) {
                for _ in 0..600 {
                    let snapshot = q.snapshot();
                    if let Some(job) = snapshot.jobs.iter().find(|j| {
                        matches!(
                            j.status,
                            Status::Failed | Status::Interrupted | Status::Cancelled
                        )
                    }) {
                        panic!("Unexpected queue failure: {}", job.error);
                    }
                    if snapshot.revision > 0
                        && snapshot.jobs.len() >= minimum
                        && snapshot.jobs.iter().all(|j| j.status == Status::Done)
                        && !panel.busy.get()
                    {
                        return;
                    }
                    glib::timeout_future(Duration::from_millis(20)).await;
                }
                panic!("Queue did not finish");
            }
            for _ in 0..300 {
                if queue.snapshot().revision > 0 && !panel.busy.get() && !local.is_loading() {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            let base = PathBuf::from(std::env::var_os("NEXUTERM_QUEUE_LOCAL").unwrap());
            let mode = std::env::var("NEXUTERM_SMOKE_TRANSFER_QUEUE").unwrap();
            if mode == "recover" {
                glib::timeout_future(Duration::from_millis(350)).await;
                assert!(
                    queue
                        .snapshot()
                        .jobs
                        .iter()
                        .any(|j| j.status == Status::Pending
                            && j.local.file_name().is_some_and(|n| n == "resume.txt"))
                );
                assert!(
                    !panel.smoke_has_entry("resume.txt"),
                    "Recovered pending queue must not execute automatically"
                );
                crate::file_browser::smoke_button(&panel.widget, "Iniciar pendientes de la cola")
                    .emit_clicked();
                for _ in 0..600 {
                    if queue.snapshot().jobs.iter().any(|j| {
                        j.status == Status::Done
                            && j.local.file_name().is_some_and(|n| n == "resume.txt")
                    }) {
                        break;
                    }
                    glib::timeout_future(Duration::from_millis(20)).await;
                }
                assert!(
                    queue
                        .snapshot()
                        .jobs
                        .iter()
                        .any(|j| j.status == Status::Done
                            && j.local.file_name().is_some_and(|n| n == "resume.txt"))
                );
                eprintln!("NEXUTERM_SMOKE_TRANSFER_QUEUE_RECOVER_OK");
                panel
                    .window
                    .upgrade()
                    .unwrap()
                    .application()
                    .unwrap()
                    .quit();
                return;
            }
            crate::file_browser::smoke_button(
                &panel.widget,
                "Pausar cola después del archivo actual",
            )
            .emit_clicked();
            local.smoke_copy_many(&["batch-a.txt", "batch-b.txt"]);
            crate::file_browser::smoke_key(&panel.list, gtk::gdk::Key::v);
            for _ in 0..300 {
                if queue.snapshot().jobs.len() == 2 {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            assert_eq!(queue.snapshot().jobs.len(), 2);
            assert!(
                queue
                    .snapshot()
                    .jobs
                    .iter()
                    .all(|j| j.status == Status::Pending)
            );
            assert!(!panel.smoke_has_entry("batch-a.txt"));
            crate::file_browser::smoke_button(&panel.widget, "Iniciar pendientes de la cola")
                .emit_clicked();
            idle(&queue, &panel, 2).await;
            assert_eq!(queue.snapshot().jobs.len(), 2);
            local.smoke_drag_many(&["drag-a.txt", "drag-b.txt"], &panel.widget);
            idle(&queue, &panel, 4).await;
            assert_eq!(queue.snapshot().jobs.len(), 4);
            local.smoke_drag_many(&["folder tree"], &panel.widget);
            idle(&queue, &panel, 8).await;
            assert!(queue.snapshot().jobs.iter().any(|j| j.mkdir));
            local.navigate(base.join("downloads"));
            for _ in 0..300 {
                if !local.is_loading() {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            let select = |names: &[&str]| {
                panel.list.unselect_all();
                for name in names {
                    let index = panel
                        .entries
                        .borrow()
                        .iter()
                        .position(|e| e.name == std::path::Path::new(name))
                        .expect(name);
                    panel
                        .list
                        .select_row(panel.list.row_at_index(index as i32).as_ref());
                }
            };
            select(&["batch-a.txt", "batch-b.txt"]);
            crate::file_browser::smoke_key(&panel.list, gtk::gdk::Key::c);
            crate::file_browser::smoke_button(&local.widget, "Pegar archivo · Ctrl+V")
                .emit_clicked();
            idle(&queue, &panel, 10).await;
            select(&["drag-a.txt", "drag-b.txt"]);
            crate::file_browser::smoke_drag(&panel.list, &local.widget);
            idle(&queue, &panel, 12).await;
            select(&["folder tree"]);
            panel.download();
            idle(&queue, &panel, 16).await;
            for name in [
                "batch-a.txt",
                "batch-b.txt",
                "drag-a.txt",
                "drag-b.txt",
                "folder tree/nested/file %20 á.txt",
            ] {
                assert_eq!(
                    std::fs::read(base.join(name)).unwrap(),
                    std::fs::read(base.join("downloads").join(name)).unwrap()
                );
            }
            assert!(base.join("downloads/folder tree/empty").is_dir());
            std::fs::write(
                base.join("batch-a.txt"),
                b"changed local content must not overwrite remote",
            )
            .unwrap();
            panel.upload_file(base.join("batch-a.txt"));
            for _ in 0..300 {
                if queue
                    .snapshot()
                    .jobs
                    .iter()
                    .any(|j| j.status == Status::Failed)
                {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            assert_eq!(
                queue
                    .snapshot()
                    .jobs
                    .iter()
                    .filter(|j| j.status == Status::Failed)
                    .count(),
                1
            );
            if let Some(state) = panel.state.upgrade() {
                queue.show(&state, &panel);
            }
            glib::timeout_future(Duration::from_millis(300)).await;
            fn find(w: &gtk::Widget, kind: glib::Type) -> Option<gtk::Widget> {
                if w.type_().is_a(kind) {
                    return Some(w.clone());
                }
                let mut c = w.first_child();
                while let Some(w) = c {
                    if let Some(v) = find(&w, kind) {
                        return Some(v);
                    }
                    c = w.next_sibling();
                }
                None
            }
            fn button(w: &gtk::Widget, label: &str) -> Option<gtk::Button> {
                if let Some(b) = w.downcast_ref::<gtk::Button>()
                    && b.label().as_deref() == Some(label)
                {
                    return Some(b.clone());
                }
                let mut c = w.first_child();
                while let Some(w) = c {
                    if let Some(v) = button(&w, label) {
                        return Some(v);
                    }
                    c = w.next_sibling();
                }
                None
            }
            let window = gtk::Window::list_toplevels()
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::Window>().ok())
                .find(|w| w.title().is_some_and(|t| t.starts_with("Transferencias ·")))
                .unwrap();
            let rows = find(window.upcast_ref(), gtk::ListBox::static_type())
                .unwrap()
                .downcast::<gtk::ListBox>()
                .unwrap();
            let failed = queue
                .snapshot()
                .jobs
                .iter()
                .position(|j| j.status == Status::Failed)
                .unwrap();
            rows.select_row(rows.row_at_index(failed as i32).as_ref());
            button(window.upcast_ref(), "Reintentar seleccionado")
                .unwrap()
                .emit_clicked();
            for _ in 0..300 {
                if queue.snapshot().jobs[failed].status == Status::Pending {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            assert_eq!(queue.snapshot().jobs[failed].status, Status::Pending);
            button(window.upcast_ref(), "Iniciar pendientes")
                .unwrap()
                .emit_clicked();
            for _ in 0..300 {
                if queue.snapshot().jobs[failed].status == Status::Failed
                    && queue.snapshot().jobs[failed].attempts == 2
                {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            assert_eq!(queue.snapshot().jobs[failed].status, Status::Failed);
            assert_eq!(queue.snapshot().jobs[failed].attempts, 2);
            // New intent is persisted, but failure has paused the queue.
            panel.upload_file(base.join("resume.txt"));
            for _ in 0..300 {
                if queue
                    .snapshot()
                    .jobs
                    .iter()
                    .any(|j| j.status == Status::Pending)
                {
                    break;
                }
                glib::timeout_future(Duration::from_millis(20)).await;
            }
            assert!(
                queue
                    .snapshot()
                    .jobs
                    .iter()
                    .any(|j| j.status == Status::Pending)
            );
            assert!(!panel.smoke_has_entry("resume.txt"));
            if let Some(state) = panel.state.upgrade() {
                queue.show(&state, &panel);
            }
            glib::timeout_future(Duration::from_millis(250)).await;
            if let Some(path) = std::env::var_os("NEXUTERM_QUEUE_SCREENSHOT") {
                let paintable = gtk::WidgetPaintable::new(Some(&window));
                let snapshot = gtk::Snapshot::new();
                paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
                if let (Some(node), Some(renderer)) = (snapshot.to_node(), window.renderer()) {
                    renderer
                        .render_texture(&node, None)
                        .save_to_png(path)
                        .unwrap();
                }
            }
            eprintln!("NEXUTERM_SMOKE_TRANSFER_QUEUE_BATCH_OK");
            std::fs::write(base.join("queue-ready"), std::process::id().to_string()).unwrap();
        });
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
                self.list.unselect_all();self.list.select_row(Some(&row_for("unicode.txt")));
                crate::file_browser::smoke_key(&self.list, gtk::gdk::Key::c);
                crate::file_browser::smoke_button(&local.widget, "Pegar archivo · Ctrl+V").emit_clicked();
            }
            2 => {
                assert_eq!(
                    std::fs::read(local.directory().join("unicode.txt")).unwrap(),
                    "Texto de prueba UTF-8: áéíóú\n".as_bytes()
                );
                self.smoke_step.set(3);
                local.smoke_copy(&upload_name);
                crate::file_browser::smoke_key(&self.list, gtk::gdk::Key::v);
            }
            3 => {
                assert!(
                    self.entries
                        .borrow()
                        .iter()
                        .any(|entry| entry.name == std::path::Path::new(&upload_name))
                );
                self.smoke_step.set(10);
                self.list.unselect_all();self.list.select_row(Some(&row_for("drag-download.txt")));
                let remote = self.clone();
                glib::timeout_add_local_once(Duration::from_millis(100), move || {
                    crate::file_browser::smoke_drag(&remote.list, &local.widget);
                });
            }
            10 => {
                assert_eq!(std::fs::read(local.directory().join("drag-download.txt")).unwrap(), b"drag download");
                self.smoke_step.set(11);
                let remote = self.clone();
                glib::timeout_add_local_once(Duration::from_millis(100), move || {
                    local.smoke_drag("drag-upload.txt", &remote.widget);
                });
            }
            11 => {
                assert!(self.entries.borrow().iter().any(|entry| entry.name == std::path::Path::new("drag-upload.txt")));
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
                    assert!(remote.widget.clone().upcast::<gtk::Widget>() == page.child() || remote.widget.is_ancestor(&page.child()));
                    let state = remote.state.upgrade().unwrap();
                    assert_eq!(state.local_browsers.borrow().iter().filter_map(std::rc::Weak::upgrade).count(), 1);
                    let local = state.local_files.borrow().as_ref().unwrap().clone();
                    assert!(local.widget.is_mapped());
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
                        "NEXUTERM_SMOKE_FILE_BROWSER_OK navigation=1 download=1 upload=1 clipboard=both drag_drop=both single_local=1 overwrite_blocked=1 closed=1"
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
    fn edit(self: &Rc<Self>, remote: PathBuf) {
        if self.busy.get() {
            return;
        }
        let Some(connection) = self.connection.borrow().clone() else {
            return;
        };
        self.set_busy(true);
        self.status
            .set_text("Descargando una copia temporal para editar…");
        let panel = Rc::downgrade(self);
        let task = nexuterm::runtime::runtime().spawn(nexuterm::remote_edit::RemoteDocument::open(
            connection, remote,
        ));
        glib::spawn_future_local(async move {
            let result = task.await.map_err(|e| e.to_string()).and_then(|r| r);
            let Some(panel) = panel.upgrade() else {
                return;
            };
            panel.set_busy(false);
            match result {
                Ok(document) => {
                    if let (Some(state), Some(window)) =
                        (panel.state.upgrade(), panel.window.upgrade())
                    {
                        panel
                            .status
                            .set_text("Copia temporal abierta; Guardar publica un reemplazo atómico SFTP.");
                        crate::files::remote_editor(&state, &window, document);
                    }
                }
                Err(error) => panel
                    .status
                    .set_text(&format!("No se pudo editar: {error}")),
            }
        });
    }
    fn upload(self: &Rc<Self>) {
        if let Some(local) = self
            .local
            .borrow()
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
        {
            let paths=local.selected_paths();
            if !paths.is_empty() {
                self.upload_files(paths);
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
                        p.upload_file(local);
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
        let local=self.local.borrow().as_ref().and_then(Weak::upgrade);
        if let Some(local)=local {
            let entries=self.entries.borrow();let mut paths=Vec::new();
            for row in self.list.selected_rows(){let Some(entry)=entries.get(row.index() as usize) else{return;};
                if !entry.regular && !entry.directory {self.status.set_text("No se transfieren enlaces ni archivos especiales");return;}
                paths.push((self.current.borrow().join(&entry.name),entry.directory));
            }
            drop(entries);self.download_paths(paths,local.directory());return;
        }

        let Some(row) = self.list.selected_rows().into_iter().next() else {
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
                        if let Some(queue)=p.queue.borrow().clone(){queue.enqueue(&p,vec![nexuterm::transfer_queue::Job::new(false,false,local,remote)]);}else{p.request(Request::Download { remote, local }, true);}
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
