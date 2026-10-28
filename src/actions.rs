use super::*;

pub(crate) fn icon_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon);
    button.set_tooltip_text(Some(label));
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button.add_css_class("flat");
    button
}

fn active_session(state: &State) -> Option<Rc<Session>> {
    let page = state.notebook.selected_page()?.child();
    let focused = state.focused_terminal.borrow().upgrade();
    let sessions = state.sessions.borrow();
    let eligible: Vec<_> = sessions
        .iter()
        .filter_map(Weak::upgrade)
        .filter(|s| s.terminal.upgrade().is_some_and(|t| t.is_ancestor(&page)))
        .collect();
    eligible
        .iter()
        .find(|s| {
            s.terminal
                .upgrade()
                .is_some_and(|t| t.has_focus() || focused.as_ref() == Some(&t))
        })
        .cloned()
        .or_else(|| eligible.first().cloned())
}

pub(super) fn install_actions(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    app: &gtk::Application,
    sidebar: &gtk::Box,
    menu_button: &gtk::MenuButton,
) {
    let menu = gio::Menu::new();
    for (title, entries) in [
        (
            "Conexiones",
            vec![
                ("Administrar sesiones…", "sessions"),
                ("Espacios de trabajo…", "workspaces"),
                ("Recuperar sesión anterior…", "recover-session"),
            ],
        ),
        (
            "Terminal",
            vec![
                ("Terminal local", "new-terminal"),
                ("Buscar…", "search"),
                ("Dividir en columnas", "split-horizontal"),
                ("Dividir en filas", "split-vertical"),
                ("Aumentar texto", "zoom-in"),
                ("Reducir texto", "zoom-out"),
            ],
        ),
        (
            "Vista y herramientas",
            vec![
                ("Mostrar explorador", "sidebar"),
                ("Agentes de IA", "ai-agents"),
                ("Abrir archivo…", "open-file"),
                ("Recuperar documentos…", "recover-documents"),
                ("Preferencias…", "preferences"),
                ("Acerca de NexuTerm", "about"),
            ],
        ),
    ] {
        let section = gio::Menu::new();
        for (label, action) in entries {
            section.append(Some(label), Some(&format!("win.{action}")));
        }
        menu.append_section(Some(title), &section);
    }
    menu_button.set_menu_model(Some(&menu));
    sidebar.set_visible(state.config.borrow().preferences.sidebar_visible);
    apply_theme(&state.config.borrow().preferences.theme);
    for name in [
        "connections",
        "local-files",
        "sessions",
        "workspaces",
        "recover-session",
        "new-connection",
        "new-terminal",
        "open-file",
        "recover-documents",
        "remote-files",
        "search",
        "split-horizontal",
        "split-vertical",
        "reconnect",
        "disconnect",
        "zoom-in",
        "zoom-out",
        "sidebar",
        "preferences",
        "about",
    ] {
        let action = gio::SimpleAction::new(name, None);
        let s = Rc::downgrade(state);
        let w = window.downgrade();
        let side = sidebar.downgrade();
        let menu = menu_button.downgrade();
        action.connect_activate(move |_, _| {
            let (Some(s), Some(w)) = (s.upgrade(), w.upgrade()) else {
                return;
            };
            // Release the popover's input grab before presenting modal windows
            // or removing/reparenting the focused terminal.
            if let Some(menu) = menu.upgrade() {
                menu.popdown();
            }
            match name {
                "connections" => {
                    let already_open = s.sidebar_stack.visible_child().as_ref() == Some(s.connections_side.upcast_ref());
                    let visible = !already_open || !s.sidebar_area.is_visible();
                    s.sidebar_stack.set_visible_child(&s.connections_side);
                    s.sidebar_area.set_visible(visible);
                }
                "local-files" => {
                    if let Some(files) = s.local_files.borrow().as_ref() {
                        let already_open = s.sidebar_stack.visible_child().as_ref() == Some(files.widget.upcast_ref());
                        let visible = !already_open || !s.sidebar_area.is_visible();
                        s.sidebar_stack.set_visible_child(&files.widget);
                        s.sidebar_area.set_visible(visible);
                    }
                }
                "sessions" => sessions::show(&s, &w),
                "workspaces" => workspace_ui::show(&s, &w),
                "recover-session" => session_ui::show(&s,&w,false),
                "new-connection" => profile_dialog(&s, &w, None),
                "new-terminal" => local_terminal(&s),
                "open-file" => crate::files::choose_editor(&s, &w),
                "recover-documents" => crate::recovery_ui::show(&s, &w),
                "remote-files" => {
                    update_sidebar(&s);
                    s.sidebar_area.set_visible(true);
                }
                "preferences" => preferences(&s, &w),
                "about" => {
                    let d = gtk::AboutDialog::builder()
                        .transient_for(&w)
                        .modal(true)
                        .destroy_with_parent(true)
                        .program_name("NexuTerm")
                        .version("0.2.0 · en desarrollo")
                        .comments("Terminal y conexiones remotas para Linux")
                        .logo_icon_name("io.nexuterm.NexuTerm")
                        .build();
                    d.present();
                }
                "sidebar" => {
                    if let Some(side) = side.upgrade() {
                        side.set_visible(!side.is_visible());
                    }
                }
                _ => {
                    if let Some(session) = active_session(&s) {
                        let Some(t) = session.terminal.upgrade() else {
                            return;
                        };
                        match name {
                            "search" => terminal_search(&t, &w),
                            "zoom-in" => t.set_font_scale((t.font_scale() + 0.1).min(3.0)),
                            "zoom-out" => t.set_font_scale((t.font_scale() - 0.1).max(0.5)),
                            "disconnect" => session.stop(),
                            "split-horizontal" | "split-vertical" => {
                                let Some(parent) = t.parent().and_downcast::<gtk::Box>() else {
                                    return;
                                };
                                let old = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                                while let Some(child) = parent.first_child() {
                                    parent.remove(&child);
                                    old.append(&child);
                                }
                                let new = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                                let orientation = if name == "split-horizontal" {
                                    gtk::Orientation::Horizontal
                                } else {
                                    gtk::Orientation::Vertical
                                };
                                let paned = gtk::Paned::new(orientation);
                                paned.add_css_class("nexuterm-terminal-split");
                                paned.set_hexpand(true);
                                paned.set_vexpand(true);
                                paned.set_start_child(Some(&old));
                                paned.set_end_child(Some(&new));
                                paned.set_position(
                                    if orientation == gtk::Orientation::Horizontal {
                                        parent.width().max(400) / 2
                                    } else {
                                        parent.height().max(240) / 2
                                    },
                                );
                                parent.append(&paned);
                                terminal_in(
                                    &s,
                                    "Terminal",
                                    session.spec.clone(),
                                    Some(&new),
                                    session.connection.clone(),
                                );
                            }
                            "reconnect" => {
                                session.stop();
                                if let Some(parent) = t.parent().and_downcast::<gtk::Box>() {
                                    while let Some(child) = parent.first_child() {
                                        parent.remove(&child);
                                    }
                                    terminal_in(
                                        &s,
                                        "Terminal",
                                        session.spec.clone(),
                                        Some(&parent),
                                        session.connection.clone(),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
        window.add_action(&action);
    }
    let s = Rc::downgrade(state);
    let update = move || {
        if let Some(s) = s.upgrade() {
            update_actions(&s);
        }
    };
    update();
    state
        .notebook
        .connect_selected_page_notify(move |_| update());
    for (action, keys) in [
        ("sessions", vec!["<Primary><Shift>s"]),
        ("new-terminal", vec!["<Primary><Shift>t"]),
        ("new-connection", vec!["<Primary>n"]),
        ("open-file", vec!["<Primary>o"]),
        ("search", vec!["<Primary><Shift>f"]),
        ("sidebar", vec!["F9"]),
    ] {
        app.set_accels_for_action(&format!("win.{action}"), &keys);
    }
}

pub(super) fn update_actions(s: &State) {
    for session in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
        if let (Some(t), Some(button)) =
            (session.terminal.upgrade(), session.close_button.upgrade())
        {
            button.set_visible(split_parent(&t).is_some());
        }
    }
    if let Some(w) = s.window.upgrade() {
        let session = active_session(s);
        let terminal = session.is_some();
        if let Some(a) = w
            .lookup_action("remote-files")
            .and_downcast::<gio::SimpleAction>()
        {
            a.set_enabled(session.as_ref().is_some_and(|session| {
                session
                    .connection
                    .as_ref()
                    .is_some_and(|connection| connection.profile.protocol == Protocol::Ssh)
            }));
        }
        for name in [
            "search",
            "split-horizontal",
            "split-vertical",
            "reconnect",
            "disconnect",
            "zoom-in",
            "zoom-out",
        ] {
            if let Some(a) = w.lookup_action(name).and_downcast::<gio::SimpleAction>() {
                let sftp = session.as_ref().is_some_and(|session| {
                    session
                        .connection
                        .as_ref()
                        .is_some_and(|connection| connection.profile.protocol == Protocol::Sftp)
                });
                a.set_enabled(
                    terminal && !(sftp && matches!(name, "split-horizontal" | "split-vertical")),
                );
            }
        }
    }
}

fn split_parent(term: &vte4::Terminal) -> Option<(gtk::Widget, gtk::Paned)> {
    let mut branch = term.parent()?;
    while let Some(parent) = branch.parent() {
        if parent.has_css_class("nexuterm-terminal-split") {
            return Some((branch, parent.downcast().ok()?));
        }
        branch = parent;
    }
    None
}

pub(super) fn close_split(state: &State, session: &Session) {
    let Some(term) = session.terminal.upgrade() else {
        return;
    };
    let Some((branch, split)) = split_parent(&term) else {
        return;
    };
    let Some(parent) = split.parent().and_downcast::<gtk::Box>() else {
        return;
    };
    let sibling = if split.start_child().as_ref() == Some(&branch) {
        split.end_child()
    } else {
        split.start_child()
    };
    let Some(sibling) = sibling else {
        return;
    };
    session.stop();
    split.set_start_child(gtk::Widget::NONE);
    split.set_end_child(gtk::Widget::NONE);
    parent.remove(&split);
    parent.append(&sibling);
    let next = state
        .sessions
        .borrow()
        .iter()
        .filter_map(Weak::upgrade)
        .filter_map(|s| s.terminal.upgrade())
        .find(|t| t.is_ancestor(&sibling));
    if let Some(next) = next {
        next.grab_focus();
    }
    update_actions(state);
}

#[cfg(debug_assertions)]
pub(super) fn smoke_close_splits(state: &State, window: &gtk::ApplicationWindow) {
    let terminals = || {
        let page = state.notebook.selected_page().unwrap().child();
        state
            .sessions
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .filter(|s| s.terminal.upgrade().is_some_and(|t| t.is_ancestor(&page)))
            .collect::<Vec<_>>()
    };
    // Close the nested leaf, then the surviving inner branch.
    for expected in [2, 1] {
        let current = active_session(state).unwrap();
        let close = current.close_button.upgrade().unwrap();
        assert!(close.is_visible());
        close.emit_clicked();
        assert!(current.cancelled.get());
        assert_eq!(terminals().len(), expected);
        assert_eq!(state.notebook.n_pages(), 1);
    }
    let survivor = terminals().pop().unwrap();
    assert!(!survivor.close_button.upgrade().unwrap().is_visible());
    assert!(split_parent(&survivor.terminal.upgrade().unwrap()).is_none());
    // Also close the original side, keeping the newly created terminal alive.
    gio::prelude::ActionGroupExt::activate_action(window, "split-horizontal", None);
    assert_eq!(terminals().len(), 2);
    survivor.close_button.upgrade().unwrap().emit_clicked();
    assert!(survivor.cancelled.get());
    let remaining = terminals();
    assert_eq!(remaining.len(), 1);
    assert!(!remaining[0].cancelled.get());
    assert!(!remaining[0].close_button.upgrade().unwrap().is_visible());
    eprintln!("NEXUTERM_SMOKE_CLOSE_SPLITS_OK");
}

fn terminal_search(term: &vte4::Terminal, window: &gtk::ApplicationWindow) {
    let dialog = gtk::Window::builder()
        .transient_for(window)
        .modal(true)
        .title("Buscar en la terminal")
        .default_width(430)
        .build();
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.set_margin_top(16);
    row.set_margin_bottom(16);
    row.set_margin_start(16);
    row.set_margin_end(16);
    let entry = gtk::SearchEntry::new();
    entry.set_hexpand(true);
    row.append(&entry);
    let next = icon_button("go-down-symbolic", "Siguiente coincidencia");
    row.append(&next);
    let t = term.downgrade();
    let e = entry.clone();
    next.connect_clicked(move |_| {
        if let Some(t) = t.upgrade() {
            let pattern = glib::Regex::escape_string(e.text());
            if let Ok(regex) = vte4::Regex::for_search(&pattern, 0x00000400) {
                t.search_set_regex(Some(&regex), 0);
                t.search_set_wrap_around(true);
                t.search_find_next();
            }
        }
    });
    let n = next.clone();
    entry.connect_activate(move |_| n.emit_clicked());
    dialog.set_child(Some(&row));
    dialog.present();
    entry.grab_focus();
}

#[cfg(debug_assertions)]
pub(super) fn smoke_menus(window: &gtk::ApplicationWindow, menu: &gtk::MenuButton) {
    let window = window.downgrade();
    let menu = menu.downgrade();
    glib::spawn_future_local(async move {
        glib::timeout_future(std::time::Duration::from_millis(300)).await;
        let (Some(window), Some(menu)) = (window.upgrade(), menu.upgrade()) else {
            return;
        };
        let display = gtk::prelude::WidgetExt::display(&window);
        assert!(gtk::IconTheme::for_display(&display).has_icon("io.nexuterm.NexuTerm"));
        // Repeat opening/closing each dialog with the menu already holding focus.
        for _ in 0..2 {
            for action in [
                "preferences",
                "about",
                "new-connection",
                "search",
                "sessions",
            ] {
                menu.popup();
                glib::timeout_future(std::time::Duration::from_millis(50)).await;
                assert!(menu.popover().unwrap().is_visible());
                gio::prelude::ActionGroupExt::activate_action(&window, action, None);
                glib::timeout_future(std::time::Duration::from_millis(70)).await;
                assert!(!menu.popover().unwrap().is_visible());
                let dialogs: Vec<gtk::Window> = gtk::Window::list_toplevels()
                    .into_iter()
                    .filter_map(|w| w.downcast::<gtk::Window>().ok())
                    .filter(|w| w.transient_for().as_ref() == Some(window.upcast_ref()))
                    .collect();
                assert_eq!(dialogs.len(), 1, "Falta el diálogo de {action}");
                for dialog in dialogs {
                    dialog.close();
                }
            }
        }
        eprintln!("NEXUTERM_SMOKE_MENUS_OK");
    });
}

fn apply_theme(theme: &str) {
    adw::StyleManager::default().set_color_scheme(match theme {
        "dark" => adw::ColorScheme::ForceDark,
        "light" => adw::ColorScheme::ForceLight,
        _ => adw::ColorScheme::Default,
    });
}

fn preferences(state: &Rc<State>, window: &gtk::ApplicationWindow) {
    let current = state.config.borrow().preferences.clone();
    let dialog = gtk::Window::builder()
        .transient_for(window)
        .modal(true)
        .title("Preferencias")
        .default_width(480)
        .build();
    let grid = gtk::Grid::builder()
        .row_spacing(12)
        .column_spacing(16)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();
    let theme = gtk::DropDown::from_strings(&["Sistema", "Claro", "Oscuro"]);
    theme.set_selected(match current.theme.as_str() {
        "light" => 1,
        "dark" => 2,
        _ => 0,
    });
    grid.attach(&gtk::Label::new(Some("Apariencia")), 0, 0, 1, 1);
    grid.attach(&theme, 1, 0, 1, 1);
    let font_description = gtk::pango::FontDescription::from_string(&current.font);
    let current_family = font_description
        .family()
        .map(|family| family.to_string())
        .unwrap_or_else(|| "Monospace".into());
    let mut families: Vec<String> = window
        .pango_context()
        .list_families()
        .iter()
        .map(|family| family.name().to_string())
        .collect();
    if !families.contains(&current_family) {
        families.push(current_family.clone());
    }
    families.sort_by_cached_key(|family| family.to_lowercase());
    families.dedup();
    let family_names: Vec<&str> = families.iter().map(String::as_str).collect();
    let font = gtk::DropDown::from_strings(&family_names);
    font.set_enable_search(true);
    font.set_hexpand(true);
    font.set_selected(
        families
            .iter()
            .position(|name| name == &current_family)
            .unwrap() as u32,
    );
    let size = gtk::SpinButton::with_range(1.0, 512.0, 0.5);
    size.set_digits(1);
    let current_size = f64::from(font_description.size()) / f64::from(gtk::pango::SCALE);
    size.set_value(if current_size > 0.0 {
        current_size
    } else {
        11.0
    });
    size.set_tooltip_text(Some("Tamaño de la fuente"));
    let font_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    font_row.append(&font);
    font_row.append(&size);
    grid.attach(&gtk::Label::new(Some("Fuente de terminal")), 0, 1, 1, 1);
    grid.attach(&font_row, 1, 1, 1, 1);
    let lines = entry(
        &grid,
        2,
        "Líneas de historial",
        &current.scrollback.to_string(),
    );
    let side = gtk::CheckButton::with_label("Mostrar explorador al iniciar");
    side.set_active(current.sidebar_visible);

    let external = gtk::CheckButton::with_label("Abrir archivos con la aplicación del sistema");
    external.set_active(current.files_use_system_app);

    let numbers = gtk::CheckButton::with_label("Mostrar números de línea en el editor");
    numbers.set_active(current.editor_line_numbers);
    grid.attach(&numbers, 0, 3, 2, 1);
    let browser = gtk::Box::new(gtk::Orientation::Vertical, 12);
    for set in [gtk::Widget::set_margin_top, gtk::Widget::set_margin_bottom, gtk::Widget::set_margin_start, gtk::Widget::set_margin_end] { set(browser.upcast_ref(), 24); }
    browser.append(&side); browser.append(&external);
    let hidden = gtk::CheckButton::with_label("Mostrar archivos ocultos al abrir el navegador");
    hidden.set_active(current.browser_show_hidden); browser.append(&hidden);
    let columns_title = gtk::Label::new(Some("Columnas de la vista de detalles")); columns_title.set_xalign(0.0); columns_title.add_css_class("heading"); browser.append(&columns_title);
    let name_column = gtk::CheckButton::with_label("Nombre (siempre visible)"); name_column.set_active(true); name_column.set_sensitive(false); browser.append(&name_column);
    let type_column = gtk::CheckButton::with_label("Tipo"); type_column.set_active(current.browser_show_type); browser.append(&type_column);
    let size_column = gtk::CheckButton::with_label("Tamaño"); size_column.set_active(current.browser_show_size); browser.append(&size_column);
    let permissions_column = gtk::CheckButton::with_label("Permisos"); permissions_column.set_active(current.browser_show_permissions); browser.append(&permissions_column);
    let help = gtk::Label::new(Some("Ctrl+H o el botón del ojo alternan los ocultos en el panel enfocado. La terminal conserva su atajo de teclado.")); help.set_wrap(true); help.set_xalign(0.0); help.add_css_class("dim-label"); browser.append(&help);
    let tabs = gtk::Notebook::new();
    tabs.append_page(&grid, Some(&gtk::Label::new(Some("General"))));
    tabs.append_page(&browser, Some(&gtk::Label::new(Some("Navegador"))));
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12); content.append(&tabs);
    let footer = gtk::Box::new(gtk::Orientation::Vertical, 8); footer.set_margin_start(24); footer.set_margin_end(24); footer.set_margin_bottom(18);
    content.append(&footer);
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    footer.append(&error);
    let save = gtk::Button::with_label("Guardar");
    save.add_css_class("suggested-action");
    save.set_sensitive(state.config_writable);
    save.set_halign(gtk::Align::End);
    footer.append(&save);
    let s = Rc::downgrade(state);
    let d = dialog.downgrade();
    save.connect_clicked(move |_| {
        let Some(s) = s.upgrade() else {
            return;
        };
        let Ok(scrollback) = lines.text().parse::<u32>() else {
            error.set_text("El historial debe ser un número entero.");
            return;
        };
        if scrollback > 1_000_000 {
            error.set_text("Indica un historial de hasta 1.000.000 líneas.");
            return;
        }
        let mut config = s.config.borrow().clone();
        config.preferences.theme = match theme.selected() {
            1 => "light",
            2 => "dark",
            _ => "system",
        }
        .into();
        let Some(family) = families.get(font.selected() as usize) else {
            error.set_text("Selecciona una fuente de terminal.");
            return;
        };
        let mut selected_font = font_description.clone();
        selected_font.set_family(family);
        let scaled_size = size.value() * f64::from(gtk::pango::SCALE);
        if selected_font.is_size_absolute() {
            selected_font.set_absolute_size(scaled_size);
        } else {
            selected_font.set_size(scaled_size.round() as i32);
        }
        config.preferences.font = selected_font.to_string();
        config.preferences.scrollback = scrollback;
        config.preferences.sidebar_visible = side.is_active();
        config.preferences.files_use_system_app = external.is_active();
        config.preferences.editor_line_numbers = numbers.is_active();
        config.preferences.browser_show_hidden = hidden.is_active();
        config.preferences.browser_show_type = type_column.is_active();
        config.preferences.browser_show_size = size_column.is_active();
        config.preferences.browser_show_permissions = permissions_column.is_active();
        if let Err(e) = config::save(&s.path, &config) {
            error.set_text(&e);
            return;
        }
        apply_theme(&config.preferences.theme);
        for session in s.sessions.borrow().iter().filter_map(Weak::upgrade) {
            if let Some(t) = session.terminal.upgrade() {
                t.set_font(Some(&gtk::pango::FontDescription::from_string(
                    &config.preferences.font,
                )));
                t.set_scrollback_lines(scrollback.into());
            }
        }
        *s.config.borrow_mut() = config;
        apply_browser_preferences(&s);
        s.sidebar_area.set_visible(side.is_active());
        if let Some(d) = d.upgrade() {
            d.close();
        }
    });
    dialog.set_child(Some(&content));
    dialog.present();
}
