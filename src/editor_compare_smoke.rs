use super::*;
fn named(w: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if w.widget_name() == name {
        return Some(w.clone());
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(found) = named(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}
fn buffer(w: &gtk::Widget) -> gtk::TextBuffer {
    if let Ok(view) = w.clone().downcast::<gtk::TextView>() {
        return view.buffer();
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(v) = find_text(&c) {
            return v;
        }
        child = c.next_sibling();
    }
    panic!("No text view")
}
fn find_text(w: &gtk::Widget) -> Option<gtk::TextBuffer> {
    if let Ok(v) = w.clone().downcast::<gtk::TextView>() {
        return Some(v.buffer());
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        if let Some(v) = find_text(&c) {
            return Some(v);
        }
        child = c.next_sibling();
    }
    None
}
async fn until(mut test: impl FnMut() -> bool) {
    for _ in 0..200 {
        if test() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(25)).await;
    }
    panic!("Comparison UI did not reach expected state");
}
fn dialog() -> Option<gtk::Window> {
    gtk::Window::list_toplevels()
        .into_iter()
        .find(|w| w.widget_name() == "editor-comparison")
        .and_then(|w| w.downcast().ok())
}
fn click(w: &gtk::Widget, name: &str) {
    named(w, name)
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}
pub(super) fn run(
    window: glib::WeakRef<gtk::ApplicationWindow>,
    text: glib::WeakRef<gtk::TextBuffer>,
    status: glib::WeakRef<gtk::Label>,
    save: glib::WeakRef<gtk::Button>,
    compare: gtk::Button,
    loaded: Rc<Cell<bool>>,
    busy: Rc<Cell<bool>>,
    recovery: Rc<Tracker>,
) {
    glib::spawn_future_local(async move {
        until(|| loaded.get()).await;
        let text = text.upgrade().unwrap();
        let status = status.upgrade().unwrap();
        let save = save.upgrade().unwrap();
        let path =
            std::path::PathBuf::from(std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR").unwrap());
        let original = "def value():\n    return 1\n";
        let local = "def value():\n    return 42\n";
        let server = "def value():\n    return 99\n";
        let newer = "def value():\n    return 100\n";
        let merged = "def value():\n    return 142\n";
        assert_eq!(recovery.base().as_deref(), Some(original));
        text.set_text(local);
        text.set_modified(true);
        std::fs::write(&path, server).unwrap();
        save.emit_clicked();
        until(|| !busy.get() && status.text().contains("Conflicto")).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), server);
        assert!(text.is_modified());
        compare.emit_clicked();
        until(|| dialog().is_some()).await;
        let d = dialog().unwrap();
        for (name, expected) in [
            ("editor-compare-base", original),
            ("editor-compare-local", local),
            ("editor-compare-remote", server),
        ] {
            assert_eq!(
                snapshot(&buffer(&named(d.upcast_ref(), name).unwrap())).unwrap(),
                expected
            );
        }
        let apply = named(d.upcast_ref(), "editor-merge-apply")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        assert!(!apply.is_sensitive());
        click(d.upcast_ref(), "editor-merge-cancel");
        until(|| !busy.get()).await;
        assert_eq!(snapshot(&text).unwrap(), local);
        compare.emit_clicked();
        until(|| dialog().is_some()).await;
        let d = dialog().unwrap();
        std::fs::write(&path, newer).unwrap();
        click(d.upcast_ref(), "editor-merge-use-remote");
        click(d.upcast_ref(), "editor-merge-apply");
        until(|| {
            named(d.upcast_ref(), "editor-merge-status")
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap()
                .text()
                .contains("volvió a cambiar")
        })
        .await;
        assert_eq!(snapshot(&text).unwrap(), local);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), newer);
        click(d.upcast_ref(), "editor-merge-cancel");
        until(|| !busy.get()).await;
        compare.emit_clicked();
        until(|| dialog().is_some()).await;
        let d = dialog().unwrap();
        assert_eq!(
            snapshot(&buffer(
                &named(d.upcast_ref(), "editor-compare-base").unwrap()
            ))
            .unwrap(),
            original
        );
        assert_eq!(
            snapshot(&buffer(
                &named(d.upcast_ref(), "editor-compare-remote").unwrap()
            ))
            .unwrap(),
            newer
        );
        let result = buffer(&named(d.upcast_ref(), "editor-merge-result").unwrap());
        result.set_text(merged);
        named(d.upcast_ref(), "editor-merge-reviewed")
            .unwrap()
            .downcast::<gtk::CheckButton>()
            .unwrap()
            .set_active(true);
        if let Some(path) = std::env::var_os("NEXUTERM_COMPARE_SCREENSHOT") {
            glib::timeout_future(std::time::Duration::from_millis(200)).await;
            let paintable = gtk::WidgetPaintable::new(Some(&d));
            let image = gtk::Snapshot::new();
            paintable.snapshot(&image, d.width() as f64, d.height() as f64);
            if let (Some(node), Some(renderer)) = (image.to_node(), d.renderer()) {
                renderer
                    .render_texture(&node, None)
                    .save_to_png(path)
                    .unwrap();
            }
        }
        click(d.upcast_ref(), "editor-merge-apply");
        result.set_text("Edición llegada mientras se verificaba");
        until(||named(d.upcast_ref(),"editor-merge-status").unwrap().downcast::<gtk::Label>().unwrap().text().contains("resultado cambió")).await;
        assert_eq!(snapshot(&text).unwrap(),local);
        assert_eq!(std::fs::read_to_string(&path).unwrap(),newer);
        assert!(!named(d.upcast_ref(),"editor-merge-apply").unwrap().is_sensitive());
        click(d.upcast_ref(),"editor-merge-cancel");until(||!busy.get()).await;
        compare.emit_clicked();until(||dialog().is_some()).await;let d=dialog().unwrap();
        let result=buffer(&named(d.upcast_ref(),"editor-merge-result").unwrap());result.set_text(merged);
        named(d.upcast_ref(),"editor-merge-reviewed").unwrap().downcast::<gtk::CheckButton>().unwrap().set_active(true);
        click(d.upcast_ref(),"editor-merge-apply");
        until(|| !busy.get()).await;
        assert_eq!(snapshot(&text).unwrap(), merged);
        assert!(text.is_modified());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), newer);
        let id = recovery.id.clone();
        let draft = nexuterm::runtime::runtime()
            .spawn_blocking(move || {
                nexuterm::recovery::read(
                    &nexuterm::recovery::directory()
                        .unwrap()
                        .join(format!("{id}.json")),
                )
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(draft.base_text.as_deref(), Some(newer));
        assert_eq!(draft.text, merged);
        save.emit_clicked();
        until(|| !busy.get() && !text.is_modified()).await;
        assert_eq!(std::fs::read_to_string(&path).unwrap(), merged);
        eprintln!("NEXUTERM_SMOKE_EDITOR_COMPARE_OK");
        window.upgrade().unwrap().application().unwrap().quit();
    });
}

pub(super) async fn recovered(root: &gtk::Widget, text: &gtk::TextBuffer, tracker: &Tracker) {
    assert!(tracker.base().is_none());
    let local = snapshot(text).unwrap();
    let nexuterm::recovery::Source::Remote { path, .. } = tracker.source() else {
        panic!("Expected remote draft");
    };
    let server = std::fs::read_to_string(&path).unwrap();
    click(root, "editor-compare");
    until(|| dialog().is_some()).await;
    let d = dialog().unwrap();
    let base = buffer(&named(d.upcast_ref(), "editor-compare-base").unwrap());
    assert!(
        snapshot(&base)
            .unwrap()
            .contains("no conserva el texto base")
    );
    click(d.upcast_ref(), "editor-merge-use-local");
    click(d.upcast_ref(), "editor-merge-apply");
    until(|| !d.is_visible()).await;
    assert_eq!(snapshot(text).unwrap(), local);
    assert!(text.is_modified());
    assert_eq!(tracker.base().as_deref(), Some(server.as_str()));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), server);
    eprintln!("NEXUTERM_SMOKE_LEGACY_COMPARISON_OK");
}
