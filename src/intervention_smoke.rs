use super::*;
fn named(w: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if w.widget_name() == name {
        return Some(w.clone());
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        if let Some(v) = named(&c, name) {
            return Some(v);
        }
    }
    None
}
fn top(name: &str) -> Option<gtk::Window> {
    gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Window>().ok())
        .find(|w| w.widget_name() == name)
}
fn click(w: &gtk::Window, name: &str) {
    named(w.upcast_ref(), name)
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}
fn text(w: &gtk::Window, name: &str, value: &str) {
    let widget = named(w.upcast_ref(), name).unwrap();
    if let Some(v) = widget.downcast_ref::<gtk::TextView>() {
        v.buffer().set_text(value);
    } else {
        widget.downcast::<gtk::Entry>().unwrap().set_text(value);
    }
}
fn contents(w: &gtk::Window, name: &str) -> String {
    let b = named(w.upcast_ref(), name)
        .unwrap()
        .downcast::<gtk::TextView>()
        .unwrap()
        .buffer();
    b.text(&b.start_iter(), &b.end_iter(), false).into()
}
async fn until(mut f: impl FnMut() -> bool) {
    for _ in 0..600 {
        if f() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(25)).await;
    }
    panic!("Intervention smoke timed out");
}
async fn ready(c: &Controller) {
    until(|| !c.busy.get()).await;
}
fn read(c: &Controller) -> Intervention {
    Store::default_path()
        .unwrap()
        .read(c.id.borrow().as_ref().unwrap())
        .unwrap()
}
fn button_label(w: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    if let Some(b) = w.downcast_ref::<gtk::Button>() {
        if b.label().is_some_and(|l| l == label) {
            return Some(b.clone());
        }
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        if let Some(b) = button_label(&c, label) {
            return Some(b);
        }
    }
    None
}
async fn answer(label: &str) {
    until(|| {
        gtk::Window::list_toplevels()
            .iter()
            .any(|w| button_label(w, label).is_some())
    })
    .await;
    for w in gtk::Window::list_toplevels() {
        if let Some(b) = button_label(&w, label) {
            b.emit_clicked();
            break;
        }
    }
}
async fn approve(w: &gtk::Window) {
    click(w, "supervised-run");
    until(|| top("supervised-approval").is_some()).await;
    click(&top("supervised-approval").unwrap(), "supervised-approve");
}
pub(super) fn run(state: &Rc<State>) {
    let state = state.clone();
    glib::spawn_future_local(async move {
        let parent = state.window().unwrap();
        show(&state, &parent);
        let w = top("interventions").unwrap();
        let c = controller(&state);
        ready(&c).await;
        let mode = std::env::var("NEXUTERM_SMOKE_INTERVENTIONS").unwrap();
        let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
        if mode == "restore" {
            let selector = named(w.upcast_ref(), "intervention-selected")
                .unwrap()
                .downcast::<gtk::DropDown>()
                .unwrap();
            until(|| selector.model().is_some_and(|m| m.n_items() > 1)).await;
            selector.set_selected(1);
            until(|| c.id.borrow().is_some() && !c.busy.get()).await;
            assert_eq!(contents(&w, "intervention-note"), "pendiente de recuperar");
            assert_eq!(read(&c).entries.len(), 7);
            assert_eq!(
                std::fs::read_to_string(home.join("crash-counter")).unwrap(),
                "once\n"
            );
            assert_eq!(
                std::fs::read_to_string(home.join("counter-proof")).unwrap(),
                "once\n"
            );
            click(&w, "intervention-preview");
            until(|| contents(&w, "intervention-report").contains("Unknown")).await;
            let record = read(&c)
                .entries
                .last()
                .unwrap()
                .evidence
                .as_ref()
                .unwrap()
                .execution_id
                .clone();
            nexuterm::execution_history::remove(
                &nexuterm::execution_history::directory().unwrap(),
                &record,
            )
            .unwrap();
            click(&w, "intervention-preview");
            until(|| contents(&w, "intervention-report").contains("evidencia ausente")).await;
            let id = c.id.borrow().clone().unwrap();
            click(&w, "intervention-delete");
            answer("Eliminar").await;
            until(|| c.id.borrow().is_none() && !c.busy.get()).await;
            assert!(Store::default_path().unwrap().read(&id).is_err());
            crate::supervised_ui::show_linked(
                &state,
                &parent,
                Link {
                    intervention_id: id.clone(),
                    phase: Phase::Change,
                },
            );
            let execution = top("supervised-execution").unwrap();
            text(&execution, "supervised-directory", &home.to_string_lossy());
            text(
                &execution,
                "supervised-command",
                "touch deleted-context-never",
            );
            approve(&execution).await;
            until(|| top("supervised-approval").is_none()).await;
            assert!(!home.join("deleted-context-never").exists());
            assert!(Store::default_path().unwrap().read(&id).is_err());
            eprintln!("NEXUTERM_SMOKE_INTERVENTION_RESTORE_OK");
            parent.application().unwrap().quit();
            return;
        }
        click(&w, "intervention-new");
        let create = top("intervention-create").unwrap();
        text(&create, "intervention-title", "Incidencia de laboratorio");
        text(
            &create,
            "intervention-objective",
            "Diagnosticar y verificar sin repetir acciones",
        );
        text(&create, "intervention-targets", "local");
        click(&create, "intervention-create-confirm");
        until(|| c.id.borrow().is_some() && !c.busy.get()).await;
        let id = c.id.borrow().clone().unwrap();
        if mode == "close" {
            text(&w, "intervention-note", "último cambio antes de cerrar");
            eprintln!("NEXUTERM_INTERVENTION_CLOSE_REQUESTED");
            parent.close();
            glib::spawn_future_local(async move {
                for _ in 0..100 {
                    for w in gtk::Window::list_toplevels() {
                        if let Some(b) = button_label(&w, "Cerrar") {
                            b.emit_clicked();
                            return;
                        }
                    }
                    glib::timeout_future(std::time::Duration::from_millis(25)).await;
                }
            });
            return;
        }
        text(&w, "intervention-note", "secreto-nota-no-exportar");
        named(w.upcast_ref(), "intervention-excluded")
            .unwrap()
            .downcast::<gtk::CheckButton>()
            .unwrap()
            .set_active(true);
        click(&w, "intervention-register");
        until(|| read(&c).entries.len() == 1 && !c.busy.get()).await;
        text(&w, "intervention-note", "Diagnóstico registrado");
        named(w.upcast_ref(), "intervention-excluded")
            .unwrap()
            .downcast::<gtk::CheckButton>()
            .unwrap()
            .set_active(false);
        click(&w, "intervention-register");
        until(|| read(&c).entries.len() == 2 && !c.busy.get()).await;
        named(w.upcast_ref(), "intervention-phase")
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap()
            .set_selected(3);
        click(&w, "intervention-execute");
        let execution = top("supervised-execution").unwrap();
        text(&execution, "supervised-directory", &home.to_string_lossy());
        let journal = Store::default_path()
            .unwrap()
            .directory
            .join(format!("{id}.json"));
        let command = format!(
            "grep -q '\"phase\":\"Approval\"' {} || exit 17; printf 'evidence-private'; echo once >> counter-proof",
            nexuterm::ssh::quote(&journal.to_string_lossy())
        );
        text(&execution, "supervised-command", &command);
        click(&execution, "supervised-run");
        until(|| top("supervised-approval").is_some()).await;
        assert_eq!(read(&c).entries.len(), 2);
        assert!(!home.join("counter-proof").exists());
        click(&top("supervised-approval").unwrap(), "supervised-reject");
        assert_eq!(read(&c).entries.len(), 2);
        approve(&execution).await;
        until(|| read(&c).entries.len() == 4).await;
        let reference = read(&c)
            .entries
            .last()
            .unwrap()
            .evidence
            .as_ref()
            .unwrap()
            .execution_id
            .clone();
        until(|| {
            nexuterm::execution_history::read(
                &nexuterm::execution_history::directory().unwrap(),
                &reference,
            )
            .is_ok_and(|r| r.status == nexuterm::execution::Status::Completed)
        })
        .await;
        let record = nexuterm::execution_history::read(
            &nexuterm::execution_history::directory().unwrap(),
            &reference,
        )
        .unwrap();
        assert_eq!(record.exit_code, Some(0));
        assert_eq!(record.stdout, b"evidence-private");
        assert_eq!(
            std::fs::read_to_string(home.join("counter-proof")).unwrap(),
            "once\n"
        );
        click(&w, "intervention-preview");
        until(|| !contents(&w, "intervention-report").is_empty() && !c.busy.get()).await;
        let public = contents(&w, "intervention-report");
        assert!(!public.contains("secreto-nota"));
        assert!(!public.contains("counter-proof"));
        assert!(!public.contains("evidence-private"));
        named(w.upcast_ref(), "intervention-export-commands")
            .unwrap()
            .downcast::<gtk::CheckButton>()
            .unwrap()
            .set_active(true);
        assert!(contents(&w, "intervention-report").is_empty());
        named(w.upcast_ref(), "intervention-export-output")
            .unwrap()
            .downcast::<gtk::CheckButton>()
            .unwrap()
            .set_active(true);
        named(w.upcast_ref(), "intervention-export-format")
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap()
            .set_selected(1);
        click(&w, "intervention-preview");
        until(|| contents(&w, "intervention-report").contains("evidence-private") && !c.busy.get())
            .await;
        let reviewed =
            contents(&w, "intervention-report").replace("evidence-private", "[REDACTADO]");
        text(&w, "intervention-report", &reviewed);
        let value: serde_json::Value = serde_json::from_str(&reviewed).unwrap();
        assert_eq!(value["intervention_id"], id);
        interventions::export(
            &home.join("reviewed.json"),
            &contents(&w, "intervention-report"),
        )
        .unwrap();
        assert!(
            !std::fs::read_to_string(home.join("reviewed.json"))
                .unwrap()
                .contains("evidence-private")
        );
        click(&w, "intervention-attach");
        until(|| top("intervention-attach-dialog").is_some()).await;
        click(
            &top("intervention-attach-dialog").unwrap(),
            "intervention-attach-confirm",
        );
        until(|| read(&c).entries.len() == 5 && !c.busy.get()).await;
        assert_eq!(
            std::fs::read_to_string(home.join("counter-proof")).unwrap(),
            "once\n"
        );
        text(&w, "intervention-note", "pendiente de recuperar");
        named(w.upcast_ref(), "intervention-phase")
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap()
            .set_selected(4);
        until(|| read(&c).draft.text == "pendiente de recuperar" && !c.busy.get()).await;
        w.present();
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        if let Some(path) = std::env::var_os("NEXUTERM_INTERVENTION_SCREENSHOT") {
            let p = gtk::WidgetPaintable::new(Some(&w));
            let snapshot = gtk::Snapshot::new();
            p.snapshot(&snapshot, w.width() as f64, w.height() as f64);
            let node=snapshot.to_node().expect("Intervention must be visibly rendered");
            let renderer=w.renderer().expect("Intervention renderer");
            renderer.render_texture(&node,None).save_to_png(path).unwrap();
        }
        if mode == "capture" {
            click(&w, "intervention-execute");
            text(
                &execution,
                "supervised-command",
                "echo once >> crash-counter; printf partial; sleep 3",
            );
            approve(&execution).await;
            until(|| read(&c).entries.len() == 7).await;
            let reference = read(&c)
                .entries
                .last()
                .unwrap()
                .evidence
                .as_ref()
                .unwrap()
                .execution_id
                .clone();
            until(|| {
                nexuterm::execution_history::read(
                    &nexuterm::execution_history::directory().unwrap(),
                    &reference,
                )
                .is_ok_and(|r| {
                    r.status == nexuterm::execution::Status::Running && r.stdout == b"partial"
                })
            })
            .await;
            eprintln!("NEXUTERM_INTERVENTION_CAPTURE_PID={}", std::process::id());
            return;
        }
        eprintln!("NEXUTERM_SMOKE_INTERVENTION_OK");
        parent.application().unwrap().quit();
    });
}
