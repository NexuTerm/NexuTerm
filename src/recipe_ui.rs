//! Recipe parameters share the existing command preview and approval workflow.
use gtk::prelude::*;
use nexuterm::recipes::Recipe;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
type ReadCommand = Rc<dyn Fn() -> Result<String, String>>;

pub(super) fn install(body: &gtk::Box, command: &gtk::TextView) -> ReadCommand {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let choice = gtk::DropDown::from_strings(&[
        "Comando libre",
        "Espacio en disco",
        "Estado de servicio",
        "Registros systemd",
    ]);
    choice.set_widget_name("recipe-choice");
    choice.set_tooltip_text(Some(
        "Recetas de solo consulta; revisa el comando antes de aprobarlo.",
    ));
    let parameter = gtk::Entry::builder().hexpand(true).build();
    parameter.set_widget_name("recipe-parameter");
    parameter.set_max_length(4096);
    row.append(&choice);
    row.append(&parameter);
    body.append(&row);
    let options = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    fn number(
        row: &gtk::Box,
        label: &str,
        name: &str,
        max: f64,
        initial: f64,
    ) -> (gtk::Box, gtk::SpinButton) {
        let group = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        group.append(&gtk::Label::new(Some(label)));
        let spin = gtk::SpinButton::with_range(1.0, max, 1.0);
        spin.set_widget_name(name);
        spin.set_value(initial);
        group.append(&spin);
        row.append(&group);
        (group, spin)
    }
    let (disk, threshold) = number(
        &options,
        "Umbral de uso (%)",
        "recipe-threshold",
        100.0,
        90.0,
    );
    let (journal_lines, lines) =
        number(&options, "Máximo de líneas", "recipe-lines", 5000.0, 100.0);
    let (journal_minutes, minutes) =
        number(&options, "Últimos minutos", "recipe-minutes", 10080.0, 60.0);
    body.append(&options);
    let hint = gtk::Label::new(None);
    hint.set_widget_name("recipe-validation");
    hint.set_wrap(true);
    hint.set_xalign(0.0);
    body.append(&hint);
    let selected = choice.downgrade();
    let input = parameter.downgrade();
    let limit = threshold.downgrade();
    let count = lines.downgrade();
    let period = minutes.downgrade();
    let preview = command.downgrade();
    let read: ReadCommand = Rc::new(move || {
        let (Some(choice), Some(input), Some(limit), Some(count), Some(period), Some(preview)) = (
            selected.upgrade(),
            input.upgrade(),
            limit.upgrade(),
            count.upgrade(),
            period.upgrade(),
            preview.upgrade(),
        ) else {
            return Err("Ventana cerrada".into());
        };
        let recipe = match choice.selected() {
            0 => {
                let buffer = preview.buffer();
                if buffer.char_count() > 16384 {
                    return Err("Máximo 16 KiB por comando".into());
                }
                return Ok(buffer
                    .text(&buffer.start_iter(), &buffer.end_iter(), false)
                    .into());
            }
            1 => Recipe::Disk {
                path: input.text().into(),
                max_used_percent: limit.value_as_int() as u8,
            },
            2 => Recipe::Service {
                unit: input.text().into(),
            },
            3 => Recipe::Journal {
                unit: input.text().into(),
                lines: count.value_as_int() as u32,
                minutes: period.value_as_int() as u32,
            },
            _ => return Err("Selecciona una receta disponible".into()),
        };
        recipe.command()
    });
    let selected = choice.downgrade();
    let input = parameter.downgrade();
    let preview = command.downgrade();
    let hint = hint.downgrade();
    let options = options.downgrade();
    let disk = disk.downgrade();
    let journal_lines = journal_lines.downgrade();
    let journal_minutes = journal_minutes.downgrade();
    let last = Cell::new(0);
    let free_command = RefCell::new(String::new());
    let disk_path = RefCell::new(String::from("/"));
    let unit = RefCell::new(String::new());
    let read_preview = read.clone();
    let guard = Cell::new(false);
    let refresh: Rc<dyn Fn()> = Rc::new(move || {
        if guard.replace(true) {
            return;
        }
        let (
            Some(choice),
            Some(input),
            Some(preview),
            Some(hint),
            Some(options),
            Some(disk),
            Some(journal_lines),
            Some(journal_minutes),
        ) = (
            selected.upgrade(),
            input.upgrade(),
            preview.upgrade(),
            hint.upgrade(),
            options.upgrade(),
            disk.upgrade(),
            journal_lines.upgrade(),
            journal_minutes.upgrade(),
        )
        else {
            guard.set(false);
            return;
        };
        let mode = choice.selected();
        let previous = last.replace(mode);
        if previous != mode {
            if previous == 0 {
                let b = preview.buffer();
                // Do not silently discard an oversized manual draft.
                if b.char_count() > 16384 {
                    last.set(0);
                    choice.set_selected(0);
                    hint.set_visible(true);
                    hint.set_text("El comando manual supera 16 KiB. Redúcelo antes de elegir una receta; el texto se ha conservado.");
                    guard.set(false);
                    return;
                }
                *free_command.borrow_mut() = b.text(&b.start_iter(), &b.end_iter(), false).into();
            } else if previous == 1 {
                *disk_path.borrow_mut() = input.text().into();
            } else {
                *unit.borrow_mut() = input.text().into();
            }
            if mode == 0 {
                preview.buffer().set_text(&free_command.borrow());
            } else if mode == 1 {
                input.set_text(&disk_path.borrow());
            } else {
                input.set_text(&unit.borrow());
            }
        }
        input.set_visible(mode != 0);
        input.set_placeholder_text(Some(if mode == 1 {
            "Ruta absoluta en el destino"
        } else {
            "Unidad, por ejemplo ssh.service"
        }));
        options.set_visible(mode == 1 || mode == 3);
        disk.set_visible(mode == 1);
        journal_lines.set_visible(mode == 3);
        journal_minutes.set_visible(mode == 3);
        preview.set_editable(mode == 0);
        hint.set_visible(mode != 0);
        if mode != 0 {
            match read_preview() {
                Ok(script) => {
                    preview.buffer().set_text(&script);
                    hint.set_text("Solo consulta. Revisa destino, carpeta y comando. Retorno 2: uso sobre el umbral; 3: servicio no activo. Journal consulta solo los registros accesibles.");
                    hint.remove_css_class("error");
                }
                Err(error) => {
                    preview.buffer().set_text("");
                    hint.set_text(&error);
                    hint.add_css_class("error");
                }
            }
        }
        guard.set(false);
    });
    let callback = refresh.clone();
    choice.connect_selected_notify(move |_| callback());
    let callback = refresh.clone();
    parameter.connect_changed(move |_| callback());
    for spin in [threshold, lines, minutes] {
        let callback = refresh.clone();
        spin.connect_value_changed(move |_| callback());
    }
    refresh();
    read
}
