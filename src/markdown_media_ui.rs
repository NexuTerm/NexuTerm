//! Native overlays reserve space using tags; no anchors or objects enter the source buffer.
use gtk::{glib, prelude::*};
use nexuterm::{
    markdown_image::{self as image, Location},
    markdown_media::{self as model, Block, Kind},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, OnceLock},
};
struct Range {
    buffer: glib::WeakRef<gtk::TextBuffer>,
    start: gtk::TextMark,
    end: gtk::TextMark,
    expected: RefCell<String>,
    active: Rc<Cell<bool>>,
}
impl Range {
    fn new(
        buffer: &gtk::TextBuffer,
        start: i32,
        end: i32,
        expected: String,
        cell: bool,
        active: Rc<Cell<bool>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            buffer: buffer.downgrade(),
            start: buffer.create_mark(None, &buffer.iter_at_offset(start), cell),
            end: buffer.create_mark(None, &buffer.iter_at_offset(end), !cell),
            expected: RefCell::new(expected),
            active,
        })
    }
    fn replace(&self, text: &str) -> Result<(), String> {
        if !self.active.get() {
            return Err("La vista ya se ha sustituido; no se modificó el documento".into());
        }
        let b = self.buffer.upgrade().ok_or("Documento cerrado")?;
        let mut a = b.iter_at_mark(&self.start);
        let mut z = b.iter_at_mark(&self.end);
        if a.offset() > z.offset()
            || b.text(&a, &z, true).as_str() != self.expected.borrow().as_str()
        {
            return Err(
                "El documento cambió; vuelve a seleccionar Visual para recargar la tabla".into(),
            );
        }
        let offset = a.offset();
        b.begin_user_action();
        b.delete(&mut a, &mut z);
        b.insert(&mut a, text);
        b.end_user_action();
        b.move_mark(&self.start, &b.iter_at_offset(offset));
        b.move_mark(
            &self.end,
            &b.iter_at_offset(offset + text.chars().count() as i32),
        );
        *self.expected.borrow_mut() = text.into();
        Ok(())
    }
}
impl Drop for Range {
    fn drop(&mut self) {
        if let Some(b) = self.buffer.upgrade() {
            if !self.start.is_deleted() {
                b.delete_mark(&self.start);
            }
            if !self.end.is_deleted() {
                b.delete_mark(&self.end);
            }
        }
    }
}
struct CellEdit {
    entry: glib::WeakRef<gtk::Entry>,
    range: Rc<Range>,
    dirty: Cell<bool>,
    scheduled: Cell<bool>,
}
impl CellEdit {
    fn finish(&self) {
        if self.scheduled.replace(false) {
            if let Some(b) = self.range.buffer.upgrade() {
                b.end_user_action();
            }
        }
    }
    fn commit(&self) -> Result<(), String> {
        if !self.dirty.get() {
            return Ok(());
        }
        let entry = self.entry.upgrade().ok_or("Celda cerrada")?;
        let result = model::cell_text(&entry.text()).and_then(|text| self.range.replace(&text));
        match &result {
            Ok(()) => {
                self.dirty.set(false);
                entry.remove_css_class("error");
                entry.set_tooltip_text(None);
            }
            Err(e) => {
                entry.add_css_class("error");
                entry.set_tooltip_text(Some(e));
            }
        }
        result
    }
}
struct Item {
    widget: gtk::Box,
    range: Rc<Range>,
    space: gtk::TextTag,
    height: i32,
}
pub struct Media {
    view: glib::WeakRef<gtk::TextView>,
    layer: gtk::Fixed,
    base: Option<PathBuf>,
    remote: Option<Rc<RefCell<Option<Arc<nexuterm::remote_edit::RemoteDocument>>>>>,
    items: RefCell<Vec<Item>>,
    signature: RefCell<Vec<String>>,
    hidden: gtk::TextTag,
    cache: RefCell<HashMap<String, gtk::gdk::Texture>>,
    cancel: tokio::sync::watch::Sender<bool>,
    pending: Rc<Cell<usize>>,
}
impl Media {
    pub fn new(
        view: &gtk::TextView,
        base: Option<PathBuf>,
        remote: Option<Rc<RefCell<Option<Arc<nexuterm::remote_edit::RemoteDocument>>>>>,
    ) -> Rc<Self> {
        let hidden = gtk::TextTag::builder().invisible(true).build();
        view.buffer().tag_table().add(&hidden);
        let (cancel, _) = tokio::sync::watch::channel(false);
        // One permanent overlay; GTK4 TextView::remove only handles anchored children.
        // Remove/rebuild our widgets through the public GtkFixed container API.
        let layer = gtk::Fixed::new();
        view.add_overlay(&layer, 0, 0);
        Rc::new(Self {
            cancel,
            layer,
            pending: Rc::new(Cell::new(0)),
            view: view.downgrade(),
            base,
            remote,
            items: RefCell::new(vec![]),
            signature: RefCell::new(vec![]),
            hidden,
            cache: RefCell::new(HashMap::new()),
        })
    }
    pub fn pause(&self) {
        self.cancel.send_replace(true);
        self.clear();
    }
    pub fn resume(&self) {
        self.cancel.send_replace(false);
    }
    pub fn editing(&self) -> bool {
        let Some(v) = self.view.upgrade() else {
            return false;
        };
        let Some(focus) = v
            .root()
            .and_then(|r| r.downcast::<gtk::Window>().ok())
            .and_then(|w| gtk::prelude::GtkWindowExt::focus(&w))
        else {
            return false;
        };
        self.items
            .borrow()
            .iter()
            .any(|i| focus.is_ancestor(&i.widget))
    }
    pub fn clear(&self) {
        let view = self.view.upgrade();
        if let Some(v) = &view {
            let b = v.buffer();
            b.remove_tag(&self.hidden, &b.start_iter(), &b.end_iter());
        }
        for item in self.items.borrow_mut().drain(..) {
            item.range.active.set(false);
            if let Some(v) = &view {
                let b = v.buffer();
                b.remove_tag(&item.space, &b.start_iter(), &b.end_iter());
                b.tag_table().remove(&item.space);
                self.layer.remove(&item.widget);
            }
        }
        self.signature.borrow_mut().clear();
    }
    pub fn sync(self: &Rc<Self>, blocks: Vec<Block>, map: &[i32]) {
        let signature: Vec<_> = blocks.iter().map(|b| b.raw.clone()).collect();
        if *self.signature.borrow() == signature {
            if let Some(v) = self.view.upgrade() {
                let b = v.buffer();
                b.remove_tag(&self.hidden, &b.start_iter(), &b.end_iter());
                for (item, block) in self.items.borrow().iter().zip(&blocks) {
                    let start = map[block.range.start];
                    let end = map[block.range.end];
                    b.remove_tag(&item.space, &b.start_iter(), &b.end_iter());
                    if end - start > 1 {
                        b.apply_tag(
                            &self.hidden,
                            &b.iter_at_offset(start),
                            &b.iter_at_offset(end - 1),
                        );
                    }
                    b.apply_tag(
                        &item.space,
                        &b.iter_at_offset(start),
                        &b.iter_at_offset(end),
                    );
                    b.move_mark(&item.range.start, &b.iter_at_offset(start));
                    b.move_mark(&item.range.end, &b.iter_at_offset(end));
                }
            }
            self.position();
            return;
        }
        self.clear();
        *self.signature.borrow_mut() = signature;
        let Some(view) = self.view.upgrade() else {
            return;
        };
        let buffer = view.buffer();
        for (index, block) in blocks.into_iter().enumerate() {
            if block.range.is_empty() {
                continue;
            }
            let start = map[block.range.start];
            let end = map[block.range.end];
            let range = Range::new(
                &buffer,
                start,
                end,
                block.raw.clone(),
                false,
                Rc::new(Cell::new(true)),
            );
            let widget = gtk::Box::new(gtk::Orientation::Vertical, 3);
            widget.add_css_class("md-media-block");
            widget.set_widget_name(&format!("md-media-{index}"));
            let height = match block.kind {
                Kind::Table(table) => {
                    self.table(&widget, range.clone(), table, map, index);
                    (52 + 32 * (block.raw.lines().count() as i32 - 1)).clamp(116, 390)
                }
                Kind::Image { url, alt } => {
                    self.image(&widget, &url, &alt, index);
                    310
                }
            };
            // Keep one source character as a transparent line with reserved vertical space.
            // This works at EOF too, without appending a newline or touching undo history.
            let space = gtk::TextTag::builder()
                .pixels_above_lines(height)
                .pixels_below_lines(4)
                .size_points(1.0)
                .foreground_rgba(&gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0))
                .build();
            buffer.tag_table().add(&space);
            if end - start > 1 {
                buffer.apply_tag(
                    &self.hidden,
                    &buffer.iter_at_offset(start),
                    &buffer.iter_at_offset(end - 1),
                );
            }
            buffer.apply_tag(
                &space,
                &buffer.iter_at_offset(start),
                &buffer.iter_at_offset(end),
            );
            self.layer.put(&widget, view.left_margin() as f64, 0.0);
            self.items.borrow_mut().push(Item {
                widget,
                range,
                space,
                height,
            });
        }
        self.position();
    }
    pub fn position(&self) {
        let Some(v) = self.view.upgrade() else {
            return;
        };
        let width = (v.width() - v.left_margin() - v.right_margin() - 12).max(180);
        for item in self.items.borrow().iter() {
            let b = v.buffer();
            let end = b.iter_at_mark(&item.range.end).offset();
            if end < 1 {
                continue;
            }
            let rect = v.iter_location(&b.iter_at_offset(end - 1));
            item.widget.set_size_request(width, item.height - 4);
            self.layer.move_(
                &item.widget,
                v.left_margin() as f64,
                (rect.y() - item.height).max(0) as f64,
            );
        }
    }
    fn table(
        self: &Rc<Self>,
        root: &gtk::Box,
        range: Rc<Range>,
        table: model::Table,
        map: &[i32],
        index: usize,
    ) {
        let Some(view) = self.view.upgrade() else {
            return;
        };
        let buffer = view.buffer();
        let rows = table.rows.len();
        let columns = table.align.len();
        let mut edits: Vec<Rc<CellEdit>> = Vec::new();
        let grid = gtk::Grid::builder()
            .column_spacing(0)
            .row_spacing(0)
            .build();
        grid.set_column_homogeneous(true);
        for (r, row) in table.rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                let entry = gtk::Entry::new();
                entry.set_text(cell.raw.trim());
                entry.set_width_chars(10);
                entry.set_max_width_chars(24);
                entry.set_hexpand(true);
                entry.set_max_length(1024);
                entry.set_widget_name(&format!("md-cell-{index}-{r}-{c}"));
                entry.add_css_class("md-table-cell");
                if r == 0 {
                    entry.add_css_class("md-table-heading");
                }
                let align = match table.align.get(c) {
                    Some(pulldown_cmark::Alignment::Center) => 0.5,
                    Some(pulldown_cmark::Alignment::Right) => 1.0,
                    _ => 0.0,
                };
                gtk::prelude::EntryExt::set_alignment(&entry, align);
                let cell_range = Range::new(
                    &buffer,
                    map[cell.range.start],
                    map[cell.range.end],
                    cell.raw.clone(),
                    true,
                    range.active.clone(),
                );
                let edit = Rc::new(CellEdit {
                    entry: entry.downgrade(),
                    range: cell_range,
                    dirty: Cell::new(false),
                    scheduled: Cell::new(false),
                });
                edits.push(edit.clone());
                let edited = edit.clone();
                entry.connect_changed(move |_| {
                    if !edited.range.active.get() {
                        return;
                    }
                    if !edited.scheduled.replace(true) {
                        if let Some(b) = edited.range.buffer.upgrade() {
                            b.begin_user_action();
                        }
                        let edit = edited.clone();
                        glib::idle_add_local_once(move || edit.finish());
                    }
                    edited.dirty.set(true);
                    let _ = edited.commit();
                });
                let keys = gtk::EventControllerKey::new();
                keys.set_propagation_phase(gtk::PropagationPhase::Capture);
                let weak = view.downgrade();
                keys.connect_key_pressed(move |_, key, _, mods| {
                    if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                        && matches!(
                            key,
                            gtk::gdk::Key::z
                                | gtk::gdk::Key::Z
                                | gtk::gdk::Key::y
                                | gtk::gdk::Key::Y
                        )
                    {
                        edit.finish();
                        if let Some(v) = weak.upgrade() {
                            let b = v.buffer();
                            let redo = mods.contains(gtk::gdk::ModifierType::SHIFT_MASK)
                                || matches!(key, gtk::gdk::Key::y | gtk::gdk::Key::Y);
                            if redo && b.can_redo() {
                                b.redo();
                            } else if !redo && b.can_undo() {
                                b.undo();
                            }
                            v.grab_focus();
                        }
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                });
                entry.add_controller(keys);
                grid.attach(&entry, c as i32, r as i32, 1, 1);
            }
        }
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .child(&grid)
            .build();
        root.append(&scroll);
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        for (operation, (label, r, c)) in [
            ("+ Fila", rows + 1, columns),
            ("− Última fila", rows.saturating_sub(1), columns),
            ("+ Columna", rows, columns + 1),
            ("− Última columna", rows, columns.saturating_sub(1)),
        ]
        .into_iter()
        .enumerate()
        {
            let button = gtk::Button::with_label(label);
            button.set_widget_name(&format!("md-table-{index}-action-{operation}"));
            button.add_css_class("flat");
            button.set_sensitive(r >= 1 && c >= 1 && r <= 32 && c <= 12 && r * c <= 256);
            controls.append(&button);
            let range = range.clone();
            let weak = view.downgrade();
            let edits = edits.clone();
            button.connect_clicked(move |button| {
                let result = (|| {
                    for edit in &edits {
                        edit.commit()?;
                        edit.finish();
                    }

                    if !range.active.get() {
                        return Err("Vista sustituida".to_owned());
                    }
                    let b = range.buffer.upgrade().ok_or("Documento cerrado")?;
                    let current = b
                        .text(
                            &b.iter_at_mark(&range.start),
                            &b.iter_at_mark(&range.end),
                            true,
                        )
                        .to_string();
                    let parsed = model::blocks(&current);
                    let Some(Block {
                        kind: Kind::Table(table),
                        ..
                    }) = parsed.first()
                    else {
                        return Err("La tabla ya no es válida".into());
                    };
                    if parsed.len() != 1 || parsed[0].raw != current {
                        return Err("La tabla cambió de estructura; recarga Visual".into());
                    }
                    let rows = table.rows.len();
                    let cols = table.align.len();
                    let (r, c) = match operation {
                        0 => (rows + 1, cols),
                        1 => (rows.saturating_sub(1), cols),
                        2 => (rows, cols + 1),
                        _ => (rows, cols.saturating_sub(1)),
                    };
                    let text = model::table_text(table, r, c)?;
                    *range.expected.borrow_mut() = current;
                    range.replace(&text)
                })();
                if let Err(e) = result {
                    button.set_tooltip_text(Some(&e));
                }
                if let Some(v) = weak.upgrade() {
                    v.grab_focus();
                }
            });
        }
        let actions = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .child(&controls)
            .build();
        root.append(&actions);
    }
    fn image(self: &Rc<Self>, root: &gtk::Box, url: &str, alt: &str, index: usize) {
        let picture = gtk::Picture::new();
        picture.set_widget_name(&format!("md-image-{index}"));
        picture.set_can_shrink(true);
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.set_vexpand(true);
        root.append(&picture);
        let caption = gtk::Label::new(Some(alt));
        caption.set_ellipsize(gtk::pango::EllipsizeMode::End);
        root.append(&caption);
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let load = gtk::Button::with_label("Cargar / actualizar imagen");
        load.set_widget_name(&format!("md-image-load-{index}"));
        load.set_tooltip_text(Some(url));
        controls.append(&load);
        let notice = gtk::Label::new(None);
        notice.set_ellipsize(gtk::pango::EllipsizeMode::End);
        notice.set_max_width_chars(40);
        notice.connect_label_notify(|label| label.set_tooltip_text(Some(&label.text())));
        notice.set_xalign(0.0);
        notice.set_hexpand(true);
        controls.append(&notice);
        root.append(&controls);
        let url = url.to_owned();
        let weak = Rc::downgrade(self);
        let pic = picture.downgrade();
        let msg = notice.downgrade();
        let btn = load.downgrade();
        let key = url.clone();
        load.connect_clicked(move |_| {
            if let Some(c) = weak.upgrade() {
                c.load_image(key.clone(), true, pic.clone(), msg.clone(), btn.clone());
            }
        });
        if let Some(texture) = self.cache.borrow().get(&url) {
            picture.set_paintable(Some(texture));
            notice.set_text("Imagen cargada; actualizar requiere una acción expresa");
        } else {
            self.load_image(
                url,
                false,
                picture.downgrade(),
                notice.downgrade(),
                load.downgrade(),
            );
        }
    }
    fn load_image(
        self: &Rc<Self>,
        url: String,
        explicit: bool,
        picture: glib::WeakRef<gtk::Picture>,
        notice: glib::WeakRef<gtk::Label>,
        button: glib::WeakRef<gtk::Button>,
    ) {
        enum Input {
            Standard(Location),
            Ssh(Arc<nexuterm::remote_edit::RemoteDocument>, String),
        }
        let source = match if self.base.is_none() && url::Url::parse(&url).is_err() {
            if !explicit {
                Err("Imagen SSH: pulsa Cargar para usar la sesión vinculada".into())
            } else if let Some(doc) = self
                .remote
                .as_ref()
                .and_then(|r| r.borrow().as_ref().cloned())
            {
                Ok(Input::Ssh(doc, url.clone()))
            } else {
                Err(
                    "Vincula el documento a una sesión SSH existente antes de cargar la imagen"
                        .into(),
                )
            }
        } else {
            image::location(self.base.as_deref(), &url, explicit).map(Input::Standard)
        } {
            Ok(s) => s,
            Err(e) => {
                if let Some(n) = notice.upgrade() {
                    n.set_text(&e);
                }
                return;
            }
        };
        if self.pending.get() >= 16 {
            if let Some(n) = notice.upgrade() {
                n.set_text("Hay cargas pendientes; vuelve a pulsar Cargar cuando terminen");
            }
            return;
        }
        self.pending.set(self.pending.get() + 1);
        struct Pending(Rc<Cell<usize>>);
        impl Drop for Pending {
            fn drop(&mut self) {
                self.0.set(self.0.get().saturating_sub(1));
            }
        }
        let pending = Pending(self.pending.clone());
        if let Some(b) = button.upgrade() {
            b.set_sensitive(false);
        }
        if let Some(n) = notice.upgrade() {
            n.set_text("Cargando imagen…");
        }
        let weak = Rc::downgrade(self);
        let mut cancel = self.cancel.subscribe();
        glib::spawn_future_local(async move {
            let _pending = pending;
            static SLOTS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
            let slots = SLOTS
                .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
                .clone();
            let _slot = tokio::select! {_ = cancel.changed()=>return,slot=slots.acquire_owned()=>match slot{Ok(s)=>s,Err(_)=>return}};
            if picture.upgrade().is_none() {
                return;
            }
            let bytes = match source {
                Input::Standard(Location::Local {
                    path,
                    base,
                    explicit,
                }) => nexuterm::runtime::runtime()
                    .spawn_blocking(move || image::read_local(&path, base.as_deref(), explicit))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string())),
                Input::Standard(Location::Http(url)) => nexuterm::runtime::runtime()
                    .spawn(async move { tokio::select!{_ = cancel.changed()=>Err("Carga cancelada al cerrar el documento".into()),result=image::download(&url)=>result} })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string())),
                Input::Ssh(doc,path)=>nexuterm::runtime::runtime().spawn(async move{tokio::select!{_ = cancel.changed()=>Err("Carga cancelada".into()),result=doc.read_image(&path)=>result}}).await.unwrap_or_else(|e|Err(e.to_string())),
            };
            let decoded = match bytes {
                Ok(bytes) => nexuterm::runtime::runtime()
                    .spawn_blocking(move || decode(bytes))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string())),
                Err(e) => Err(e),
            };
            if let Some(b) = button.upgrade() {
                b.set_sensitive(true);
            }
            let (Some(c), Some(p), Some(n)) = (weak.upgrade(), picture.upgrade(), notice.upgrade())
            else {
                return;
            };
            match decoded {
                Ok((bytes, width, height, stride)) => {
                    let bytes = glib::Bytes::from_owned(bytes);
                    let texture = gtk::gdk::MemoryTexture::new(
                        width,
                        height,
                        gtk::gdk::MemoryFormat::R8g8b8a8,
                        &bytes,
                        stride,
                    )
                    .upcast::<gtk::gdk::Texture>();
                    p.set_paintable(Some(&texture));
                    let mut cache = c.cache.borrow_mut();
                    if cache.len() >= 8 && !cache.contains_key(&url) {
                        cache.clear();
                    }
                    cache.insert(url, texture);
                    n.set_text("Vista previa · la imagen original no se modifica");
                }
                Err(e) => n.set_text(&e),
            }
        });
    }
}
impl Drop for Media {
    fn drop(&mut self) {
        self.cancel.send_replace(true);
        self.clear();
    }
}
fn decode(bytes: Vec<u8>) -> Result<(Vec<u8>, i32, i32, usize), String> {
    use std::io::Write;
    let raster = bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(&[0xff, 0xd8, 0xff])
        || (bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"));
    if !raster {
        return Err(
            "Vista previa disponible para PNG, JPEG y WebP; no se ejecutan SVG ni HTML".into(),
        );
    }
    let mut file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    let (_, width, height) =
        gtk::gdk_pixbuf::Pixbuf::file_info(file.path()).ok_or("Imagen no reconocida")?;
    if width <= 0 || height <= 0 || width as i64 * height as i64 > 4_000_000 {
        return Err("Imagen superior a cuatro millones de píxeles".into());
    }
    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(file.path(), 640, 224, true)
        .map_err(|e| e.to_string())?
        .add_alpha(false, 0, 0, 0)
        .map_err(|e| e.to_string())?;
    Ok((
        pixbuf.read_pixel_bytes().as_ref().to_vec(),
        pixbuf.width(),
        pixbuf.height(),
        pixbuf.rowstride() as usize,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn named(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
        if root.widget_name() == name {
            return Some(root.clone());
        }
        let mut child = root.first_child();
        while let Some(w) = child {
            if let Some(found) = named(&w, name) {
                return Some(found);
            }
            child = w.next_sibling();
        }
        None
    }
    async fn until(mut f: impl FnMut() -> bool) {
        for _ in 0..300 {
            if f() {
                return;
            }
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        panic!("Media condition timed out");
    }
    #[test]
    #[ignore = "requires Xvfb and a private D-Bus session"]
    fn native_media_preserves_source_cells_images_and_undo() {
        gtk::init().unwrap();
        if let Some(settings) = gtk::Settings::default() {
            settings.set_property(
                "gtk-application-prefer-dark-theme",
                std::env::var("NEXUTERM_MARKDOWN_THEME").as_deref() == Ok("dark"),
            );
        }
        let dir = tempfile::tempdir().unwrap();
        let image_path = dir.path().join("imagen.png");
        let pixels = glib::Bytes::from_owned([30, 140, 210, 255].repeat(96 * 64));
        gtk::gdk::MemoryTexture::new(96, 64, gtk::gdk::MemoryFormat::R8g8b8a8, &pixels, 96 * 4)
            .save_to_png(&image_path)
            .unwrap();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
        root.append(&gtk::Label::new(Some(
            "Informe Markdown · tablas e imágenes",
        )));
        let view = gtk::TextView::new();
        view.set_left_margin(12);
        view.set_right_margin(12);
        let scroll = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .child(&view)
            .build();
        root.append(&scroll);
        let gutter = gtk::DrawingArea::new();
        crate::markdown_editor::bind(&root, &view, &gutter, Some(dir.path().to_owned()), None);
        let source = "# Informe de mantenimiento\n\nResumen 👩‍💻\n\n| Servicio | Estado |\n| :--- | :---: |\n| SSH | Activo |\n| Copias | Revisar |\n\n![Imagen de prueba](imagen.png)\n\nDespués de la imagen.\n";
        let mut document = source.to_owned();
        let server = std::env::var("NEXUTERM_MEDIA_HTTP").ok();
        if let Some(server) = &server {
            if let Some(path) = std::env::var_os("NEXUTERM_MEDIA_HTTP_PNG") {
                std::fs::copy(&image_path, path).unwrap();
            }
            document.push_str(&format!(
                "\n![HTTP]({server}/image.png)\n\n![Lenta]({server}/slow)\n"
            ));
        }
        let source = document.as_str();
        let buffer = view.buffer();
        buffer.set_text(source);
        buffer.set_modified(false);
        let window = gtk::Window::builder()
            .default_width(1000)
            .default_height(860)
            .child(&root)
            .build();
        window.present();
        glib::MainContext::default().block_on(async {
            until(|| named(root.upcast_ref(), "md-cell-0-1-1").is_some()).await;
            eprintln!("MEDIA tables ready");
            until(|| {
                named(root.upcast_ref(), "md-image-1")
                    .and_then(|w| w.downcast::<gtk::Picture>().ok())
                    .is_some_and(|p| p.paintable().is_some())
            })
            .await;
            if let Some(server) = &server {
                let picture = named(root.upcast_ref(), "md-image-2")
                    .unwrap()
                    .downcast::<gtk::Picture>()
                    .unwrap();
                assert!(
                    picture.paintable().is_none(),
                    "HTTP requires explicit loading"
                );
                named(root.upcast_ref(), "md-image-load-2")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .emit_clicked();
                until(|| picture.paintable().is_some()).await;
                for path in ["missing", "large", "stream", "redirect"] {
                    let url = format!("{server}/{path}");
                    assert!(
                        nexuterm::runtime::runtime()
                            .spawn(async move { image::download(&url).await })
                            .await
                            .unwrap()
                            .is_err(),
                        "{path} should fail"
                    );
                }
            }
            glib::timeout_future(std::time::Duration::from_millis(400)).await;
            assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
            assert!(!buffer.is_modified());
            let cell = named(root.upcast_ref(), "md-cell-0-1-1")
                .unwrap()
                .downcast::<gtk::Entry>()
                .unwrap();
            cell.grab_focus();
            cell.set_text("Disponible | estable");
            assert!(
                buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), true)
                    .contains(r"Disponible \| estable")
            );
            until(|| {
                crate::recovery_ui::snapshot(&buffer)
                    .unwrap()
                    .contains("Disponible \\| estable")
            })
            .await;
            assert!(
                crate::recovery_ui::snapshot(&buffer)
                    .unwrap()
                    .contains("Después de la imagen.\n")
            );
            eprintln!("MEDIA cell changed");
            view.grab_focus();
            glib::timeout_future(std::time::Duration::from_millis(500)).await;
            let current = crate::recovery_ui::snapshot(&buffer).unwrap();
            cell.set_text("STALE MUST NOT WRITE");
            assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), current);
            buffer.undo();
            until(|| crate::recovery_ui::snapshot(&buffer).unwrap() == source).await;
            // Structural edits must preserve a cell edited before its idle callback.
            until(|| {
                named(root.upcast_ref(), "md-cell-0-1-1")
                    .is_some_and(|w| w.downcast::<gtk::Entry>().unwrap().text() == "Activo")
            })
            .await;
            let fresh = named(root.upcast_ref(), "md-cell-0-1-1")
                .unwrap()
                .downcast::<gtk::Entry>()
                .unwrap();
            fresh.grab_focus();
            fresh.set_text("Conservado");
            fresh.select_region(0, -1);
            named(root.upcast_ref(), "md-format-0")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            assert_eq!(fresh.text(), "**Conservado**");
            named(root.upcast_ref(), "md-table-0-action-0")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            until(|| named(root.upcast_ref(), "md-cell-0-3-1").is_some()).await;
            let changed = crate::recovery_ui::snapshot(&buffer).unwrap();
            assert!(changed.contains("**Conservado**"));
            assert!(changed.contains(":---:"));
            let selected = named(root.upcast_ref(), "md-cell-0-1-1")
                .unwrap()
                .downcast::<gtk::Entry>()
                .unwrap();
            selected.grab_focus();
            selected.select_region(0, -1);
            named(root.upcast_ref(), "md-copy")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            assert_eq!(
                selected
                    .clipboard()
                    .read_text_future()
                    .await
                    .unwrap()
                    .unwrap(),
                "**Conservado**"
            );
            named(root.upcast_ref(), "md-table-0-action-2")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            until(|| named(root.upcast_ref(), "md-cell-0-3-2").is_some()).await;
            assert!(
                crate::recovery_ui::snapshot(&buffer)
                    .unwrap()
                    .contains("**Conservado**")
            );
            named(root.upcast_ref(), "md-table-0-action-3")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            until(|| named(root.upcast_ref(), "md-cell-0-3-2").is_none()).await;
            for _ in 0..4 {
                buffer.undo();
            }
            until(|| crate::recovery_ui::snapshot(&buffer).unwrap() == source).await;
            let mode = named(root.upcast_ref(), "markdown-visual")
                .unwrap()
                .downcast::<gtk::ToggleButton>()
                .unwrap();
            eprintln!("MEDIA undo ready");
            mode.set_active(false);
            until(|| named(root.upcast_ref(), "md-cell-0-1-1").is_none()).await;
            assert_eq!(
                buffer.text(&buffer.start_iter(), &buffer.end_iter(), false),
                source
            );
            mode.set_active(true);
            until(|| named(root.upcast_ref(), "md-cell-0-1-1").is_some()).await;
            glib::timeout_future(std::time::Duration::from_millis(500)).await;
            view.scroll_to_iter(&mut buffer.start_iter(), 0.0, true, 0.0, 0.0);
            glib::timeout_future(std::time::Duration::from_millis(500)).await;
            let table = named(root.upcast_ref(), "md-media-0").unwrap();
            let picture = named(root.upcast_ref(), "md-media-1").unwrap();
            let tb = table.compute_bounds(&view).unwrap();
            let ib = picture.compute_bounds(&view).unwrap();
            eprintln!(
                "MEDIA geometry table={tb:?} image={ib:?} scroll={}",
                view.vadjustment().unwrap().value()
            );
            assert!(tb.y() > 20.0, "Table must not conceal the heading");
            assert!(ib.y() >= tb.y() + tb.height(), "Media blocks overlap");
            if let Some(path) = std::env::var_os("NEXUTERM_MEDIA_SCREENSHOT") {
                let p = gtk::WidgetPaintable::new(Some(&window));
                let shot = gtk::Snapshot::new();
                p.snapshot(&shot, window.width() as f64, window.height() as f64);
                window
                    .renderer()
                    .unwrap()
                    .render_texture(&shot.to_node().unwrap(), None)
                    .save_to_png(&path)
                    .unwrap();
                window.set_default_size(520, 860);
                glib::timeout_future(std::time::Duration::from_millis(350)).await;
                assert!(window.width() <= 550, "Media must not force a wide window");
                assert!(
                    table.width() <= view.width(),
                    "Table exceeds the document viewport"
                );
                let p = gtk::WidgetPaintable::new(Some(&window));
                let shot = gtk::Snapshot::new();
                p.snapshot(&shot, window.width() as f64, window.height() as f64);
                let narrow = std::path::PathBuf::from(path).with_extension("narrow.png");
                window
                    .renderer()
                    .unwrap()
                    .render_texture(&shot.to_node().unwrap(), None)
                    .save_to_png(narrow)
                    .unwrap();
            }
        });
        if server.is_some() {
            glib::MainContext::default().block_on(async {
                named(root.upcast_ref(), "md-image-load-3")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .emit_clicked();
                glib::timeout_future(std::time::Duration::from_millis(300)).await;
                window.destroy();
                glib::timeout_future(std::time::Duration::from_millis(700)).await;
            });
        } else {
            window.destroy();
        }
    }
}
