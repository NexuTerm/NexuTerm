use super::*;
fn named(w: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if w.widget_name() == name {
        return Some(w.clone());
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        if let Some(found) = named(&c, name) {
            return Some(found);
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
fn label_button(w: &gtk::Widget, label: &str) -> Option<gtk::Button> {
    if let Some(b) = w.downcast_ref::<gtk::Button>() {
        if b.label().is_some_and(|l| l == label) {
            return Some(b.clone());
        }
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        child = c.next_sibling();
        if let Some(b) = label_button(&c, label) {
            return Some(b);
        }
    }
    None
}
async fn until(mut predicate: impl FnMut() -> bool) {
    for _ in 0..500 {
        if predicate() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(25)).await;
    }
    panic!("SSH tools GUI timed out");
}
fn field(w: &gtk::Window, name: &str, text: &str) {
    named(w.upcast_ref(), name)
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap()
        .set_text(text);
}
async fn answer(label: &str) {
    until(|| {
        gtk::Window::list_toplevels()
            .iter()
            .any(|w| label_button(w, label).is_some())
    })
    .await;
    for w in gtk::Window::list_toplevels() {
        if let Some(b) = label_button(&w, label) {
            b.emit_clicked();
            break;
        }
    }
}
pub(super) fn run(state: &Rc<State>) {
    let state = state.clone();
    glib::spawn_future_local(async move {
        let parent = state.window().unwrap();
        if std::env::var("NEXUTERM_SMOKE_SSH_TOOLS").as_deref() == Ok("restore") {
            show(&state, &parent);
            let w = top("ssh-tunnels").unwrap();
            let c = controller(&state);
            assert!(c.jobs.borrow().is_empty());
            assert_eq!(state.config.borrow().tunnels.len(), 1);
            let saved = named(w.upcast_ref(), "tunnel-saved")
                .unwrap()
                .downcast::<gtk::DropDown>()
                .unwrap();
            saved.set_selected(1);
            assert!(!busy(&state));
            click(&w, "tunnel-start");
            assert!(c.jobs.borrow().is_empty());
            assert!(
                !gtk::Window::list_toplevels()
                    .iter()
                    .any(|w| label_button(w, "Activar").is_some())
            );
            saved.set_selected(0);
            click(&w, "tunnel-save");
            assert_eq!(state.config.borrow().tunnels.len(), 2);
            assert!(c.jobs.borrow().is_empty());
            saved.set_selected(2);
            label_button(w.upcast_ref(), "Borrar definición…")
                .unwrap()
                .emit_clicked();
            answer("Borrar").await;
            until(|| state.config.borrow().tunnels.len() == 1).await;
            assert!(c.jobs.borrow().is_empty());
            eprintln!("NEXUTERM_SMOKE_SSH_TOOLS_RESTORE_OK");
            parent.application().unwrap().quit();
            return;
        }
        let profile = state
            .config
            .borrow()
            .profiles
            .iter()
            .find(|p| p.id == "remote-editor-fixture")
            .unwrap()
            .clone();
        connect_profile(&state, &parent, &profile);
        until(|| {
            state
                .sessions
                .borrow()
                .iter()
                .filter_map(Weak::upgrade)
                .any(|s| s.connection.as_ref().is_some_and(|c| c.socket().exists()))
        })
        .await;
        let connection = state
            .sessions
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .find_map(|s| s.connection.clone())
            .unwrap();
        let ready = nexuterm::runtime::runtime().spawn({
            let c = connection.clone();
            async move { c.ready().await }
        });
        assert!(ready.await.unwrap().is_ok());
        let root = PathBuf::from(std::env::var_os("NEXUTERM_REMOTE_FIXTURE_ROOT").unwrap());
        let source = root.join("ssh-import-config");
        let before = std::fs::read(&source).unwrap();
        let home = root.clone();
        let path = source.clone();
        let preview = nexuterm::runtime::runtime()
            .spawn_blocking(move || ssh_import::read(&path, &home))
            .await
            .unwrap()
            .unwrap();
        assert!(!preview.warnings.is_empty());
        let count = state.config.borrow().profiles.len();
        preview_import(&state, &parent, preview);
        let preview = top("ssh-import-preview").unwrap();
        assert_eq!(state.config.borrow().profiles.len(), count);
        preview.close();
        let parsed = ssh_import::read(&source, &root).unwrap();
        preview_import(&state, &parent, parsed);
        click(&top("ssh-import-preview").unwrap(), "ssh-import-confirm");
        assert_eq!(state.config.borrow().profiles.len(), count + 1);
        assert_eq!(std::fs::read(&source).unwrap(), before);
        assert!(!root.join("never-imported").exists());
        assert!(
            state
                .config
                .borrow()
                .profiles
                .last()
                .unwrap()
                .ssh_ignore_config
        );
        show(&state, &parent);
        let w = top("ssh-tunnels").unwrap();
        let c = controller(&state);
        let echo = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        echo.set_nonblocking(true).unwrap();
        let echo_port = echo.local_addr().unwrap().port();
        let echo = nexuterm::runtime::runtime().spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let listener = tokio::net::TcpListener::from_std(echo).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut b = [0; 32];
            let n = stream.read(&mut b).await.unwrap();
            stream.write_all(&b[..n]).await.unwrap();
        });
        let port = std::net::TcpListener::bind(("127.0.0.1", 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        field(&w, "tunnel-port", &port.to_string());
        field(&w, "tunnel-destination-port", &echo_port.to_string());
        field(&w, "tunnel-name", "Fixture local");
        click(&w, "tunnel-save");
        assert_eq!(state.config.borrow().tunnels.len(), 1);
        assert!(c.jobs.borrow().is_empty());
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_err());
        click(&w, "tunnel-start");
        answer("Cancelar").await;
        assert!(c.jobs.borrow().is_empty());
        click(&w, "tunnel-start");
        answer("Activar").await;
        until(|| {
            c.jobs
                .borrow()
                .first()
                .is_some_and(|h| h.snapshot().status == Status::Active)
        })
        .await;
        let traffic = nexuterm::runtime::runtime().spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            stream.write_all(b"gui-tunnel").await.unwrap();
            let mut b = [0; 10];
            stream.read_exact(&mut b).await.unwrap();
            assert_eq!(&b, b"gui-tunnel");
        });
        traffic.await.unwrap();
        echo.await.unwrap();
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        if let Some(path) = std::env::var_os("NEXUTERM_SSH_TOOLS_SCREENSHOT") {
            let paintable = gtk::WidgetPaintable::new(Some(&w));
            let snapshot = gtk::Snapshot::new();
            paintable.snapshot(&snapshot, w.width() as f64, w.height() as f64);
            if let (Some(node), Some(renderer)) = (snapshot.to_node(), w.renderer()) {
                renderer
                    .render_texture(&node, None)
                    .save_to_png(path)
                    .unwrap();
            }
        }
        if std::env::var("NEXUTERM_SMOKE_SSH_TOOLS").as_deref() == Ok("close") {
            std::fs::write(root.join("tunnel-close-port"), port.to_string()).unwrap();
            parent.close();
            answer("Cerrar").await;
            glib::timeout_future(std::time::Duration::from_secs(12)).await;
            panic!("Close did not finish");
        }
        click(&w, "tunnel-stop");
        until(|| {
            c.jobs
                .borrow()
                .first()
                .is_some_and(|h| h.snapshot().status == Status::Stopped)
        })
        .await;
        assert!(std::net::TcpStream::connect(("127.0.0.1", port)).is_err());
        eprintln!("NEXUTERM_SMOKE_SSH_TOOLS_OK");
        parent.application().unwrap().quit();
    });
}
