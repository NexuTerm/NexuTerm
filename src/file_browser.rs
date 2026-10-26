//! Local/remote panes shared by the standalone FTP and SFTP views.
use crate::{
    remote_panel::Panel,
    ui::{State, icon_button},
};
use gtk::{gio, glib, prelude::*};
use nexuterm::transfer::{Entry, PAGE_SIZE};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::{Rc, Weak},
};
const PRIORITY: glib::Priority = glib::Priority::DEFAULT;

pub struct LocalPane {
    pub widget: gtk::Box,
    current: RefCell<PathBuf>,
    path: gtk::Entry,
    list: gtk::ListBox,
    entries: RefCell<Vec<Entry>>,
    cursor: RefCell<Option<gio::FileEnumerator>>,
    busy: Cell<bool>,
    next: gtk::Button,
    status: gtk::Label,
    remote: Weak<Panel>,
    cancel: gio::Cancellable,
    state: Weak<State>,
    standalone: bool,
    hidden: gtk::ToggleButton,
    #[cfg(debug_assertions)]
    hidden_keys: gtk::EventControllerKey,
    #[cfg(debug_assertions)]
    features_test_started: Cell<bool>,
}
impl LocalPane {
    pub fn apply_preferences(&self, preferences: &nexuterm::config::Preferences) {
        apply_columns(self.widget.upcast_ref(), preferences);
        self.hidden.set_active(preferences.browser_show_hidden);
    }
    fn create(self: &Rc<Self>, target: PathBuf, folder: bool) {
        if self.busy.get() {
            return;
        }
        self.busy.set(true);
        self.widget.set_sensitive(false);
        let pane = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let file = gio::File::for_path(&target);
            let result = if folder {
                file.make_directory_future(PRIORITY).await
            } else {
                match file
                    .create_future(gio::FileCreateFlags::NONE, PRIORITY)
                    .await
                {
                    Ok(stream) => stream.close_future(PRIORITY).await.map(|_| ()),
                    Err(error) => Err(error),
                }
            };
            if let Some(pane) = pane.upgrade() {
                pane.busy.set(false);
                pane.widget.set_sensitive(true);
                match result {
                    Ok(()) => pane.refresh(),
                    Err(error) => {
                        pane.status.set_text(&format!("No se pudo crear: {error}"));
                        pane.status.set_visible(true);
                    }
                }
            }
        });
    }
    pub fn is_loading(&self) -> bool {
        self.busy.get()
    }
    pub fn directory(&self) -> PathBuf {
        self.current.borrow().clone()
    }
    pub fn selected_file(&self) -> Option<PathBuf> {
        let row = self.list.selected_row()?;
        let entries = self.entries.borrow();
        let entry = entries.get(row.index() as usize)?;
        entry
            .regular
            .then(|| self.current.borrow().join(&entry.name))
    }
    pub fn refresh(self: &Rc<Self>) {
        self.navigate(self.directory());
    }
    pub fn stop(&self) {
        self.cancel.cancel();
    }
    pub fn local(state: &Rc<State>) -> Rc<Self> {
        Self::new(state, None)
    }
    fn new(state: &Rc<State>, remote: Option<&Rc<Panel>>) -> Rc<Self> {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("nexuterm-section");
        widget.add_css_class("nexuterm-panel");
        widget.add_css_class("local-explorer");
        widget.set_margin_top(2);
        widget.set_margin_bottom(8);
        widget.set_width_request(240);
        let title = gtk::Label::new(Some("ARCHIVOS LOCALES"));
        title.set_xalign(0.0);
        title.add_css_class("heading");
        let heading = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        heading.add_css_class("explorer-heading");
        title.set_hexpand(true);
        heading.append(&title);
        widget.append(&heading);
        let tools = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let up = icon_button("go-up-symbolic", "Carpeta local superior");
        let home = icon_button("user-home-symbolic", "Carpeta personal local");
        let refresh = icon_button("view-refresh-symbolic", "Actualizar archivos locales");
        let upload = icon_button("go-next-symbolic", "Subir el archivo local seleccionado");
        for button in [&up, &home, &refresh, &upload] {
            tools.append(button);
        }
        upload.set_visible(remote.is_some());
        if remote.is_none() {
            let open = icon_button("document-open-symbolic", "Abrir archivo · Ctrl+O");
            open.set_action_name(Some("win.open-file"));
            tools.append(&open);
        }
        widget.append(&tools);
        let path = gtk::Entry::new();
        path.set_tooltip_text(Some("Carpeta local · Enter para abrir"));
        widget.append(&path);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.set_activate_on_single_click(false);
        list.add_css_class("file-details");
        list.set_overflow(gtk::Overflow::Hidden);
        let (hidden, _keys) = hidden_controls(
            &widget,
            &list,
            state.config.borrow().preferences.browser_show_hidden,
        );
        heading.append(&hidden);
        for (icon, label, folder) in [
            ("folder-new-symbolic", "Nueva carpeta", true),
            ("document-new-symbolic", "Nuevo archivo", false),
        ] {
            let button = icon_button(icon, label);
            let weak = Rc::downgrade(state);
            let directory = path.downgrade();
            button.connect_clicked(move |_| {
                let Some(state) = weak.upgrade() else {
                    return;
                };
                let Some(window) = state.window() else {
                    return;
                };
                let Some(directory) = directory.upgrade() else {
                    return;
                };
                let pane = state
                    .local_browsers
                    .borrow()
                    .iter()
                    .filter_map(Weak::upgrade)
                    .find(|p| p.path == directory);
                let Some(pane) = pane else {
                    return;
                };
                let current = pane.directory();
                let weak = Rc::downgrade(&pane);
                create_dialog(&window, folder, move |name| {
                    if let Some(pane) = weak.upgrade() {
                        pane.create(current.join(name), folder);
                    }
                });
            });
            tools.append(&button);
        }
        let columns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        for (caption, width, expand) in [
            ("Nombre", 144, true),
            ("Tipo", 70, false),
            ("Tamaño", 76, false),
            ("Permisos", 96, false),
        ] {
            let label = gtk::Label::new(Some(caption));
            label.set_xalign(0.0);
            match caption {
                "Tipo" => label.add_css_class("browser-type"),
                "Tamaño" => label.add_css_class("browser-size"),
                "Permisos" => label.add_css_class("browser-permissions"),
                _ => {}
            }
            label.set_width_request(width);
            label.set_hexpand(expand);
            columns.append(&label);
        }
        columns.add_css_class("detail-columns");
        let details = gtk::Box::new(gtk::Orientation::Vertical, 0);
        details.append(&columns);
        details.append(&list);
        widget.append(
            &gtk::ScrolledWindow::builder()
                .child(&details)
                .overlay_scrolling(false)
                .vexpand(true)
                .hscrollbar_policy(gtk::PolicyType::Automatic)
                .build(),
        );
        let next = icon_button("go-next-symbolic", "Mostrar los siguientes archivos");
        next.set_sensitive(false);
        next.set_visible(remote.is_some());
        tools.append(&next);
        let status = gtk::Label::new(None);
        status.set_xalign(0.0);
        status.set_wrap(true);
        status.set_visible(remote.is_some());
        status.add_css_class("error");
        widget.append(&status);
        let cancel = gio::Cancellable::new();
        state.operations.borrow_mut().push(cancel.downgrade());
        let panel = Rc::new(Self {
            widget,
            current: RefCell::new(if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some() {
                std::env::var_os("NEXUTERM_SMOKE_FILE_BROWSER_LOCAL")
                    .map(PathBuf::from)
                    .unwrap_or_else(glib::home_dir)
            } else {
                std::env::current_dir().unwrap_or_else(|_| glib::home_dir())
            }),
            path,
            list,
            entries: RefCell::new(Vec::new()),
            cursor: RefCell::new(None),
            busy: Cell::new(false),
            next,
            status,
            remote: remote.map(Rc::downgrade).unwrap_or_default(),
            state: Rc::downgrade(state),
            standalone: remote.is_none(),
            hidden,
            #[cfg(debug_assertions)]
            hidden_keys: _keys,
            #[cfg(debug_assertions)]
            features_test_started: Cell::new(false),
            cancel,
        });
        state
            .local_browsers
            .borrow_mut()
            .retain(|p| p.strong_count() > 0);
        state
            .local_browsers
            .borrow_mut()
            .push(Rc::downgrade(&panel));
        panel.apply_preferences(&state.config.borrow().preferences);
        let p = Rc::downgrade(&panel);
        up.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                let path = p
                    .directory()
                    .parent()
                    .unwrap_or(std::path::Path::new("/"))
                    .to_path_buf();
                p.navigate(path);
            }
        });
        let p = Rc::downgrade(&panel);
        home.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.navigate(glib::home_dir());
            }
        });
        let p = Rc::downgrade(&panel);
        refresh.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.refresh();
            }
        });
        let p = Rc::downgrade(&panel);
        upload.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.upload();
            }
        });
        let p = Rc::downgrade(&panel);
        panel.path.connect_activate(move |entry| {
            if let Some(p) = p.upgrade() {
                p.navigate(entry.text().as_str().into());
            }
        });
        let p = Rc::downgrade(&panel);
        panel.next.connect_clicked(move |_| {
            if let Some(p) = p.upgrade() {
                p.page();
            }
        });
        let p = Rc::downgrade(&panel);
        panel.list.connect_row_activated(move |_, row| {
            if let Some(p) = p.upgrade() {
                let entry = p.entries.borrow().get(row.index() as usize).cloned();
                if let Some(entry) = entry {
                    if entry.directory {
                        p.navigate(p.directory().join(entry.name));
                    } else if p.standalone && entry.regular {
                        if let Some(state) = p.state.upgrade() {
                            crate::ui::open_local_file(&state, p.directory().join(entry.name));
                        }
                    } else if !p.standalone {
                        p.upload();
                    }
                }
            }
        });
        panel.refresh();
        panel
    }
    #[cfg(debug_assertions)]
    pub fn smoke_upload(&self, name: &str) {
        let index = self
            .entries
            .borrow()
            .iter()
            .position(|entry| entry.name == std::path::Path::new(name))
            .expect("Local upload fixture listed");
        self.list
            .select_row(self.list.row_at_index(index as i32).as_ref());
        self.upload();
    }
    fn upload(&self) {
        if let (Some(remote), Some(path)) = (self.remote.upgrade(), self.selected_file()) {
            remote.upload_file(path);
        } else {
            self.status
                .set_text("Selecciona un archivo local para subir.");
        }
    }
    pub fn navigate(self: &Rc<Self>, directory: PathBuf) {
        if self.busy.replace(true) || self.cancel.is_cancelled() {
            return;
        }
        self.widget.set_sensitive(false);
        let p = Rc::downgrade(self);
        let cancel = self.cancel.clone();
        glib::spawn_future_local(async move {
            let directory = if directory.is_absolute() {
                directory
            } else if let Some(p) = p.upgrade() {
                p.directory().join(directory)
            } else {
                return;
            };
            let future = gio::File::for_path(&directory).enumerate_children_future(
                "standard::name,standard::type,standard::size,unix::mode",
                gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                PRIORITY,
            );
            let result = gio::CancellableFuture::new(future, cancel.clone())
                .await
                .unwrap_or_else(|error| Err(error.into()));
            if let Some(p) = p.upgrade() {
                p.busy.set(false);
                p.widget.set_sensitive(true);
                if cancel.is_cancelled() {
                    return;
                }
                match result {
                    Ok(cursor) => {
                        *p.cursor.borrow_mut() = Some(cursor);
                        *p.current.borrow_mut() = directory.clone();
                        p.path.set_text(&directory.to_string_lossy());
                        p.page();
                    }
                    Err(error) => {
                        p.status.set_text(&error.to_string());
                        p.status.set_visible(true);
                    }
                }
            }
        });
    }
    fn page(self: &Rc<Self>) {
        if self.busy.replace(true) || self.cancel.is_cancelled() {
            return;
        }
        let Some(cursor) = self.cursor.borrow().clone() else {
            self.busy.set(false);
            return;
        };
        self.widget.set_sensitive(false);
        let p = Rc::downgrade(self);
        let cancel = self.cancel.clone();
        glib::spawn_future_local(async move {
            let future = cursor.next_files_future(PAGE_SIZE as i32, PRIORITY);
            let result = gio::CancellableFuture::new(future, cancel)
                .await
                .unwrap_or_else(|error| Err(error.into()));
            if let Some(p) = p.upgrade() {
                p.busy.set(false);
                p.widget.set_sensitive(true);
                if p.cancel.is_cancelled() {
                    return;
                }
                match result {
                    Err(error) => {
                        p.status.set_text(&error.to_string());
                        p.status.set_visible(true);
                    }
                    Ok(infos) => {
                        p.next.set_sensitive(infos.len() == PAGE_SIZE);
                        p.next.set_visible(infos.len() == PAGE_SIZE);
                        p.status.set_visible(!p.standalone);
                        let mut entries: Vec<_> = infos
                            .into_iter()
                            .map(|info| Entry {
                                name: info.name(),
                                directory: info.file_type() == gio::FileType::Directory,
                                regular: info.file_type() == gio::FileType::Regular,
                                size: u64::try_from(info.size()).ok(),
                                permissions: info
                                    .has_attribute("unix::mode")
                                    .then(|| info.attribute_uint32("unix::mode")),
                            })
                            .collect();
                        entries.sort_by(|a, b| {
                            b.directory
                                .cmp(&a.directory)
                                .then_with(|| a.name.cmp(&b.name))
                        });
                        while let Some(row) = p.list.first_child() {
                            p.list.remove(&row);
                        }
                        for (index, entry) in entries.iter().enumerate() {
                            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                            let name_cell = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                            name_cell.set_width_request(144);
                            name_cell.set_hexpand(true);
                            name_cell.append(&gtk::Image::from_icon_name(if entry.directory {
                                "folder-symbolic"
                            } else {
                                "text-x-generic-symbolic"
                            }));
                            let name = gtk::Label::new(Some(&entry.name.to_string_lossy()));
                            name.set_xalign(0.0);
                            name.set_hexpand(true);
                            name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                            name_cell.append(&name);
                            row.append(&name_cell);
                            let kind = gtk::Label::new(Some(if entry.directory {
                                "Carpeta"
                            } else if entry.regular {
                                "Archivo"
                            } else {
                                "Enlace / otro"
                            }));
                            kind.add_css_class("browser-type");
                            kind.set_width_request(70);
                            kind.set_xalign(0.0);
                            kind.add_css_class("dim-label");
                            row.append(&kind);
                            let size = gtk::Label::new(
                                entry
                                    .size
                                    .filter(|_| entry.regular)
                                    .map(glib::format_size)
                                    .as_deref(),
                            );
                            size.add_css_class("browser-size");
                            size.set_width_request(76);
                            size.set_xalign(1.0);
                            row.append(&size);
                            row.add_css_class(if index % 2 == 0 {
                                "file-even"
                            } else {
                                "file-odd"
                            });
                            let permissions = gtk::Label::new(Some(
                                &nexuterm::transfer::permission_text(entry.permissions),
                            ));
                            permissions.add_css_class("browser-permissions");
                            permissions.add_css_class("monospace");
                            permissions.set_width_request(96);
                            row.append(&permissions);
                            let item = gtk::ListBoxRow::new();
                            item.set_child(Some(&row));
                            if entry.name.as_os_str().as_encoded_bytes().starts_with(b".") {
                                item.add_css_class("hidden-file");
                            }
                            p.list.append(&item);
                        }
                        p.status.set_text(&format!("{} entradas", entries.len()));
                        *p.entries.borrow_mut() = entries;
                        if let Some(state) = p.state.upgrade() {
                            apply_columns(
                                p.widget.upcast_ref(),
                                &state.config.borrow().preferences,
                            );
                        }
                        p.list.invalidate_filter();
                        #[cfg(debug_assertions)]
                        if p.standalone
                            && std::env::var_os("NEXUTERM_SMOKE_BROWSER_FEATURES").is_some()
                            && !p.features_test_started.replace(true)
                        {
                            p.smoke_features();
                        }
                        #[cfg(debug_assertions)]
                        if p.standalone
                            && std::env::var_os("NEXUTERM_SMOKE_LOCAL_EXPLORER").is_some()
                        {
                            let name = if p
                                .directory()
                                .file_name()
                                .is_some_and(|name| name == "nested")
                            {
                                "sample con á.txt"
                            } else {
                                "nested"
                            };
                            let index = p
                                .entries
                                .borrow()
                                .iter()
                                .position(|entry| entry.name == std::path::Path::new(name))
                                .expect("Explorer fixture listed");
                            assert!(!p.list.activates_on_single_click());
                            let row = p.list.row_at_index(index as i32).unwrap();
                            if name == "sample con á.txt" {
                                eprintln!("NEXUTERM_SMOKE_LOCAL_NAVIGATION_OK");
                            }
                            if name == "nested" {
                                let list = p.list.downgrade();
                                glib::timeout_add_local_once(
                                    std::time::Duration::from_millis(1300),
                                    move || {
                                        if let Some(list) = list.upgrade() {
                                            list.emit_by_name::<()>("row-activated", &[&row]);
                                        }
                                    },
                                );
                            } else {
                                p.list.emit_by_name::<()>("row-activated", &[&row]);
                            }
                        }
                    }
                }
            }
        });
    }
}

pub fn workspace(state: &Rc<State>, remote: &Rc<Panel>) -> (gtk::Paned, Rc<LocalPane>) {
    let local = LocalPane::new(state, Some(remote));
    remote.bind_local(&local);
    let split = gtk::Paned::new(gtk::Orientation::Horizontal);
    split.add_css_class("nexuterm-split");
    split.set_hexpand(true);
    split.set_vexpand(true);
    split.set_position(400);
    split.set_start_child(Some(&local.widget));
    split.set_end_child(Some(&remote.widget));
    split.set_shrink_start_child(false);
    split.set_shrink_end_child(false);
    (split, local)
}

pub(crate) fn apply_columns(root: &gtk::Widget, preferences: &nexuterm::config::Preferences) {
    for (class, visible) in [
        ("browser-type", preferences.browser_show_type),
        ("browser-size", preferences.browser_show_size),
        ("browser-permissions", preferences.browser_show_permissions),
    ] {
        if root.has_css_class(class) {
            root.set_visible(visible);
        }
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        apply_columns(&widget, preferences);
        child = widget.next_sibling();
    }
}

pub(crate) fn hidden_controls(
    root: &gtk::Box,
    list: &gtk::ListBox,
    initial: bool,
) -> (gtk::ToggleButton, gtk::EventControllerKey) {
    let button = gtk::ToggleButton::builder()
        .icon_name("view-reveal-symbolic")
        .tooltip_text("Mostrar u ocultar archivos ocultos · Ctrl+H")
        .active(initial)
        .build();
    button.add_css_class("flat");
    button.add_css_class("browser-hidden-toggle");
    button.update_property(&[gtk::accessible::Property::Label("Mostrar archivos ocultos")]);
    let weak = button.downgrade();
    list.set_filter_func(move |row| {
        !row.has_css_class("hidden-file") || weak.upgrade().is_some_and(|b| b.is_active())
    });
    let weak = list.downgrade();
    button.connect_toggled(move |_| {
        if let Some(list) = weak.upgrade() {
            list.unselect_all();
            list.invalidate_filter();
        }
    });
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = button.downgrade();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if matches!(key, gtk::gdk::Key::h | gtk::gdk::Key::H)
            && modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            && !modifiers
                .intersects(gtk::gdk::ModifierType::ALT_MASK | gtk::gdk::ModifierType::SUPER_MASK)
            && let Some(button) = weak.upgrade()
        {
            button.set_active(!button.is_active());
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    root.add_controller(keys.clone());
    (button, keys)
}

pub(crate) fn create_dialog(
    parent: &gtk::ApplicationWindow,
    folder: bool,
    create: impl Fn(PathBuf) + 'static,
) {
    let dialog = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .title(if folder {
            "Nueva carpeta"
        } else {
            "Nuevo archivo"
        })
        .default_width(360)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    for set in [
        gtk::Widget::set_margin_top,
        gtk::Widget::set_margin_bottom,
        gtk::Widget::set_margin_start,
        gtk::Widget::set_margin_end,
    ] {
        set(root.upcast_ref(), 18);
    }
    let name = gtk::Entry::builder()
        .placeholder_text("Nombre")
        .activates_default(true)
        .build();
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.add_css_class("error");
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    actions.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label("Cancelar");
    let submit = gtk::Button::with_label("Crear");
    submit.add_css_class("suggested-action");
    actions.append(&cancel);
    actions.append(&submit);
    root.append(&name);
    root.append(&error);
    root.append(&actions);
    dialog.set_child(Some(&root));
    dialog.set_default_widget(Some(&submit));
    let weak = dialog.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(dialog) = weak.upgrade() {
            dialog.close();
        }
    });
    let weak = dialog.downgrade();
    submit.connect_clicked(move |_| {
        match nexuterm::transfer::creation_name(name.text().as_str()) {
            Ok(name) => {
                create(name);
                if let Some(dialog) = weak.upgrade() {
                    dialog.close();
                }
            }
            Err(message) => error.set_text(&message),
        }
    });
    dialog.present();
}

#[cfg(debug_assertions)]
impl LocalPane {
    fn smoke_features(self: &Rc<Self>) {
        fn widgets(root: &gtk::Widget) -> Vec<gtk::Widget> {
            let mut result = vec![root.clone()];
            let mut child = root.first_child();
            while let Some(widget) = child {
                result.extend(widgets(&widget));
                child = widget.next_sibling();
            }
            result
        }
        fn dialog(title: &str) -> gtk::Window {
            gtk::Window::list_toplevels()
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::Window>().ok())
                .find(|w| w.title().as_deref() == Some(title))
                .expect("Expected test dialog")
        }
        fn button(root: &gtk::Widget, text: &str) -> gtk::Button {
            widgets(root)
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::Button>().ok())
                .find(|b| {
                    b.label().as_deref() == Some(text) || b.tooltip_text().as_deref() == Some(text)
                })
                .expect("Expected test button")
        }
        async fn settle(pane: &LocalPane) {
            for _ in 0..100 {
                glib::timeout_future(std::time::Duration::from_millis(20)).await;
                if !pane.busy.get() {
                    return;
                }
            }
            panic!("Browser operation did not settle");
        }
        let pane = self.clone();
        glib::spawn_future_local(async move {
            settle(&pane).await;
            let hidden_row = {
                let entries = pane.entries.borrow();
                let index = entries
                    .iter()
                    .position(|e| e.name == std::path::Path::new(".hidden-file"))
                    .unwrap();
                assert_eq!(
                    nexuterm::transfer::permission_text(entries[index].permissions),
                    "rw-r-----"
                );
                let folder = entries
                    .iter()
                    .find(|e| e.name == std::path::Path::new("nested"))
                    .unwrap();
                assert_eq!(
                    nexuterm::transfer::permission_text(folder.permissions),
                    "rwxr-x---"
                );
                pane.list.row_at_index(index as i32).unwrap()
            };
            assert!(!hidden_row.is_child_visible());
            pane.hidden.set_active(true);
            settle(&pane).await;
            assert!(hidden_row.is_child_visible());
            assert!(pane.hidden_keys.emit_by_name::<bool>(
                "key-pressed",
                &[
                    &gtk::gdk::Key::h,
                    &0_u32,
                    &gtk::gdk::ModifierType::CONTROL_MASK
                ]
            ));
            settle(&pane).await;
            assert!(!hidden_row.is_child_visible());
            eprintln!("NEXUTERM_SMOKE_BROWSER_HIDDEN_OK");
            let state = pane.state.upgrade().unwrap();
            let window = state.window().unwrap();
            gtk::gio::prelude::ActionGroupExt::activate_action(&window, "preferences", None);
            let preferences = dialog("Preferencias");
            let tabs = widgets(preferences.upcast_ref())
                .into_iter()
                .find_map(|w| w.downcast::<gtk::Notebook>().ok())
                .unwrap();
            assert_eq!(tabs.n_pages(), 2);
            assert_eq!(
                tabs.tab_label_text(&tabs.nth_page(Some(1)).unwrap())
                    .as_deref(),
                Some("Navegador")
            );
            tabs.set_current_page(Some(1));
            glib::timeout_future(std::time::Duration::from_millis(100)).await;
            if let Some(path) = std::env::var_os("NEXUTERM_BROWSER_PREFERENCES_SCREENSHOT") {
                let paintable = gtk::WidgetPaintable::new(Some(&preferences));
                let snapshot = gtk::Snapshot::new();
                paintable.snapshot(
                    &snapshot,
                    f64::from(preferences.width()),
                    f64::from(preferences.height()),
                );
                if let (Some(node), Some(renderer)) = (snapshot.to_node(), preferences.renderer()) {
                    renderer
                        .render_texture(&node, None)
                        .save_to_png(path)
                        .unwrap();
                }
            }
            for check in widgets(preferences.upcast_ref())
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::CheckButton>().ok())
            {
                if matches!(
                    check.label().as_deref(),
                    Some("Tipo" | "Tamaño" | "Permisos")
                ) {
                    check.set_active(false);
                }
            }
            button(preferences.upcast_ref(), "Guardar").emit_clicked();
            settle(&pane).await;
            for widget in widgets(pane.widget.upcast_ref()) {
                if ["browser-type", "browser-size", "browser-permissions"]
                    .iter()
                    .any(|c| widget.has_css_class(c))
                {
                    assert!(!widget.is_visible());
                }
            }
            assert!(!state.config.borrow().preferences.browser_show_permissions);
            // Reopen Preferences to verify persistence and restore just Permissions.
            gtk::gio::prelude::ActionGroupExt::activate_action(&window, "preferences", None);
            let preferences = dialog("Preferencias");
            for check in widgets(preferences.upcast_ref())
                .into_iter()
                .filter_map(|w| w.downcast::<gtk::CheckButton>().ok())
            {
                if check.label().as_deref() == Some("Permisos") {
                    assert!(!check.is_active());
                    check.set_active(true);
                }
            }
            button(preferences.upcast_ref(), "Guardar").emit_clicked();
            settle(&pane).await;
            assert!(state.config.borrow().preferences.browser_show_permissions);
            eprintln!("NEXUTERM_SMOKE_BROWSER_PREFERENCES_OK");
            for (action, name, folder) in [
                ("Nueva carpeta", "created-dir", true),
                ("Nuevo archivo", "created.txt", false),
                ("Nuevo archivo", "existing.txt", false),
            ] {
                button(pane.widget.upcast_ref(), action).emit_clicked();
                let prompt = dialog(action);
                let entry = widgets(prompt.upcast_ref())
                    .into_iter()
                    .find_map(|w| w.downcast::<gtk::Entry>().ok())
                    .unwrap();
                entry.set_text(name);
                button(prompt.upcast_ref(), "Crear").emit_clicked();
                settle(&pane).await;
                let target = pane.directory().join(name);
                assert_eq!(target.is_dir(), folder);
                if name == "existing.txt" {
                    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep original");
                    assert!(pane.status.is_visible());
                } else if !folder {
                    assert_eq!(std::fs::metadata(&target).unwrap().len(), 0);
                }
            }
            button(pane.widget.upcast_ref(), "Nuevo archivo").emit_clicked();
            let prompt = dialog("Nuevo archivo");
            let entry = widgets(prompt.upcast_ref())
                .into_iter()
                .find_map(|w| w.downcast::<gtk::Entry>().ok())
                .unwrap();
            entry.set_text("../outside");
            button(prompt.upcast_ref(), "Crear").emit_clicked();
            assert!(prompt.is_visible());
            button(prompt.upcast_ref(), "Cancelar").emit_clicked();
            let scroll = widgets(pane.widget.upcast_ref())
                .into_iter()
                .find_map(|w| w.downcast::<gtk::ScrolledWindow>().ok())
                .unwrap();
            assert!(!scroll.is_overlay_scrolling());
            let scrollbar = scroll.vscrollbar().compute_bounds(&scroll).unwrap();
            let viewport = scroll.child().unwrap().compute_bounds(&scroll).unwrap();
            assert!(
                viewport.x() + viewport.width() <= scrollbar.x() + 1.0,
                "List overlaps scrollbar"
            );
            eprintln!("NEXUTERM_SMOKE_BROWSER_CREATE_AND_SCROLL_OK");
            pane.refresh();
        });
    }
}
