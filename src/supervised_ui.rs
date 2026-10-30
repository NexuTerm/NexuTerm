//! Review exact destination/cwd/command before starting a separately supervised process.
use crate::ui::State;
use gtk::{glib, prelude::*};
use nexuterm::execution::{self, Handle, Record, Spec, Status, Target};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
pub struct Controller {
    window: RefCell<Option<gtk::Window>>,
    jobs: RefCell<Vec<Arc<Handle>>>,
    selected: RefCell<Option<Arc<Handle>>>,
    closing: Cell<bool>,
}
fn controller(state: &Rc<State>) -> Rc<Controller> {
    if let Some(c) = state.supervised.borrow().as_ref() {
        return c.clone();
    }
    let c = Rc::new(Controller {
        window: RefCell::new(None),
        jobs: RefCell::new(Vec::new()),
        selected: RefCell::new(None),
        closing: Cell::new(false),
    });
    *state.supervised.borrow_mut() = Some(c.clone());
    c
}
pub fn approve(
    state: &Rc<State>,
    parent: &gtk::ApplicationWindow,
    spec: Spec,
    target: Target,
    requester: &str,
    callback: Rc<dyn Fn(Result<Arc<Handle>, String>)>,
) {
    approve_when(
        state,
        parent,
        spec,
        target,
        requester,
        Rc::new(|| true),
        callback,
    );
}
fn approve_when(
    state: &Rc<State>,
    parent: &gtk::ApplicationWindow,
    spec: Spec,
    target: Target,
    requester: &str,
    alive: Rc<dyn Fn() -> bool>,
    callback: Rc<dyn Fn(Result<Arc<Handle>, String>)>,
) {
    if let Err(e) = spec.validate() {
        callback(Err(e));
        return;
    }
    let details = format!(
        "Solicitante: {requester}\nDestino: {}\nCarpeta: {}\nTiempo límite: {} s\nHistorial local: {}\n\n{}",
        target.destination().label,
        spec.directory.display(),
        spec.timeout_secs,
        if spec.keep_history {
            "conservar"
        } else {
            "no conservar"
        },
        spec.command
    );
    let dialog = gtk::Window::builder()
        .title("Aprobar ejecución supervisada")
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(650)
        .default_height(430)
        .build();
    dialog.set_widget_name("supervised-approval");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_top(12);
    body.set_margin_bottom(12);
    body.set_margin_start(12);
    body.set_margin_end(12);
    let view = gtk::TextView::builder()
        .editable(false)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .build();
    view.buffer().set_text(&details.replace('\0', "␀"));
    body.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&view)
            .build(),
    );
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let no = gtk::Button::with_label("Rechazar");
    no.set_widget_name("supervised-reject");
    let yes = gtk::Button::with_label("Aprobar y ejecutar");
    yes.set_widget_name("supervised-approve");
    yes.add_css_class("suggested-action");
    actions.append(&no);
    actions.append(&yes);
    body.append(&actions);
    dialog.set_child(Some(&body));
    let decided = Rc::new(Cell::new(false));
    let closed = decided.clone();
    let rejected = callback.clone();
    dialog.connect_close_request(move |_| {
        if !closed.replace(true) {
            rejected(Err(
                "Ejecución rechazada; no se inició ningún proceso".into()
            ));
        }
        glib::Propagation::Proceed
    });
    let weak = dialog.downgrade();
    no.connect_clicked(move |_| {
        if let Some(d) = weak.upgrade() {
            d.close();
        }
    });
    let weak = dialog.downgrade();
    let state = Rc::downgrade(state);
    yes.connect_clicked(move |_| {
        if decided.replace(true) {
            return;
        }
        if !alive() {
            callback(Err("La solicitud caducó; no se ejecutó el comando".into()));
            if let Some(d) = weak.upgrade() {
                d.close();
            }
            return;
        }
        let result = if let Some(state) = state.upgrade() {
            start_approved(&state, spec.clone(), target.clone())
        } else {
            Err("Ventana cerrada; no se inició el comando".into())
        };
        callback(result);
        if let Some(d) = weak.upgrade() {
            d.close();
        }
    });
    dialog.present();
}
fn start_approved(state: &Rc<State>, spec: Spec, target: Target) -> Result<Arc<Handle>, String> {
    let c = controller(state);
    if c.closing.get() {
        return Err("La ventana se está cerrando; no se inició el comando".into());
    }
    let mut jobs = c.jobs.borrow_mut();
    while jobs.len() >= 8 {
        let Some(i) = jobs.iter().position(|h| {
            if !h.finished() {
                return false;
            }
            let r = h.view();
            !r.spec.keep_history || r.recording_error.is_empty()
        }) else {
            return Err("Hay ocho resultados activos o pendientes de conservar. Exporta los que no se hayan podido guardar antes de cerrar; no se ha iniciado otro proceso.".into());
        };
        jobs.remove(i);
    }
    let handle = execution::start(spec, target)?;
    jobs.push(handle.clone());
    *c.selected.borrow_mut() = Some(handle.clone());
    Ok(handle)
}
fn status(record: &Record) -> String {
    let name = match record.status {
        Status::Preparing => "Preparando",
        Status::Running => "En ejecución",
        Status::Completed => {
            if record.exit_code == Some(0) {
                "Finalizada"
            } else {
                "Finalizada con error"
            }
        }
        Status::Cancelled => "Cancelada",
        Status::TimedOut => "Tiempo agotado",
        Status::Unknown => "Resultado desconocido",
        Status::Failed => "No completada",
    };
    format!(
        "{name} · {} ms · retorno {}{}\n{}{}",
        record.duration_ms,
        record
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "no confirmado".into()),
        if record.truncated {
            " · captura truncada a 4 MiB"
        } else {
            ""
        },
        record.error,
        if record.recording_error.is_empty() {
            String::new()
        } else {
            format!("\nHistorial no confirmado: {}", record.recording_error)
        }
    )
    .replace('\0', "␀")
}
fn text_view() -> gtk::TextView {
    gtk::TextView::builder()
        .editable(false)
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .build()
}
fn output(record: &Record) -> String {
    let out = &record.stdout[..record.stdout.len().min(128 * 1024)];
    let err = &record.stderr[..record.stderr.len().min(128 * 1024)];
    format!(
        "Destino: {}\nCarpeta: {}\nComando:\n{}\n\n--- stdout ---\n{}\n\n--- stderr ---\n{}{}",
        record.destination.label,
        record.spec.directory.display(),
        record.spec.command,
        String::from_utf8_lossy(out),
        String::from_utf8_lossy(err),
        if record.stdout.len() > out.len() || record.stderr.len() > err.len() {
            "\n\nVista acotada a 128 KiB por flujo. Exportar salida conserva toda la captura disponible."
        } else {
            ""
        }
    ).replace('\0',"␀")
}
fn export(parent: &gtk::Window, record: Record) {
    let chooser = gtk::FileDialog::builder()
        .title("Exportar salida a un archivo nuevo")
        .initial_name(format!("ejecucion-{}.log", record.id))
        .build();
    let parent = parent.clone();
    glib::spawn_future_local(async move {
        if let Ok(file) = chooser.save_future(Some(&parent)).await {
            let result = if let Some(path) = file.path() {
                nexuterm::runtime::runtime()
                    .spawn_blocking(move || nexuterm::execution_history::export(&record, &path))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
            } else {
                Err("Selecciona un archivo local nuevo".into())
            };
            if let Err(e) = result {
                gtk::AlertDialog::builder()
                    .message("No se pudo exportar")
                    .detail(e)
                    .build()
                    .show(Some(&parent));
            }
        }
    });
}
pub fn show(state: &Rc<State>, parent: &gtk::ApplicationWindow) {
    let c = controller(state);
    if let Some(window) = c.window.borrow().as_ref() {
        window.present();
        return;
    }
    let window = gtk::Window::builder()
        .title("Ejecución supervisada")
        .transient_for(parent)
        .destroy_with_parent(true)
        .default_width(1000)
        .default_height(720)
        .build();
    window.set_widget_name("supervised-execution");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_top(12);
    body.set_margin_bottom(12);
    body.set_margin_start(12);
    body.set_margin_end(12);
    let initial_targets = state.execution_targets();
    let targets = Rc::new(RefCell::new(initial_targets));
    let binding = targets.borrow();
    let labels = binding
        .iter()
        .map(|(_, label, _)| label.as_str())
        .collect::<Vec<_>>();
    let destination = gtk::DropDown::from_strings(&labels);
    destination.set_widget_name("supervised-target");
    let refresh = gtk::Button::with_label("Actualizar destinos");
    let target_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    destination.set_hexpand(true);
    target_row.append(&destination);
    target_row.append(&refresh);
    body.append(&target_row);
    drop(binding);
    let folder = gtk::Entry::builder()
        .placeholder_text("Carpeta absoluta del proceso")
        .build();
    folder.set_widget_name("supervised-directory");
    folder.set_text(
        &state
            .local_files
            .borrow()
            .as_ref()
            .map(|p| p.directory())
            .unwrap_or_else(|| {
                std::env::var_os("HOME")
                    .map(Into::into)
                    .unwrap_or_else(|| "/".into())
            })
            .to_string_lossy(),
    );
    body.append(&folder);
    let command = gtk::TextView::builder()
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .build();
    command.set_widget_name("supervised-command");
    body.append(
        &gtk::ScrolledWindow::builder()
            .min_content_height(90)
            .max_content_height(140)
            .child(&command)
            .build(),
    );
    let options = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    options.append(&gtk::Label::new(Some("Tiempo límite (s)")));
    let timeout = gtk::SpinButton::with_range(1.0, 120.0, 1.0);
    timeout.set_value(60.0);
    timeout.set_widget_name("supervised-timeout");
    options.append(&timeout);
    let keep = gtk::CheckButton::with_label("Conservar historial local");
    keep.set_active(true);
    keep.set_tooltip_text(Some(
        "Desmarca para no conservar el comando y su salida en el historial de ejecuciones.",
    ));
    options.append(&keep);
    let run = gtk::Button::with_label("Revisar ejecución…");
    run.set_widget_name("supervised-run");
    run.add_css_class("suggested-action");
    options.append(&run);
    let history = gtk::Button::with_label("Historial…");
    history.set_widget_name("supervised-history");
    options.append(&history);
    body.append(&options);
    let cancel = gtk::Button::with_label("Cancelar proceso seleccionado");
    cancel.set_widget_name("supervised-cancel");
    let export_button = gtk::Button::with_label("Exportar salida…");
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.append(&cancel);
    controls.append(&export_button);
    let forget = gtk::Button::with_label("Retirar salida de memoria…");
    forget.set_widget_name("supervised-forget");
    controls.append(&forget);
    body.append(&controls);
    let jobs = gtk::ListBox::new();
    jobs.set_selection_mode(gtk::SelectionMode::Single);
    let view = text_view();
    view.set_widget_name("supervised-output");
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.set_vexpand(true);
    split.set_start_child(Some(
        &gtk::ScrolledWindow::builder()
            .min_content_width(230)
            .child(&jobs)
            .build(),
    ));
    split.set_end_child(Some(&gtk::ScrolledWindow::builder().child(&view).build()));
    split.set_position(250);
    body.append(&split);
    let progress = gtk::Label::new(None);
    progress.set_xalign(0.0);
    progress.set_wrap(true);
    progress.set_lines(3);
    progress.set_ellipsize(gtk::pango::EllipsizeMode::End);
    body.append(&progress);
    let notice = gtk::Label::new(Some(
        "Cada aprobación inicia un proceso separado del shell interactivo. Los resultados desconocidos no se reejecutan automáticamente.",
    ));
    notice.set_wrap(true);
    notice.set_xalign(0.0);
    body.append(&notice);
    window.set_child(Some(&body));
    let directory = folder.downgrade();
    destination.connect_selected_notify(move |_| {
        if let Some(f) = directory.upgrade() {
            f.set_text("");
        }
    });
    let update_state = Rc::downgrade(state);
    let update_targets = targets.clone();
    let choice = destination.downgrade();
    refresh.connect_clicked(move |_| {
        if let (Some(s), Some(d)) = (update_state.upgrade(), choice.upgrade()) {
            let list = s.execution_targets();
            let labels = list
                .iter()
                .map(|(_, label, _)| label.as_str())
                .collect::<Vec<_>>();
            if let Some(names) = d.model().and_downcast::<gtk::StringList>() {
                *update_targets.borrow_mut() = list.clone();
                names.splice(0, names.n_items(), &labels);
                d.set_selected(0);
            }
        }
    });
    let weak = window.downgrade();
    window.connect_close_request(move |_| {
        if let Some(w) = weak.upgrade() {
            w.set_visible(false);
        }
        glib::Propagation::Stop
    });
    *c.window.borrow_mut() = Some(window.clone());
    let state_weak = Rc::downgrade(state);
    let parent = parent.downgrade();
    let folder = folder.downgrade();
    let command = command.downgrade();
    let destination = destination.downgrade();
    let timeout = timeout.downgrade();
    let keep = keep.downgrade();
    let msg = notice.downgrade();
    run.connect_clicked(move |_| {
        let (
            Some(state),
            Some(parent),
            Some(folder),
            Some(command),
            Some(destination),
            Some(timeout),
            Some(keep),
        ) = (
            state_weak.upgrade(),
            parent.upgrade(),
            folder.upgrade(),
            command.upgrade(),
            destination.upgrade(),
            timeout.upgrade(),
            keep.upgrade(),
        )
        else {
            return;
        };
        let target = targets
            .borrow()
            .get(destination.selected() as usize)
            .map(|(_, _, target)| target.clone());
        let Some(target) = target else {
            return;
        };
        let b = command.buffer();
        if b.char_count() > 16384 {
            if let Some(msg) = msg.upgrade() {
                msg.set_text("Máximo 16 KiB por comando");
            }
            return;
        }
        let spec = Spec {
            command: b.text(&b.start_iter(), &b.end_iter(), false).into(),
            directory: folder.text().as_str().into(),
            timeout_secs: timeout.value_as_int() as u64,
            keep_history: keep.is_active(),
        };
        let msg = msg.clone();
        approve(
            &state,
            &parent,
            spec,
            target.clone(),
            "Usuario",
            Rc::new(move |result| {
                if let Some(msg) = msg.upgrade() {
                    msg.set_text(&match result {
                        Ok(_) => "Ejecución aprobada; consulta el resultado seleccionado.".into(),
                        Err(e) => e,
                    });
                }
            }),
        );
    });
    let weak = Rc::downgrade(&c);
    cancel.connect_clicked(move |_| {
        if let Some(c) = weak.upgrade() {
            if let Some(h) = c.selected.borrow().as_ref() {
                h.cancel();
            }
        }
    });
    let weak = Rc::downgrade(&c);
    let win = window.downgrade();
    export_button.connect_clicked(move |_| {
        if let (Some(c), Some(win)) = (weak.upgrade(), win.upgrade()) {
            if let Some(h) = c.selected.borrow().as_ref() {
                export(&win, h.snapshot());
            }
        }
    });
    let weak = Rc::downgrade(&c);
    let parent = window.downgrade();
    forget.connect_clicked(move |_|{
        let(Some(c),Some(parent))=(weak.upgrade(),parent.upgrade())else{return;};let Some(handle)=c.selected.borrow().clone()else{return;};if !handle.finished(){return;}
        let r=handle.view();let detail=format!("{} · {}\nSe retirará esta copia en memoria. El historial guardado en disco se conserva. Si no hay una copia guardada, exporta la salida antes de retirarla.",r.destination.label,r.spec.command.chars().take(120).collect::<String>());
        let dialog=gtk::AlertDialog::builder().message("¿Retirar esta salida de memoria?").detail(&detail).buttons(["Cancelar","Retirar"]).cancel_button(0).default_button(0).build();
        glib::spawn_future_local(async move{if dialog.choose_future(Some(&parent)).await==Ok(1){c.jobs.borrow_mut().retain(|h|!Arc::ptr_eq(h,&handle));let matches=c.selected.borrow().as_ref().is_some_and(|h|Arc::ptr_eq(h,&handle));if matches{*c.selected.borrow_mut()=None;}}});
    });
    let weak = Rc::downgrade(&c);
    jobs.connect_row_selected(move |_, row| {
        if let (Some(c), Some(row)) = (weak.upgrade(), row) {
            if let Some(h) = c.jobs.borrow().get(row.index() as usize) {
                *c.selected.borrow_mut() = Some(h.clone());
            }
        }
    });
    let win = window.downgrade();
    history.connect_clicked(move |_| {
        if let Some(win) = win.upgrade() {
            show_history(&win);
        }
    });
    let weak = Rc::downgrade(&c);
    let list = jobs.downgrade();
    let output_view = view.downgrade();
    let stop = cancel.downgrade();
    let forget = forget.downgrade();
    let progress = progress.downgrade();
    let visible = window.downgrade();
    let mut signatures = Vec::new();
    let mut previous = String::new();
    glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
        let (Some(c), Some(list), Some(view), Some(stop)) = (
            weak.upgrade(),
            list.upgrade(),
            output_view.upgrade(),
            stop.upgrade(),
        ) else {
            return glib::ControlFlow::Break;
        };
        if visible.upgrade().is_none_or(|w| !w.is_visible()) {
            return glib::ControlFlow::Continue;
        }
        let handles = c.jobs.borrow();
        let ids = handles.iter().map(|h| h.id()).collect::<Vec<_>>();
        if signatures != ids {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            for h in handles.iter() {
                let r = h.view();
                let label = gtk::Label::new(Some(&format!(
                    "{}\n{}",
                    r.destination.label,
                    r.spec.command.chars().take(50).collect::<String>()
                )));
                label.set_xalign(0.0);
                label.set_wrap(true);
                list.append(&label);
            }
            signatures = ids;
        }
        if let Some(h) = c.selected.borrow().as_ref() {
            let r = h.view();
            if let Some(progress) = progress.upgrade() {
                let text = status(&r);
                progress.set_text(&text);
                progress.set_tooltip_text(Some(&text));
            }
            let current = format!(
                "{}\nVista en directo: hasta 128 KiB por flujo. Exportar salida conserva la captura completa.",
                output(&r)
            );
            if current != previous {
                view.buffer().set_text(&current);
                previous = current;
            }
            stop.set_sensitive(!h.finished());
            if let Some(forget) = forget.upgrade() {
                forget.set_sensitive(h.finished());
            }
        } else {
            stop.set_sensitive(false);
            if let Some(forget) = forget.upgrade() {
                forget.set_sensitive(false);
            }
            if !previous.is_empty() {
                view.buffer()
                    .set_text("Selecciona una ejecución para consultar su resultado.");
                previous.clear();
                if let Some(progress) = progress.upgrade() {
                    progress.set_text("");
                }
            }
        }
        glib::ControlFlow::Continue
    });
    window.present();
}
fn show_history(parent: &gtk::Window) {
    let window = gtk::Window::builder()
        .title("Historial de ejecuciones")
        .transient_for(parent)
        .destroy_with_parent(true)
        .default_width(900)
        .default_height(650)
        .build();
    window.set_widget_name("supervised-history-window");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let names = gtk::StringList::new(&[]);
    let selector = gtk::DropDown::builder()
        .model(&names)
        .enable_search(true)
        .build();
    body.append(&selector);
    let view = text_view();
    body.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&view)
            .build(),
    );
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let remove = gtk::Button::with_label("Eliminar registro…");
    let save = gtk::Button::with_label("Exportar salida…");
    actions.append(&remove);
    actions.append(&save);
    body.append(&actions);
    window.set_child(Some(&body));
    let entries = Rc::new(RefCell::new(Vec::<String>::new()));
    let record = Rc::new(RefCell::new(None::<Record>));
    let weak_view = view.downgrade();
    let ids = entries.clone();
    let selected_record = record.clone();
    let revision = Rc::new(Cell::new(0u64));
    let loading = Rc::new(Cell::new(false));
    let pending = Rc::new(Cell::new(false));
    selector.connect_selected_notify(move |s| {
        revision.set(revision.get() + 1);
        let epoch = revision.get();
        if loading.get() {
            if !pending.replace(true) {
                let pending = pending.clone();
                let selector = s.downgrade();
                glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
                    pending.set(false);
                    if let Some(s) = selector.upgrade() {
                        s.notify("selected");
                    }
                });
            }
            return;
        }
        let revision = revision.clone();
        let Some(id) = ids.borrow().get(s.selected() as usize).cloned() else {
            return;
        };
        loading.set(true);
        let loading = loading.clone();
        let view = weak_view.clone();
        let record = selected_record.clone();
        *record.borrow_mut() = None;
        glib::spawn_future_local(async move {
            let result = nexuterm::runtime::runtime()
                .spawn_blocking(move || {
                    nexuterm::execution_history::read(
                        &nexuterm::execution_history::directory()?,
                        &id,
                    )
                })
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            loading.set(false);
            if revision.get() != epoch {
                return;
            }
            if let Some(view) = view.upgrade() {
                match result {
                    Ok(r) => {
                        view.buffer()
                            .set_text(&format!("{}\n\n{}", status(&r), output(&r)));
                        *record.borrow_mut() = Some(r);
                    }
                    Err(e) => view.buffer().set_text(&e),
                }
            }
        });
    });
    let weak = window.downgrade();
    let selected = record.clone();
    save.connect_clicked(move |_| {
        if let (Some(w), Some(r)) = (weak.upgrade(), selected.borrow().clone()) {
            export(&w, r);
        }
    });
    let weak = window.downgrade();
    let ids = entries.clone();
    let choice = selector.downgrade();
    let model = names.clone();
    let record_ref = record.clone();
    let weak_view = view.downgrade();
    remove.connect_clicked(move |_| {
        let (Some(w), Some(choice)) = (weak.upgrade(), choice.upgrade()) else {
            return;
        };
        let Some(id) = ids.borrow().get(choice.selected() as usize).cloned() else {
            return;
        };
        let dialog = gtk::AlertDialog::builder()
            .message("¿Eliminar este registro y su salida?")
            .buttons(["Cancelar", "Eliminar"])
            .cancel_button(0)
            .default_button(0)
            .build();
        let (view, ids, model, record) = (
            weak_view.clone(),
            ids.clone(),
            model.clone(),
            record_ref.clone(),
        );
        glib::spawn_future_local(async move {
            if dialog.choose_future(Some(&w)).await != Ok(1) {
                return;
            }
            let target = id.clone();
            let result = nexuterm::runtime::runtime()
                .spawn_blocking(move || {
                    nexuterm::execution_history::remove(
                        &nexuterm::execution_history::directory()?,
                        &target,
                    )
                })
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            if let Some(view) = view.upgrade() {
                match result {
                    Ok(()) => {
                        *record.borrow_mut() = None;
                        let index = ids.borrow().iter().position(|v| v == &id);
                        if let Some(i) = index {
                            ids.borrow_mut().remove(i);
                            model.splice(i as u32, 1, &[]);
                        }
                        view.buffer().set_text("Registro eliminado.");
                    }
                    Err(e) => view.buffer().set_text(&e),
                }
            }
        });
    });
    let names = names.clone();
    let view = view.downgrade();
    let selector = selector.downgrade();
    glib::spawn_future_local(async move {
        let result = nexuterm::runtime::runtime()
            .spawn_blocking(move || {
                nexuterm::execution_history::list(&nexuterm::execution_history::directory()?)
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()));
        match result {
            Ok(list) => {
                let labels = list
                    .iter()
                    .map(|e| {
                        let date = glib::DateTime::from_unix_local(e.started_at as i64)
                            .ok()
                            .and_then(|d| d.format("%d/%m %H:%M:%S").ok())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Sin fecha".into());
                        format!("{date} · {}", e.label)
                    })
                    .collect::<Vec<_>>();
                let labels = labels.iter().map(String::as_str).collect::<Vec<_>>();
                *entries.borrow_mut() = list.iter().map(|e| e.id.clone()).collect();
                names.splice(0, names.n_items(), &labels);
                if let Some(s) = selector.upgrade() {
                    s.set_selected(0);
                    s.notify("selected");
                }
            }
            Err(e) => {
                if let Some(view) = view.upgrade() {
                    view.buffer().set_text(&e);
                }
            }
        }
    });
    window.present();
}

pub fn handle_mcp(state: &Rc<State>, request: nexuterm::terminal_mcp::Request) {
    use nexuterm::terminal_mcp::result;
    use serde_json::json;
    if request.name == "supervised_targets" {
        let values=state.execution_targets().into_iter().map(|(id,label,target)|json!({"target_id":id,"label":label,"remote":target.destination().remote})).collect::<Vec<_>>();
        let _ = request
            .response
            .send(result(json!({"targets":values}).to_string(), false));
        return;
    }
    let id = request.arguments["target_id"].as_str().unwrap_or("");
    let target = state
        .execution_targets()
        .into_iter()
        .find(|(target, _, _)| target == id)
        .map(|(_, _, t)| t);
    let Some(target) = target else {
        let _ = request.response.send(result(
            "Destino inexistente: consulta supervised_targets y especifica su identificador exacto"
                .into(),
            true,
        ));
        return;
    };
    let Some(directory) = request.arguments["directory"].as_str() else {
        let _ = request
            .response
            .send(result("Falta la carpeta absoluta explícita".into(), true));
        return;
    };
    if request
        .arguments
        .get("timeout_seconds")
        .is_some_and(|v| v.as_u64().is_none())
        || request
            .arguments
            .get("keep_history")
            .is_some_and(|v| v.as_bool().is_none())
    {
        let _ = request.response.send(result(
            "Tiempo límite o conservación de historial con tipo inválido".into(),
            true,
        ));
        return;
    }
    let spec = Spec {
        command: request.arguments["command"].as_str().unwrap_or("").into(),
        directory: directory.into(),
        timeout_secs: request.arguments["timeout_seconds"].as_u64().unwrap_or(60),
        keep_history: request.arguments["keep_history"].as_bool().unwrap_or(true),
    };
    let Some(parent) = state.window() else {
        let _ = request
            .response
            .send(result("La ventana está cerrada".into(), true));
        return;
    };
    let sender = Rc::new(RefCell::new(Some(request.response)));
    let live = sender.clone();
    let alive: Rc<dyn Fn() -> bool> =
        Rc::new(move || live.borrow().as_ref().is_some_and(|s| !s.is_closed()));
    approve_when(
        state,
        &parent,
        spec,
        target,
        &request.agent,
        alive,
        Rc::new(move |answer| {
            let sender = sender.clone();
            match answer {
                Err(e) => {
                    if let Some(s) = sender.borrow_mut().take() {
                        let _ = s.send(result(e, true));
                    }
                }
                Ok(h) => {
                    glib::spawn_future_local(async move {
                        while !h.finished() {
                            if sender.borrow().as_ref().is_none_or(|s| s.is_closed()) {
                                h.cancel();
                            }
                            glib::timeout_future(std::time::Duration::from_millis(50)).await;
                        }
                        let r = h.snapshot();
                        let tail = |bytes: &[u8]| {
                            String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(4096)..])
                                .into_owned()
                        };
                        let text=json!({"execution_id":r.id,"status":format!("{:?}",r.status),"exit_code":r.exit_code,"duration_ms":r.duration_ms,"stdout":tail(&r.stdout),"stderr":tail(&r.stderr),"capture_truncated":r.truncated,"reply_truncated":r.stdout.len()>4096||r.stderr.len()>4096,"error":r.error.chars().take(512).collect::<String>(),"recording_error":r.recording_error.chars().take(512).collect::<String>(),"history_saved":r.spec.keep_history&&r.recording_error.is_empty()}).to_string();
                        if let Some(s) = sender.borrow_mut().take() {
                            let _ = s.send(result(
                                text,
                                r.status != Status::Completed || r.exit_code != Some(0),
                            ));
                        }
                    });
                }
            }
        }),
    );
}

#[cfg(debug_assertions)]
#[path = "supervised_smoke.rs"]
mod smoke;
#[cfg(debug_assertions)]
pub fn smoke(state: &Rc<State>) {
    smoke::run(state);
}

pub fn busy(state: &State) -> bool {
    state
        .supervised
        .borrow()
        .as_ref()
        .is_some_and(|c| c.jobs.borrow().iter().any(|h| !h.finished()))
}
pub fn closing(state: &State) -> bool {
    state
        .supervised
        .borrow()
        .as_ref()
        .is_some_and(|c| c.closing.get())
}
pub async fn finish_before_close(state: &Rc<State>) -> Result<(), String> {
    let Some(c) = state.supervised.borrow().clone() else {
        return Ok(());
    };
    c.closing.set(true);
    let handles = c.jobs.borrow().clone();
    for handle in &handles {
        if !handle.finished() {
            handle.cancel();
        }
    }
    while handles.iter().any(|h| !h.finished()) {
        glib::timeout_future(std::time::Duration::from_millis(50)).await;
    }
    c.closing.set(false);
    for handle in &handles {
        let r = handle.view();
        if r.spec.keep_history && !r.recording_error.is_empty() {
            return Err(format!(
                "{} · {}: {}. La salida sigue disponible en Ejecución supervisada para exportarla.",
                r.destination.label,
                r.spec.command.chars().take(80).collect::<String>(),
                r.recording_error
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binary_output_is_displayed_without_embedded_nuls() {
        let r = Record {
            version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            spec: Spec {
                command: "printf fixture".into(),
                directory: "/tmp".into(),
                timeout_secs: 60,
                keep_history: false,
            },
            destination: execution::Destination {
                id: "local".into(),
                label: "Local".into(),
                remote: false,
            },
            started_at: 0,
            duration_ms: 0,
            status: Status::Completed,
            exit_code: Some(0),
            stdout: vec![b'A', 0, 255],
            stderr: vec![0],
            truncated: false,
            error: String::new(),
            recording_error: String::new(),
        };
        let text = output(&r);
        assert!(!text.contains('\0'));
        assert!(text.contains('␀'));
        assert_eq!(r.stdout, vec![b'A', 0, 255]);
    }
}
