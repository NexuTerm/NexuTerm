//! Native visual/source views on the existing editor buffer and recovery journal.
use gtk::{glib, prelude::*};
use nexuterm::markdown::{self, Style};
use std::{cell::Cell, rc::Rc, time::Duration};

// Table entries edit their own source range; inline commands must follow their focus.
fn focused_cell(view: &gtk::TextView) -> Option<gtk::Entry> {
    let mut focus = view
        .root()?
        .downcast::<gtk::Window>()
        .ok()
        .and_then(|w| gtk::prelude::GtkWindowExt::focus(&w));
    while let Some(widget) = focus {
        if widget.widget_name().starts_with("md-cell-") {
            return widget.downcast::<gtk::Entry>().ok();
        }
        focus = widget.parent();
    }
    None
}
fn wrap(view: &gtk::TextView, before: &str, after: &str) {
    if let Some(entry) = focused_cell(view) {
        let (a, z) = entry
            .selection_bounds()
            .unwrap_or((entry.position(), entry.position()));
        if before.contains('\n') || after.contains('\n') {
            return;
        }
        let text = entry.text().to_string();
        let chars: Vec<_> = text.chars().collect();
        let (a, z) = (a.min(z) as usize, a.max(z) as usize);
        let selected: String = chars[a..z].iter().collect();
        let replacement = if selected.starts_with(before)
            && selected.ends_with(after)
            && selected.len() >= before.len() + after.len()
        {
            selected[before.len()..selected.len() - after.len()].to_owned()
        } else {
            format!("{before}{selected}{after}")
        };
        let value = format!(
            "{}{}{}",
            chars[..a].iter().collect::<String>(),
            replacement,
            chars[z..].iter().collect::<String>()
        );
        entry.set_text(&value);
        entry.select_region(a as i32, (a + replacement.chars().count()) as i32);
        return;
    }
    if !view.is_editable() {
        return;
    }
    let buffer = view.buffer();
    let (mut a, mut z) = buffer.selection_bounds().unwrap_or_else(|| {
        let p = buffer.iter_at_offset(buffer.cursor_position());
        (p, p)
    });
    let mut start = a.offset();
    let value = buffer.text(&a, &z, true).to_string();
    let left_count = before.chars().count() as i32;
    let right_count = after.chars().count() as i32;
    let mut surrounded = start >= left_count
        && z.offset() + right_count <= buffer.char_count()
        && buffer.text(&buffer.iter_at_offset(start - left_count), &a, true) == before
        && buffer.text(&z, &buffer.iter_at_offset(z.offset() + right_count), true) == after;
    // A two-star delimiter belongs to bold, not italic.
    if surrounded && before == "*" {
        let prefix = buffer.text(&buffer.iter_at_offset((start - 16).max(0)), &a, true);
        let suffix = buffer.text(
            &z,
            &buffer.iter_at_offset((z.offset() + 16).min(buffer.char_count())),
            true,
        );
        if prefix.chars().rev().take_while(|c| *c == '*').count() == 2
            || suffix.chars().take_while(|c| *c == '*').count() == 2
        {
            surrounded = false;
        }
    }
    let embedded = value.starts_with(before)
        && value.ends_with(after)
        && value.len() >= before.len() + after.len();
    let contents = if surrounded {
        start -= left_count;
        a = buffer.iter_at_offset(start);
        z = buffer.iter_at_offset(z.offset() + right_count);
        value.clone()
    } else if embedded {
        value[before.len()..value.len() - after.len()].to_owned()
    } else {
        format!("{before}{value}{after}")
    };
    buffer.begin_user_action();
    buffer.delete(&mut a, &mut z);
    buffer.insert(&mut a, &contents);
    buffer.end_user_action();
    let unwrapped = surrounded || embedded;
    let left = start + if unwrapped { 0 } else { left_count };
    let right = start + contents.chars().count() as i32 - if unwrapped { 0 } else { right_count };
    view.grab_focus();
    buffer.select_range(&buffer.iter_at_offset(left), &buffer.iter_at_offset(right));
    // Button activation finishes after its callback. Restore the text selection
    // once that activation has released focus, without overriding later edits.
    let weak = view.downgrade();
    let length = buffer.char_count();
    glib::idle_add_local_once(move || {
        if let Some(view) = weak.upgrade() {
            let b = view.buffer();
            if b.char_count() == length
                && b.text(
                    &b.iter_at_offset(start),
                    &b.iter_at_offset(start + contents.chars().count() as i32),
                    true,
                ) == contents
            {
                view.grab_focus();
                b.select_range(&b.iter_at_offset(left), &b.iter_at_offset(right));
            }
        }
    });
}
fn prefix(view: &gtk::TextView, prefix: &str) {
    if focused_cell(view).is_some() {
        return;
    }
    if !view.is_editable() {
        return;
    }
    let buffer = view.buffer();
    let selected = buffer.selection_bounds();
    let single = selected.is_none();
    let (a, z) = selected.unwrap_or_else(|| {
        let p = buffer.iter_at_offset(buffer.cursor_position());
        (p, p)
    });
    let mut start = a;
    start.set_line_offset(0);
    let mut end = z;
    if single || end.line_offset() > 0 {
        end.forward_to_line_end();
    }
    let original = buffer.text(&start, &end, true).to_string();
    let remove = !original.is_empty()
        && !prefix.starts_with('#')
        && prefix != "> "
        && !prefix.is_empty()
        && original.lines().all(|l| l.starts_with(prefix));
    let replacement = original
        .strip_suffix('\n')
        .unwrap_or(&original)
        .split('\n')
        .map(|line| {
            if remove {
                line.strip_prefix(prefix).unwrap_or(line).to_owned()
            } else {
                let line = if prefix.starts_with('#') || prefix.is_empty() || prefix == "> " {
                    let n = line.bytes().take_while(|b| *b == b'#').count();
                    if (1..=6).contains(&n) && line.as_bytes().get(n) == Some(&b' ') {
                        &line[n + 1..]
                    } else {
                        line.strip_prefix("> ").unwrap_or(line)
                    }
                } else {
                    line
                };
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let replacement = if original.ends_with('\n') {
        format!("{replacement}\n")
    } else {
        replacement
    };
    if replacement == original {
        view.grab_focus();
        return;
    }
    buffer.begin_user_action();
    buffer.delete(&mut start, &mut end);
    buffer.insert(&mut start, &replacement);
    buffer.end_user_action();
    view.grab_focus();
}
fn reference(view: &gtk::TextView, image: bool, base: Option<&std::path::Path>) {
    if focused_cell(view).is_some() {
        return;
    }
    if !view.is_editable() {
        return;
    }
    let Some(parent) = view.root().and_then(|w| w.downcast::<gtk::Window>().ok()) else {
        return;
    };
    let dialog = gtk::Window::builder()
        .title(if image {
            "Insertar imagen"
        } else {
            "Insertar enlace"
        })
        .transient_for(&parent)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(460)
        .build();
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_top(12);
    body.set_margin_bottom(12);
    body.set_margin_start(12);
    body.set_margin_end(12);
    let caption = gtk::Entry::builder()
        .placeholder_text(if image {
            "Descripción de la imagen"
        } else {
            "Texto del enlace"
        })
        .build();
    let buffer = view.buffer();
    if let Some((a, z)) = buffer.selection_bounds() {
        caption.set_text(&buffer.text(&a, &z, false));
    }
    let target = gtk::Entry::builder()
        .placeholder_text("Ruta relativa o URL https://…")
        .build();
    let notice = gtk::Label::new(Some(
        "Se insertará una referencia Markdown. No se descargará ni abrirá el recurso.",
    ));
    notice.set_wrap(true);
    let accept = gtk::Button::with_label("Insertar");
    body.append(&caption);
    body.append(&target);
    if image {
        notice.set_text(
            "Se insertará una referencia; la imagen no se copiará ni subirá al servidor.",
        );
        let browse = gtk::Button::with_label("Explorar…");
        browse.set_widget_name("md-image-browse");
        body.append(&browse);
        let parent = dialog.downgrade();
        let field = target.downgrade();
        let base = base.map(std::path::Path::to_owned);
        let cancel = gtk::gio::Cancellable::new();
        let closed = cancel.clone();
        dialog.connect_unrealize(move |_| closed.cancel());
        browse.connect_clicked(move |button| {
            let Some(parent) = parent.upgrade() else {
                return;
            };
            let chooser = gtk::FileDialog::builder()
                .title("Seleccionar imagen")
                .build();
            let images = gtk::FileFilter::new();
            images.set_name(Some("Imágenes PNG, JPEG y WebP"));
            for mime in ["image/png", "image/jpeg", "image/webp"] {
                images.add_mime_type(mime);
            }
            let all = gtk::FileFilter::new();
            all.set_name(Some("Todos los archivos"));
            all.add_pattern("*");
            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&images);
            filters.append(&all);
            chooser.set_filters(Some(&filters));
            chooser.set_default_filter(Some(&images));
            if let Some(base) = &base {
                chooser.set_initial_folder(Some(&gtk::gio::File::for_path(base)));
            }
            let field = field.clone();
            let base = base.clone();
            let weak = button.downgrade();
            button.set_sensitive(false);
            chooser.open(Some(&parent), Some(&cancel), move |result| {
                if let Some(button) = weak.upgrade() {
                    button.set_sensitive(true);
                }
                if let Ok(file) = result
                    && let Some(field) = field.upgrade()
                {
                    // A file URI also works for a remote document without mistaking it for an SSH path.
                    let uri = file.uri().to_string();
                    let value = base
                        .as_ref()
                        .and_then(|base| {
                            let path = file.path()?;
                            if !path.starts_with(base) {
                                return None;
                            }
                            url::Url::from_directory_path(base)
                                .ok()?
                                .make_relative(&url::Url::parse(&uri).ok()?)
                        })
                        .unwrap_or(uri);
                    field.set_text(&value);
                }
            });
        });
    }
    body.append(&notice);
    body.append(&accept);
    dialog.set_child(Some(&body));
    let weak = view.downgrade();
    let window = dialog.downgrade();
    accept.connect_clicked(move |_| {
        let (Some(view), Some(window)) = (weak.upgrade(), window.upgrade()) else {
            return;
        };
        let path = target.text();
        let path = path.trim();
        if path.is_empty() || path.len() > 4096 || path.chars().any(char::is_control) {
            notice.set_text("Indica una ruta o URL válida de hasta 4096 bytes.");
            return;
        }
        if let Ok(url) = url::Url::parse(path) {
            if !matches!(url.scheme(), "https" | "http" | "file") {
                notice.set_text("Solo se admiten rutas y enlaces HTTP, HTTPS o file.");
                return;
            }
        }
        let label = caption
            .text()
            .replace('\\', "\\\\")
            .replace('[', "\\[")
            .replace(']', "\\]");
        let path = path
            .replace(' ', "%20")
            .replace('(', "%28")
            .replace(')', "%29");
        let content = format!("{}[{label}]({path})", if image { "!" } else { "" });
        let buffer = view.buffer();
        let (mut a, mut z) = buffer.selection_bounds().unwrap_or_else(|| {
            let p = buffer.iter_at_offset(buffer.cursor_position());
            (p, p)
        });
        buffer.begin_user_action();
        buffer.delete(&mut a, &mut z);
        buffer.insert(&mut a, &content);
        buffer.end_user_action();
        window.close();
        view.grab_focus();
    });
    dialog.present();
}
fn ribbon_group(row: &gtk::Box, title: &str) -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Vertical, 2);
    group.add_css_class("md-ribbon-group");
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    content.set_vexpand(true);
    group.append(&content);
    let caption = gtk::Label::new(Some(title));
    caption.add_css_class("md-group-caption");
    group.append(&caption);
    row.append(&group);
    content
}
fn ribbon_button(label: &str, tip: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.set_tooltip_text(Some(tip));
    button.set_focus_on_click(false);
    button.add_css_class("flat");
    button
}
fn ribbon_css() {
    thread_local! {static INSTALLED:Cell<bool>=const{Cell::new(false)};}
    INSTALLED.with(|installed| {
        if !installed.replace(true) {
            let provider = gtk::CssProvider::new();
            provider.load_from_data(include_str!("markdown_ribbon.css"));
            if let Some(display) = gtk::gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
                );
            }
        }
    });
}
pub fn bind(
    root: &gtk::Box,
    view: &gtk::TextView,
    gutter: &gtk::DrawingArea,
    document_base: Option<std::path::PathBuf>,
    remote: Option<
        Rc<std::cell::RefCell<Option<std::sync::Arc<nexuterm::remote_edit::RemoteDocument>>>>,
    >,
) {
    view.set_widget_name("markdown-editor");
    view.buffer().set_enable_undo(true);
    view.buffer().set_max_undo_levels(200);
    let ribbon = gtk::Box::new(gtk::Orientation::Vertical, 4);
    ribbon.set_widget_name("markdown-ribbon");
    ribbon.add_css_class("markdown-ribbon");
    ribbon.set_vexpand(false);
    ribbon_css();
    let tools = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let insert = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let display = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let panels = gtk::Stack::new();
    panels.set_vexpand(false);
    for (name, label, row) in [
        ("home", "Inicio", &tools),
        ("insert", "Insertar", &insert),
        ("view", "Vista", &display),
    ] {
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .child(row)
            .build();
        panels.add_titled(&scroll, Some(name), label);
    }
    let tabs = gtk::StackSwitcher::builder().stack(&panels).build();
    let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    navigation.append(&tabs);
    let mode = gtk::ToggleButton::with_label("Visual");
    mode.set_active(true);
    mode.set_widget_name("markdown-visual");
    navigation.append(&mode);
    ribbon.append(&navigation);
    ribbon.append(&panels);
    mode.set_tooltip_text(Some("Alternar edición visual y código Markdown. Vista visual hasta 256 KiB; tablas editables y vistas previas de imágenes; Código conserva la fuente completa."));
    root.insert_child_after(&ribbon, Some(&root.first_child().unwrap()));
    // Clipboard: large Paste with two compact actions alongside it.
    let clipboard = ribbon_group(&tools, "Portapapeles");
    let paste = ribbon_button("", "Pegar (Ctrl+V)");
    let paste_body = gtk::Box::new(gtk::Orientation::Vertical, 3);
    let icon = gtk::Image::from_icon_name("edit-paste-symbolic");
    icon.set_pixel_size(24);
    paste_body.append(&icon);
    paste_body.append(&gtk::Label::new(Some("Pegar")));
    paste.set_widget_name("md-paste");
    paste.set_child(Some(&paste_body));
    clipboard.append(&paste);
    let weak = view.downgrade();
    paste.connect_clicked(move |_| {
        if let Some(v) = weak.upgrade() {
            if let Some(entry) = focused_cell(&v) {
                if let Some(text) = entry
                    .delegate()
                    .and_then(|d| d.downcast::<gtk::Text>().ok())
                {
                    let _ = text.activate_action("clipboard.paste", None);
                }
            } else {
                v.emit_paste_clipboard();
            }
        }
    });
    let clip_small = gtk::Box::new(gtk::Orientation::Vertical, 0);
    clipboard.append(&clip_small);
    for (label, cut) in [("✂  Cortar", true), ("▣  Copiar", false)] {
        let button = ribbon_button(label, label);
        button.set_widget_name(if cut { "md-cut" } else { "md-copy" });
        clip_small.append(&button);
        let weak = view.downgrade();
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                if let Some(entry) = focused_cell(&v) {
                    if let Some(text) = entry
                        .delegate()
                        .and_then(|d| d.downcast::<gtk::Text>().ok())
                    {
                        let _ = text.activate_action(
                            if cut {
                                "clipboard.cut"
                            } else {
                                "clipboard.copy"
                            },
                            None,
                        );
                    }
                } else if cut {
                    v.emit_cut_clipboard();
                } else {
                    v.emit_copy_clipboard();
                }
            }
        });
    }
    let font = ribbon_group(&tools, "Fuente");
    let font_grid = gtk::Grid::builder()
        .column_spacing(2)
        .row_spacing(2)
        .build();
    font.append(&font_grid);
    for (i, (label, tip, before, after)) in [
        ("B", "Negrita (Ctrl+B)", "**", "**"),
        ("I", "Cursiva (Ctrl+I)", "*", "*"),
        ("S̶", "Tachado", "~~", "~~"),
        ("‹/›", "Código en línea", "`", "`"),
    ]
    .into_iter()
    .enumerate()
    {
        let button = ribbon_button(label, tip);
        button.add_css_class("md-format");
        button.set_widget_name(&format!("md-format-{i}"));
        font_grid.attach(&button, (i % 2) as i32, (i / 2) as i32, 1, 1);
        if i == 0 {
            button.add_css_class("md-bold");
        }
        if i == 1 {
            button.add_css_class("md-italic");
        }
        let weak = view.downgrade();
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                wrap(&v, before, after);
            }
        });
    }
    let paragraph = ribbon_group(&tools, "Párrafo");
    let pg = gtk::Grid::builder()
        .column_spacing(2)
        .row_spacing(2)
        .build();
    paragraph.append(&pg);
    for (i, (label, tip, mark)) in [
        ("• ≡", "Viñetas", "- "),
        ("1. ≡", "Numeración", "1. "),
        ("☐", "Lista de tareas", "- [ ] "),
        ("❞", "Cita", "> "),
    ]
    .into_iter()
    .enumerate()
    {
        let button = ribbon_button(label, tip);
        pg.attach(&button, (i % 2) as i32, (i / 2) as i32, 1, 1);
        let weak = view.downgrade();
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                prefix(&v, mark);
            }
        });
    }
    let styles = ribbon_group(&tools, "Estilos");
    let gallery = gtk::Grid::builder()
        .column_spacing(3)
        .row_spacing(3)
        .build();
    styles.append(&gallery);
    let mut style_buttons = Vec::new();
    for (i, (label, mark)) in [
        ("Normal", ""),
        ("Título 1", "# "),
        ("Título 2", "## "),
        ("Título 3", "### "),
        ("Título 4", "#### "),
        ("Título 5", "##### "),
        ("Título 6", "###### "),
        ("Cita", "> "),
    ]
    .into_iter()
    .enumerate()
    {
        let button = ribbon_button(label, label);
        button.remove_css_class("flat");
        button.add_css_class("md-style");
        button.add_css_class(&format!("md-style-{i}"));
        button.set_widget_name(&format!("md-style-{i}"));
        gallery.attach(&button, (i % 4) as i32, (i / 4) as i32, 1, 1);
        style_buttons.push(button.downgrade());
        let weak = view.downgrade();
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                prefix(&v, mark);
            }
        });
    }
    let buffer = view.buffer().downgrade();
    let update: Rc<dyn Fn()> = Rc::new(move || {
        let Some(b) = buffer.upgrade() else {
            return;
        };
        let mut a = b.iter_at_offset(b.cursor_position());
        a.set_line_offset(0);
        let mut z = a;
        z.forward_to_line_end();
        let text = b.text(&a, &z, true);
        let count = text.bytes().take_while(|c| *c == b'#').count();
        let selected = if (1..=6).contains(&count) && text.as_bytes().get(count) == Some(&b' ') {
            count
        } else if text.starts_with("> ") {
            7
        } else {
            0
        };
        for (i, weak) in style_buttons.iter().enumerate() {
            if let Some(button) = weak.upgrade() {
                if i == selected {
                    button.add_css_class("selected");
                } else {
                    button.remove_css_class("selected");
                }
            }
        }
    });
    let u = update.clone();
    view.buffer().connect_mark_set(move |_, _, _| u());
    let u = update.clone();
    view.buffer().connect_changed(move |_| u());
    update();
    let editing = ribbon_group(&tools, "Edición");
    let editing_column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    editing.append(&editing_column);
    let undo_redo = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    editing_column.append(&undo_redo);
    for (label, undo) in [("↶", true), ("↷", false)] {
        let button = ribbon_button(
            label,
            if undo {
                "Deshacer (Ctrl+Z)"
            } else {
                "Rehacer (Ctrl+Mayús+Z)"
            },
        );
        undo_redo.append(&button);
        view.buffer()
            .bind_property(
                if undo { "can-undo" } else { "can-redo" },
                &button,
                "sensitive",
            )
            .sync_create()
            .build();
        let weak = view.downgrade();
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                let b = v.buffer();
                if undo && b.can_undo() {
                    b.undo();
                } else if !undo && b.can_redo() {
                    b.redo();
                }
                v.grab_focus();
            }
        });
    }
    let select = ribbon_button("Seleccionar todo", "Seleccionar todo (Ctrl+A)");
    editing_column.append(&select);
    let weak = view.downgrade();
    select.connect_clicked(move |_| {
        if let Some(v) = weak.upgrade() {
            if let Some(entry) = focused_cell(&v) {
                entry.select_region(0, -1);
            } else {
                v.grab_focus();
                v.emit_select_all(true);
            }
        }
    });
    let insert_group = ribbon_group(&insert, "Elementos del documento");
    for (label, before, after) in [
        ("Enlace", "[", "](https://ejemplo.invalid)"),
        ("Imagen", "![", "](imagen.png)"),
        (
            "Tabla",
            "\n| Columna 1 | Columna 2 |\n| --- | --- |\n| ",
            " | |\n",
        ),
        ("Bloque de código", "\n```\n", "\n```\n"),
    ] {
        let button = ribbon_button(label, label);
        insert_group.append(&button);
        let weak = view.downgrade();
        let base = document_base.clone();
        button.connect_clicked(move |_| {
            if let Some(v) = weak.upgrade() {
                if label == "Enlace" || label == "Imagen" {
                    reference(&v, label == "Imagen", base.as_deref());
                } else {
                    wrap(&v, before, after);
                }
            }
        });
    }
    let zoom = gtk::SpinButton::with_range(60.0, 200.0, 10.0);
    zoom.set_value(100.0);
    zoom.set_tooltip_text(Some("Zoom del documento (%)"));
    let zoom_group = ribbon_group(&display, "Zoom");
    zoom_group.append(&gtk::Label::new(Some("Porcentaje")));
    zoom_group.append(&zoom);
    let keys = gtk::EventControllerKey::new();
    let weak = view.downgrade();
    keys.connect_key_pressed(move |_, key, _, mods| {
        if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK)
            && !mods
                .intersects(gtk::gdk::ModifierType::ALT_MASK | gtk::gdk::ModifierType::SUPER_MASK)
        {
            if let Some(view) = weak.upgrade() {
                match key {
                    gtk::gdk::Key::b | gtk::gdk::Key::B => wrap(&view, "**", "**"),
                    gtk::gdk::Key::i | gtk::gdk::Key::I => wrap(&view, "*", "*"),
                    _ => return glib::Propagation::Proceed,
                };
                return glib::Propagation::Stop;
            }
        }
        glib::Propagation::Proceed
    });
    view.add_controller(keys);
    let table = view.buffer().tag_table();
    let base = gtk::TextTag::builder()
        .name("md-base")
        .family("sans-serif")
        .size_points(12.0)
        .pixels_above_lines(0)
        .pixels_below_lines(2)
        .build();
    table.add(&base);
    let hidden = gtk::TextTag::builder()
        .name("md-hidden")
        .invisible(true)
        .build();
    table.add(&hidden);
    let bold = gtk::TextTag::builder().weight(700).build();
    table.add(&bold);
    let italic = gtk::TextTag::builder()
        .style(gtk::pango::Style::Italic)
        .build();
    table.add(&italic);
    let strike = gtk::TextTag::builder().strikethrough(true).build();
    table.add(&strike);
    let code = gtk::TextTag::builder().family("monospace").build();
    table.add(&code);
    let link = gtk::TextTag::builder()
        .underline(gtk::pango::Underline::Single)
        .build();
    table.add(&link);
    let quote = gtk::TextTag::builder()
        .left_margin(24)
        .style(gtk::pango::Style::Italic)
        .build();
    table.add(&quote);
    let headings: Vec<_> = (0..6)
        .map(|n| {
            let tag = gtk::TextTag::builder()
                .weight(700)
                .scale(1.8 - n as f64 * 0.14)
                .build();
            table.add(&tag);
            tag
        })
        .collect();
    let base_zoom = base.clone();
    zoom.connect_value_changed(move |s| base_zoom.set_size_points(12.0 * s.value() / 100.0));
    let dirty = Rc::new(Cell::new(true));
    let d = dirty.clone();
    view.buffer().connect_changed(move |_| d.set(true));
    let d = dirty.clone();
    let weak = view.downgrade();
    let numbers = gutter.downgrade();
    let numbers_visible = gutter.is_visible();
    mode.connect_toggled(move |mode| {
        mode.set_label(if mode.is_active() {
            "Visual"
        } else {
            "Código"
        });
        if let Some(v) = weak.upgrade() {
            v.set_monospace(!mode.is_active());
            v.set_wrap_mode(if mode.is_active() {
                gtk::WrapMode::WordChar
            } else {
                gtk::WrapMode::None
            });
        }
        if let Some(g) = numbers.upgrade() {
            g.set_visible(!mode.is_active() && numbers_visible);
        }
        d.set(true);
    });
    view.set_monospace(false);
    view.set_wrap_mode(gtk::WrapMode::WordChar);
    gutter.set_visible(false);
    let weak = view.downgrade();
    let mode = mode.downgrade();
    let tags: Vec<_> = vec![
        base.clone(),
        hidden.clone(),
        bold.clone(),
        italic.clone(),
        strike.clone(),
        code.clone(),
        link.clone(),
        quote.clone(),
    ]
    .into_iter()
    .chain(headings.iter().cloned())
    .collect();
    let media = crate::markdown_media_ui::Media::new(view, document_base, remote);
    let suspended = Rc::downgrade(&media);
    view.connect_unrealize(move |_| {
        if let Some(m) = suspended.upgrade() {
            m.pause();
        }
    });
    let resumed = Rc::downgrade(&media);
    let render = dirty.clone();
    view.connect_realize(move |_| {
        if let Some(m) = resumed.upgrade() {
            m.resume();
            render.set(true);
        }
    });
    let position = Rc::downgrade(&media);
    view.add_tick_callback(move |_, _| {
        if let Some(m) = position.upgrade() {
            m.position();
            glib::ControlFlow::Continue
        } else {
            glib::ControlFlow::Break
        }
    });
    let running = Rc::new(Cell::new(false));
    glib::timeout_add_local(Duration::from_millis(180), move || {
        let (Some(view), Some(mode)) = (weak.upgrade(), mode.upgrade()) else {
            return glib::ControlFlow::Break;
        };
        if !view.is_realized() {
            return glib::ControlFlow::Continue;
        }
        if mode.is_active() && media.editing() {
            return glib::ControlFlow::Continue;
        }
        if running.get() || !dirty.replace(false) {
            return glib::ControlFlow::Continue;
        }
        let buffer = view.buffer();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        for tag in &tags {
            buffer.remove_tag(tag, &start, &end);
        }
        if !mode.is_active() {
            media.clear();
            return glib::ControlFlow::Continue;
        }
        if buffer.char_count() > 262144 {
            media.clear();
            return glib::ControlFlow::Continue;
        }
        let source = buffer.text(&start, &end, true).to_string();
        if source.len() > 262144 {
            media.clear();
            return glib::ControlFlow::Continue;
        }
        buffer.apply_tag(&base, &start, &end);
        let weak = view.downgrade();
        let dirty = dirty.clone();
        let running = running.clone();
        let mode = mode.downgrade();
        let (hidden, bold, italic, strike, code, link, quote, headings) = (
            hidden.clone(),
            bold.clone(),
            italic.clone(),
            strike.clone(),
            code.clone(),
            link.clone(),
            quote.clone(),
            headings.clone(),
        );
        let media = media.clone();
        running.set(true);
        glib::spawn_future_local(async move {
            let parsed = nexuterm::runtime::runtime()
                .spawn_blocking(move || {
                    let spans = markdown::spans(&source);
                    let map = markdown::offsets(&source);
                    let blocks = nexuterm::markdown_media::blocks(&source);
                    (spans, map, blocks)
                })
                .await;
            running.set(false);
            let (Some(view), Some(mode)) = (weak.upgrade(), mode.upgrade()) else {
                return;
            };
            if dirty.get() || !mode.is_active() || !view.is_realized() {
                return;
            }
            let Ok((spans, map, blocks)) = parsed else {
                return;
            };
            let buffer = view.buffer();
            for span in spans {
                let tag = match span.style {
                    Style::Heading(n) => &headings[(n as usize - 1).min(5)],
                    Style::Bold => &bold,
                    Style::Italic => &italic,
                    Style::Strike => &strike,
                    Style::Code => &code,
                    Style::Link => &link,
                    Style::Quote => &quote,
                    Style::Hidden => &hidden,
                };
                buffer.apply_tag(
                    tag,
                    &buffer.iter_at_offset(map[span.range.start]),
                    &buffer.iter_at_offset(map[span.range.end]),
                );
            }
            media.sync(blocks, &map);
        });
        glib::ControlFlow::Continue
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settle() {
        for _ in 0..40 {
            while glib::MainContext::default().pending() {
                glib::MainContext::default().iteration(false);
            }
            std::thread::sleep(Duration::from_millis(15));
        }
    }
    fn named(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
        if root.widget_name() == name {
            return Some(root.clone());
        }
        let mut c = root.first_child();
        while let Some(w) = c {
            if let Some(found) = named(&w, name) {
                return Some(found);
            }
            c = w.next_sibling();
        }
        None
    }
    #[test]
    #[ignore = "requires Xvfb and private D-Bus"]
    fn visual_source_undo_and_recovery_preserve_markdown() {
        gtk::init().unwrap();
        if let Some(settings) = gtk::Settings::default() {
            settings.set_property(
                "gtk-application-prefer-dark-theme",
                std::env::var("NEXUTERM_MARKDOWN_THEME").as_deref() == Ok("dark"),
            );
        }
        let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
        root.append(&gtk::Label::new(Some("Documento Markdown")));
        let view = gtk::TextView::new();
        let gutter = gtk::DrawingArea::new();
        root.append(&view);
        bind(&root, &view, &gutter, None, None);
        let buffer = view.buffer();
        prefix(&view, "# ");
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), "# ");
        buffer.undo();
        buffer.set_text("uno\ndos\n");
        buffer.select_range(&buffer.start_iter(), &buffer.end_iter());
        prefix(&view, "- ");
        assert_eq!(
            crate::recovery_ui::snapshot(&buffer).unwrap(),
            "- uno\n- dos\n"
        );
        let source = "# Documento de prueba\n\nTexto con **negrita**, *cursiva* y `código`.\n\n## Próximos pasos\n\n- [ ] Revisar el informe\n- [x] Guardar el original\n\n> Una cita importante\n\n```rust\nlet mensaje = \"¡Hola 👩‍💻!\";\n```\n";
        buffer.set_text(source);
        buffer.set_modified(false);
        let window = gtk::Window::builder()
            .default_width(1000)
            .default_height(650)
            .child(&root)
            .build();
        window.present();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        assert!(!buffer.is_modified());
        assert!(
            !buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .contains("**negrita**")
        );
        let mode = named(root.upcast_ref(), "markdown-visual")
            .unwrap()
            .downcast::<gtk::ToggleButton>()
            .unwrap();
        let ribbon = named(root.upcast_ref(), "markdown-ribbon").unwrap();
        assert!(
            ribbon.height() <= 155,
            "Ribbon too tall: {}",
            ribbon.height()
        );
        let h2 = named(root.upcast_ref(), "md-style-2")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        buffer.place_cursor(&buffer.iter_at_offset(0));
        h2.emit_clicked();
        settle();
        assert!(
            crate::recovery_ui::snapshot(&buffer)
                .unwrap()
                .starts_with("## Documento")
        );
        assert!(h2.has_css_class("selected"));
        let styled = crate::recovery_ui::snapshot(&buffer).unwrap();
        buffer.set_modified(false);
        h2.emit_clicked();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), styled);
        assert!(!buffer.is_modified());
        buffer.undo();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        let normal = named(root.upcast_ref(), "md-style-0")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        buffer.place_cursor(&buffer.iter_at_offset(0));
        normal.emit_clicked();
        settle();
        assert!(
            crate::recovery_ui::snapshot(&buffer)
                .unwrap()
                .starts_with("Documento")
        );
        buffer.undo();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        if let Some(path) = std::env::var_os("NEXUTERM_MARKDOWN_SCREENSHOT") {
            let paint = gtk::WidgetPaintable::new(Some(&window));
            let snapshot = gtk::Snapshot::new();
            paint.snapshot(&snapshot, window.width() as f64, window.height() as f64);
            let node = snapshot.to_node().unwrap();
            window
                .renderer()
                .unwrap()
                .render_texture(&node, None)
                .save_to_png(&path)
                .unwrap();
            let path = std::path::PathBuf::from(path);
            let target = path.with_file_name(format!(
                "{}-ribbon.png",
                path.file_stem().unwrap().to_string_lossy()
            ));
            window
                .renderer()
                .unwrap()
                .render_texture(&node, Some(&ribbon.compute_bounds(&window).unwrap()))
                .save_to_png(target)
                .unwrap();
        }
        // Establish the saved baseline after the separate gallery/undo exercises.
        buffer.set_modified(false);
        mode.set_active(false);
        settle();
        assert_eq!(
            buffer.text(&buffer.start_iter(), &buffer.end_iter(), false),
            source
        );
        assert!(!buffer.is_modified());
        mode.set_active(true);
        settle();
        let at = source[..source.find("Texto").unwrap()].chars().count() as i32;
        // A real text selection is made after the editor has received focus.
        view.grab_focus();
        settle();
        buffer.select_range(&buffer.iter_at_offset(at), &buffer.iter_at_offset(at + 5));
        named(root.upcast_ref(), "md-format-0")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle();
        assert!(
            crate::recovery_ui::snapshot(&buffer)
                .unwrap()
                .contains("**Texto**")
        );
        assert!(buffer.can_undo());
        wrap(&view, "**", "**");
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        buffer.undo();
        settle();
        assert!(
            crate::recovery_ui::snapshot(&buffer)
                .unwrap()
                .contains("**Texto**")
        );
        buffer.undo();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        buffer.select_range(&buffer.iter_at_offset(at), &buffer.iter_at_offset(at + 5));
        reference(&view, false, None);
        settle();
        let dialog = gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Window>().ok())
            .find(|w| w.title().as_deref() == Some("Insertar enlace"))
            .unwrap();
        let body = dialog.child().unwrap();
        let caption = body
            .first_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap();
        assert_eq!(caption.text(), "Texto");
        let target = caption
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap();
        target.set_text("javascript:alert(1)");
        let accept = body
            .last_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        accept.emit_clicked();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        target.set_text("https://example.invalid/a b");
        accept.emit_clicked();
        settle();
        assert!(
            crate::recovery_ui::snapshot(&buffer)
                .unwrap()
                .contains("[Texto](https://example.invalid/a%20b)")
        );
        buffer.undo();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        let images = tempfile::tempdir().unwrap();
        let image = images.path().join("foto con espacio.png");
        std::fs::write(&image, b"image picker fixture").unwrap();
        buffer.select_range(&buffer.iter_at_offset(at), &buffer.iter_at_offset(at + 5));
        reference(&view, true, Some(images.path()));
        settle();
        let form = gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Window>().ok())
            .find(|w| w.title().as_deref() == Some("Insertar imagen"))
            .unwrap();
        named(form.upcast_ref(), "md-image-browse")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle();
        let chooser = gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::FileChooserDialog>().ok())
            .find(|w| w.title().as_deref() == Some("Seleccionar imagen"))
            .expect("Native image picker must open");
        chooser.set_file(&gtk::gio::File::for_path(&image)).unwrap();
        settle();
        chooser.response(gtk::ResponseType::Accept);
        settle();
        let body = form.child().unwrap();
        let path = body
            .first_child()
            .unwrap()
            .next_sibling()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap();
        assert_eq!(path.text(), "foto%20con%20espacio.png");
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        body.last_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle();
        assert!(
            crate::recovery_ui::snapshot(&buffer)
                .unwrap()
                .contains("![Texto](foto%20con%20espacio.png)")
        );
        buffer.undo();
        settle();
        assert_eq!(crate::recovery_ui::snapshot(&buffer).unwrap(), source);
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), crate::recovery_ui::snapshot(&buffer).unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(file.path()).unwrap(), source);
        window.set_default_size(550, 650);
        settle();
        assert!(
            window.width() <= 570,
            "Window minimum width too large: {}",
            window.width()
        );
        assert!(
            ribbon.height() <= 180,
            "Narrow ribbon too tall: {}",
            ribbon.height()
        );
        window.destroy();
    }
}
