//! Small contextual menus reuse the same operations as the visible controls.
use gtk::{gdk, gio, glib, prelude::*};
use std::rc::Rc;
use vte4::prelude::*;

pub struct Item {
    label: String,
    enabled: bool,
    activate: Box<dyn Fn()>,
}
impl Item {
    pub fn new(label: &str, enabled: bool, activate: impl Fn() + 'static) -> Self {
        Self {
            label: label.into(),
            enabled,
            activate: Box::new(activate),
        }
    }
    fn button(label: &str, button: &gtk::Button) -> Self {
        let weak = button.downgrade();
        Self::new(label, button.is_sensitive(), move || {
            if let Some(button) = weak.upgrade().filter(|b| b.is_sensitive()) {
                button.emit_clicked();
            }
        })
    }
}
pub fn attach(
    widget: &impl IsA<gtk::Widget>,
    build: impl Fn(Option<(f64, f64)>) -> Vec<Item> + 'static,
) -> gtk::GestureClick {
    let widget = widget.as_ref();
    let popover = gtk::PopoverMenu::from_model(gio::MenuModel::NONE);
    popover.set_widget_name("nexuterm-context-menu");
    popover.set_has_arrow(false);
    let weak = popover.downgrade();
    widget.connect_destroy(move |_| {
        if let Some(popover) = weak.upgrade().filter(|p| p.parent().is_some()) {
            popover.unparent();
        }
    });
    let build = Rc::new(build);
    let show: Rc<dyn Fn(Option<(f64, f64)>)> = {
        let weak = widget.downgrade();
        Rc::new(move |point| {
            let Some(widget) = weak.upgrade().filter(|w| w.is_sensitive()) else {
                return;
            };
            if !widget.is::<gtk::Label>() {
                widget.grab_focus();
            }
            let items = build(point);
            if items.is_empty() {
                return;
            }
            let anchor = if widget.is::<gtk::Box>() {
                widget.clone()
            } else {
                let Some(anchor) = widget.ancestor(gtk::Box::static_type()) else {
                    return;
                };
                anchor
            };
            if popover.parent().as_ref() != Some(&anchor) {
                if popover.parent().is_some() {
                    popover.unparent();
                }
                popover.set_parent(&anchor);
            }
            let menu = gio::Menu::new();
            let group = gio::SimpleActionGroup::new();
            for (index, item) in items.into_iter().enumerate() {
                let name = format!("item{index}");
                let action = gio::SimpleAction::new(&name, None);
                action.set_enabled(item.enabled);
                let weak = popover.downgrade();
                action.connect_activate(move |_, _| {
                    // Release the input grab before opening dialogs or changing the view.
                    if let Some(popover) = weak.upgrade() {
                        popover.popdown();
                    }
                    (item.activate)();
                });
                group.add_action(&action);
                menu.append(Some(&item.label), Some(&format!("context.{name}")));
            }
            popover.insert_action_group("context", Some(&group));
            popover.set_menu_model(Some(&menu));
            let (x, y) = point.unwrap_or((12.0, 12.0));
            let point = widget
                .compute_point(&anchor, &gtk::graphene::Point::new(x as f32, y as f32))
                .unwrap_or_else(|| gtk::graphene::Point::new(12.0, 12.0));
            popover.set_pointing_to(Some(&gdk::Rectangle::new(
                point.x() as i32,
                point.y() as i32,
                1,
                1,
            )));
            popover.popup();
        })
    };
    let click = gtk::GestureClick::new();
    click.set_button(3);
    click.set_name(Some("nexuterm-context-menu"));
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let open = show.clone();
    click.connect_pressed(move |gesture, _, x, y| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        open(Some((x, y)));
    });
    widget.add_controller(click.clone());
    let keys = gtk::EventControllerKey::new();
    keys.set_name(Some("nexuterm-context-keys"));
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if key == gdk::Key::Menu
            || (key == gdk::Key::F10 && modifiers == gdk::ModifierType::SHIFT_MASK)
        {
            show(None);
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    widget.add_controller(keys);
    click
}
fn buttons(root: &gtk::Widget, result: &mut Vec<gtk::Button>) {
    if root.is::<gtk::Popover>() {
        return;
    }
    if let Some(button) = root.downcast_ref::<gtk::Button>() {
        result.push(button.clone());
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        buttons(&widget, result);
    }
}
pub fn browser(root: &gtk::Box, list: &gtk::ListBox, can_copy: impl Fn() -> bool + 'static) {
    let root = root.downgrade();
    let rows = list.downgrade();
    attach(list, move |point| {
        let (Some(root), Some(list)) = (root.upgrade(), rows.upgrade()) else {
            return vec![];
        };
        if let Some((_,y))=point {
            if let Some(row)=list.row_at_y(y as i32) {if !row.is_selected(){list.unselect_all();list.select_row(Some(&row));}}
        }
        let mut controls = vec![];
        buttons(root.upcast_ref(), &mut controls);
        let upload_selection = root.has_css_class("local-explorer")
            && can_copy()
            && controls.iter().any(|b| {
                b.is_visible()
                    && b.tooltip_text()
                        .is_some_and(|text| text.starts_with("Subir"))
            });
        let weak = list.downgrade();
        let upload=if upload_selection {controls.iter().find(|b|b.tooltip_text().is_some_and(|t|t.starts_with("Subir"))).map(|b|b.downgrade())}else{None};
        let mut items = vec![Item::new(
            if upload_selection {
                "Subir selección"
            } else {
                "Abrir"
            },
            list.selected_rows().into_iter().next().is_some() && list.is_sensitive(),
            move || {
                if let Some(button)=upload.as_ref().and_then(|b|b.upgrade()){button.emit_clicked();return;}
                if let Some(list) = weak.upgrade().filter(|l| l.is_sensitive()) {
                    if let Some(row) = list.selected_rows().into_iter().next() {
                        list.emit_by_name::<()>("row-activated", &[&row]);
                    }
                }
            },
        )];
        for (prefix, label) in [
            ("Copiar archivo", "Copiar archivo"),
            ("Pegar archivo", "Pegar archivo"),
            ("Actualizar", "Actualizar"),
            ("Carpeta local superior", "Carpeta superior"),
            ("Carpeta superior", "Carpeta superior"),
            (
                "Mostrar u ocultar archivos ocultos",
                "Mostrar / ocultar archivos ocultos",
            ),
            ("Nueva carpeta", "Nueva carpeta"),
            ("Nuevo archivo", "Nuevo archivo"),
            ("Eliminar…", "Eliminar… (mover a la papelera)"),
            ("Subir", "Subir archivo"),
            ("Descargar", "Descargar archivo"),
        ] {
            if prefix == "Subir" && upload_selection {
                continue;
            }
            if let Some(button) = controls.iter().find(|b| {
                b.is_visible()
                    && b.tooltip_text()
                        .is_some_and(|text| text.starts_with(prefix))
            }) {
                let mut item = Item::button(label, button);
                if prefix == "Copiar archivo" {
                    item.enabled &= can_copy();
                }
                if prefix == "Pegar archivo" {
                    let formats = root.clipboard().formats();
                    item.enabled &= formats.contains_type(String::static_type())
                        || formats.contain_mime_type("text/plain");
                }
                if prefix == "Descargar"
                    || (prefix == "Subir" && root.has_css_class("local-explorer"))
                {
                    item.enabled &= can_copy();
                }
                items.push(item);
            }
        }
        items
    });
}
pub fn terminal(terminal: &vte4::Terminal) {
    let weak = terminal.downgrade();
    attach(terminal, move |_| {
        let Some(terminal) = weak.upgrade() else {
            return vec![];
        };
        let copy = terminal.downgrade();
        let paste = terminal.downgrade();
        let select = terminal.downgrade();
        let mut items = vec![
            Item::new("Copiar", terminal.has_selection(), move || {
                if let Some(t) = copy.upgrade() {
                    t.copy_clipboard_format(vte4::Format::Text);
                }
            }),
            Item::new(
                "Pegar",
                terminal
                    .clipboard()
                    .formats()
                    .contain_mime_type("text/plain")
                    || terminal
                        .clipboard()
                        .formats()
                        .contains_type(String::static_type()),
                move || {
                    if let Some(t) = paste.upgrade() {
                        t.paste_clipboard();
                    }
                },
            ),
            Item::new("Seleccionar todo", true, move || {
                if let Some(t) = select.upgrade() {
                    t.select_all();
                }
            }),
        ];
        for (label, action) in [
            ("Buscar…", "search"),
            ("Nueva terminal", "new-terminal"),
            ("Dividir en columnas", "split-horizontal"),
            ("Dividir en filas", "split-vertical"),
        ] {
            let target = terminal.downgrade();
            items.push(Item::new(label, true, move || {
                if let Some(t) = target.upgrade() {
                    let _ = t.activate_action(&format!("win.{action}"), None);
                }
            }));
        }
        items
    });
}
pub fn message(label: &gtk::Label) {
    let weak = label.downgrade();
    attach(label, move |_| {
        let Some(label) = weak.upgrade() else {
            return vec![];
        };
        let bounds = label.selection_bounds();
        let selection = label.downgrade();
        let whole = label.downgrade();
        let select = label.downgrade();
        vec![
            Item::new("Copiar selección", bounds.is_some(), move || {
                if let Some(label) = selection.upgrade() {
                    if let Some((a, b)) = label.selection_bounds() {
                        let (a, b) = (a.min(b) as usize, a.max(b) as usize);
                        label.clipboard().set_text(
                            &label.text().chars().skip(a).take(b - a).collect::<String>(),
                        );
                    }
                }
            }),
            Item::new(
                "Copiar mensaje completo",
                !label.text().is_empty(),
                move || {
                    if let Some(label) = whole.upgrade() {
                        label.clipboard().set_text(&label.text());
                    }
                },
            ),
            Item::new("Seleccionar todo", true, move || {
                if let Some(label) = select.upgrade() {
                    label.select_region(0, -1);
                }
            }),
        ]
    });
}
fn transcript(widget: &gtk::Widget, text: &mut String) {
    if widget.is::<gtk::Popover>() {
        return;
    }
    if let Some(expander) = widget.downcast_ref::<gtk::Expander>() {
        if let Some(child) = expander.child() { transcript(&child, text); }
        return;
    }
    if let Some(label) = widget.downcast_ref::<gtk::Label>() {
        text.push_str(&label.text());
        text.push('\n');
        return;
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        transcript(&widget, text);
    }
}
pub fn conversation(root: &gtk::Box) {
    let weak = root.downgrade();
    let gesture = attach(root, move |_| {
        let Some(root) = weak.upgrade() else {
            return vec![];
        };
        let copy = root.downgrade();
        let bottom = root.downgrade();
        vec![
            Item::new(
                "Copiar conversación",
                root.first_child().is_some(),
                move || {
                    if let Some(root) = copy.upgrade() {
                        let mut text = String::new();
                        transcript(root.upcast_ref(), &mut text);
                        root.clipboard().set_text(&text);
                    }
                },
            ),
            Item::new("Ir al final", true, move || {
                if let Some(root) = bottom.upgrade() {
                    if let Some(scroll) = root
                        .ancestor(gtk::ScrolledWindow::static_type())
                        .and_downcast::<gtk::ScrolledWindow>()
                    {
                        let adjustment = scroll.vadjustment();
                        adjustment.set_value(adjustment.upper() - adjustment.page_size());
                    }
                }
            }),
        ]
    });
    // Message labels handle their own menu first; the background handles the rest.
    gesture.set_propagation_phase(gtk::PropagationPhase::Bubble);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};
    fn settle() {
        let context = glib::MainContext::default();
        while context.pending() {
            context.iteration(false);
        }
    }
    fn open(widget: &impl IsA<gtk::Widget>, y: f64) -> gtk::PopoverMenu {
        let widget = widget.as_ref();
        let controllers = widget.observe_controllers();
        let click = (0..controllers.n_items())
            .filter_map(|i| controllers.item(i))
            .filter_map(|o| o.downcast::<gtk::GestureClick>().ok())
            .find(|g| g.name().as_deref() == Some("nexuterm-context-menu"))
            .unwrap();
        click.emit_by_name::<()>("pressed", &[&1i32, &5f64, &y]);
        settle();
        let anchor = if widget.is::<gtk::Box>() {
            widget.clone()
        } else {
            widget.ancestor(gtk::Box::static_type()).unwrap()
        };
        let mut child = anchor.first_child();
        while let Some(w) = child {
            child = w.next_sibling();
            if let Ok(popover) = w.downcast::<gtk::PopoverMenu>() {
                if popover.is_visible() {
                    return popover;
                }
            }
        }
        panic!("Context menu not found");
    }
    fn labels(popover: &gtk::PopoverMenu) -> Vec<String> {
        let menu = popover.menu_model().unwrap();
        (0..menu.n_items())
            .map(|i| {
                menu.item_attribute_value(i, "label", None)
                    .unwrap()
                    .get::<String>()
                    .unwrap()
            })
            .collect()
    }
    #[test]
    #[ignore = "requires Xvfb and a private D-Bus session"]
    fn menus_target_clicked_rows_copy_unicode_and_release_grabs() {
        gtk::init().unwrap();
        let window = gtk::Window::builder()
            .default_width(640)
            .default_height(600)
            .build();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let rows = gtk::ListBox::new();
        for text in ["uno.txt", "dos.txt"] {
            rows.append(&gtk::Label::new(Some(text)));
        }
        root.append(&rows);
        let refresh = gtk::Button::with_label("Refresh");
        refresh.set_tooltip_text(Some("Actualizar archivos locales"));
        root.append(&refresh);
        let copied = Rc::new(Cell::new(false));
        let copy = gtk::Button::with_label("Copy");
        copy.set_tooltip_text(Some("Copiar archivo"));
        root.append(&copy);
        let done = copied.clone();
        copy.connect_clicked(move |_| done.set(true));
        let upload = gtk::Button::with_label("Upload");
        upload.set_tooltip_text(Some("Subir archivo…"));
        root.append(&upload);
        let uploaded = Rc::new(Cell::new(false));
        let done = uploaded.clone();
        upload.connect_clicked(move |_| done.set(true));
        let weak = rows.downgrade();
        browser(&root, &rows, move || {
            weak.upgrade()
                .is_some_and(|list| list.selected_rows().into_iter().next().is_some())
        });
        let activated = Rc::new(Cell::new(-1));
        let value = activated.clone();
        rows.connect_row_activated(move |_, row| value.set(row.index()));
        let label = gtk::Label::new(Some("área de pruebas"));
        label.set_selectable(true);
        root.append(&label);
        message(&label);
        let terminal = vte4::Terminal::new();
        root.append(&terminal);
        super::terminal(&terminal);
        let conversation = gtk::Box::new(gtk::Orientation::Vertical, 0);
        conversation.append(&gtk::Label::new(Some("Mensaje completo")));
        root.append(&conversation);
        super::conversation(&conversation);
        window.set_child(Some(&root));
        window.present();
        settle();
        let second = rows.row_at_index(1).unwrap();
        let menu = open(&rows, f64::from(second.allocation().y() + 2));
        assert_eq!(rows.selected_rows().into_iter().next().unwrap().index(), 1);
        assert!(labels(&menu).contains(&"Actualizar".into()));
        menu.activate_action("context.item0", None).unwrap();
        settle();
        assert_eq!(activated.get(), 1);
        assert!(!menu.is_visible());
        let menu = open(&rows, f64::from(second.allocation().y() + 2));
        menu.activate_action("context.item1", None).unwrap();
        settle();
        assert!(copied.get());
        let menu = open(&rows, 10000.0);
        assert!(rows.selected_rows().into_iter().next().is_none());
        menu.activate_action("context.item0", None).unwrap();
        settle();
        assert_eq!(activated.get(), 1);
        assert!(menu.is_visible());
        let index = labels(&menu)
            .iter()
            .position(|label| label == "Subir archivo")
            .unwrap();
        menu.activate_action(&format!("context.item{index}"), None)
            .unwrap();
        assert!(
            uploaded.get(),
            "Upload into an empty remote folder must remain available"
        );
        menu.popdown();
        let controllers = rows.observe_controllers();
        let keys = (0..controllers.n_items())
            .filter_map(|i| controllers.item(i))
            .filter_map(|o| o.downcast::<gtk::EventControllerKey>().ok())
            .find(|k| k.name().as_deref() == Some("nexuterm-context-keys"))
            .unwrap();
        let handled = keys.emit_by_name::<bool>(
            "key-pressed",
            &[&gdk::Key::F10, &0u32, &gdk::ModifierType::SHIFT_MASK],
        );
        assert!(handled);
        settle();
        // The same popup survives reopening and refuses operations with no selection.
        let menu = open(&rows, 10000.0);
        menu.popdown();
        label.select_region(0, 4);
        settle();
        let menu = open(&label, 5.0);
        assert!(labels(&menu).contains(&"Copiar mensaje completo".into()));
        menu.activate_action("context.item0", None).unwrap();
        settle();
        let text = glib::MainContext::default()
            .block_on(label.clipboard().read_text_future())
            .unwrap()
            .unwrap();
        assert_eq!(text.as_str(), "área");
        let menu = open(&terminal, 5.0);
        assert!(labels(&menu).contains(&"Pegar".into()));
        menu.popdown();
        let menu = open(&conversation, 5.0);
        menu.activate_action("context.item0", None).unwrap();
        settle();
        let text = glib::MainContext::default()
            .block_on(label.clipboard().read_text_future())
            .unwrap()
            .unwrap();
        assert_eq!(text.as_str(), "Mensaje completo\n");
        let weak = rows.downgrade();
        refresh.connect_clicked(move |_| {
            if let Some(rows) = weak.upgrade() {
                while let Some(child) = rows.first_child() {
                    rows.remove(&child);
                }
            }
        });
        let menu = open(&rows, 2.0);
        let index = labels(&menu)
            .iter()
            .position(|label| label == "Actualizar")
            .unwrap();
        menu.activate_action(&format!("context.item{index}"), None)
            .unwrap();
        settle();
        assert!(rows.first_child().is_none());
        window.destroy();
        settle();
    }
}
