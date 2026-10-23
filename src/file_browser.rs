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
}
impl LocalPane {
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
    fn new(state: &Rc<State>, remote: &Rc<Panel>) -> Rc<Self> {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("nexuterm-section");
        widget.add_css_class("nexuterm-panel");
        for set in [
            gtk::Widget::set_margin_top,
            gtk::Widget::set_margin_bottom,
            gtk::Widget::set_margin_start,
            gtk::Widget::set_margin_end,
        ] {
            set(widget.upcast_ref(), 8);
        }
        widget.set_width_request(240);
        let title = gtk::Label::new(Some("ARCHIVOS LOCALES"));
        title.set_xalign(0.0);
        title.add_css_class("heading");
        widget.append(&title);
        let tools = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let up = icon_button("go-up-symbolic", "Carpeta local superior");
        let home = icon_button("user-home-symbolic", "Carpeta personal local");
        let refresh = icon_button("view-refresh-symbolic", "Actualizar archivos locales");
        let upload = icon_button("go-next-symbolic", "Subir el archivo local seleccionado");
        for button in [&up, &home, &refresh, &upload] {
            tools.append(button);
        }
        widget.append(&tools);
        let path = gtk::Entry::new();
        path.set_tooltip_text(Some("Carpeta local · Enter para abrir"));
        widget.append(&path);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.add_css_class("navigation-sidebar");
        widget.append(
            &gtk::ScrolledWindow::builder()
                .child(&list)
                .vexpand(true)
                .hscrollbar_policy(gtk::PolicyType::Never)
                .build(),
        );
        let next = gtk::Button::with_label("Página siguiente");
        next.set_sensitive(false);
        widget.append(&next);
        let status = gtk::Label::new(None);
        status.set_xalign(0.0);
        status.set_wrap(true);
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
                glib::home_dir()
            }),
            path,
            list,
            entries: RefCell::new(Vec::new()),
            cursor: RefCell::new(None),
            busy: Cell::new(false),
            next,
            status,
            remote: Rc::downgrade(remote),
            cancel,
        });
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
                    } else {
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
    fn navigate(self: &Rc<Self>, directory: PathBuf) {
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
                "standard::name,standard::type,standard::size",
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
                    Err(error) => p.status.set_text(&error.to_string()),
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
                    Err(error) => p.status.set_text(&error.to_string()),
                    Ok(infos) => {
                        p.next.set_sensitive(infos.len() == PAGE_SIZE);
                        let mut entries: Vec<_> = infos
                            .into_iter()
                            .map(|info| Entry {
                                name: info.name(),
                                directory: info.file_type() == gio::FileType::Directory,
                                regular: info.file_type() == gio::FileType::Regular,
                                size: u64::try_from(info.size()).ok(),
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
                        for entry in &entries {
                            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                            row.append(&gtk::Image::from_icon_name(if entry.directory {
                                "folder-symbolic"
                            } else {
                                "text-x-generic-symbolic"
                            }));
                            let name = gtk::Label::new(Some(&entry.name.to_string_lossy()));
                            name.set_xalign(0.0);
                            name.set_hexpand(true);
                            name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                            row.append(&name);
                            if let Some(size) = entry.size.filter(|_| entry.regular) {
                                row.append(&gtk::Label::new(Some(&glib::format_size(size))));
                            }
                            p.list.append(&row);
                        }
                        p.status.set_text(&format!("{} entradas", entries.len()));
                        *p.entries.borrow_mut() = entries;
                    }
                }
            }
        });
    }
}

pub fn workspace(state: &Rc<State>, remote: &Rc<Panel>) -> (gtk::Paned, Rc<LocalPane>) {
    let local = LocalPane::new(state, remote);
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
