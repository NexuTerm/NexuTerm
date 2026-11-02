//! A recoverable intervention notebook; selecting it never launches an operation.
use crate::ui::State;
use gtk::{glib, prelude::*};
use nexuterm::interventions::{self, Draft, ExportOptions, Intervention, Link, Phase, Store};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub struct Controller {
    window: RefCell<Option<gtk::Window>>,
    id: RefCell<Option<String>>,
    busy: Cell<bool>,
    saved: RefCell<Draft>,
    input: glib::WeakRef<gtk::TextView>,
    phase: glib::WeakRef<gtk::DropDown>,
    excluded: glib::WeakRef<gtk::CheckButton>,
    notice: glib::WeakRef<gtk::Label>,
    closing: Cell<bool>,
    discard_close: Cell<bool>,
    report: glib::WeakRef<gtk::TextView>,
    report_for: Rc<RefCell<Option<String>>>,
}
fn controller(state: &Rc<State>) -> Rc<Controller> {
    if let Some(c) = state.interventions.borrow().as_ref() {
        return c.clone();
    }
    let c = Rc::new(Controller {
        window: RefCell::new(None),
        id: RefCell::new(None),
        busy: Cell::new(false),
        saved: RefCell::new(Draft::default()),
        input: glib::WeakRef::new(),
        phase: glib::WeakRef::new(),
        excluded: glib::WeakRef::new(),
        notice: glib::WeakRef::new(),
        closing: Cell::new(false),
        discard_close: Cell::new(false),
        report: glib::WeakRef::new(),
        report_for: Rc::new(RefCell::new(None)),
    });
    *state.interventions.borrow_mut() = Some(c.clone());
    c
}
fn phase(index: u32) -> Phase {
    match index {
        0 => Phase::Diagnosis,
        1 => Phase::Proposal,
        2 => Phase::Approval,
        3 => Phase::Change,
        4 => Phase::Verification,
        _ => Phase::Note,
    }
}
fn phase_index(value: Phase) -> u32 {
    match value {
        Phase::Diagnosis => 0,
        Phase::Proposal => 1,
        Phase::Approval => 2,
        Phase::Change => 3,
        Phase::Verification => 4,
        Phase::Note => 5,
    }
}
fn draft(c: &Controller) -> Result<Draft, String> {
    let (Some(input), Some(choice), Some(excluded)) =
        (c.input.upgrade(), c.phase.upgrade(), c.excluded.upgrade())
    else {
        return Err("Ventana cerrada".into());
    };
    let b = input.buffer();
    if b.char_count() > 4096 {
        return Err("Máximo 4096 bytes por paso".into());
    }
    let text = b.text(&b.start_iter(), &b.end_iter(), false).to_string();
    if text.len() > 4096 {
        return Err("Máximo 4096 bytes por paso".into());
    }
    Ok(Draft {
        phase: phase(choice.selected()),
        text,
        excluded: excluded.is_active(),
    })
}
fn notify(c: &Controller, message: &str) {
    if let Some(label) = c.notice.upgrade() {
        label.set_text(message);
    }
}
fn set_busy(c: &Controller, busy: bool) {
    c.busy.set(busy);
    if let Some(window) = c.window.borrow().as_ref() {
        window.set_sensitive(!busy);
    }
}
async fn flush(c: &Rc<Controller>) -> Result<(), String> {
    while c.busy.get() {
        glib::timeout_future(std::time::Duration::from_millis(20)).await;
    }
    let Some(id) = c.id.borrow().clone() else {
        return Ok(());
    };
    let current = draft(c)?;
    if current == *c.saved.borrow() {
        return Ok(());
    }
    set_busy(c, true);
    let content = current.clone();
    let expected = c.saved.borrow().clone();
    let result = nexuterm::runtime::runtime()
        .spawn_blocking(move || Store::default_path()?.draft_if(&id, content, expected))
        .await
        .unwrap_or_else(|e| Err(e.to_string()));
    set_busy(c, false);
    result?;
    *c.saved.borrow_mut() = current;
    Ok(())
}
pub fn prepare_close(state: &Rc<State>, parent: &gtk::ApplicationWindow) -> bool {
    let Some(c) = state.interventions.borrow().as_ref().cloned() else {
        return false;
    };
    if c.discard_close.get() {
        return false;
    }
    if c.closing.get() {
        return true;
    }
    if c.id.borrow().is_none() {
        return false;
    }
    if !c.busy.get() && draft(&c).is_ok_and(|d| d == *c.saved.borrow()) {
        return false;
    }
    c.closing.set(true);
    let parent = parent.downgrade();
    glib::spawn_future_local(async move {
        let result = flush(&c).await;
        c.closing.set(false);
        let Some(parent) = parent.upgrade() else {
            return;
        };
        match result {
            Ok(()) => parent.close(),
            Err(e) => {
                notify(&c, &e);
                let warning=gtk::AlertDialog::builder().message("No se pudo conservar el borrador de intervención").detail(format!("{e}\nLa última copia guardada se conserva. Cerrar sin el último texto requiere descartarlo expresamente.")).buttons(["Cancelar cierre","Cerrar sin el último borrador"]).cancel_button(0).default_button(0).build();
                if warning.choose_future(Some(&parent)).await == Ok(1) {
                    c.discard_close.set(true);
                    parent.close();
                }
            }
        }
    });
    true
}
fn text_view(name: &str, editable: bool) -> gtk::TextView {
    let v = gtk::TextView::builder()
        .editable(editable)
        .wrap_mode(gtk::WrapMode::WordChar)
        .monospace(true)
        .build();
    v.set_widget_name(name);
    v
}
fn clear_preview(preview: &gtk::TextView, for_id: &RefCell<Option<String>>) {
    preview.buffer().set_text("");
    for_id.borrow_mut().take();
}
pub fn show(state: &Rc<State>, parent: &gtk::ApplicationWindow) {
    let c = controller(state);
    if let Some(window) = c.window.borrow().as_ref() {
        window.present();
        return;
    }
    let window = gtk::Window::builder()
        .title("Intervenciones")
        .transient_for(parent)
        .destroy_with_parent(true)
        .default_width(1100)
        .default_height(760)
        .build();
    window.set_widget_name("interventions");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let selected = gtk::DropDown::from_strings(&["Selecciona una intervención"]);
    selected.set_widget_name("intervention-selected");
    selected.set_hexpand(true);
    let refresh = gtk::Button::with_label("Actualizar");
    let create = gtk::Button::with_label("Nueva…");
    create.set_widget_name("intervention-new");
    let remove = gtk::Button::with_label("Eliminar…");
    remove.set_widget_name("intervention-delete");
    for w in [
        &selected.clone().upcast::<gtk::Widget>(),
        &refresh.clone().upcast(),
        &create.clone().upcast(),
        &remove.clone().upcast(),
    ] {
        header.append(w);
    }
    root.append(&header);
    let summary = gtk::Label::new(Some(
        "Objetivo, destinos y evidencias privados. Recuperar no ejecuta acciones. Las notas de aprobación no autorizan herramientas.",
    ));
    summary.set_wrap(true);
    summary.set_lines(4);
    summary.set_ellipsize(gtk::pango::EllipsizeMode::End);
    summary.set_xalign(0.0);
    summary.set_widget_name("intervention-summary");
    root.append(&summary);
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.set_vexpand(true);
    let timeline = gtk::Box::new(gtk::Orientation::Vertical, 6);
    split.set_start_child(Some(
        &gtk::ScrolledWindow::builder()
            .min_content_width(320)
            .child(&timeline)
            .build(),
    ));
    let right = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let options = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let commands = gtk::CheckButton::with_label("Incluir comandos y carpetas");
    commands.set_widget_name("intervention-export-commands");
    let outputs = gtk::CheckButton::with_label("Incluir salidas");
    outputs.set_widget_name("intervention-export-output");
    let format = gtk::DropDown::from_strings(&["Markdown", "JSON"]);
    format.set_widget_name("intervention-export-format");
    options.append(&commands);
    options.append(&outputs);
    options.append(&format);
    right.append(&options);
    let report = text_view("intervention-report", true);
    right.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&report)
            .build(),
    );
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let preview = gtk::Button::with_label("Generar vista para revisar");
    preview.set_widget_name("intervention-preview");
    let save = gtk::Button::with_label("Guardar informe revisado…");
    save.set_widget_name("intervention-export");
    actions.append(&preview);
    actions.append(&save);
    right.append(&actions);
    split.set_end_child(Some(&right));
    split.set_position(360);
    root.append(&split);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let stage = gtk::DropDown::from_strings(&[
        "Diagnóstico",
        "Propuesta",
        "Aprobación (nota)",
        "Cambio",
        "Verificación",
        "Nota",
    ]);
    stage.set_widget_name("intervention-phase");
    let excluded = gtk::CheckButton::with_label("Excluir este paso del informe");
    excluded.set_widget_name("intervention-excluded");
    row.append(&stage);
    row.append(&excluded);
    root.append(&row);
    let input = text_view("intervention-note", true);
    root.append(
        &gtk::ScrolledWindow::builder()
            .min_content_height(70)
            .max_content_height(100)
            .child(&input)
            .build(),
    );
    let tools = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let register = gtk::Button::with_label("Registrar paso");
    register.set_widget_name("intervention-register");
    let attach = gtk::Button::with_label("Adjuntar ejecución existente…");
    attach.set_widget_name("intervention-attach");
    let execute = gtk::Button::with_label("Preparar ejecución vinculada…");
    execute.set_widget_name("intervention-execute");
    for b in [&register, &attach, &execute] {
        tools.append(b);
    }
    root.append(&tools);
    let notice = gtk::Label::new(Some(
        "Borrador conservado periódicamente. Revisa y edita el informe antes de compartir; no hay detección automática de secretos.",
    ));
    notice.set_widget_name("intervention-notice");
    notice.set_xalign(0.0);
    notice.set_wrap(true);
    root.append(&notice);
    window.set_child(Some(&root));
    c.input.set(Some(&input));
    c.phase.set(Some(&stage));
    c.excluded.set(Some(&excluded));
    c.notice.set(Some(&notice));
    *c.window.borrow_mut() = Some(window.clone());
    let edited = Rc::downgrade(&c);
    input.buffer().connect_changed(move |_| {
        if let Some(c) = edited.upgrade() {
            c.discard_close.set(false);
        }
    });
    let ids = Rc::new(RefCell::new(Vec::<String>::new()));
    let report_for = c.report_for.clone();
    c.report.set(Some(&report));
    let loading = Rc::new(Cell::new(false));
    let load: Rc<dyn Fn(String)> = {
        let weak = Rc::downgrade(&c);
        let selected = selected.downgrade();
        let summary = summary.downgrade();
        let timeline = timeline.downgrade();
        let preview = report.downgrade();
        let report_for = report_for.clone();
        Rc::new(move |id| {
            let Some(c) = weak.upgrade() else {
                return;
            };
            if c.busy.get() {
                return;
            }
            let selected = selected.clone();
            let summary = summary.clone();
            let timeline = timeline.clone();
            let preview = preview.clone();
            let report_for = report_for.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = flush(&c).await {
                    notify(&c, &e);
                    return;
                }
                set_busy(&c, true);
                let read_id = id.clone();
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || Store::default_path()?.read(&read_id))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()));
                match result {
                    Ok(value) => {
                        *c.id.borrow_mut() = Some(id.clone());
                        *c.saved.borrow_mut() = value.draft.clone();
                        if let (Some(input), Some(stage), Some(excluded)) =
                            (c.input.upgrade(), c.phase.upgrade(), c.excluded.upgrade())
                        {
                            input.buffer().set_text(&value.draft.text);
                            stage.set_selected(phase_index(value.draft.phase));
                            excluded.set_active(value.draft.excluded);
                        }
                        if let Some(summary) = summary.upgrade() {
                            summary.set_text(&format!(
                                "{}\nObjetivo: {}\nDestinos: {}\nID: {}",
                                value.title,
                                value.objective,
                                value.targets.join(", "),
                                value.id
                            ));
                        }
                        if let Some(preview) = preview.upgrade() {
                            clear_preview(&preview, &report_for);
                        }
                        if let Some(timeline) = timeline.upgrade() {
                            render(&c, &timeline, &value);
                        }
                        notify(
                            &c,
                            "Intervención recuperada sin ejecutar acciones. Actualizar recarga pasos; Generar vista consulta evidencias actuales.",
                        );
                    }
                    Err(e) => {
                        *c.id.borrow_mut() = Some(id);
                        *c.saved.borrow_mut() = Draft::default();
                        if let Some(input) = c.input.upgrade() {
                            input.buffer().set_text("");
                        }
                        if let Some(stage) = c.phase.upgrade() {
                            stage.set_selected(0);
                        }
                        if let Some(excluded) = c.excluded.upgrade() {
                            excluded.set_active(false);
                        }
                        if let Some(timeline) = timeline.upgrade() {
                            while let Some(child) = timeline.first_child() {
                                timeline.remove(&child);
                            }
                        }
                        if let Some(preview) = preview.upgrade() {
                            clear_preview(&preview, &report_for);
                        }
                        if let Some(summary) = summary.upgrade() {
                            summary.set_text("Registro no legible: se conserva intacto. Puedes eliminarlo expresamente.");
                        }
                        notify(&c, &e);
                    }
                }
                set_busy(&c, false);
                let _ = selected;
            });
        })
    };
    let update: Rc<dyn Fn()> = {
        let weak = Rc::downgrade(&c);
        let selected = selected.downgrade();
        let ids = ids.clone();
        let loading = loading.clone();
        Rc::new(move || {
            let (Some(c), Some(selected)) = (weak.upgrade(), selected.upgrade()) else {
                return;
            };
            if c.busy.get() {
                return;
            }
            let ids = ids.clone();
            let loading = loading.clone();
            glib::spawn_future_local(async move {
                if let Err(e) = flush(&c).await {
                    notify(&c, &e);
                    return;
                }
                set_busy(&c, true);
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || Store::default_path()?.list())
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()));
                match result {
                    Ok(values) => {
                        loading.set(true);
                        let mut names = vec!["Selecciona una intervención".to_owned()];
                        names.extend(values.iter().map(|(_, title)| title.clone()));
                        let keys = values.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();
                        let index =
                            c.id.borrow()
                                .as_ref()
                                .and_then(|id| keys.iter().position(|v| v == id))
                                .map(|i| i as u32 + 1)
                                .unwrap_or(0);
                        *ids.borrow_mut() = keys;
                        selected.set_model(Some(&gtk::StringList::new(
                            &names.iter().map(String::as_str).collect::<Vec<_>>(),
                        )));
                        selected.set_selected(index);
                        loading.set(false);
                    }
                    Err(e) => notify(&c, &e),
                }
                set_busy(&c, false);
            });
        })
    };
    let choices = ids.clone();
    let get = load.clone();
    let guard = loading.clone();
    selected.connect_selected_notify(move |choice| {
        if guard.get() {
            return;
        }
        if let Some(id) = choice
            .selected()
            .checked_sub(1)
            .and_then(|i| choices.borrow().get(i as usize).cloned())
        {
            get(id);
        }
    });
    let u = update.clone();
    let get = load.clone();
    let weak = Rc::downgrade(&c);
    refresh.connect_clicked(move |_| {
        if let Some(c) = weak.upgrade() {
            if let Some(id) = c.id.borrow().clone() {
                get(id);
            } else {
                u();
            }
        }
    });
    let weak = Rc::downgrade(&c);
    let create_parent = parent.downgrade();
    let update_list = update.clone();
    let load_created = load.clone();
    create.connect_clicked(move |_| {
        let (Some(c), Some(parent)) = (weak.upgrade(), create_parent.upgrade()) else {
            return;
        };
        new_dialog(&c, &parent, update_list.clone(), load_created.clone());
    });
    let weak = Rc::downgrade(&c);
    let get = load.clone();
    register.connect_clicked(move |_| {
        let Some(c) = weak.upgrade() else {
            return;
        };
        let Some(id) = c.id.borrow().clone() else {
            notify(&c, "Crea o abre una intervención");
            return;
        };
        let content = match draft(&c) {
            Ok(v) => v,
            Err(e) => {
                notify(&c, &e);
                return;
            }
        };
        let expected = c.saved.borrow().clone();
        set_busy(&c, true);
        let get = get.clone();
        glib::spawn_future_local(async move {
            let key = id.clone();
            let result = nexuterm::runtime::runtime()
                .spawn_blocking(move || Store::default_path()?.note_if(&key, content, expected))
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            if result.is_ok() {
                *c.saved.borrow_mut() = Draft::default();
                if let Some(input) = c.input.upgrade() {
                    input.buffer().set_text("");
                }
            }
            set_busy(&c, false);
            match result {
                Ok(_) => get(id),
                Err(e) => notify(&c, &e),
            }
        });
    });
    let s = Rc::downgrade(state);
    let weak = Rc::downgrade(&c);
    let execution_parent = state.window.clone();
    execute.connect_clicked(move |_|{let(Some(state),Some(c),Some(parent))=(s.upgrade(),weak.upgrade(),execution_parent.upgrade())else{return;};let Some(id)=c.id.borrow().clone()else{notify(&c,"Abre una intervención");return;};let phase=c.phase.upgrade().map(|s|phase(s.selected())).unwrap_or(Phase::Diagnosis);if !matches!(phase,Phase::Diagnosis|Phase::Change|Phase::Verification){notify(&c,"Elige Diagnóstico, Cambio o Verificación para ejecutar; las notas de aprobación no conceden permisos.");return;}crate::supervised_ui::show_linked(&state,&parent,Link{intervention_id:id,phase});});
    let weak = Rc::downgrade(&c);
    let get = load.clone();
    let parent = parent.downgrade();
    attach.connect_clicked(move |_| {
        if let (Some(c), Some(parent)) = (weak.upgrade(), parent.upgrade()) {
            attach_dialog(&c, &parent, get.clone());
        }
    });
    let weak = Rc::downgrade(&c);
    let report_view = report.downgrade();
    let commands_weak = commands.downgrade();
    let outputs_weak = outputs.downgrade();
    let format_weak = format.downgrade();
    let report_for_preview = report_for.clone();
    preview.connect_clicked(move |_|{let(Some(c),Some(view),Some(commands),Some(outputs),Some(format))=(weak.upgrade(),report_view.upgrade(),commands_weak.upgrade(),outputs_weak.upgrade(),format_weak.upgrade())else{return;};let Some(id)=c.id.borrow().clone()else{notify(&c,"Abre una intervención");return;};let options=ExportOptions{commands:commands.is_active(),output:outputs.is_active(),json:format.selected()==1};let marker=report_for_preview.clone();glib::spawn_future_local(async move{if let Err(e)=flush(&c).await{notify(&c,&e);return;}set_busy(&c,true);let key=id.clone();let result=nexuterm::runtime::runtime().spawn_blocking(move||{let store=Store::default_path()?;interventions::report(&store.read(&key)?,&nexuterm::execution_history::directory()?,options)}).await.unwrap_or_else(|e|Err(e.to_string()));set_busy(&c,false);match result{Ok(text)=>{view.buffer().set_text(&text);*marker.borrow_mut()=Some(id);notify(&c,"Revisa y redacta el texto; solo esta vista revisada se exportará. Salidas limitadas a 64 KiB por flujo; el historial conserva la captura disponible.");},Err(e)=>notify(&c,&e)}});});
    for check in [commands, outputs] {
        let view = report.downgrade();
        let marker = report_for.clone();
        check.connect_toggled(move |_| {
            if let Some(view) = view.upgrade() {
                clear_preview(&view, &marker);
            }
        });
    }
    let view = report.downgrade();
    let marker = report_for.clone();
    format.connect_selected_notify(move |_| {
        if let Some(view) = view.upgrade() {
            clear_preview(&view, &marker);
        }
    });
    let weak = Rc::downgrade(&c);
    let view = report.downgrade();
    let fmt = format.downgrade();
    let marker = report_for.clone();
    let parent = window.downgrade();
    save.connect_clicked(move |_| {
        let (Some(c), Some(view), Some(fmt), Some(parent)) = (
            weak.upgrade(),
            view.upgrade(),
            fmt.upgrade(),
            parent.upgrade(),
        ) else {
            return;
        };
        if marker.borrow().as_ref() != c.id.borrow().as_ref() || marker.borrow().is_none() {
            notify(
                &c,
                "Genera y revisa el informe de esta intervención antes de guardarlo",
            );
            return;
        }
        let b = view.buffer();
        if b.char_count() > interventions::REPORT_MAX as i32 {
            notify(&c, "Informe demasiado grande");
            return;
        }
        let text = b.text(&b.start_iter(), &b.end_iter(), false).to_string();
        let json = fmt.selected() == 1;
        if json && serde_json::from_str::<serde_json::Value>(&text).is_err() {
            notify(&c, "El texto revisado no es JSON válido");
            return;
        }
        glib::spawn_future_local(async move {
            let chooser = gtk::FileDialog::builder()
                .title("Guardar informe revisado en un archivo nuevo")
                .initial_name(if json {
                    "intervencion.json"
                } else {
                    "intervencion.md"
                })
                .build();
            if let Ok(file) = chooser.save_future(Some(&parent)).await {
                if let Some(path) = file.path() {
                    let result = nexuterm::runtime::runtime()
                        .spawn_blocking(move || interventions::export(&path, &text))
                        .await
                        .unwrap_or_else(|e| Err(e.to_string()));
                    notify(
                        &c,
                        &match result {
                            Ok(()) => "Informe guardado; no se sobrescribió ningún archivo".into(),
                            Err(e) => e,
                        },
                    );
                }
            }
        });
    });
    let weak = Rc::downgrade(&c);
    let parent = window.downgrade();
    let u = update.clone();
    let list = timeline.downgrade();
    let summary = summary.downgrade();
    let view = report.downgrade();
    let marker = report_for.clone();
    remove.connect_clicked(move |_|{let(Some(c),Some(parent))=(weak.upgrade(),parent.upgrade())else{return;};let Some(id)=c.id.borrow().clone()else{return;};let u=u.clone();let list=list.clone();let summary=summary.clone();let view=view.clone();let marker=marker.clone();glib::spawn_future_local(async move{if gtk::AlertDialog::builder().message("¿Eliminar esta intervención?").detail("Elimina notas y vínculos, incluido su borrador. No cancela acciones ni elimina el historial de ejecuciones. No se recupera automáticamente.").buttons(["Cancelar","Eliminar"]).cancel_button(0).default_button(0).build().choose_future(Some(&parent)).await!=Ok(1){return;}while c.busy.get(){glib::timeout_future(std::time::Duration::from_millis(20)).await;}set_busy(&c,true);let result=nexuterm::runtime::runtime().spawn_blocking(move||Store::default_path()?.remove(&id)).await.unwrap_or_else(|e|Err(e.to_string()));if result.is_ok(){c.id.borrow_mut().take();*c.saved.borrow_mut()=Draft::default();if let Some(input)=c.input.upgrade(){input.buffer().set_text("");}if let Some(list)=list.upgrade(){while let Some(child)=list.first_child(){list.remove(&child);}}if let Some(summary)=summary.upgrade(){summary.set_text("Intervención eliminada. Las ejecuciones conservadas siguen en su historial.");}if let Some(view)=view.upgrade(){clear_preview(&view,&marker);}}set_busy(&c,false);match result{Ok(())=>u(),Err(e)=>notify(&c,&e)}});});
    let weak = Rc::downgrade(&c);
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let Some(c) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if !c.busy.get()
            && c.id.borrow().is_some()
            && draft(&c).is_ok_and(|d| d != *c.saved.borrow())
        {
            glib::spawn_future_local(async move {
                if let Err(e) = flush(&c).await {
                    notify(&c, &format!("Borrador no conservado: {e}"));
                }
            });
        }
        glib::ControlFlow::Continue
    });
    window.connect_close_request(|w| {
        w.set_visible(false);
        glib::Propagation::Stop
    });
    update();
    window.present();
}
fn render(c: &Rc<Controller>, box_: &gtk::Box, value: &Intervention) {
    while let Some(w) = box_.first_child() {
        box_.remove(&w);
    }
    for entry in &value.entries {
        let label = gtk::Label::new(Some(&format!(
            "{} · {}\n{}{}",
            entry.phase.label(),
            entry.at,
            entry.text,
            entry
                .evidence
                .as_ref()
                .map(|e| format!("\nEjecución: {} · {}", e.execution_id, e.phase.label()))
                .unwrap_or_default()
        )));
        label.set_wrap(true);
        label.set_xalign(0.0);
        label.set_selectable(true);
        box_.append(&label);
        let check = gtk::CheckButton::with_label("Excluir este paso del informe");
        check.set_active(entry.excluded);
        box_.append(&check);
        let weak = Rc::downgrade(c);
        let id = value.id.clone();
        let entry_id = entry.id.clone();
        check.connect_toggled(move |check| {
            let Some(c) = weak.upgrade() else {
                return;
            };
            if c.busy.get() {
                return;
            }
            let toggle = check.downgrade();
            let id = id.clone();
            let entry = entry_id.clone();
            let excluded = check.is_active();
            if let Some(report) = c.report.upgrade() {
                clear_preview(&report, &c.report_for);
            }
            set_busy(&c, true);
            glib::spawn_future_local(async move {
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || Store::default_path()?.exclude(&id, &entry, excluded))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()));
                if let Err(e) = result {
                    if let Some(toggle) = toggle.upgrade() {
                        toggle.set_active(!excluded);
                    }
                    notify(&c, &e);
                }
                set_busy(&c, false);
            });
        });
    }
}
fn new_dialog(
    c: &Rc<Controller>,
    parent: &gtk::ApplicationWindow,
    update: Rc<dyn Fn()>,
    load: Rc<dyn Fn(String)>,
) {
    let w = gtk::Window::builder()
        .title("Nueva intervención")
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(600)
        .default_height(400)
        .build();
    w.set_widget_name("intervention-create");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let title = gtk::Entry::builder()
        .placeholder_text("Título (hasta 120 bytes)")
        .build();
    title.set_widget_name("intervention-title");
    title.set_max_length(120);
    let objective = text_view("intervention-objective", true);
    let targets = gtk::Entry::builder()
        .placeholder_text("Destinos previstos (separados por comas)")
        .build();
    targets.set_widget_name("intervention-targets");
    targets.set_max_length(4096);
    body.append(&title);
    body.append(&gtk::Label::new(Some("Objetivo")));
    body.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&objective)
            .build(),
    );
    body.append(&targets);
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    body.append(&error);
    let accept = gtk::Button::with_label("Crear y conservar");
    accept.set_widget_name("intervention-create-confirm");
    body.append(&accept);
    w.set_child(Some(&body));
    let window = w.downgrade();
    let weak = Rc::downgrade(c);
    accept.connect_clicked(move |_| {
        let (Some(c), Some(w)) = (weak.upgrade(), window.upgrade()) else {
            return;
        };
        let b = objective.buffer();
        if title.text().len() > 120 || b.char_count() > 16384 {
            error.set_text("Título u objetivo demasiado grande");
            return;
        }
        let title = title.text().to_string();
        let objective = b.text(&b.start_iter(), &b.end_iter(), false).to_string();
        let targets = targets
            .text()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect::<Vec<_>>();
        let error = error.clone();
        let update = update.clone();
        let load = load.clone();
        w.set_sensitive(false);
        glib::spawn_future_local(async move {
            if let Err(e) = flush(&c).await {
                error.set_text(&e);
                w.set_sensitive(true);
                return;
            }
            let result = nexuterm::runtime::runtime()
                .spawn_blocking(move || Store::default_path()?.create(title, objective, targets))
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
            match result {
                Ok(value) => {
                    w.close();
                    load(value.id);
                    update();
                }
                Err(e) => {
                    error.set_text(&e);
                    w.set_sensitive(true);
                }
            }
        });
    });
    w.present();
}
fn attach_dialog(c: &Rc<Controller>, parent: &gtk::ApplicationWindow, load: Rc<dyn Fn(String)>) {
    let Some(id) = c.id.borrow().clone() else {
        notify(c, "Abre una intervención");
        return;
    };
    let phase = c
        .phase
        .upgrade()
        .map(|s| phase(s.selected()))
        .unwrap_or(Phase::Diagnosis);
    let c = c.clone();
    let parent = parent.downgrade();
    glib::spawn_future_local(async move {
        set_busy(&c, true);
        let result = nexuterm::runtime::runtime()
            .spawn_blocking(move || {
                nexuterm::execution_history::list(&nexuterm::execution_history::directory()?)
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()));
        set_busy(&c, false);
        let values = match result {
            Ok(v) => v,
            Err(e) => {
                notify(&c, &e);
                return;
            }
        };
        let Some(parent) = parent.upgrade() else {
            return;
        };
        let w = gtk::Window::builder()
            .title("Adjuntar evidencia existente")
            .transient_for(&parent)
            .modal(true)
            .destroy_with_parent(true)
            .default_width(700)
            .build();
        w.set_widget_name("intervention-attach-dialog");
        let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let labels = values.iter().map(|v| v.label.as_str()).collect::<Vec<_>>();
        let choice = gtk::DropDown::from_strings(&labels);
        body.append(&choice);
        let accept = gtk::Button::with_label("Adjuntar sin ejecutar");
        accept.set_widget_name("intervention-attach-confirm");
        body.append(&accept);
        let error = gtk::Label::new(None);
        error.set_wrap(true);
        body.append(&error);
        w.set_child(Some(&body));
        let window = w.downgrade();
        accept.connect_clicked(move |_| {
            let Some(w) = window.upgrade() else {
                return;
            };
            let Some(key) = values.get(choice.selected() as usize).map(|v| v.id.clone()) else {
                return;
            };
            let id = id.clone();
            let load = load.clone();
            let error = error.clone();
            w.set_sensitive(false);
            glib::spawn_future_local(async move {
                let target = id.clone();
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        Store::default_path()?.attach(
                            &target,
                            phase,
                            &nexuterm::execution_history::directory()?,
                            &key,
                        )
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()));
                match result {
                    Ok(_) => {
                        w.close();
                        load(id);
                    }
                    Err(e) => {
                        error.set_text(&e);
                        w.set_sensitive(true);
                    }
                }
            });
        });
        w.present();
    });
}

#[cfg(debug_assertions)]
#[path = "intervention_smoke.rs"]
mod smoke;
#[cfg(debug_assertions)]
pub fn smoke(state: &Rc<State>) {
    smoke::run(state);
}

/// Release retained transient windows before GTK finalizes their parent (GTK 4.14).
pub fn shutdown(state:&State){
    let controller=state.interventions.borrow().as_ref().cloned();
    if let Some(c)=controller {let window=c.window.borrow_mut().take();if let Some(window)=window{window.set_transient_for(None::<&gtk::Window>);window.destroy();}}
}
