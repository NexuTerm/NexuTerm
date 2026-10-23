use super::*;

pub(super) fn show(state: &Rc<State>, parent: &gtk::ApplicationWindow) {
    if let Some(window) = state.manager.borrow().upgrade() {
        window.present();
        return;
    }
    let window = gtk::ApplicationWindow::builder()
        .application(&parent.application().expect("application"))
        .transient_for(parent)
        .destroy_with_parent(true)
        .title("Administrar sesiones — NexuTerm")
        .default_width(760)
        .default_height(540)
        .build();
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(
        "Administrar sesiones",
        "Crear, editar, duplicar y organizar conexiones",
    )));
    let add = icon_button("list-add-symbolic", "Nueva sesión");
    header.pack_start(&add);
    window.set_titlebar(Some(&header));
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(12);
    root.set_margin_end(12);
    let filter = gtk::SearchEntry::builder()
        .placeholder_text("Buscar por nombre, grupo o destino…")
        .build();
    root.append(&filter);
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    root.append(
        &gtk::ScrolledWindow::builder()
            .child(&list)
            .vexpand(true)
            .build(),
    );
    window.set_child(Some(&root));
    *state.manager.borrow_mut() = window.downgrade();
    let s = Rc::downgrade(state);
    let w = window.downgrade();
    add.set_sensitive(state.config_writable);
    add.connect_clicked(move |_| {
        if let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) {
            profile_dialog(&s, &w, None);
        }
    });
    let weak_list = list.downgrade();
    filter.connect_search_changed(move |entry| {
        if let Some(list) = weak_list.upgrade() {
            let query = entry.text().to_lowercase();
            list.set_filter_func(move |row| {
                row.tooltip_text()
                    .is_some_and(|text| text.to_lowercase().contains(&query))
            });
        }
    });
    refresh(state, &window, &list);
    window.present();
}

pub(super) fn refresh_open(state: &Rc<State>) {
    if let Some(window) = state.manager.borrow().upgrade()
        && let Some(root) = window.child()
        && let Some(scroll) = root.last_child().and_downcast::<gtk::ScrolledWindow>()
        && let Some(list) = scroll.child().and_downcast::<gtk::ListBox>()
    {
        refresh(state, &window, &list);
    }
}

fn refresh(state: &Rc<State>, window: &gtk::ApplicationWindow, list: &gtk::ListBox) {
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }
    let mut profiles = state.config.borrow().profiles.clone();
    profiles.sort_by(|a, b| {
        b.favorite
            .cmp(&a.favorite)
            .then(a.group.cmp(&b.group))
            .then(a.name.cmp(&b.name))
    });
    if profiles.is_empty() {
        let label = gtk::Label::new(Some("No hay sesiones guardadas. Usa + para crear una."));
        label.set_margin_top(24);
        label.set_margin_bottom(24);
        list.append(&label);
    }
    for profile in profiles {
        let row = gtk::ListBoxRow::new();
        row.set_tooltip_text(Some(&format!(
            "{} {} {} {}",
            profile.name, profile.host, profile.username, profile.group
        )));
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        content.set_margin_top(10);
        content.set_margin_bottom(10);
        content.set_margin_start(12);
        content.set_margin_end(12);
        let details = gtk::Box::new(gtk::Orientation::Vertical, 4);
        details.set_hexpand(true);
        let name = gtk::Label::new(Some(&format!(
            "{}{}",
            if profile.favorite { "★ " } else { "" },
            profile.name
        )));
        name.set_xalign(0.0);
        name.add_css_class("heading");
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        details.append(&name);
        let protocol = match profile.protocol {
            Protocol::Ssh => "SSH",
            Protocol::Rdp => "RDP",
            Protocol::Vnc => "VNC",
            Protocol::Telnet => "Telnet",
            Protocol::Ftp => "FTP",
            Protocol::Sftp => "SFTP",
        };
        let destination = gtk::Label::new(Some(&format!(
            "{} · {}@{}:{}{}",
            protocol,
            profile.username,
            profile.host,
            profile.port,
            if profile.group.is_empty() {
                String::new()
            } else {
                format!(" · {}", profile.group)
            }
        )));
        destination.set_xalign(0.0);
        destination.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        destination.add_css_class("dim-label");
        details.append(&destination);
        content.append(&details);
        for (icon, label, operation) in [
            ("media-playback-start-symbolic", "Conectar", "connect"),
            ("document-edit-symbolic", "Editar sesión", "edit"),
            ("edit-copy-symbolic", "Duplicar sesión", "duplicate"),
            ("user-trash-symbolic", "Eliminar sesión", "delete"),
        ] {
            let button = icon_button(icon, label);
            button.set_sensitive(operation == "connect" || state.config_writable);
            content.append(&button);
            let s = Rc::downgrade(state);
            let w = window.downgrade();
            let id = profile.id.clone();
            button.connect_clicked(move |_| {
                if let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) {
                    operate(&s, &w, &id, operation);
                }
            });
        }
        row.set_child(Some(&content));
        list.append(&row);
    }
    list.invalidate_filter();
}

fn operate(state: &Rc<State>, window: &gtk::ApplicationWindow, id: &str, operation: &str) {
    let Some(index) = state
        .config
        .borrow()
        .profiles
        .iter()
        .position(|p| p.id == id)
    else {
        return;
    };
    if operation == "edit" {
        profile_dialog(state, window, Some(index));
        return;
    }
    if operation == "connect" {
        let profile = state.config.borrow().profiles[index].clone();
        if let Some(main) = state.window.upgrade() {
            connect_profile(state, &main, &profile);
            main.present();
        }
        return;
    }
    if operation == "delete" {
        let dialog = gtk::AlertDialog::builder()
            .modal(true)
            .message("¿Eliminar la sesión guardada?")
            .detail("Las conexiones que ya estén abiertas seguirán activas.")
            .buttons(["Cancelar", "Eliminar"])
            .cancel_button(0)
            .default_button(0)
            .build();
        let s = Rc::downgrade(state);
        let w = window.downgrade();
        let id = id.to_owned();
        glib::spawn_future_local(async move {
            if let Some(w) = w.upgrade()
                && dialog.choose_future(Some(&w)).await == Ok(1)
                && let Some(s) = s.upgrade()
            {
                change(&s, &id, false);
            }
        });
    } else {
        change(state, id, true);
    }
}

fn change(state: &Rc<State>, id: &str, duplicate: bool) {
    let mut config = state.config.borrow().clone();
    let Some(index) = config.profiles.iter().position(|p| p.id == id) else {
        return;
    };
    let removed = (!duplicate).then(|| config.profiles[index].clone());
    if duplicate {
        let mut profile = config.profiles[index].clone();
        profile.id = uuid::Uuid::new_v4().to_string();
        profile.secret_ref = None;
        let suffix = " (copia)";
        while profile.name.len() + suffix.len() > 120 {
            profile.name.pop();
        }
        profile.name.push_str(suffix);
        config.profiles.push(profile);
    } else {
        config.profiles.remove(index);
    }
    if let Err(e) = config::save(&state.path, &config) {
        state.status.set_text(&e);
        return;
    }
    *state.config.borrow_mut() = config;
    if let Some(profile) = removed
        && profile.protocol == Protocol::Ftp
    {
        forget_ftp_profile(state, profile);
    }
    if let Some(window) = state.window.upgrade() {
        refresh_sidebar(state, &window);
    }
    refresh_open(state);
}

#[cfg(debug_assertions)]
pub(super) fn smoke(state: &Rc<State>) {
    assert!(
        state.path.starts_with(std::env::temp_dir()),
        "CRUD smoke requires a temporary XDG_CONFIG_HOME"
    );
    let original = state.config.borrow().clone();
    let first = original
        .profiles
        .first()
        .expect("Seed a fictitious profile")
        .clone();
    let manager = state.manager.borrow().upgrade().unwrap();
    change(state, &first.id, true);
    let copy = state.config.borrow().profiles.last().unwrap().clone();
    assert_ne!(copy.id, first.id);
    assert_eq!(copy.group, first.group);
    assert_eq!(copy.favorite, first.favorite);
    fn form() -> (gtk::Window, gtk::Grid) {
        let window = gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Window>().ok())
            .find(|w| w.is_visible() && w.title().is_some_and(|t| t == "Conexión remota"))
            .expect("Session form visible");
        let grid = window.child().unwrap().downcast::<gtk::Grid>().unwrap();
        (window, grid)
    }
    fn save(grid: &gtk::Grid) {
        grid.child_at(0, 12)
            .unwrap()
            .last_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
    }
    let index = state
        .config
        .borrow()
        .profiles
        .iter()
        .position(|p| p.id == copy.id)
        .unwrap();
    profile_dialog(state, &manager, Some(index));
    let (_, grid) = form();
    grid.child_at(1, 0)
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap()
        .set_text("Copia editada");
    save(&grid);
    let edited = state
        .config
        .borrow()
        .profiles
        .iter()
        .find(|p| p.id == copy.id)
        .unwrap()
        .clone();
    assert_eq!(edited.name, "Copia editada");
    assert_eq!(edited.group, first.group);
    assert_eq!(edited.favorite, first.favorite);
    profile_dialog(state, &manager, None);
    let (_, grid) = form();
    let protocol = grid
        .child_at(1, 1)
        .unwrap()
        .downcast::<gtk::DropDown>()
        .unwrap();
    let port = grid
        .child_at(1, 3)
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap();
    let visible = |row| {
        assert_eq!(
            grid.child_at(0, row).unwrap().is_visible(),
            grid.child_at(1, row).unwrap().is_visible()
        );
        grid.child_at(1, row).unwrap().is_visible()
    };
    assert!(visible(4) && visible(5) && visible(6) && !visible(7));
    port.set_text("2222");
    protocol.set_selected(1);
    assert!(visible(4) && !visible(5) && !visible(6) && visible(7));
    assert_eq!(port.text(), "3389");
    port.set_text("3395");
    protocol.set_selected(2);
    assert!(!visible(4) && !visible(5) && !visible(6) && !visible(7));
    assert_eq!(port.text(), "5900");
    let user_toggle = grid
        .child_at(0, 9)
        .unwrap()
        .downcast::<gtk::CheckButton>()
        .unwrap();
    user_toggle.set_active(true);
    assert!(visible(4));
    user_toggle.set_active(false);
    port.set_text("5901");
    protocol.set_selected(0);
    assert_eq!(port.text(), "2222");
    protocol.set_selected(1);
    assert_eq!(port.text(), "3395");
    protocol.set_selected(2);
    assert_eq!(port.text(), "5901");
    for (selected, default_port, user, ssh_fields) in [
        (3, "23", false, false),
        (4, "21", true, false),
        (5, "22", true, true),
    ] {
        protocol.set_selected(selected);
        assert_eq!(port.text(), default_port);
        assert_eq!(visible(4), user);
        assert_eq!(visible(5), ssh_fields);
        assert_eq!(visible(6), ssh_fields);
        assert!(!visible(7));
        assert_eq!(visible(8), selected == 4);
    }
    protocol.set_selected(2);
    for row in [4, 5, 6, 7] {
        grid.child_at(1, row)
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_text("unused");
    }
    grid.child_at(1, 0)
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap()
        .set_text("Nueva sesión de prueba");
    grid.child_at(1, 2)
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap()
        .set_text("test.example.invalid");
    save(&grid);
    let created = state.config.borrow().profiles.last().unwrap().clone();
    assert_eq!(created.name, "Nueva sesión de prueba");
    assert_eq!(created.port, 5901);
    assert!(
        created.username.is_empty()
            && created.identity.is_empty()
            && created.jump_host.is_empty()
            && created.domain.is_empty()
    );
    change(state, &created.id, false);
    change(state, &copy.id, false);
    let persisted = config::load(&state.path).unwrap();
    assert_eq!(persisted.profiles.len(), original.profiles.len());
    for (before, after) in original.profiles.iter().zip(&persisted.profiles) {
        assert_eq!(before.id, after.id);
        assert_eq!(before.name, after.name);
    }
    eprintln!("NEXUTERM_SMOKE_SESSIONS_OK create=1 edit=1 duplicate=1 delete=2 persisted=1");
}
