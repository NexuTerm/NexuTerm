#![allow(deprecated)] // Match the project GTK dialog compatibility.
//! Voice input stays local; a model and whisper.cpp executable are user configured.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
pub(super) fn button(state: &Rc<State>, composer: &gtk::TextView) -> gtk::Button {
    let button = icon_button(
        "audio-input-microphone-symbolic",
        "Dictar por voz · configuración local si no está disponible",
    );
    let active: Rc<RefCell<Option<(Arc<AtomicBool>, tokio::task::AbortHandle)>>> =
        Rc::new(RefCell::new(None));
    let cancel = active.clone();
    button.connect_unrealize(move |_| {
        if let Some((stop, task)) = cancel.borrow_mut().take() {
            stop.store(true, Ordering::Relaxed);
            task.abort();
        }
    });
    let s = Rc::downgrade(state);
    let input = composer.downgrade();
    button.connect_clicked(move |button| {
        if let Some((stop, _)) = active.borrow().as_ref() {
            stop.store(true, Ordering::Relaxed);
            button.set_sensitive(false);
            button.set_tooltip_text(Some("Transcribiendo…"));
            return;
        }
        let Some(state) = s.upgrade() else {
            return;
        };
        let prefs = state.config.borrow().preferences.clone();
        let (program, model) =
            nexuterm::dictation::resolve_engine(&prefs.dictation_program, &prefs.dictation_model);
        if connection::find_program(&[&program]).is_none()
            || !std::path::Path::new(&model).is_file()
        {
            configure(&state);
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let mut task = nexuterm::runtime::runtime().spawn(nexuterm::dictation::transcribe(
            program,
            model,
            prefs.dictation_language,
            stop.clone(),
        ));
        *active.borrow_mut() = Some((stop, task.abort_handle()));
        button.set_icon_name("media-playback-stop-symbolic");
        button.add_css_class("destructive-action");
        button.set_tooltip_text(Some("Grabando · pulsa para terminar (máximo 60 segundos)"));
        let button = button.downgrade();
        let input = input.clone();
        let active = active.clone();
        let s = s.clone();
        glib::spawn_future_local(async move {
            while !task.is_finished() {
                glib::timeout_future(std::time::Duration::from_millis(100)).await;
            }
            let result = (&mut task).await;
            active.borrow_mut().take();
            if let Some(button) = button.upgrade() {
                button.set_sensitive(true);
                button.set_icon_name("audio-input-microphone-symbolic");
                button.remove_css_class("destructive-action");
                button.set_tooltip_text(Some("Dictar por voz"));
            }
            match result {
                Ok(Ok(text)) => {
                    if let Some(input) = input.upgrade() {
                        let buffer = input.buffer();
                        if buffer.char_count() > 0 {
                            buffer.insert(&mut buffer.end_iter(), " ");
                        }
                        buffer.insert(&mut buffer.end_iter(), &text);
                        input.grab_focus();
                        #[cfg(debug_assertions)]
                        if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("dictation") {
                            assert!(
                                buffer
                                    .text(&buffer.start_iter(), &buffer.end_iter(), false)
                                    .ends_with(&text)
                            );
                            eprintln!("NEXUTERM_SMOKE_DICTATION_LOCAL_OK");
                        }
                    }
                }
                Ok(Err(error)) => {
                    if let Some(parent) = s.upgrade().and_then(|s| s.window.upgrade()) {
                        let dialog = gtk::MessageDialog::builder()
                            .transient_for(&parent)
                            .modal(true)
                            .text("No se pudo completar el dictado")
                            .secondary_text(error)
                            .buttons(gtk::ButtonsType::Close)
                            .build();
                        dialog.connect_response(|d, _| d.close());
                        dialog.present();
                    }
                }
                Err(_) => {}
            }
        });
    });
    #[cfg(debug_assertions)]
    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("dictation")
        && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
    {
        let start = button.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(700), move || {
            if let Some(button) = start.upgrade() {
                button.emit_clicked();
            }
        });
        let stop = button.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1300), move || {
            if let Some(button) = stop.upgrade() {
                button.emit_clicked();
            }
        });
    }
    button
}
pub(super) fn configure(state: &Rc<State>) {
    let Some(parent) = state.window.upgrade() else {
        return;
    };
    let dialog = gtk::Window::builder()
        .title("Configurar dictado local")
        .transient_for(&parent)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(480)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.set_margin_top(18);
    root.set_margin_bottom(18);
    root.set_margin_start(18);
    root.set_margin_end(18);
    let info = gtk::Label::new(Some(
        "El dictado usa PipeWire y whisper.cpp en tu equipo. Selecciona un ejecutable whisper-cli y un modelo GGML ya instalado. No se envía audio a servicios externos.",
    ));
    info.set_wrap(true);
    info.set_xalign(0.0);
    root.append(&info);
    let prefs = state.config.borrow().preferences.clone();
    let (engine, model_path) =
        nexuterm::dictation::resolve_engine(&prefs.dictation_program, &prefs.dictation_model);
    let program = gtk::Entry::new();
    program.set_text(&engine);
    program.set_placeholder_text(Some("Ejecutable whisper-cli"));
    root.append(&program);
    let model = gtk::Entry::new();
    model.set_text(&model_path);
    model.set_placeholder_text(Some("Ruta absoluta al modelo GGML"));
    root.append(&model);
    let choose = gtk::Button::with_label("Seleccionar modelo…");
    root.append(&choose);
    let weak = dialog.downgrade();
    let field = model.downgrade();
    choose.connect_clicked(move |_| {
        let Some(parent) = weak.upgrade() else {
            return;
        };
        let chooser = gtk::FileChooserNative::new(
            Some("Modelo de voz GGML"),
            Some(&parent),
            gtk::FileChooserAction::Open,
            Some("Seleccionar"),
            Some("Cancelar"),
        );
        let field = field.clone();
        chooser.connect_response(move |c, r| {
            if r == gtk::ResponseType::Accept {
                if let (Some(field), Some(path)) =
                    (field.upgrade(), c.file().and_then(|f| f.path()))
                {
                    field.set_text(&path.to_string_lossy());
                }
            }
            c.destroy();
        });
        chooser.show();
    });
    let languages = gtk::DropDown::from_strings(&[
        "Español",
        "Detectar automáticamente",
        "Otro idioma configurado",
    ]);
    languages.set_selected(match prefs.dictation_language.as_str() {
        "es" => 0,
        "auto" => 1,
        _ => 2,
    });
    root.append(&gtk::Label::new(Some("Idioma del dictado")));
    root.append(&languages);
    let status = gtk::Label::new(None);
    status.set_wrap(true);
    root.append(&status);
    let save = gtk::Button::with_label("Guardar configuración");
    save.add_css_class("suggested-action");
    save.set_sensitive(state.config_writable);
    root.append(&save);
    let s = Rc::downgrade(state);
    let w = dialog.downgrade();
    save.connect_clicked(move |_| {
        let Some(state) = s.upgrade() else {
            return;
        };
        if connection::find_program(&[program.text().as_str()]).is_none()
            || !std::path::Path::new(model.text().as_str()).is_file()
        {
            status.set_text("No se encuentra el ejecutable o el archivo del modelo.");
            return;
        }
        let mut config = state.config.borrow().clone();
        config.preferences.dictation_program = program.text().to_string();
        config.preferences.dictation_model = model.text().to_string();
        config.preferences.dictation_language = match languages.selected() {
            0 => "es".into(),
            1 => "auto".into(),
            _ => prefs.dictation_language.clone(),
        };
        match config::save(&state.path, &config) {
            Ok(()) => {
                *state.config.borrow_mut() = config;
                if let Some(w) = w.upgrade() {
                    w.close();
                }
            }
            Err(e) => status.set_text(&e),
        }
    });
    dialog.set_child(Some(&root));
    dialog.present();
}
