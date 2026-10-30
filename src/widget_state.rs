//! Small GTK view snapshots; no file access, connection or input replay.
use gtk::{glib, prelude::*};
use nexuterm::workspaces::{BrowserView, EditorView};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn find(widget: &gtk::Widget, kind: glib::Type) -> Option<gtk::Widget> {
    if widget.type_().is_a(kind) {
        return Some(widget.clone());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(found) = find(&current, kind) {
            return Some(found);
        }
        child = current.next_sibling();
    }
    None
}
fn scroll(widget: &gtk::Widget) -> Option<gtk::ScrolledWindow> {
    find(widget, gtk::ScrolledWindow::static_type())?
        .downcast()
        .ok()
}
fn position(a: &gtk::Adjustment) -> f64 {
    let range = (a.upper() - a.page_size() - a.lower()).max(0.0);
    if range > 0.0 {
        ((a.value() - a.lower()) / range).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
fn apply(a: &gtk::Adjustment, value: f64) {
    a.set_value(a.lower() + (a.upper() - a.page_size() - a.lower()).max(0.0) * value);
}
pub fn browser(
    widget: &gtk::Widget,
    hidden: bool,
    selected: Option<std::path::PathBuf>,
) -> BrowserView {
    let (vertical, horizontal) = scroll(widget)
        .map(|s| (position(&s.vadjustment()), position(&s.hadjustment())))
        .unwrap_or_default();
    BrowserView {
        hidden,
        selected,
        vertical,
        horizontal,
    }
}
pub fn restore_browser_scroll(
    widget: &gtk::Widget,
    pending: Rc<RefCell<Option<BrowserView>>>,
    epoch: Rc<Cell<u64>>,
) {
    if let Some(scroll) = scroll(widget) {
        let Some(view) = pending.borrow().clone() else {
            return;
        };
        let (vertical, horizontal) = (view.vertical, view.horizontal);
        let revision = epoch.get();
        let frames = Cell::new(0);
        scroll.add_tick_callback(move |s, _| {
            if epoch.get() != revision {
                return glib::ControlFlow::Break;
            }
            frames.set(frames.get() + 1);
            if frames.get() < 3 {
                return glib::ControlFlow::Continue;
            }
            apply(&s.vadjustment(), vertical);
            apply(&s.hadjustment(), horizontal);
            pending.borrow_mut().take();
            glib::ControlFlow::Break
        });
    }
}
pub fn editor(widget: &gtk::Widget) -> Option<EditorView> {
    let text = find(widget, gtk::TextView::static_type())?
        .downcast::<gtk::TextView>()
        .ok()?;
    let buffer = text.buffer();
    let (vertical, horizontal) = scroll(widget)
        .map(|s| (position(&s.vadjustment()), position(&s.hadjustment())))
        .unwrap_or_default();
    Some(EditorView {
        cursor: buffer.cursor_position().max(0) as u32,
        anchor: buffer
            .iter_at_mark(&buffer.selection_bound())
            .offset()
            .max(0) as u32,
        vertical,
        horizontal,
    })
}
pub fn restore_editor(widget: &gtk::Widget, pending: Rc<Cell<Option<EditorView>>>) {
    let Some(text) =
        find(widget, gtk::TextView::static_type()).and_then(|w| w.downcast::<gtk::TextView>().ok())
    else {
        return;
    };
    let Some(scroll) = scroll(widget) else {
        return;
    };
    let frames = Cell::new(0);
    let text = text.downgrade();
    scroll.add_tick_callback(move |s, _| {
        let (Some(view), Some(text)) = (pending.get(), text.upgrade()) else {
            return glib::ControlFlow::Break;
        };
        frames.set(frames.get() + 1);
        if frames.get() == 2 {
            let b = text.buffer();
            let count = b.char_count().max(0) as u32;
            b.select_range(
                &b.iter_at_offset(view.cursor.min(count) as i32),
                &b.iter_at_offset(view.anchor.min(count) as i32),
            );
        }
        if frames.get() < 4 {
            return glib::ControlFlow::Continue;
        }
        apply(&s.vadjustment(), view.vertical);
        apply(&s.hadjustment(), view.horizontal);
        pending.set(None);
        glib::ControlFlow::Break
    });
}
