//! GTK checks with isolated, synthetic providers only.
use super::*;
async fn until(mut condition: impl FnMut() -> bool) {
    for _ in 0..200 {
        if condition() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(25)).await;
    }
    panic!("Recent-chat UI did not reach the expected state");
}
fn input(c: &Conversation) -> gtk::TextView {
    find_named(c.page.upcast_ref(), "codex-composer")
        .unwrap()
        .downcast()
        .unwrap()
}
fn send(c: &Conversation) -> gtk::Button {
    find_named(c.page.upcast_ref(), "codex-send")
        .unwrap()
        .downcast()
        .unwrap()
}
fn pages(c: &Conversation) -> gtk::Stack {
    find_named(c.page.upcast_ref(), "ai-chat-pages")
        .unwrap()
        .downcast()
        .unwrap()
}
fn rows(c: &Conversation) -> Vec<gtk::Button> {
    fn visit(w: &gtk::Widget, out: &mut Vec<gtk::Button>) {
        if w.widget_name() == "ai-recent-conversation" {
            out.push(w.clone().downcast().unwrap());
        }
        let mut child = w.first_child();
        while let Some(c) = child {
            visit(&c, out);
            child = c.next_sibling();
        }
    }
    let mut out = Vec::new();
    visit(c.page.upcast_ref(), &mut out);
    out
}
fn text(c: &Conversation) -> String {
    let b = input(c).buffer();
    b.text(&b.start_iter(), &b.end_iter(), false).to_string()
}
async fn native(c: &Conversation, history: &gtk::Button, notebook: &gtk::Notebook) {
    until(|| {
        send(c).is_sensitive()
            && find_named(c.page.upcast_ref(), "ai-native-recent-old-thread").is_some()
    })
    .await;
    assert_eq!(pages(c).visible_child_name().as_deref(), Some("recent"));
    find_named(c.page.upcast_ref(), "workspace-more-recent-chats")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    until(|| find_named(c.page.upcast_ref(), "ai-native-recent-older-thread").is_some()).await;
    find_named(c.page.upcast_ref(), "ai-native-recent-older-thread")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    until(|| (c.chat().snapshot)().messages.len() == 2 && !c.process.current_busy()).await;
    assert_eq!(
        pages(c).visible_child_name().as_deref(),
        Some("conversation")
    );
    c.chat().archive.flush().await.unwrap();
    let id=c.chat().archive.id.clone();
    let total=nexuterm::runtime::runtime().spawn_blocking(move ||nexuterm::chat_history::page(&nexuterm::chat_history::directory().unwrap(),&id,0).unwrap().total).await.unwrap();
    assert_eq!(total,2);
    let previous = c.id.borrow().clone();
    history.emit_clicked();
    until(|| *c.id.borrow() != previous && !rows(c).is_empty() && send(c).is_sensitive()).await;
    rows(c)[0].emit_clicked();
    until(|| send(c).is_sensitive() && (c.chat().snapshot)().codex_thread.is_some()).await;
    assert_eq!(
        pages(c).visible_child_name().as_deref(),
        Some("conversation")
    );
    let previous = c.id.borrow().clone();
    history.emit_clicked();
    until(|| *c.id.borrow() != previous && !rows(c).is_empty() && send(c).is_sensitive()).await;
    input(c).buffer().set_text("Nueva desde recientes");
    assert!((c.chat().snapshot)().codex_thread.is_none());
    assert_eq!(
        pages(c).visible_child_name().as_deref(),
        Some("conversation")
    );
    send(c).emit_clicked();
    until(|| !c.process.current_busy() && (c.chat().snapshot)().messages.len() >= 2).await;
    assert_eq!(notebook.n_pages(), 1);
}
pub(super) fn run(
    state: &Rc<State>,
    side: &gtk::Box,
    notebook: &gtk::Notebook,
    active: &Rc<RefCell<Rc<Conversation>>>,
    history: &gtk::Button,
    launch: Rc<dyn Fn(bool)>,
    selector: &gtk::DropDown,
    order: &Rc<RefCell<Vec<usize>>>,
) {
    let state = state.clone();
    let side = side.clone();
    let notebook = notebook.clone();
    let active = active.clone();
    let history = history.clone();
    let selector = selector.clone();
    let order = order.clone();
    glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(200)).await;
        side.set_visible(true);
        launch(false);
        let c = active.borrow().clone();
        if std::env::var("NEXUTERM_SMOKE_RECENTS").as_deref() == Ok("native") {
            native(&c, &history, &notebook).await;
            eprintln!("NEXUTERM_SMOKE_RECENTS_OK");
            state.window().unwrap().application().unwrap().quit();
            return;
        }
        until(|| send(&c).is_sensitive() && !rows(&c).is_empty()).await;
        assert_eq!(pages(&c).visible_child_name().as_deref(), Some("recent"));
        assert!(input(&c).is_mapped());
        assert!(pages(&c).height() > 180);
        assert!(
            rows(&c)
                .iter()
                .all(|b| !b.label().unwrap().contains("GitHub"))
        );
        let reload = std::env::var("NEXUTERM_SMOKE_RECENTS").as_deref() == Ok("reload");
        if !reload {
            input(&c).buffer().set_text("Nueva conversación Codex");
            assert_eq!(
                pages(&c).visible_child_name().as_deref(),
                Some("conversation")
            );
            let original_id = c.id.borrow().clone();
            send(&c).emit_clicked();
            assert!(c.process.current_busy());
            history.emit_clicked();
            assert_eq!(*c.id.borrow(),original_id,"Opening recents must not close an active response");
            assert_eq!(pages(&c).visible_child_name().as_deref(),Some("recent"));
            until(|| {
                *c.id.borrow() != original_id && rows(&c).len() >= 2 && send(&c).is_sensitive()
            })
            .await;
            assert_eq!(pages(&c).visible_child_name().as_deref(), Some("recent"));
            assert_eq!(text(&c), "");
            assert_eq!(notebook.n_pages(), 1);
            assert!(
                rows(&c)
                    .iter()
                    .all(|b| !b.label().unwrap().contains("GitHub"))
            );
            rows(&c)
                .into_iter()
                .find(|b| b.label().as_deref() == Some("Codex anterior"))
                .unwrap()
                .emit_clicked();
            until(|| send(&c).is_sensitive() && text(&c) == "Borrador Codex").await;
            assert_eq!(
                pages(&c).visible_child_name().as_deref(),
                Some("conversation")
            );
            assert_eq!(notebook.n_pages(), 1);
            input(&c).buffer().set_text("Continuar Codex");
            send(&c).emit_clicked();
            until(|| {
                !c.process.current_busy()
                    && (c.chat().snapshot)()
                        .messages
                        .iter()
                        .any(|m| m.text == "respuesta-2")
            })
            .await;
            let previous = c.id.borrow().clone();
            history.emit_clicked();
            until(|| *c.id.borrow() != previous && send(&c).is_sensitive() && !rows(&c).is_empty())
                .await;
        }
        let second = order
            .borrow()
            .iter()
            .position(|&i| state.config.borrow().ai_agents[i].id == "github-fixture")
            .unwrap();
        selector.set_selected(second as u32);
        until(|| {
            c.agent_id.borrow().as_deref() == Some("github-fixture")
                && rows(&c)
                    .iter()
                    .any(|b| b.label().as_deref() == Some("GitHub anterior"))
        })
        .await;
        assert_eq!(pages(&c).visible_child_name().as_deref(), Some("recent"));
        assert!(
            rows(&c)
                .iter()
                .all(|b| !b.label().unwrap().contains("Codex"))
        );
        if !reload {
            rows(&c)
                .into_iter()
                .find(|b| b.label().as_deref() == Some("GitHub anterior"))
                .unwrap()
                .emit_clicked();
            until(|| send(&c).is_sensitive() && text(&c) == "Borrador GitHub").await;
            input(&c).buffer().set_text("Continuar GitHub");
            send(&c).emit_clicked();
            until(|| {
                !c.process.current_busy()
                    && (c.chat().snapshot)()
                        .messages
                        .iter()
                        .any(|m| m.text == "respuesta-3")
            })
            .await;
            let previous = c.id.borrow().clone();
            history.emit_clicked();
            until(|| *c.id.borrow() != previous && send(&c).is_sensitive() && !rows(&c).is_empty())
                .await;
        }
        assert_eq!(notebook.n_pages(), 1);
        if let (Some(path), Some(window)) = (
            std::env::var_os("NEXUTERM_RECENTS_SCREENSHOT"),
            state.window(),
        ) {
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
        eprintln!("NEXUTERM_SMOKE_RECENTS_OK");
        state.window().unwrap().application().unwrap().quit();
    });
}
