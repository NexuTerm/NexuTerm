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
    ai_process: RefCell<Option<Rc<ai_panel::Process>>>,
    ai_keys: RefCell<std::collections::HashMap<(String, String), zeroize::Zeroizing<String>>>,
    pub notebook: adw::TabView,
    pub status: gtk::Label,
    sidebar: gtk::ListBox,
    connections_side: gtk::Box,
    local_directory: RefCell<Option<(glib::WeakRef<vte4::Terminal>, PathBuf)>>,
    local_files: RefCell<Option<Rc<crate::file_browser::LocalPane>>>,
    pub(crate) local_browsers: RefCell<Vec<Weak<crate::file_browser::LocalPane>>>,
    pub(crate) remote_browsers: RefCell<Vec<Weak<crate::remote_panel::Panel>>>,
    sidebar_area: gtk::Box,
    sidebar_stack: gtk::Stack,
    remote_panels: RefCell<Vec<(glib::WeakRef<gtk::Widget>, glib::WeakRef<gtk::Box>)>>,
    window: glib::WeakRef<gtk::ApplicationWindow>,
    manager: RefCell<glib::WeakRef<gtk::ApplicationWindow>>,
    pub(crate) config: RefCell<Config>,
    path: PathBuf,
    config_writable: bool,
    sessions: RefCell<Vec<Weak<Session>>>,
    ftp_passwords: RefCell<std::collections::HashMap<crate::ftp::Key, zeroize::Zeroizing<String>>>,
    focused_terminal: RefCell<glib::WeakRef<vte4::Terminal>>,
    pub documents: RefCell<Vec<glib::WeakRef<gtk::TextBuffer>>>,
    pub writes: RefCell<Vec<Weak<Cell<bool>>>>,
    pub operations: RefCell<Vec<glib::WeakRef<gio::Cancellable>>>,
    pub desktops: RefCell<Vec<std::sync::Weak<nexuterm::desktop::Desktop>>>,
    pub sftp_masters: RefCell<Vec<std::sync::Weak<crate::sftp_session::Master>>>,
}

struct Session {
    id:String,
    title:String,
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
#[path="ai_terminals.rs"]
mod ai_terminals;
#[path = "ai_panel.rs"]
mod ai_panel;
#[path = "ai_credentials.rs"]
mod ai_credentials;
#[path = "codex_chat.rs"]
mod codex_chat;
#[path = "ai_dictation.rs"]
mod ai_dictation;
#[path = "sessions.rs"]
mod sessions;
pub(crate) use actions::icon_button;
use actions::install_actions;

impl State {
    pub(crate) fn window(&self) -> Option<gtk::ApplicationWindow> { self.window.upgrade() }
}

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
    let toggle_side = icon_button("folder-symbolic", "Abrir u ocultar explorador de archivos · F9");
    toggle_side.set_action_name(Some("win.local-files"));
    header.pack_start(&toggle_side);
    let manage = icon_button("network-server-symbolic", "Mostrar u ocultar paleta de conexiones");
    manage.set_action_name(Some("win.connections"));
    header.pack_start(&manage);
    let search_button = icon_button(
        "system-search-symbolic",
        "Buscar en la terminal · Ctrl+Mayús+F",
    );
    search_button.set_action_name(Some("win.search"));
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Menú principal")
        .build();
    header.pack_end(&menu_button);
    root.add_top_bar(&header);
    let context = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    context.set_margin_start(6);
    context.set_margin_end(6);
    context.set_margin_top(0);
    context.set_margin_bottom(0);
    context.add_css_class("toolbar");
    context.add_css_class("nexuterm-tools");
    context.append(&new_terminal);
    for (icon, label, action) in [
        (
            "view-dual-symbolic",
            "Dividir en columnas",
            "split-horizontal",
        ),
        ("view-list-symbolic", "Dividir en filas", "split-vertical"),
    ] {
        let button = icon_button(icon, label);
        button.set_action_name(Some(&format!("win.{action}")));
        context.append(&button);
    }
    context.append(&search_button);
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.add_css_class("nexuterm-split");
    split.set_vexpand(true);
    split.set_position(420);
    split.add_tick_callback(move |split, _| {
        if split.width() > 0 {
            split.set_position(split.width() / 3);
            glib::ControlFlow::Break
        } else { glib::ControlFlow::Continue }
    });
    split.set_shrink_start_child(false);
    let side = gtk::Box::new(gtk::Orientation::Vertical, 10);
    side.add_css_class("nexuterm-section");
    side.add_css_class("nexuterm-panel");
    side.set_margin_start(0);
    side.set_margin_end(0);
    side.set_margin_top(2);
    side.set_margin_bottom(8);
    side.set_width_request(220);
    let title = gtk::Label::new(Some("CONEXIONES GUARDADAS"));
    title.set_xalign(0.0);
    title.add_css_class("heading");
    let connection_tools = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    title.set_hexpand(true);
    connection_tools.append(&title);
    for (icon, label, action) in [
        ("list-add-symbolic", "Nueva conexión · Ctrl+N", "new-connection"),
        ("view-list-symbolic", "Administrar sesiones · Ctrl+Mayús+S", "sessions"),
    ] {
        let button = icon_button(icon, label);
        button.set_action_name(Some(&format!("win.{action}")));
        connection_tools.append(&button);
    }
    side.append(&connection_tools);
    let connection_actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    for (icon, label, action) in [
        ("view-refresh-symbolic", "Reconectar terminal", "reconnect"),
        ("network-offline-symbolic", "Desconectar terminal", "disconnect"),
        ("folder-remote-symbolic", "Mostrar archivos remotos", "remote-files"),
    ] {
        let button = icon_button(icon, label);
        button.set_action_name(Some(&format!("win.{action}")));
        connection_actions.append(&button);
    }
    side.append(&connection_actions);
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
    workspace.set_margin_end(0);
    workspace.set_margin_bottom(8);
    workspace.append(&context);
    workspace.append(&tabbar);
    workspace.append(&notebook);
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    let sidebar_stack = gtk::Stack::new();
    sidebar_stack.set_vexpand(true);
    sidebar_stack.set_hhomogeneous(false);
    sidebar_stack.set_vhomogeneous(false);
    sidebar_stack.add_child(&side);
    let sidebar_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar_area.append(&sidebar_stack);
    split.set_start_child(Some(&sidebar_area));
    split.set_end_child(Some(&workspace));
    // Both children must resize so GTK preserves the explorer/terminal ratio.
    split.set_resize_start_child(true);
    split.set_resize_end_child(true);
    let ai_side = gtk::Box::new(gtk::Orientation::Vertical, 0);
    ai_side.set_visible(false);
    let ai_split = gtk::Paned::new(gtk::Orientation::Horizontal);
    ai_split.add_css_class("nexuterm-split");
    ai_split.set_margin_start(12);
    ai_split.set_margin_end(12);
    ai_split.set_start_child(Some(&split));
    ai_split.set_end_child(Some(&ai_side));
    ai_split.set_resize_end_child(false);
    ai_split.set_shrink_end_child(false);
    root.set_content(Some(&ai_split));
    let status = gtk::Label::new(Some("Listo"));
    status.set_xalign(0.0);
    status.set_margin_start(12);
    status.set_margin_top(6);
    status.set_margin_bottom(6);
    root.add_bottom_bar(&status);
    let state = Rc::new(State {
        ai_process: RefCell::new(None),
        ai_keys: RefCell::new(std::collections::HashMap::new()),
        notebook,
        status,
        sidebar,
        connections_side: side.clone(),
        local_directory: RefCell::new(None),
        local_files: RefCell::new(None),
        local_browsers: RefCell::new(Vec::new()),
        remote_browsers: RefCell::new(Vec::new()),
        sidebar_area: sidebar_area.clone(),
        sidebar_stack,
        remote_panels: RefCell::new(Vec::new()),
        window: window.downgrade(),
        manager: RefCell::new(glib::WeakRef::new()),
        config: RefCell::new(config),
        path,
        config_writable: warning.is_none(),
        sessions: RefCell::new(Vec::new()),
        ftp_passwords: RefCell::new(std::collections::HashMap::new()),
        focused_terminal: RefCell::new(glib::WeakRef::new()),
        documents: RefCell::new(Vec::new()),
        writes: RefCell::new(Vec::new()),
        operations: RefCell::new(Vec::new()),
        desktops: RefCell::new(Vec::new()),
        sftp_masters: RefCell::new(Vec::new()),
    });
    ai_panel::install(&state, &window, app, &ai_side);
    let ai_button = icon_button("system-users-symbolic", "Agentes de IA · Ctrl+Mayús+I");
    ai_button.set_action_name(Some("win.ai-agents"));
    header.pack_end(&ai_button);
    adw_window.set_content(Some(&root));
    let local_files = crate::file_browser::LocalPane::local(&state);
    state.sidebar_stack.add_child(&local_files.widget);
    *state.local_files.borrow_mut() = Some(local_files);
    install_actions(&state, &window, app, &sidebar_area, &menu_button);
    let s = Rc::downgrade(&state);
    state.notebook.connect_selected_page_notify(move |_| {
        if let Some(s) = s.upgrade() {
            update_sidebar(&s);
        }
    });
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
    let weak = Rc::downgrade(&state);
    glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
        let Some(state) = weak.upgrade() else { return glib::ControlFlow::Break; };
        if let Some(term) = state.focused_terminal.borrow().upgrade() { sync_local_directory(&state, &term); }
        glib::ControlFlow::Continue
    });
    refresh_sidebar(&state, &window);
    new_terminal.set_action_name(Some("win.new-terminal"));
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
        let active = s.ai_process.borrow().as_ref().is_some_and(|p| p.busy()) || s.sftp_masters.borrow().iter().filter_map(std::sync::Weak::upgrade).any(|master|master.pid.load(std::sync::atomic::Ordering::Relaxed)>0) || s.desktops.borrow().iter().filter_map(std::sync::Weak::upgrade)
            .any(|d| !d.finished.load(std::sync::atomic::Ordering::Relaxed)) || s
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
                .detail("Se cerrarán las conexiones y agentes de IA activos y se perderán los cambios sin guardar.")
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
        if let Some(process) = s.ai_process.borrow().as_ref() { process.shutdown(); }
        for h in s.sessions.borrow().iter().filter_map(Weak::upgrade) { h.stop(); }
        for master in s.sftp_masters.borrow().iter().filter_map(std::sync::Weak::upgrade) { master.close(); }
        for d in s.desktops.borrow().iter().filter_map(std::sync::Weak::upgrade) { d.close(); }
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
        for d in s
            .desktops
            .borrow()
            .iter()
            .filter_map(std::sync::Weak::upgrade)
        {
            d.close();
        }
        for h in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
            h.stop();
        }
        for master in s
            .sftp_masters
            .borrow()
            .iter()
            .filter_map(std::sync::Weak::upgrade)
        {
            master.close();
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
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR").is_some() {
        let root = PathBuf::from(std::env::var_os("NEXUTERM_REMOTE_FIXTURE_ROOT").unwrap());
        connect_profile(&state, &window, &Profile {
            id: "remote-editor-fixture".into(), name: "SSH editor fixture".into(), protocol: Protocol::Ssh,
            host: "127.0.0.1".into(), port: std::env::var("NEXUTERM_REMOTE_FIXTURE_PORT").unwrap().parse().unwrap(),
            username: std::env::var("NEXUTERM_REMOTE_FIXTURE_USER").unwrap(),
            identity: root.join("client").to_string_lossy().into(), ..Profile::default()
        });
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && let Ok(value) = std::env::var("NEXUTERM_SMOKE_BASIC")
        && let Some((kind, port)) = value.split_once(':')
        && let Ok(port) = port.parse::<u16>()
    {
        let state = state.clone();
        let window = window.clone();
        let kind = kind.to_string();
        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            if kind == "ftp" {
                smoke_ftp_configuration(&state, &window, port);
                return;
            }
            connect_profile(
                &state,
                &window,
                &Profile {
                    name: format!("Laboratorio {kind}"),
                    host: "127.0.0.1".into(),
                    port,
                    protocol: if kind == "telnet" {
                        Protocol::Telnet
                    } else {
                        Protocol::Ftp
                    },
                    ..Profile::default()
                },
            );
        });
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && let Ok(port) = std::env::var("NEXUTERM_SMOKE_VNC")
        && let Ok(port) = port.parse::<u16>()
    {
        connect_profile(
            &state,
            &window,
            &Profile {
                id: "vnc-loopback-test".into(),
                name: "VNC integrado — prueba local".into(),
                protocol: Protocol::Vnc,
                host: "127.0.0.1".into(),
                port,
                ..Profile::default()
            },
        );
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_RDP").is_some()
    {
        let lab = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab");
        connect_profile(
            &state,
            &window,
            &Profile {
                name: "RDP integrado — laboratorio".into(),
                protocol: Protocol::Rdp,
                host: "127.0.0.1".into(),
                port: 13389,
                username: "tester".into(),
                certificate_authority: if std::env::var_os("NEXUTERM_SMOKE_RDP_CERTIFICATE")
                    .is_some()
                {
                    String::new()
                } else {
                    lab.join("rdp-ca.pem").to_string_lossy().into()
                },
                ..Profile::default()
            },
        );
    }
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
                protocol: if std::env::var_os("NEXUTERM_SMOKE_SFTP").is_some() {
                    Protocol::Sftp
                } else {
                    Protocol::Ssh
                },
                host: "127.0.0.1".into(),
                port: 22222,
                username: "tester".into(),
                identity: if std::env::var_os("NEXUTERM_SMOKE_SFTP_PASSWORD").is_some() {
                    String::new()
                } else {
                    lab.join("id_ed25519").to_string_lossy().into()
                },
                ..Profile::default()
            },
        );
        if std::env::var_os("NEXUTERM_SMOKE_SFTP").is_none() {
            let ssh_page = state.notebook.selected_page().unwrap();
            let panel = state.sidebar_stack.visible_child().unwrap();
            assert!(panel.is_ancestor(&state.sidebar_area));
            assert!(!panel.is_ancestor(&ssh_page.child()));
            let local = state.notebook.nth_page(0);
            state.notebook.set_selected_page(&local);
            assert_eq!(
                state.sidebar_stack.visible_child(),
                state.local_files.borrow().as_ref().map(|p| p.widget.clone().upcast())
            );
            state.notebook.set_selected_page(&ssh_page);
            assert_eq!(state.sidebar_stack.visible_child(), Some(panel));
            eprintln!("NEXUTERM_SMOKE_SSH_SIDEBAR_OK");
        } else {
            assert_eq!(
                state.sidebar_stack.visible_child(),
                state.local_files.borrow().as_ref().map(|p| p.widget.clone().upcast())
            );
        }
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
        if std::env::var_os("NEXUTERM_SMOKE_LOCAL_EXPLORER").is_some() {
            let local = state.local_files.borrow().as_ref().unwrap().widget.downgrade();
            let agents = ai_side.downgrade();
            let tabs = tabbar.downgrade();
            let palette_state = Rc::downgrade(&state);
            let split = split.downgrade();
            let window = window.downgrade();
            let step = Cell::new(0);
            glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
                let (Some(split), Some(window)) = (split.upgrade(), window.upgrade()) else {
                    return glib::ControlFlow::Break;
                };
                let stage = step.get();
                let tab_height = tabs.upgrade().unwrap().height();
                assert!((24..=32).contains(&tab_height), "Tab bar height {tab_height} is not compact");
                if stage == 0 {
                    eprintln!("NEXUTERM_SMOKE_COMPACT_TABS height={tab_height}");
                    let state = palette_state.upgrade().unwrap();
                    gio::prelude::ActionGroupExt::activate_action(&window, "connections", None);
                    assert!(state.sidebar_area.is_visible());
                    assert_eq!(state.sidebar_stack.visible_child(), Some(state.connections_side.clone().upcast::<gtk::Widget>()));
                    gio::prelude::ActionGroupExt::activate_action(&window, "local-files", None);
                    assert!(state.sidebar_area.is_visible());
                    assert_eq!(state.sidebar_stack.visible_child(), local.upgrade().map(|w| w.upcast::<gtk::Widget>()));
                    eprintln!("NEXUTERM_SMOKE_PALETTE_SWITCH_OK");
                }
                let (expected_width, ratio) = match stage {
                    0 => (1280, 1.0 / 3.0),
                    1 => (1800, 1.0 / 3.0),
                    2 => (1024, 1.0 / 3.0),
                    3 => (1024, 0.4),
                    _ => (1500, 0.4),
                };
                // Client-side window decorations consume up to 24 logical pixels.
                assert!((window.width() - expected_width).abs() <= 24,
                    "Window width {}, requested {expected_width}", window.width());
                let actual = split.start_child().unwrap().width();
                let expected = f64::from(split.width()) * ratio;
                assert!((f64::from(actual) - expected).abs() < 5.0,
                    "Explorer width {actual}, workspace {}, expected {expected}", split.width());
                let local = local.upgrade().unwrap();
                let terminal = split.end_child().unwrap();
                let left = local.compute_bounds(&window).unwrap();
                let right = terminal.compute_bounds(&window).unwrap();
                assert!((left.y() - right.y()).abs() <= 1.0, "Panels must align at the top");
                assert!((left.y() + left.height() - right.y() - right.height()).abs() <= 1.0, "Panels must align at the bottom");
                let gap = right.x() - left.x() - left.width();
                assert!((0.0..=3.1).contains(&gap), "Explorer/terminal gap {gap} exceeds 3px");
                if stage == 5 {
                    let agents = agents.upgrade().unwrap().compute_bounds(&window).unwrap();
                    let gap = agents.x() - right.x() - right.width();
                    assert!((0.0..=3.1).contains(&gap), "Terminal/agents gap {gap} exceeds 3px");
                }
                eprintln!("NEXUTERM_SMOKE_EXPLORER_WIDTH stage={stage} explorer={actual} workspace={} gap={gap}", split.width());
                match stage {
                    0 => window.set_default_size(1800, 820),
                    1 => window.set_default_size(1024, 768),
                    2 => split.set_position((f64::from(split.width()) * 0.4).round() as i32),
                    3 => window.set_default_size(1500, 820),
                    4 => agents.upgrade().unwrap().set_visible(true),
                    _ => {
                        agents.upgrade().unwrap().set_visible(false);
                        split.set_position(split.width() / 3);
                        eprintln!("NEXUTERM_SMOKE_EXPLORER_RESIZE_OK");
                        return glib::ControlFlow::Break;
                    }
                }
                step.set(stage + 1);
                glib::ControlFlow::Continue
            });
        }
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
        let duration = if std::env::var_os("NEXUTERM_SMOKE_FILE_BROWSER").is_some()
            || std::env::var_os("NEXUTERM_SMOKE_RDP").is_some()
        {
            20
        } else if matches!(std::env::var("NEXUTERM_SMOKE_AI").as_deref(),Ok("dictation"|"terminals"|"new-chat")) {
            10
        } else if std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR").is_some()
            || std::env::var_os("NEXUTERM_SMOKE_BROWSER_FEATURES").is_some()
            || std::env::var_os("NEXUTERM_SMOKE_FTP_KEYRING").is_some() {
            8
        } else {
            3
        };
        glib::timeout_add_local_once(std::time::Duration::from_secs(duration), move || {
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
                    if cfg!(debug_assertions) && std::env::var_os("NEXUTERM_SMOKE_COLORS").is_some()
                    {
                        "printf 'Terminal · texto normal y colores ANSI\\n\\n'; for c in 30 31 32 33 34 35 36 37 90 91 92 93 94 95 96 97; do printf '\\033[%sm Color %s: archivos, avisos y resultados\\033[0m\\n' $c $c; done; printf '\\nTexto normal sin blanco o negro puros.\\n'; sleep 1".into()
                    } else if std::env::var_os("NEXUTERM_SMOKE_LOCAL_EXPLORER").is_some() {
                        "printf 'NexuTerm: PTY operativo\\n'; sleep 10".into()
                    } else {
                        "printf 'NexuTerm: PTY operativo\\n'; sleep 1".into()
                    },
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
    crate::terminal_colors::bind(&term);
    term.set_scrollback_lines(state.config.borrow().preferences.scrollback.into());
    term.set_hexpand(true);
    term.set_vexpand(true);
    let focus = gtk::EventControllerFocus::new();
    let s = Rc::downgrade(state);
    let t = term.downgrade();
    focus.connect_enter(move |_| {
        if let Some(s) = s.upgrade() {
            *s.focused_terminal.borrow_mut() = t.clone();
            if let Some(term) = t.upgrade() { sync_local_directory(&s, &term); }
            actions::update_actions(&s);
        }
    });
    term.add_controller(focus);
    if connection.is_none() {
        let s = Rc::downgrade(state);
        term.connect_current_directory_uri_notify(move |term| {
            if let Some(s) = s.upgrade() {
                sync_local_directory(&s, term);
            }
        });
    }
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
        id:uuid::Uuid::new_v4().to_string(),
        title:title.to_owned(),
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
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && let Ok(value) = std::env::var("NEXUTERM_SMOKE_BASIC")
        && let Some((_kind, port)) = value.split_once(':')
        && spec.args.iter().any(|arg| arg == "127.0.0.1")
        && spec.args.last().is_some_and(|arg| arg == port)
    {
        let step = Cell::new(false);
        term.connect_contents_changed(move |term| {
            use glib::translate::{ToGlibPtr, from_glib_full};
            let text: Option<glib::GString> = unsafe {
                from_glib_full(vte4::ffi::vte_terminal_get_text(
                    term.to_glib_none().0,
                    None,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ))
            };
            if !step.get() && text.is_some_and(|text| text.contains("NEXUTERM_BASIC_OK")) {
                step.set(true);
                term.feed_child(b"hello-lab\n");
                eprintln!("NEXUTERM_SMOKE_BASIC_OK");
            }
        });
    }
    let status = state.status.downgrade();
    let launch_terminal = term.downgrade();
    let launch = move || {
        if handle.cancelled.get() {
            return;
        }
        let Some(term) = launch_terminal.upgrade() else {
            return;
        };
        let argv: Vec<&str> = std::iter::once(spec.program.as_str())
            .chain(spec.args.iter().map(String::as_str))
            .collect();
        let t = term.downgrade();
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
    };
    launch();
    term.grab_focus();
    actions::update_actions(state);
}

fn connect_profile(state: &Rc<State>, window: &gtk::ApplicationWindow, profile: &Profile) {
    if matches!(profile.protocol, Protocol::Vnc | Protocol::Rdp) {
        if let Err(error) = connection::validate(profile) {
            state.status.set_text(&error);
            return;
        }
        crate::desktop_view::show(state, profile);
        return;
    }
    if profile.protocol == Protocol::Ftp {
        let profile = profile.clone();
        let state = state.clone();
        glib::spawn_future_local(async move {
            let cached = state
                .ftp_passwords
                .borrow()
                .get(&crate::ftp::key(&profile))
                .cloned();
            let password = match cached {
                Some(password) => Ok(Some(password)),
                None => crate::ftp::lookup(&profile).await,
            };
            match password {
                Ok(Some(password)) => {
                    let Some(window) = state.window.upgrade() else {
                        return;
                    };
                    let client = crate::ftp_transfer::start(&state, profile.clone(), password);
                    let remote = crate::remote_panel::Panel::with_client(&state, &window, client);
                    let (workspace, local) = crate::file_browser::workspace(&state, &remote);
                    append_tab(&state.notebook, &workspace, &profile.name, move || {
                        remote.stop();
                        local.stop();
                    });
                }
                Ok(None) => state
                    .status
                    .set_text("Configura el usuario y la contraseña FTP antes de conectar."),
                Err(error) => state.status.set_text(&error),
            }
        });
        return;
    }
    if profile.protocol == Protocol::Telnet {
        match connection::command(profile) {
            Ok(spec) => terminal(state, &profile.name, spec),
            Err(error) => state.status.set_text(&error),
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
                "-F".into(),
                "/dev/null".into(),
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
        if std::env::var_os("NEXUTERM_SMOKE_SFTP_PASSWORD").is_some() {
            spec.args.splice(
                0..0,
                [
                    "-o".into(),
                    "BatchMode=no".into(),
                    "-o".into(),
                    "PreferredAuthentications=password".into(),
                    "-o".into(),
                    "PasswordAuthentication=yes".into(),
                    "-o".into(),
                    "PubkeyAuthentication=no".into(),
                ],
            );
        }
    }
    #[cfg(debug_assertions)]
    if profile.id == "remote-editor-fixture" && std::env::var_os("NEXUTERM_SMOKE_REMOTE_EDITOR").is_some() {
        let root = PathBuf::from(std::env::var_os("NEXUTERM_REMOTE_FIXTURE_ROOT").unwrap());
        spec.args.splice(0..0, ["-F".into(), "/dev/null".into(), "-o".into(),
            format!("UserKnownHostsFile={}", root.join("known_hosts").display()),
            "-o".into(), "StrictHostKeyChecking=yes".into(), "-o".into(), "BatchMode=yes".into(),
            "-o".into(), "IdentitiesOnly=yes".into()]);
        spec.args.push("/bin/sleep 30".into());
    }
    if profile.protocol == Protocol::Sftp {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let status = gtk::Label::new(Some(
            "Conectando SFTP… La autenticación se solicita en una ventana.",
        ));
        status.set_xalign(0.0);
        status.set_wrap(true);
        page.append(&status);
        let master = crate::sftp_session::Master::start(state, connection.clone(), spec, &status);
        let remote = crate::remote_panel::Panel::new(state, window, connection);
        remote.hide_auth_when_ready(status.upcast_ref());
        let (workspace, local) = crate::file_browser::workspace(state, &remote);
        page.append(&workspace);
        append_tab(&state.notebook, &page, &profile.name, move || {
            remote.stop();
            local.stop();
            master.close();
        });
        return;
    }
    let page = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let remote = crate::remote_panel::Panel::new(state, window, connection.clone());
    {
        state.sidebar_stack.add_child(&remote.widget);
        state.remote_panels.borrow_mut().push((
            page.upcast_ref::<gtk::Widget>().downgrade(),
            remote.widget.downgrade(),
        ));
    }
    let terminals = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    terminals.set_hexpand(true);
    terminals.set_vexpand(true);
    page.append(&terminals);
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
        if let Some(s) = s.upgrade() {
            s.sidebar_stack.remove(&remote.widget);
            s.remote_panels
                .borrow_mut()
                .retain(|(_, panel)| panel.upgrade().is_some_and(|p| p != remote.widget));
        }
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
    state.sidebar_area.set_visible(true);
    update_sidebar(state);
}

fn update_sidebar(state: &State) {
    let panel = state.notebook.selected_page().and_then(|selected| {
        state
            .remote_panels
            .borrow()
            .iter()
            .find_map(|(page, panel)| {
                (page.upgrade().as_ref() == Some(&selected.child()))
                    .then(|| panel.upgrade())
                    .flatten()
            })
    });
    if let Some(panel) = panel {
        state.sidebar_stack.set_visible_child(&panel);
    } else if let Some(local) = state.local_files.borrow().as_ref() {
        state.sidebar_stack.set_visible_child(&local.widget);
    }
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
            Protocol::Telnet => "Telnet",
            Protocol::Ftp => "FTP",
            Protocol::Sftp => "SFTP",
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

#[cfg(debug_assertions)]
fn smoke_ftp_configuration(state: &Rc<State>, window: &gtk::ApplicationWindow, port: u16) {
    profile_dialog(state, window, None);
    let dialog = gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Window>().ok())
        .find(|window| {
            window.is_visible()
                && window
                    .title()
                    .is_some_and(|title| title == "Conexión remota")
        })
        .unwrap();
    let grid = dialog.child().unwrap().downcast::<gtk::Grid>().unwrap();
    grid.child_at(1, 1)
        .unwrap()
        .downcast::<gtk::DropDown>()
        .unwrap()
        .set_selected(4);
    for (row, text) in [
        (0, "FTP configurado".to_string()),
        (2, "127.0.0.1".into()),
        (3, port.to_string()),
        (4, "demo".into()),
    ] {
        grid.child_at(1, row)
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_text(&text);
    }
    let fields = grid.child_at(1, 8).unwrap();
    fields
        .first_child()
        .unwrap()
        .downcast::<gtk::PasswordEntry>()
        .unwrap()
        .set_text("test only\"\\fixture");
    let remember = std::env::var_os("NEXUTERM_SMOKE_FTP_KEYRING").is_some();
    fields
        .last_child()
        .unwrap()
        .downcast::<gtk::CheckButton>()
        .unwrap()
        .set_active(remember);
    grid.child_at(0, 12)
        .unwrap()
        .last_child()
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    let state = state.clone();
    let window = window.clone();
    let attempts = Cell::new(0);
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        attempts.set(attempts.get() + 1);
        assert!(attempts.get() < 100, "FTP form did not save");
        if dialog.is_visible() {
            return glib::ControlFlow::Continue;
        }
        let profile = state.config.borrow().profiles.last().unwrap().clone();
        assert_eq!(profile.username, "demo");
        assert_eq!(profile.secret_ref.is_some(), remember);
        assert!(
            !std::fs::read_to_string(&state.path)
                .unwrap()
                .contains("test only")
        );
        let state = state.clone();
        let window = window.clone();
        glib::spawn_future_local(async move {
            let mut changed = profile.clone();
            changed.port = if changed.port == 65535 {
                1
            } else {
                changed.port + 1
            };
            assert!(
                save_ftp_profile(
                    &state,
                    changed,
                    Some(profile.clone()),
                    Default::default(),
                    remember
                )
                .await
                .is_err()
            );
            save_ftp_profile(
                &state,
                profile.clone(),
                Some(profile),
                Default::default(),
                remember,
            )
            .await
            .unwrap();
            let profile = config::load(&state.path)
                .unwrap()
                .profiles
                .last()
                .unwrap()
                .clone();
            if remember {
                state.ftp_passwords.borrow_mut().clear();
            }
            connect_profile(&state, &window, &profile);
        });
        glib::ControlFlow::Break
    });
}

fn forget_ftp_profile(state: &Rc<State>, profile: Profile) {
    state
        .ftp_passwords
        .borrow_mut()
        .retain(|key, _| key.0 != profile.id);
    let status = state.status.downgrade();
    glib::spawn_future_local(async move {
        if let Err(error) = crate::ftp::forget(&profile).await
            && let Some(status) = status.upgrade()
        {
            status.set_text(&error);
        }
    });
}

async fn save_ftp_profile(
    state: &Rc<State>,
    mut profile: Profile,
    old: Option<Profile>,
    mut password: zeroize::Zeroizing<String>,
    remember: bool,
) -> Result<(), String> {
    let editing = old.is_some();
    let old = old.filter(|p| p.protocol == Protocol::Ftp);
    let same_destination = old
        .as_ref()
        .is_some_and(|p| crate::ftp::key(p) == crate::ftp::key(&profile));
    if password.is_empty() && same_destination {
        if let Some(cached) = state.ftp_passwords.borrow().get(&crate::ftp::key(&profile)) {
            password = cached.clone();
        }
        if password.is_empty()
            && let Some(p) = old.as_ref()
        {
            password = crate::ftp::lookup(p).await?.unwrap_or_default();
        }
    }
    if password.is_empty() {
        return Err("Indica la contraseña FTP; al cambiar servidor, puerto o usuario debe introducirse de nuevo.".into());
    }
    let reference = if remember {
        Some(crate::ftp::store(&profile, &password).await?)
    } else {
        None
    };
    profile.secret_ref = reference;
    // Re-read after awaiting the keyring so other form edits are preserved.
    let mut config = state.config.borrow().clone();
    if let Some(index) = config.profiles.iter().position(|p| p.id == profile.id) {
        config.profiles[index] = profile.clone();
    } else if !editing {
        config.profiles.push(profile.clone());
    } else {
        let _ = crate::ftp::forget(&profile).await;
        return Err("La conexión se eliminó mientras se guardaba.".into());
    }
    if let Err(error) = config::save(&state.path, &config) {
        let _ = crate::ftp::forget(&profile).await;
        return Err(error);
    }
    *state.config.borrow_mut() = config;
    state
        .ftp_passwords
        .borrow_mut()
        .retain(|key, _| key.0 != profile.id);
    state
        .ftp_passwords
        .borrow_mut()
        .insert(crate::ftp::key(&profile), password);
    if let Some(old) = old
        && let Err(error) = crate::ftp::forget(&old).await
    {
        state.status.set_text(&error);
    }
    Ok(())
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
    let protocol = gtk::DropDown::from_strings(&["SSH", "RDP", "VNC", "Telnet", "FTP", "SFTP"]);
    protocol.set_selected(match p.protocol {
        Protocol::Ssh => 0,
        Protocol::Rdp => 1,
        Protocol::Vnc => 2,
        Protocol::Telnet => 3,
        Protocol::Ftp => 4,
        Protocol::Sftp => 5,
    });
    grid.attach(&gtk::Label::new(Some("Protocolo")), 0, 1, 1, 1);
    grid.attach(&protocol, 1, 1, 1, 1);
    let host = entry(&grid, 2, "Host / IP", &p.host);
    let port = entry(&grid, 3, "Puerto", &p.port.to_string());
    let user = entry(&grid, 4, "Usuario", &p.username);
    let identity = entry(&grid, 5, "Clave SSH (ruta, opcional)", &p.identity);
    if let Some(label)=grid.child_at(0,5){grid.remove(&label);}
    let ssh_auth=gtk::DropDown::from_strings(&["SSH · automático", "SSH · contraseña", "SSH · archivo de clave"]);ssh_auth.set_selected(if p.ssh_password{1}else if !p.identity.is_empty(){2}else{0});ssh_auth.set_tooltip_text(Some("Autenticación SSH. La contraseña se solicita al conectar y no se guarda en el perfil."));grid.attach(&ssh_auth,0,5,1,1);
    identity.set_placeholder_text(Some("Archivo de clave privada (opcional en automático)"));
    let browse_key=icon_button("document-open-symbolic","Explorar archivo de clave SSH");grid.attach(&browse_key,2,5,1,1);
    let field=identity.downgrade();let parent=dialog.downgrade();browse_key.connect_clicked(move |_|{let Some(parent)=parent.upgrade()else{return;};let chooser=gtk::FileDialog::builder().title("Seleccionar clave privada SSH").build();let field=field.clone();glib::spawn_future_local(async move{if let Ok(file)=chooser.open_future(Some(&parent)).await&& let(Some(field),Some(path))=(field.upgrade(),file.path()){field.set_text(&path.to_string_lossy());}});});
    let key=identity.downgrade();let browse=browse_key.downgrade();ssh_auth.connect_selected_notify(move |auth|{let enabled=auth.selected()!=1;if let Some(key)=key.upgrade(){key.set_sensitive(enabled);}
        if let Some(browse)=browse.upgrade(){browse.set_sensitive(enabled);}});identity.set_sensitive(!p.ssh_password);browse_key.set_sensitive(!p.ssh_password);
    let browse=browse_key.downgrade();identity.connect_visible_notify(move |key|{if let Some(browse)=browse.upgrade(){browse.set_visible(gtk::prelude::WidgetExt::is_visible(key));}});
    let jump = entry(&grid, 6, "Salto SSH (opcional)", &p.jump_host);
    let domain = entry(&grid, 7, "Dominio (opcional)", &p.domain);
    let vnc_user = gtk::CheckButton::with_label("El servidor VNC requiere usuario");
    vnc_user.set_active(p.protocol == Protocol::Vnc && !p.username.is_empty());
    vnc_user.set_tooltip_text(Some(
        "La contraseña VNC se solicita al conectar y no se guarda en el perfil.",
    ));
    let ftp_password = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .placeholder_text("Vacío conserva la contraseña configurada")
        .build();
    let remember = gtk::CheckButton::with_label("Guardar contraseña en el llavero");
    remember.set_active(index.is_none() || p.secret_ref.is_some());
    remember.set_tooltip_text(Some(
        "Si se desmarca, solo se conserva mientras NexuTerm esté abierto.",
    ));
    let password_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    password_box.append(&ftp_password);
    password_box.append(&remember);
    grid.attach(&gtk::Label::new(Some("Contraseña FTP")), 0, 8, 1, 1);
    grid.attach(&password_box, 1, 8, 1, 1);
    grid.attach(&vnc_user, 0, 9, 2, 1);
    let organization = gtk::Expander::new(Some("Organización (opcional)"));
    let metadata = gtk::Grid::builder()
        .row_spacing(8)
        .column_spacing(12)
        .build();
    let group = entry(&metadata, 0, "Grupo", &p.group);
    let favorite = gtk::CheckButton::with_label("Favorita");
    favorite.set_active(p.favorite);
    metadata.attach(&favorite, 1, 1, 1, 1);
    organization.set_child(Some(&metadata));
    grid.attach(&organization, 0, 10, 2, 1);
    let err = gtk::Label::new(None);
    err.set_wrap(true);
    grid.attach(&err, 0, 11, 2, 1);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let cancel = gtk::Button::with_label("Cancelar");
    let save = gtk::Button::with_label("Guardar");
    save.add_css_class("suggested-action");
    let delete = gtk::Button::with_label("Eliminar");
    delete.set_visible(index.is_some());
    buttons.append(&delete);
    buttons.append(&cancel);
    buttons.append(&save);
    grid.attach(&buttons, 0, 12, 2, 1);
    dialog.set_child(Some(&grid));
    let weak = dialog.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(d) = weak.upgrade() {
            d.close();
        }
    });
    let form = grid.downgrade();
    let kind = protocol.downgrade();
    let toggle = vnc_user.downgrade();
    let notice = err.downgrade();
    let update_fields = Rc::new(move || {
        let (Some(grid), Some(protocol), Some(toggle)) =
            (form.upgrade(), kind.upgrade(), toggle.upgrade())
        else {
            return;
        };
        let selected = protocol.selected();
        if let Some(notice) = notice.upgrade() {
            notice.set_text(if matches!(selected, 3 | 4) {
                "Telnet y FTP transmiten datos y credenciales sin cifrar."
            } else {
                ""
            });
        }
        for (row, visible) in [
            (
                4,
                matches!(selected, 0 | 1 | 4 | 5) || (selected == 2 && toggle.is_active()),
            ),
            (5, matches!(selected, 0 | 5)),
            (6, matches!(selected, 0 | 5)),
            (7, selected == 1),
            (8, selected == 4),
        ] {
            for column in 0..2 {
                if let Some(child) = grid.child_at(column, row) {
                    child.set_visible(visible);
                }
            }
        }
        toggle.set_visible(selected == 2);
    });
    update_fields();
    let update = update_fields.clone();
    vnc_user.connect_toggled(move |_| update());
    let mut initial_ports = ["22", "3389", "5900", "23", "21", "22"].map(str::to_string);
    initial_ports[protocol.selected() as usize] = p.port.to_string();
    let ports = RefCell::new(initial_ports);
    let previous = Cell::new(protocol.selected());
    let port_field = port.downgrade();
    let error_label = err.downgrade();
    protocol.connect_selected_notify(move |c| {
        if let Some(port) = port_field.upgrade() {
            let mut ports = ports.borrow_mut();
            ports[previous.replace(c.selected()) as usize] = port.text().to_string();
            port.set_text(&ports[c.selected() as usize]);
        }
        if let Some(error) = error_label.upgrade() {
            error.set_text("");
        }
        update_fields();
    });
    let s = Rc::downgrade(state);
    let weak = dialog.downgrade();
    let previous_ftp = (p.protocol == Protocol::Ftp).then(|| p.clone());
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
            let selected = match protocol.selected() {
                1 => Protocol::Rdp,
                2 => Protocol::Vnc,
                3 => Protocol::Telnet,
                4 => Protocol::Ftp,
                5 => Protocol::Sftp,
                _ => Protocol::Ssh,
            };
            if matches!(selected,Protocol::Ssh|Protocol::Sftp)&&ssh_auth.selected()==2&&!std::path::Path::new(identity.text().as_str()).is_file(){err.set_text("Selecciona un archivo de clave SSH existente.");return;}
            let p = Profile {
                name: name.text().trim().into(),
                group: group.text().trim().into(),
                favorite: favorite.is_active(),
                protocol: selected,
                host: host.text().trim().into(),
                port,
                username: if matches!(
                    selected,
                    Protocol::Ssh | Protocol::Sftp | Protocol::Rdp | Protocol::Ftp
                ) || (selected == Protocol::Vnc && vnc_user.is_active())
                {
                    user.text().trim().into()
                } else {
                    String::new()
                },
                ssh_password:matches!(selected,Protocol::Ssh|Protocol::Sftp)&&ssh_auth.selected()==1,
                identity: if matches!(selected, Protocol::Ssh | Protocol::Sftp) && ssh_auth.selected()!=1 {
                    identity.text().into()
                } else {
                    String::new()
                },
                jump_host: if matches!(selected, Protocol::Ssh | Protocol::Sftp) {
                    jump.text().trim().into()
                } else {
                    String::new()
                },
                domain: if selected == Protocol::Rdp {
                    domain.text().trim().into()
                } else {
                    String::new()
                },
                secret_ref: None,
                id: index
                    .and_then(|i| config.profiles.get(i).map(|p| p.id.clone()))
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                ..p.clone()
            };
            if let Err(e) = connection::validate(&p) {
                err.set_text(&e);
                return;
            }
            if p.protocol == Protocol::Ftp {
                if p.username.is_empty() {
                    err.set_text("Indica el usuario FTP");
                    return;
                }
                let pass = zeroize::Zeroizing::new(ftp_password.text().to_string());
                if let Err(error) = crate::ftp::validate_password(&pass) {
                    err.set_text(&error);
                    return;
                }
                let old = index.map(|i| config.profiles[i].clone());
                let remember = remember.is_active();
                let error_label = err.clone();
                let password_field = ftp_password.clone();
                d.set_sensitive(false);
                glib::spawn_future_local(async move {
                    if let Err(error) = save_ftp_profile(&s, p, old, pass, remember).await {
                        error_label.set_text(&error);
                        d.set_sensitive(true);
                        return;
                    }
                    password_field.set_text("");
                    if let Some(window) = s.window.upgrade() {
                        refresh_sidebar(&s, &window);
                    }
                    sessions::refresh_open(&s);
                    s.status.set_text("Conexión FTP guardada");
                    d.close();
                });
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
        if let Some(previous) = previous_ftp.clone() {
            forget_ftp_profile(&s, previous);
        }
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

fn sync_local_directory(state: &State, term: &vte4::Terminal) {
    let local = state
        .sessions
        .borrow()
        .iter()
        .filter_map(Weak::upgrade)
        .find(|session| session.terminal.upgrade().as_ref() == Some(term))
        .filter(|session| session.connection.is_none() && session.title == "Terminal local");
    let Some(session) = local else {
        return;
    };
    if state.focused_terminal.borrow().upgrade().as_ref() != Some(term) {
        return;
    }
    let path = term
        .current_directory_uri()
        .and_then(|uri| gio::File::for_uri(&uri).path())
        .or_else(|| {
            session
                .pid
                .get()
                .and_then(|pid| std::fs::read_link(format!("/proc/{pid}/cwd")).ok())
        });
    if let (Some(path), Some(panel)) = (path, state.local_files.borrow().as_ref()) {
        let changed =
            state
                .local_directory
                .borrow()
                .as_ref()
                .is_none_or(|(previous, directory)| {
                    previous.upgrade().as_ref() != Some(term) || directory != &path
                });
        if changed && !panel.is_loading() {
            *state.local_directory.borrow_mut() = Some((term.downgrade(), path.clone()));
            if path != panel.directory() {
                panel.navigate(path);
            }
        }
    }
}

pub(crate) fn open_local_file(state: &Rc<State>, path: PathBuf) {
    let file = gio::File::for_path(path);
    if state.config.borrow().preferences.files_use_system_app {
        let s = Rc::downgrade(state);
        glib::spawn_future_local(async move {
            if let Err(error) = gio::AppInfo::launch_default_for_uri_future(
                &file.uri(),
                None::<&gio::AppLaunchContext>,
            )
            .await
                && let Some(s) = s.upgrade()
            {
                s.status.set_text(&format!("No se pudo abrir: {error}"));
            }
        });
    } else if let Some(window) = state.window.upgrade() {
        crate::files::editor(state, &window, file);
    }
}

pub(crate) fn apply_browser_preferences(state: &State) {
    let preferences = state.config.borrow().preferences.clone();
    state.local_browsers.borrow_mut().retain(|weak| {
        if let Some(pane) = weak.upgrade() { pane.apply_preferences(&preferences); true } else { false }
    });
    state.remote_browsers.borrow_mut().retain(|weak| {
        if let Some(pane) = weak.upgrade() { pane.apply_preferences(&preferences); true } else { false }
    });
}
