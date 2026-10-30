//! A shared, explicitly resumable queue for each saved connection destination.
use crate::{
    remote_panel::Panel,
    ui::{State, icon_button},
};
use gtk::{glib, prelude::*};
use nexuterm::{
    config::Profile,
    transfer::Planned,
    transfer_queue::{self, Edit, Job, Journal, Snapshot, Status},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
    time::Duration,
};
type Loaded = Result<(Arc<Journal>, Snapshot), String>;
pub(crate) struct Controller {
    ready: tokio::sync::watch::Receiver<Option<Loaded>>,
    initialized: Cell<bool>,
    state: RefCell<Snapshot>,
    message: RefCell<String>,
    recovered: Cell<bool>,
    running: Cell<bool>,
    selecting: Cell<bool>,
    scheduled: Cell<bool>,
    pending_batches: Cell<usize>,
    writing: Rc<Cell<bool>>,
    writers: Cell<usize>,
    owners: Cell<usize>,
    title: String,
    active: RefCell<Option<String>>,
    runner: RefCell<Weak<Panel>>,
    window: RefCell<glib::WeakRef<gtk::Window>>,
}
struct Saving(Rc<Controller>);
impl Drop for Saving {
    fn drop(&mut self) {
        let count = self.0.writers.get() - 1;
        self.0.writers.set(count);
        self.0.writing.set(count > 0);
    }
}
impl Controller {
    pub fn attach(state: &Rc<State>, profile: &Profile, panel: &Rc<Panel>) -> Rc<Self> {
        let scope = transfer_queue::scope(profile);
        let existing = state
            .transfer_queues
            .borrow()
            .get(&scope)
            .and_then(Weak::upgrade);
        let controller = existing.unwrap_or_else(|| {
            let (send, ready) = tokio::sync::watch::channel(None);
            let key = scope.clone();
            nexuterm::runtime::runtime().spawn(async move {
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        let journal = Arc::new(Journal::open(&transfer_queue::directory()?, &key)?);
                        let snapshot = journal.snapshot()?;
                        Ok((journal, snapshot))
                    })
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|v| v);
                send.send_replace(Some(result));
            });
            let queue = Rc::new(Self {
                ready,
                initialized: Cell::new(false),
                state: RefCell::new(Snapshot {
                    version: 1,
                    scope: scope.clone(),
                    revision: 0,
                    jobs: Vec::new(),
                }),
                message: RefCell::new("Cargando cola…".into()),
                recovered: Cell::new(true),
                running: Cell::new(false),
                selecting: Cell::new(false),
                scheduled: Cell::new(false),
                pending_batches: Cell::new(0),
                writing: Rc::new(Cell::new(false)),
                writers: Cell::new(0),
                owners: Cell::new(0),
                title: format!("{} · {}:{}", profile.name, profile.host, profile.port),
                active: RefCell::new(None),
                runner: RefCell::new(Weak::new()),
                window: RefCell::new(glib::WeakRef::new()),
            });
            state
                .writes
                .borrow_mut()
                .push(Rc::downgrade(&queue.writing));
            state
                .transfer_queues
                .borrow_mut()
                .retain(|_, q| q.upgrade().is_some());
            state
                .transfer_queues
                .borrow_mut()
                .insert(scope.clone(), Rc::downgrade(&queue));
            let q = queue.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = q.load().await {
                    q.error(e);
                }
            });
            queue
        });
        controller.owners.set(controller.owners.get() + 1);
        let tools = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let open = gtk::Button::with_label("Transferencias…");
        let start = icon_button(
            "media-playback-start-symbolic",
            "Iniciar pendientes de la cola",
        );
        let pause = icon_button(
            "media-playback-pause-symbolic",
            "Pausar cola después del archivo actual",
        );
        let label = gtk::Label::new(None);
        label.set_hexpand(true);
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        tools.append(&open);
        tools.append(&start);
        tools.append(&pause);
        tools.append(&label);
        panel.widget.append(&tools);
        let q = controller.clone();
        let p = Rc::downgrade(panel);
        let s = Rc::downgrade(state);
        open.connect_clicked(move |_| {
            if let (Some(state), Some(panel)) = (s.upgrade(), p.upgrade()) {
                q.show(&state, &panel);
            }
        });
        let q = controller.clone();
        let p = Rc::downgrade(panel);
        start.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                q.start(&p);
            }
        });
        let q = controller.clone();
        pause.connect_clicked(move |_| q.pause());
        let q = Rc::downgrade(&controller);
        let label = label.downgrade();
        glib::timeout_add_local(Duration::from_millis(200), move || {
            let (Some(q), Some(label)) = (q.upgrade(), label.upgrade()) else {
                return glib::ControlFlow::Break;
            };
            let model = q.state.borrow();
            let done = model
                .jobs
                .iter()
                .filter(|j| j.status == Status::Done)
                .count();
            let text = if q.running.get() {
                format!("{done}/{} · en curso", model.jobs.len())
            } else {
                format!("{done}/{} · {}", model.jobs.len(), q.message.borrow())
            };
            label.set_text(&text);
            label.set_tooltip_text(Some(&text));
            glib::ControlFlow::Continue
        });
        controller
    }
    fn saving(self: &Rc<Self>) -> Saving {
        self.writers.set(self.writers.get() + 1);
        self.writing.set(true);
        Saving(self.clone())
    }
    fn pause(&self) {
        self.running.set(false);
        self.recovered.set(true);
        *self.message.borrow_mut() = "Cola pausada; pulsa Iniciar para continuar.".into();
    }
    fn error(&self, error: String) {
        self.running.set(false);
        self.recovered.set(true);
        *self.message.borrow_mut() = error;
    }
    fn apply(&self, snapshot: Snapshot) {
        if snapshot.revision >= self.state.borrow().revision {
            *self.state.borrow_mut() = snapshot;
        }
    }
    async fn load(self: &Rc<Self>) -> Result<Arc<Journal>, String> {
        let mut ready = self.ready.clone();
        loop {
            let value = ready.borrow().clone();
            if let Some(result) = value {
                let (journal, snapshot) = result?;
                if !self.initialized.replace(true) {
                    self.recovered
                        .set(snapshot.jobs.iter().any(|j| j.status != Status::Done));
                    *self.message.borrow_mut() = if self.recovered.get() {
                        "Cola recuperada y detenida; revisa e inicia los pendientes.".into()
                    } else {
                        "Sin transferencias pendientes".into()
                    };
                    self.apply(snapshot);
                }
                return Ok(journal);
            }
            ready
                .changed()
                .await
                .map_err(|_| "No se pudo cargar la cola")?;
        }
    }
    async fn edit(self: &Rc<Self>, edit: Edit) -> Result<(Snapshot, Option<Job>), String> {
        if self.writers.get() >= 16
            && matches!(&edit, Edit::Retry(_) | Edit::Cancel(_) | Edit::ClearDone)
        {
            return Err("Espera a que terminen las operaciones pendientes de la cola".into());
        }
        let _saving = self.saving();
        let journal = self.load().await?;
        let result = nexuterm::runtime::runtime()
            .spawn_blocking(move || journal.edit(edit))
            .await
            .map_err(|e| e.to_string())??;
        self.apply(result.0.clone());
        Ok(result)
    }
    pub fn enqueue(self: &Rc<Self>, panel: &Rc<Panel>, jobs: Vec<Job>) {
        if jobs.is_empty() {
            return;
        }
        if jobs.len() > 128 || self.pending_batches.get() >= 4 {
            self.error(
                "Espera a guardar los lotes pendientes (máximo 128 elementos por lote).".into(),
            );
            return;
        }
        self.pending_batches.set(self.pending_batches.get() + 1);
        let q = self.clone();
        let panel = Rc::downgrade(panel);
        let saving = self.saving();
        glib::spawn_future_local(async move {
            let _saving = saving;
            let result = q.edit(Edit::Add(jobs)).await;
            q.pending_batches.set(q.pending_batches.get() - 1);
            match result {
                Ok(_) => {
                    if q.active.borrow().is_none() && !q.selecting.get() {
                        *q.runner.borrow_mut() = panel;
                    }
                    if !q.recovered.get() {
                        q.running.set(true);
                    }
                    q.kick();
                }
                Err(e) => q.error(e),
            }
        });
    }
    pub fn start(self: &Rc<Self>, panel: &Rc<Panel>) {
        if self.active.borrow().is_none() && !self.selecting.get() {
            *self.runner.borrow_mut() = Rc::downgrade(panel);
        }
        self.running.set(true);
        self.recovered.set(false);
        *self.message.borrow_mut() = "Pendientes preparados".into();
        self.kick();
    }
    fn kick(self: &Rc<Self>) {
        if !self.running.get() || self.selecting.get() || self.active.borrow().is_some() {
            return;
        }
        let Some(panel) = self.runner.borrow().upgrade() else {
            self.running.set(false);
            return;
        };
        if panel.queue_closed() {
            self.running.set(false);
            return;
        }
        if panel.queue_busy() {
            if !self.scheduled.replace(true) {
                let q = Rc::downgrade(self);
                glib::timeout_add_local_once(Duration::from_millis(50), move || {
                    if let Some(q) = q.upgrade() {
                        q.scheduled.set(false);
                        q.kick();
                    }
                });
            }
            return;
        }
        self.selecting.set(true);
        let q = self.clone();
        glib::spawn_future_local(async move {
            let result = q.edit(Edit::Begin).await;
            q.selecting.set(false);
            match result {
                Ok((_, Some(job))) => {
                    if !q.running.get() || panel.queue_closed() {
                        let _ = q.edit(Edit::Defer(job.id)).await;
                        q.kick();
                        return;
                    }
                    *q.active.borrow_mut() = Some(job.id.clone());
                    if let Err(e) = panel.begin_queued(&job) {
                        q.finished(job.id, Err(e), false);
                    }
                }
                Ok((_, None)) => {
                    q.running.set(false);
                    *q.message.borrow_mut() =
                        "Sin pendientes; los fallidos requieren Reintentar.".into();
                }
                Err(e) => q.error(e),
            }
        });
    }
    pub fn finished(self: &Rc<Self>, id: String, result: Result<(), String>, cancelled: bool) {
        let cancelled = cancelled && result.is_err();
        if result.is_err() || cancelled {
            self.running.set(false);
            self.recovered.set(true);
        }
        let q = self.clone();
        let saving = self.saving();
        glib::spawn_future_local(async move {
            let _saving = saving;
            let error = result.err();
            let message = error.clone().unwrap_or_else(|| "Archivo completado".into());
            match q
                .edit(Edit::Finish {
                    id,
                    error,
                    cancelled,
                })
                .await
            {
                Ok(_) => {
                    q.active.borrow_mut().take();
                    *q.message.borrow_mut() = message;
                    q.kick();
                }
                Err(e) => q.error(e),
            }
        });
    }
    pub fn planned(self: &Rc<Self>, id: String, plan: Vec<Planned>) {
        let q = self.clone();
        let saving = self.saving();
        glib::spawn_future_local(async move {
            let _saving = saving;
            let jobs = plan
                .into_iter()
                .map(|p| {
                    let mut job = Job::new(p.upload, false, p.local, p.remote);
                    job.mkdir = p.directory;
                    job
                })
                .collect();
            match q
                .edit(Edit::Expand {
                    id: id.clone(),
                    jobs,
                })
                .await
            {
                Ok(_) => {
                    q.active.borrow_mut().take();
                    *q.message.borrow_mut() = "Carpeta desglosada y guardada en la cola".into();
                    q.kick();
                }
                Err(e) => {
                    q.running.set(false);
                    let _ = q
                        .edit(Edit::Finish {
                            id,
                            error: Some(e.clone()),
                            cancelled: false,
                        })
                        .await;
                    q.active.borrow_mut().take();
                    q.error(e);
                }
            }
        });
    }
    pub fn detach(self: &Rc<Self>, panel: &Panel) {
        self.owners.set(self.owners.get().saturating_sub(1));
        if self.owners.get() == 0 {
            if let Some(window) = self.window.borrow().upgrade() {
                window.close();
            }
        }
        let is_runner = self
            .runner
            .borrow()
            .upgrade()
            .is_some_and(|p| std::ptr::eq(p.as_ref(), panel));
        if is_runner {
            self.running.set(false);
            self.recovered.set(true);
            *self.runner.borrow_mut() = Weak::new();
            if let Some(id) = self.active.borrow_mut().take() {
                let q = self.clone();
                let saving = self.saving();
                glib::spawn_future_local(async move {
                    let _saving = saving;
                    if let Err(e) = q.edit(Edit::Interrupt(id)).await {
                        q.error(e);
                    }
                });
            }
        }
    }
    pub(crate) fn show(self: &Rc<Self>, state: &Rc<State>, panel: &Rc<Panel>) {
        if let Some(window) = self.window.borrow().upgrade() {
            window.present();
            return;
        }
        let window = gtk::Window::builder()
            .title(format!("Transferencias · {}", self.title))
            .default_width(780)
            .default_height(440)
            .build();
        if let Some(parent) = state.window.upgrade() {
            window.set_transient_for(Some(&parent));
        }
        let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let note = gtk::Label::new(Some(
            "Los pendientes recuperados requieren Iniciar. Reintentar empieza el archivo desde cero y no sobrescribe destinos.",
        ));
        note.set_wrap(true);
        root.append(&note);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        root.append(
            &gtk::ScrolledWindow::builder()
                .child(&list)
                .vexpand(true)
                .build(),
        );
        let tools = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        for (text, action) in [
            ("Iniciar pendientes", 0),
            ("Pausar cola", 1),
            ("Reintentar seleccionado", 2),
            ("Cancelar seleccionado", 3),
            ("Retirar completados", 4),
        ] {
            let button = gtk::Button::with_label(text);
            let q = self.clone();
            let p = Rc::downgrade(panel);
            let list = list.downgrade();
            button.connect_clicked(move |_| {
                if action == 0 {
                    if let Some(p) = p.upgrade() {
                        q.start(&p);
                    }
                    return;
                }
                if action == 1 {
                    q.pause();
                    return;
                }
                let id = list.upgrade().and_then(|l| l.selected_row()).and_then(|r| {
                    q.state
                        .borrow()
                        .jobs
                        .get(r.index() as usize)
                        .map(|j| j.id.clone())
                });
                if action == 3 && id.is_some() && id.as_ref() == q.active.borrow().as_ref() {
                    q.pause();
                    if let Some(p) = q.runner.borrow().upgrade() {
                        p.cancel_queued();
                    }
                    return;
                }
                let edit = match (action, id) {
                    (2, Some(id)) => Edit::Retry(id),
                    (3, Some(id)) => Edit::Cancel(id),
                    (4, _) => Edit::ClearDone,
                    _ => {
                        *q.message.borrow_mut() = "Selecciona una transferencia".into();
                        return;
                    }
                };
                let q = q.clone();
                glib::spawn_future_local(async move {
                    if let Err(e) = q.edit(edit).await {
                        q.error(e);
                    } else {
                        *q.message.borrow_mut() =
                            "Cola actualizada; pulsa Iniciar para continuar.".into();
                    }
                });
            });
            tools.append(&button);
        }
        let status = gtk::Label::new(None);
        status.set_wrap(true);
        status.set_lines(2);
        status.set_ellipsize(gtk::pango::EllipsizeMode::End);
        status.set_xalign(0.0);
        root.append(&status);
        root.append(&tools);
        window.set_child(Some(&root));
        *self.window.borrow_mut() = window.downgrade();
        window.present();
        let q = Rc::downgrade(self);
        let list = list.downgrade();
        let status = status.downgrade();
        let revision = Cell::new(u64::MAX);
        glib::timeout_add_local(Duration::from_millis(200), move || {
            let (Some(q), Some(list)) = (q.upgrade(), list.upgrade()) else {
                return glib::ControlFlow::Break;
            };
            if let Some(status) = status.upgrade() {
                status.set_text(&q.message.borrow());
                status.set_tooltip_text(Some(&q.message.borrow()));
            }
            let model = q.state.borrow();
            if revision.get() == model.revision {
                return glib::ControlFlow::Continue;
            }
            revision.set(model.revision);
            let selected = list.selected_row().map(|r| r.index());
            while let Some(row) = list.row_at_index(0) {
                list.remove(&row);
            }
            for job in &model.jobs {
                let status = match job.status {
                    Status::Pending => "Pendiente",
                    Status::Running => "En curso",
                    Status::Done => "Completada",
                    Status::Failed => "Fallida",
                    Status::Cancelled => "Cancelada",
                    Status::Interrupted => "Interrumpida",
                };
                let action = if job.mkdir {
                    "Crear carpeta"
                } else if job.tree {
                    "Preparar carpeta"
                } else if job.upload {
                    "Subir"
                } else {
                    "Descargar"
                };
                let label = gtk::Label::new(Some(&format!(
                    "{status} · {action} · {} → {}",
                    if job.upload {
                        job.local.display()
                    } else {
                        job.remote.display()
                    },
                    if job.upload {
                        job.remote.display()
                    } else {
                        job.local.display()
                    }
                )));
                label.set_xalign(0.0);
                label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                label.set_tooltip_text(Some(&format!(
                    "Local: {}\nRemoto: {}\nIntentos: {}\n{}",
                    job.local.display(),
                    job.remote.display(),
                    job.attempts,
                    job.error
                )));
                list.append(&label);
            }
            if let Some(i) = selected {
                list.select_row(list.row_at_index(i).as_ref());
            }
            glib::ControlFlow::Continue
        });
    }
    #[cfg(debug_assertions)]
    pub fn snapshot(&self) -> Snapshot {
        self.state.borrow().clone()
    }
}
