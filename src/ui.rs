use adw::prelude::AdwApplicationWindowExt;
use gtk::{gio, glib, prelude::*};
use nexuterm::{
    config::{self, Config, Profile, Protocol},
    connection,
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::{Rc, Weak},
};
use vte4::prelude::*;

pub struct State {
    pub notebook: adw::TabView,
    pub status: gtk::Label,
    sidebar: gtk::ListBox,
    connections_side: gtk::Box,
    window: glib::WeakRef<gtk::ApplicationWindow>,
    manager: RefCell<glib::WeakRef<gtk::ApplicationWindow>>,
    config: RefCell<Config>,
    path: PathBuf,
    config_writable: bool,
    sessions: RefCell<Vec<Weak<Session>>>,
    focused_terminal: RefCell<glib::WeakRef<vte4::Terminal>>,
    pub documents: RefCell<Vec<glib::WeakRef<gtk::TextBuffer>>>,
    pub writes: RefCell<Vec<Weak<Cell<bool>>>>,
    pub operations: RefCell<Vec<glib::WeakRef<gio::Cancellable>>>,
}

struct Session {
    pid: Cell<Option<i32>>,
    terminal: glib::WeakRef<vte4::Terminal>,
    close_button: glib::WeakRef<gtk::Button>,
    spec: connection::CommandSpec,
    connection: Option<std::sync::Arc<nexuterm::ssh::SshConnection>>,
    cancelled: Cell<bool>,
    cancellable: gio::Cancellable,
}

#[path = "actions.rs"]
mod actions;
#[path = "sessions.rs"]
mod sessions;
pub(crate) use actions::icon_button;
use actions::install_actions;

impl Session {
    fn stop(&self) {
        self.cancelled.set(true);
        self.cancellable.cancel();
        if let Some(pid) = self.pid.take() {
            // VTE creates the child in its own PTY session; signal its process group.
            unsafe {
                libc::kill(-pid, libc::SIGHUP);
            }
        }
    }
}

pub fn build(app: &gtk::Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }
    adw::init().expect("No se pudo iniciar libadwaita");
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::IconTheme::for_display(&display).add_resource_path("/io/nexuterm/NexuTerm/icons");
        let css = gtk::CssProvider::new();
        css.load_from_data(include_str!("style.css"));
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    gtk::Window::set_default_icon_name("io.nexuterm.NexuTerm");
    let path = match config::config_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let loaded = config::load(&path);
    let warning = loaded.as_ref().err().cloned();
    let config = loaded.unwrap_or_default();
    let adw_window = adw::ApplicationWindow::builder()
        .application(app)
        .title("NexuTerm")
        .default_width(1280)
        .default_height(820)
        .build();
    let window: gtk::ApplicationWindow = adw_window.clone().upcast();
    let root = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    header.add_css_class("nexuterm-header");
    let brand = adw::WindowTitle::new("NexuTerm", "Terminal y conexiones remotas");
    header.set_title_widget(Some(&brand));
    let logo = gtk::Image::from_resource("/io/nexuterm/NexuTerm/icons/nexuterm-toolbar.png");
    logo.set_pixel_size(44);
    logo.set_margin_top(0);
    logo.set_margin_bottom(0);
    logo.set_margin_end(10);
    logo.set_can_target(false);
    header.pack_start(&logo);
    let new_terminal = icon_button(
        "utilities-terminal-symbolic",
        "Terminal local · Ctrl+Mayús+T",
    );
    let new_session = icon_button("network-server-symbolic", "Nueva conexión · Ctrl+N");
    let open_file = icon_button("document-open-symbolic", "Abrir archivo · Ctrl+O");
    let toggle_side = icon_button("sidebar-show-symbolic", "Mostrar u ocultar conexiones · F9");
    toggle_side.set_action_name(Some("win.sidebar"));
    header.pack_start(&toggle_side);
    header.pack_start(&new_session);
    let manage = icon_button("view-list-symbolic", "Administrar sesiones · Ctrl+Mayús+S");
    manage.set_action_name(Some("win.sessions"));
    header.pack_start(&manage);
    header.pack_start(&new_terminal);
    let search_button = icon_button(
        "system-search-symbolic",
        "Buscar en la terminal · Ctrl+Mayús+F",
    );
    search_button.set_action_name(Some("win.search"));
    header.pack_end(&search_button);
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menú principal")
        .build();
    header.pack_end(&menu_button);
    root.add_top_bar(&header);
    let context = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    context.set_margin_start(12);
    context.set_margin_end(12);
    context.set_margin_top(0);
    context.set_margin_bottom(0);
    context.add_css_class("toolbar");
    context.add_css_class("nexuterm-tools");
    for (icon, label, action) in [
        (
            "view-dual-symbolic",
            "Dividir en columnas",
            "split-horizontal",
        ),
        ("view-list-symbolic", "Dividir en filas", "split-vertical"),
        ("view-refresh-symbolic", "Reconectar terminal", "reconnect"),
        (
            "network-offline-symbolic",
            "Desconectar terminal",
            "disconnect",
        ),
        (
            "folder-remote-symbolic",
            "Mostrar u ocultar archivos remotos",
            "remote-files",
        ),
    ] {
        let button = icon_button(icon, label);
        button.set_action_name(Some(&format!("win.{action}")));
        context.append(&button);
    }
    root.add_top_bar(&context);
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.add_css_class("nexuterm-split");
    split.set_vexpand(true);
    split.set_position(300);
    split.set_shrink_start_child(false);
    let side = gtk::Box::new(gtk::Orientation::Vertical, 10);
    side.add_css_class("nexuterm-section");
    side.add_css_class("nexuterm-panel");
    side.set_margin_start(12);
    side.set_margin_end(12);
    side.set_margin_top(2);
    side.set_margin_bottom(8);
    side.set_width_request(220);
    let title = gtk::Label::new(Some("CONEXIONES GUARDADAS"));
    title.set_xalign(0.0);
    title.add_css_class("heading");
    side.append(&title);
    let sidebar = gtk::ListBox::new();
    sidebar.set_selection_mode(gtk::SelectionMode::None);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&sidebar)
        .build();
    side.append(&scroll);
    let note = gtk::Label::new(Some("Las credenciales se solicitan al conectar."));
    note.set_wrap(true);
    note.set_xalign(0.0);
    note.add_css_class("dim-label");
    side.append(&note);
    let filter = gtk::SearchEntry::builder()
        .placeholder_text("Buscar conexiones…")
        .build();
    side.prepend(&filter);
    let notebook = adw::TabView::new();
    let tabbar = adw::TabBar::builder()
        .view(&notebook)
        .autohide(false)
        .build();
    let workspace = gtk::Box::new(gtk::Orientation::Vertical, 0);
    workspace.add_css_class("nexuterm-section");
    workspace.add_css_class("nexuterm-workspace");
    workspace.set_overflow(gtk::Overflow::Hidden);
    tabbar.set_overflow(gtk::Overflow::Hidden);
    workspace.set_margin_top(2);
    workspace.set_margin_end(12);
    workspace.set_margin_bottom(8);
    workspace.append(&tabbar);
    workspace.append(&notebook);
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    split.set_start_child(Some(&side));
    split.set_end_child(Some(&workspace));
    split.set_resize_start_child(false);
    root.set_content(Some(&split));
    let status = gtk::Label::new(Some("Listo"));
    status.set_xalign(0.0);
    status.set_margin_start(12);
    status.set_margin_top(6);
    status.set_margin_bottom(6);
    root.add_bottom_bar(&status);
    let state = Rc::new(State {
        notebook,
        status,
        sidebar,
        connections_side: side.clone(),
        window: window.downgrade(),
        manager: RefCell::new(glib::WeakRef::new()),
        config: RefCell::new(config),
        path,
        config_writable: warning.is_none(),
        sessions: RefCell::new(Vec::new()),
        focused_terminal: RefCell::new(glib::WeakRef::new()),
        documents: RefCell::new(Vec::new()),
        writes: RefCell::new(Vec::new()),
        operations: RefCell::new(Vec::new()),
    });
    adw_window.set_content(Some(&root));
    install_actions(&state, &window, app, &side, &menu_button);
    let list = state.sidebar.clone();
    filter.connect_search_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let query = Rc::new(query);
        list.set_filter_func(move |row| {
            fn text(w: &gtk::Widget) -> String {
                if let Some(l) = w.downcast_ref::<gtk::Label>() {
                    return l.text().to_string();
                }
                let mut result = String::new();
                let mut child = w.first_child();
                while let Some(c) = child {
                    result.push_str(&text(&c));
                    child = c.next_sibling();
                }
                result
            }
            text(row.upcast_ref())
                .to_lowercase()
                .contains(query.as_str())
        });
    });
    refresh_sidebar(&state, &window);
    new_terminal.set_action_name(Some("win.new-terminal"));
    new_session.set_action_name(Some("win.new-connection"));
    open_file.set_action_name(Some("win.open-file"));
    let force_close = Rc::new(Cell::new(false));
    let force = force_close.clone();
    let s = state.clone();
    window.connect_close_request(move |w| {
        if s.writes
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .any(|busy| busy.get())
        {
            s.status
                .set_text("Espera a que finalicen las escrituras o cancela las transferencias antes de cerrar.");
            return glib::Propagation::Stop;
        }
        let active = s
            .sessions
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .any(|h| h.pid.get().is_some());
        let dirty = s
            .documents
            .borrow()
            .iter()
            .filter_map(glib::WeakRef::upgrade)
            .any(|b| b.is_modified());
        if !force.get() && (active || dirty) {
            let dialog = gtk::AlertDialog::builder()
                .modal(true)
                .message("¿Cerrar NexuTerm?")
                .detail("Se cerrarán las conexiones activas y se perderán los cambios sin guardar.")
                .buttons(["Cancelar", "Cerrar"])
                .cancel_button(0)
                .default_button(0)
                .build();
            let w = w.clone();
            let force = force.clone();
            glib::spawn_future_local(async move {
                if dialog.choose_future(Some(&w)).await == Ok(1) {
                    force.set(true);
                    w.close();
                }
            });
            return glib::Propagation::Stop;
        }
        for h in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
            h.stop();
        }
        for c in s
            .operations
            .borrow()
            .iter()
            .filter_map(glib::WeakRef::upgrade)
        {
            c.cancel();
        }
        glib::Propagation::Proceed
    });
    let s = state.clone();
    app.connect_shutdown(move |_| {
        for h in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
            h.stop();
        }
        for c in s
            .operations
            .borrow()
            .iter()
            .filter_map(glib::WeakRef::upgrade)
        {
            c.cancel();
        }
    });
    local_terminal(&state);
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_SSH_LAB").is_some()
    {
        let lab = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab");
        connect_profile(
            &state,
            &window,
            &Profile {
                id: "nexuterm-smoke-lab".into(),
                name: "Laboratorio SSH local".into(),
                host: "127.0.0.1".into(),
                port: 22222,
                username: "tester".into(),
                identity: lab.join("id_ed25519").to_string_lossy().into(),
                ..Profile::default()
            },
        );
    }
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_SESSIONS").is_some()
    {
        sessions::show(&state, &window);
    }
    if let Some(e) = warning {
        state.status.set_text(&format!(
            "Configuración no cargada; guardado deshabilitado para conservarla: {e}"
        ));
    }
    window.present();
    // Optional deterministic local UI smoke check, with no network or user-profile writes.
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some() {
        #[cfg(debug_assertions)]
        if std::env::var_os("NEXUTERM_SMOKE_MENUS").is_some() {
            actions::smoke_menus(&window, &menu_button);
        }
        #[cfg(debug_assertions)]
        if std::env::var_os("NEXUTERM_SMOKE_SESSION_CRUD").is_some() {
            let s = Rc::downgrade(&state);
            glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
                if let Some(s) = s.upgrade() {
                    sessions::smoke(&s);
                }
            });
        }
        if std::env::var_os("NEXUTERM_SMOKE_COMPACT").is_some() {
            window.set_default_size(1024, 768);
        }
        if std::env::var_os("NEXUTERM_SMOKE_ACTIONS").is_some() {
            let w = window.downgrade();
            let s = Rc::downgrade(&state);
            glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
                if let (Some(w), Some(s)) = (w.upgrade(), s.upgrade()) {
                    for action in [
                        "split-horizontal",
                        "split-vertical",
                        "zoom-in",
                        "zoom-out",
                        "sidebar",
                        "sidebar",
                    ] {
                        gio::prelude::ActionGroupExt::activate_action(&w, action, None);
                    }
                    let count = s.sessions.borrow().iter().filter_map(Weak::upgrade).count();
                    assert_eq!(count, 3, "Las divisiones deben crear tres terminales");
                    #[cfg(debug_assertions)]
                    if std::env::var_os("NEXUTERM_SMOKE_CLOSE_SPLITS").is_some() {
                        actions::smoke_close_splits(&s, &w);
                    }
                    eprintln!("NEXUTERM_SMOKE_ACTIONS_OK terminals={count}");
                }
            });
        }
        let manager = state.manager.borrow().clone();
        let app = app.downgrade();
        let window = window.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_secs(3), move || {
            if let (Some(path), Some(window)) =
                (std::env::var_os("NEXUTERM_SCREENSHOT"), window.upgrade())
            {
                let window = manager.upgrade().unwrap_or(window);
                let paintable = gtk::WidgetPaintable::new(Some(&window));
                let snapshot = gtk::Snapshot::new();
                paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
                if let (Some(node), Some(renderer)) = (snapshot.to_node(), window.renderer())
                    && let Err(e) = renderer.render_texture(&node, None).save_to_png(path)
                {
                    eprintln!("Captura: {e}");
                }
            }
            if let Some(app) = app.upgrade() {
                app.quit();
            }
        });
    }
}

pub fn append_tab(
    notebook: &adw::TabView,
    child: &impl IsA<gtk::Widget>,
    name: &str,
    on_close: impl Fn() + 'static,
) {
    let page = notebook.append(child);
    page.set_title(name);
    notebook.set_selected_page(&page);
    let target = page.downgrade();
    let handler = Rc::new(RefCell::new(None));
    let own_handler = handler.clone();
    *handler.borrow_mut() = Some(notebook.connect_close_page(move |view, page| {
        if target.upgrade().as_ref() == Some(page) {
            on_close();
            view.close_page_finish(page, true);
            if let Some(id) = own_handler.borrow_mut().take() {
                view.disconnect(id);
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    }));
}

fn local_terminal(state: &Rc<State>) {
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some() {
        terminal(
            state,
            "Terminal local",
            connection::CommandSpec {
                program: "/bin/sh".into(),
                args: vec![
                    "-c".into(),
                    "printf 'NexuTerm: PTY operativo\\n'; sleep 1".into(),
                ],
            },
        );
        return;
    }
    let shell = std::env::var("SHELL")
        .ok()
        .filter(|s| std::path::Path::new(s).is_absolute())
        .unwrap_or_else(|| "/bin/sh".into());
    terminal(
        state,
        "Terminal local",
        connection::CommandSpec {
            program: shell,
            args: vec!["-l".into()],
        },
    );
}

fn terminal(state: &Rc<State>, title: &str, spec: connection::CommandSpec) {
    terminal_in(state, title, spec, None, None);
}
fn terminal_in(
    state: &Rc<State>,
    title: &str,
    spec: connection::CommandSpec,
    container: Option<&gtk::Box>,
    connection: Option<std::sync::Arc<nexuterm::ssh::SshConnection>>,
) {
    let term = vte4::Terminal::new();
    term.set_color_background(&gtk::gdk::RGBA::new(0.12, 0.12, 0.12, 1.0));
    term.set_scrollback_lines(state.config.borrow().preferences.scrollback.into());
    term.set_hexpand(true);
    term.set_vexpand(true);
    let focus = gtk::EventControllerFocus::new();
    let s = Rc::downgrade(state);
    let t = term.downgrade();
    focus.connect_enter(move |_| {
        if let Some(s) = s.upgrade() {
            *s.focused_terminal.borrow_mut() = t.clone();
            actions::update_actions(&s);
        }
    });
    term.add_controller(focus);
    term.set_font(Some(&gtk::pango::FontDescription::from_string(
        &state.config.borrow().preferences.font,
    )));
    let keys = gtk::EventControllerKey::new();
    let t = term.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if modifiers
            .contains(gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK)
            && let Some(t) = t.upgrade()
        {
            if key == gtk::gdk::Key::C || key == gtk::gdk::Key::c {
                t.copy_clipboard_format(vte4::Format::Text);
                return glib::Propagation::Stop;
            }
            if key == gtk::gdk::Key::V || key == gtk::gdk::Key::v {
                t.paste_clipboard();
                return glib::Propagation::Stop;
            }
        }
        glib::Propagation::Proceed
    });
    term.add_controller(keys);
    let page = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    page.add_css_class("nexuterm-terminal");
    page.set_overflow(gtk::Overflow::Hidden);
    page.append(&term);
    let scrollbar = gtk::Scrollbar::new(gtk::Orientation::Vertical, term.vadjustment().as_ref());
    let controls = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let close = icon_button("window-close-symbolic", "Cerrar esta división");
    close.set_visible(false);
    controls.append(&close);
    scrollbar.set_vexpand(true);
    controls.append(&scrollbar);
    page.append(&controls);
    let handle = Rc::new(Session {
        terminal: term.downgrade(),
        close_button: close.downgrade(),
        spec: spec.clone(),
        connection,
        pid: Cell::new(None),
        cancelled: Cell::new(false),
        cancellable: gio::Cancellable::new(),
    });
    let s = Rc::downgrade(state);
    let h = Rc::downgrade(&handle);
    close.connect_clicked(move |_| {
        if let (Some(s), Some(h)) = (s.upgrade(), h.upgrade()) {
            actions::close_split(&s, &h);
        }
    });
    state.sessions.borrow_mut().retain(|s| s.strong_count() > 0);
    state.sessions.borrow_mut().push(Rc::downgrade(&handle));
    if let Some(container) = container {
        container.append(&page);
    } else {
        let s = Rc::downgrade(state);
        let p = page.downgrade();
        append_tab(&state.notebook, &page, title, move || {
            if let (Some(s), Some(p)) = (s.upgrade(), p.upgrade()) {
                for session in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
                    if session
                        .terminal
                        .upgrade()
                        .is_some_and(|t| t.is_ancestor(&p))
                    {
                        session.stop();
                    }
                }
            }
        });
    }
    let h = handle.clone();
    term.connect_child_exited(move |t, code| {
        h.pid.set(None);
        if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some() {
            eprintln!("NEXUTERM_SMOKE_CHILD_EXIT={code}");
        }
        t.feed(format!("\r\n[Proceso terminado: {code}]\r\n").as_bytes());
    });
    let argv: Vec<&str> = std::iter::once(spec.program.as_str())
        .chain(spec.args.iter().map(String::as_str))
        .collect();
    let t = term.downgrade();
    let status = state.status.downgrade();
    let h = handle.clone();
    term.spawn_async(
        vte4::PtyFlags::DEFAULT,
        None,
        &argv,
        &["TERM=xterm-256color"],
        glib::SpawnFlags::SEARCH_PATH,
        || {},
        10000,
        Some(&handle.cancellable),
        move |r| match r {
            Ok(pid) => {
                h.pid.set(Some(pid.0));
                if h.cancelled.get() {
                    h.stop();
                }
            }
            Err(e) => {
                if let Some(t) = t.upgrade() {
                    t.feed(format!("\r\nNo se pudo iniciar: {e}\r\n").as_bytes());
                }
                if let Some(s) = status.upgrade() {
                    s.set_text(&e.to_string());
                }
            }
        },
    );
    term.grab_focus();
    actions::update_actions(state);
}

fn connect_profile(state: &Rc<State>, window: &gtk::ApplicationWindow, profile: &Profile) {
    if profile.protocol != Protocol::Ssh {
        match connection::command(profile) {
            Ok(cmd) => terminal(state, &profile.name, cmd),
            Err(e) => state.status.set_text(&e),
        }
        return;
    }
    let connection = match nexuterm::ssh::SshConnection::new(profile.clone()) {
        Ok(c) => c,
        Err(e) => {
            state.status.set_text(&e);
            return;
        }
    };
    let mut spec = match connection.terminal_command() {
        Ok(spec) => spec,
        Err(e) => {
            state.status.set_text(&e);
            return;
        }
    };
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_SSH_LAB").is_some()
        && profile.id == "nexuterm-smoke-lab"
        && profile.host == "127.0.0.1"
        && profile.port == 22222
    {
        let known = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab/known_hosts");
        spec.args.splice(
            0..0,
            [
                "-o".into(),
                format!("UserKnownHostsFile={}", known.display()),
                "-o".into(),
                "StrictHostKeyChecking=yes".into(),
                "-o".into(),
                "BatchMode=yes".into(),
                "-o".into(),
                "IdentitiesOnly=yes".into(),
            ],
        );
    }
    let page = gtk::Paned::new(gtk::Orientation::Horizontal);
    page.set_position(300);
    page.set_shrink_start_child(false);
    page.set_resize_start_child(false);
    let remote = crate::remote_panel::Panel::new(state, window, connection.clone());
    page.set_start_child(Some(&remote.widget));
    let terminals = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    terminals.set_hexpand(true);
    terminals.set_vexpand(true);
    page.set_end_child(Some(&terminals));
    terminal_in(
        state,
        &profile.name,
        spec,
        Some(&terminals),
        Some(connection.clone()),
    );
    let s = Rc::downgrade(state);
    let p = page.downgrade();
    append_tab(&state.notebook, &page, &profile.name, move || {
        remote.stop();
        connection
            .closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let (Some(s), Some(p)) = (s.upgrade(), p.upgrade()) {
            for session in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
                if session
                    .terminal
                    .upgrade()
                    .is_some_and(|t| t.is_ancestor(&p))
                {
                    session.stop();
                }
            }
        }
    });
    state.connections_side.set_visible(false);
}

fn refresh_sidebar(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    while let Some(row) = state.sidebar.first_child() {
        state.sidebar.remove(&row);
    }
    if state.config.borrow().profiles.is_empty() {
        let empty = gtk::Label::new(Some(
            "Todavía no hay conexiones.\nAñade una con el icono de servidor.",
        ));
        empty.set_wrap(true);
        empty.add_css_class("dim-label");
        empty.set_margin_top(24);
        empty.set_margin_bottom(24);
        state.sidebar.append(&empty);
    }
    for (index, profile) in state.config.borrow().profiles.iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Vertical, 4);
        row.set_margin_top(6);
        row.set_margin_bottom(8);
        let protocol = match profile.protocol {
            Protocol::Ssh => "SSH",
            Protocol::Rdp => "RDP",
            Protocol::Vnc => "VNC",
        };
        let label = gtk::Label::new(Some(&format!("{} · {protocol}", profile.name)));
        label.set_xalign(0.0);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_max_width_chars(26);
        label.add_css_class("heading");
        row.append(&label);
        let host = gtk::Label::new(Some(&format!("{}:{}", profile.host, profile.port)));
        host.set_xalign(0.0);
        host.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        row.append(&host);
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let connect = icon_button("media-playback-start-symbolic", "Conectar");
        let edit = icon_button("document-edit-symbolic", "Editar conexión");
        actions.append(&connect);
        actions.append(&edit);
        let p = profile.clone();
        let s = Rc::downgrade(state);
        connect.connect_clicked(move |_| {
            if let Some(s) = s.upgrade()
                && let Some(window) = s.window.upgrade()
            {
                connect_profile(&s, &window, &p);
            }
        });
        if profile.protocol == Protocol::Ssh {
            let files = icon_button("folder-remote-symbolic", "Explorar archivos SFTP");
            actions.append(&files);
            let p = profile.clone();
            let s = Rc::downgrade(state);
            let w = window.downgrade();
            files.connect_clicked(move |_| {
                if let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) {
                    connect_profile(&s, &w, &p);
                }
            });
        }
        let s = Rc::downgrade(state);
        let w = window.downgrade();
        edit.connect_clicked(move |_| {
            if let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) {
                profile_dialog(&s, &w, Some(index));
            }
        });
        row.append(&actions);
        state.sidebar.append(&row);
    }
}

fn entry(grid: &gtk::Grid, row: i32, label: &str, value: &str) -> gtk::Entry {
    let l = gtk::Label::new(Some(label));
    l.set_xalign(0.0);
    let e = gtk::Entry::new();
    e.set_text(value);
    e.set_hexpand(true);
    grid.attach(&l, 0, row, 1, 1);
    grid.attach(&e, 1, row, 1, 1);
    e
}

fn profile_dialog(state: &Rc<State>, window: &gtk::ApplicationWindow, index: Option<usize>) {
    if !state.config_writable {
        state
            .status
            .set_text("Repara o recupera la configuración antes de guardar sesiones.");
        return;
    }
    let p = index
        .and_then(|i| state.config.borrow().profiles.get(i).cloned())
        .unwrap_or(Profile {
            port: 22,
            ..Profile::default()
        });
    let dialog = gtk::Window::builder()
        .transient_for(window)
        .modal(true)
        .title("Conexión remota")
        .default_width(520)
        .build();
    let grid = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(12)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();
    let name = entry(&grid, 0, "Nombre", &p.name);
    let protocol = gtk::DropDown::from_strings(&["SSH + SFTP", "RDP / FreeRDP", "VNC / TigerVNC"]);
    protocol.set_selected(match p.protocol {
        Protocol::Ssh => 0,
        Protocol::Rdp => 1,
        Protocol::Vnc => 2,
    });
    grid.attach(&gtk::Label::new(Some("Protocolo")), 0, 1, 1, 1);
    grid.attach(&protocol, 1, 1, 1, 1);
    let host = entry(&grid, 2, "Host / IP", &p.host);
    let port = entry(&grid, 3, "Puerto", &p.port.to_string());
    let user = entry(&grid, 4, "Usuario", &p.username);
    let identity = entry(&grid, 5, "Clave SSH (ruta)", &p.identity);
    let jump = entry(&grid, 6, "Salto SSH", &p.jump_host);
    let domain = entry(&grid, 7, "Dominio RDP", &p.domain);
    let group = entry(&grid, 8, "Grupo", &p.group);
    let favorite = gtk::CheckButton::with_label("Favorita");
    favorite.set_active(p.favorite);
    grid.attach(&favorite, 1, 9, 1, 1);
    let err = gtk::Label::new(None);
    err.set_wrap(true);
    grid.attach(&err, 0, 10, 2, 1);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let cancel = gtk::Button::with_label("Cancelar");
    let save = gtk::Button::with_label("Guardar");
    save.add_css_class("suggested-action");
    let delete = gtk::Button::with_label("Eliminar");
    delete.set_visible(index.is_some());
    buttons.append(&delete);
    buttons.append(&cancel);
    buttons.append(&save);
    grid.attach(&buttons, 0, 11, 2, 1);
    dialog.set_child(Some(&grid));
    let weak = dialog.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(d) = weak.upgrade() {
            d.close();
        }
    });
    let port_clone = port.clone();
    protocol.connect_selected_notify(move |c| {
        let v = match c.selected() {
            1 => "3389",
            2 => "5900",
            _ => "22",
        };
        port_clone.set_text(v);
    });
    let s = Rc::downgrade(state);
    let weak = dialog.downgrade();
    let target_id = index.map(|_| p.id.clone());
    let commit = Rc::new(move |remove: bool| {
        let Some(d) = weak.upgrade() else {
            return;
        };
        let Some(s) = s.upgrade() else {
            d.close();
            return;
        };
        let mut config = s.config.borrow().clone();
        let index = match &target_id {
            Some(id) => match config.profiles.iter().position(|p| &p.id == id) {
                Some(i) => Some(i),
                None => {
                    err.set_text(
                        "La sesión ya no existe. Cierra el formulario y actualiza el listado.",
                    );
                    return;
                }
            },
            None => None,
        };
        if remove {
            if let Some(i) = index {
                config.profiles.remove(i);
            }
        } else {
            let Ok(port) = port.text().parse::<u16>() else {
                err.set_text("Puerto inválido");
                return;
            };
            let p = Profile {
                name: name.text().trim().into(),
                group: group.text().trim().into(),
                favorite: favorite.is_active(),
                protocol: match protocol.selected() {
                    1 => Protocol::Rdp,
                    2 => Protocol::Vnc,
                    _ => Protocol::Ssh,
                },
                host: host.text().trim().into(),
                port,
                username: user.text().trim().into(),
                identity: identity.text().into(),
                jump_host: jump.text().trim().into(),
                domain: domain.text().trim().into(),
                id: index
                    .and_then(|i| config.profiles.get(i).map(|p| p.id.clone()))
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                ..p.clone()
            };
            if let Err(e) = connection::validate(&p) {
                err.set_text(&e);
                return;
            }
            if let Some(i) = index {
                config.profiles[i] = p;
            } else {
                config.profiles.push(p);
            }
        }
        if let Err(e) = config::save(&s.path, &config) {
            err.set_text(&e);
            return;
        }
        *s.config.borrow_mut() = config;
        if let Some(w) = s.window.upgrade() {
            refresh_sidebar(&s, &w);
        }
        sessions::refresh_open(&s);
        s.status.set_text("Sesiones guardadas");
        d.close();
    });
    let commit_save = commit.clone();
    save.connect_clicked(move |_| commit_save(false));
    let parent = dialog.downgrade();
    delete.connect_clicked(move |_| {
        let parent = parent.clone();
        let commit = commit.clone();
        glib::spawn_future_local(async move {
            if let Some(parent) = parent.upgrade() {
                let confirm = gtk::AlertDialog::builder()
                    .modal(true)
                    .message("¿Eliminar la sesión guardada?")
                    .buttons(["Cancelar", "Eliminar"])
                    .cancel_button(0)
                    .default_button(0)
                    .build();
                if confirm.choose_future(Some(&parent)).await == Ok(1) {
                    commit(true);
                }
            }
        });
    });
    dialog.present();
}
