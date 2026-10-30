//! Paged local transcript browser; never executes or sends recorded content.
use crate::ui::State;
use gtk::{glib, prelude::*};
use nexuterm::chat_history::{self, PAGE_SIZE, Recorder};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub async fn flush(state: &Rc<State>) {
    let pending=state.chat_snapshots().into_iter().map(chat_history::RecentSave::new).collect::<Vec<_>>();
    if let Err(error)=nexuterm::runtime::runtime().spawn_blocking(move || {
        let dir=chat_history::directory()?;
        for chat in pending {chat.save(&dir)?;}
        Ok::<(),String>(())
    }).await.unwrap_or_else(|e|Err(e.to_string())) {
        state.status.set_text(&format!("No se pudieron guardar los chats recientes: {error}"));
    }

    let recorders = state.chat_archives.borrow().clone();
    for recorder in recorders {
        if let Err(error) = recorder.flush().await {
            state.status.set_text(&format!("Captura de historial detenida: {error}. Se conserva la copia de recuperación del chat."));
        }
    }
}
pub fn show(state: &Rc<State>, recorder: &Rc<Recorder>) {
    let Some(window) = state.window() else {
        return;
    };
    let dialog = gtk::Window::builder()
        .title("Historial local de chats")
        .transient_for(&window)
        .destroy_with_parent(true)
        .default_width(850)
        .default_height(640)
        .build();
    dialog.set_widget_name("chat-local-history");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_start(12);
    body.set_margin_end(12);
    body.set_margin_top(12);
    body.set_margin_bottom(12);
    let explanation = gtk::Label::new(Some(
        "Historial local completo, separado de las 128 tarjetas visibles. Vista de 25 mensajes con texto acotado; exportar JSONL conserva todas las entradas y fragmentos. Nada se envía al agente.",
    ));
    explanation.set_wrap(true);
    body.append(&explanation);
    let capture = gtk::CheckButton::with_label("Conservar historial completo de este chat");
    capture.set_active(recorder.enabled());
    capture.set_sensitive(recorder.error().is_none());
    capture.set_tooltip_text(Some("Controla el archivo de historial. La copia automática de recuperación conserva los últimos mensajes y el borrador por separado."));
    body.append(&capture);
    let r = recorder.clone();
    capture.connect_toggled(move |b| {
        if b.is_active() && r.error().is_some() {
            b.set_active(false);
            b.set_sensitive(false);
            return;
        }
        r.set_enabled(b.is_active());
    });
    let choices = gtk::StringList::new(&[]);
    let selector = gtk::DropDown::builder()
        .model(&choices)
        .enable_search(true)
        .build();
    body.append(&selector);
    let messages = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&messages)
        .build();
    body.append(&scroll);
    let status = gtk::Label::new(None);
    status.set_widget_name("chat-history-status");
    status.set_wrap(true);
    status.set_xalign(0.0);
    body.append(&status);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let previous = gtk::Button::with_label("Anterior");
    let next = gtk::Button::with_label("Siguiente");
    next.set_widget_name("chat-history-next");
    let export = gtk::Button::with_label("Exportar JSONL…");
    let remove = gtk::Button::with_label("Borrar historial…");
    for b in [&previous, &next, &export, &remove] {
        row.append(b);
    }
    body.append(&row);
    dialog.set_child(Some(&body));
    dialog.present();
    let ids = Rc::new(RefCell::new(Vec::<String>::new()));
    let offset = Rc::new(Cell::new(0usize));
    let generation = Rc::new(Cell::new(0u64));
    let loading = Rc::new(Cell::new(false));
    let load: Rc<dyn Fn()> = {
        let ids = ids.clone();
        let selector = selector.downgrade();
        let offset = offset.clone();
        let messages = messages.downgrade();
        let status = status.downgrade();
        let previous = previous.downgrade();
        let next = next.downgrade();
        let generation = generation.clone();
        let loading = loading.clone();
        Rc::new(move || {
            let Some(id) = selector
                .upgrade()
                .and_then(|s| ids.borrow().get(s.selected() as usize).cloned())
            else {
                return;
            };
            if loading.replace(true) {
                return;
            }
            if let Some(s) = selector.upgrade() {
                s.set_sensitive(false);
            }
            if let Some(b) = previous.upgrade() {
                b.set_sensitive(false);
            }
            if let Some(b) = next.upgrade() {
                b.set_sensitive(false);
            }
            let loading = loading.clone();
            let selector = selector.clone();
            generation.set(generation.get() + 1);
            let epoch = generation.get();
            let generation = generation.clone();
            let start = offset.get();
            let (messages, status, previous, next) = (
                messages.clone(),
                status.clone(),
                previous.clone(),
                next.clone(),
            );
            glib::spawn_future_local(async move {
                let result = nexuterm::runtime::runtime()
                    .spawn_blocking(move || {
                        chat_history::page(&chat_history::directory()?, &id, start)
                    })
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r);
                loading.set(false);
                if let Some(s) = selector.upgrade() {
                    s.set_sensitive(true);
                }
                if generation.get() != epoch {
                    return;
                }
                let (Some(messages), Some(status)) = (messages.upgrade(), status.upgrade()) else {
                    return;
                };
                match result {
                    Ok(page) => {
                        while let Some(child) = messages.first_child() {
                            messages.remove(&child);
                        }
                        for m in page.messages {
                            let title = gtk::Label::new(Some(&m.title));
                            title.add_css_class("heading");
                            title.set_xalign(0.0);
                            messages.append(&title);
                            let text = gtk::Label::new(Some(&m.text));
                            text.set_selectable(true);
                            text.set_wrap(true);
                            text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                            text.set_xalign(0.0);
                            messages.append(&text);
                        }
                        status.set_text(&format!(
                            "Mensajes {}–{} de {}. La exportación conserva el texto completo.",
                            if page.total == 0 { 0 } else { start + 1 },
                            (start + PAGE_SIZE).min(page.total),
                            page.total
                        ));
                        if let Some(b) = previous.upgrade() {
                            b.set_sensitive(start > 0);
                        }
                        if let Some(b) = next.upgrade() {
                            b.set_sensitive(start + PAGE_SIZE < page.total);
                        }
                    }
                    Err(error) => status.set_text(&error),
                }
            });
        })
    };
    let l = load.clone();
    let start = offset.clone();
    selector.connect_selected_notify(move |_| {
        start.set(0);
        l();
    });
    let l = load.clone();
    let start = offset.clone();
    previous.connect_clicked(move |_| {
        start.set(start.get().saturating_sub(PAGE_SIZE));
        l();
    });
    let l = load.clone();
    let start = offset.clone();
    next.connect_clicked(move |_| {
        start.set(start.get().saturating_add(PAGE_SIZE));
        l();
    });
    for (button, deleting) in [(export, false), (remove, true)] {
        let selector = selector.downgrade();
        let ids = ids.clone();
        let dialog = dialog.downgrade();
        let status = status.downgrade();
        let recorder = recorder.clone();
        let capture = capture.downgrade();
        button.connect_clicked(move |_|{
            let Some(id)=selector.upgrade().and_then(|s|ids.borrow().get(s.selected() as usize).cloned()) else{return;};
            let (dialog,status,recorder,capture)=(dialog.clone(),status.clone(),recorder.clone(),capture.clone());
            glib::spawn_future_local(async move {
                let Some(window)=dialog.upgrade() else{return;};
                if deleting {
                    let alert=gtk::AlertDialog::builder().modal(true).message("¿Borrar este historial local?").detail("No se borran originales ni las copias de recuperación existentes. Para impedir nuevas capturas del chat actual también se desactiva su historial.").buttons(["Cancelar","Borrar historial"]).cancel_button(0).default_button(0).build();
                    if alert.choose_future(Some(&window)).await!=Ok(1){return;}
                    if id==recorder.id {recorder.set_enabled(false);if let Some(c)=capture.upgrade(){c.set_active(false);c.set_sensitive(false);}}
                    let _=recorder.flush().await;
                    let result=nexuterm::runtime::runtime().spawn_blocking(move ||chat_history::remove(&chat_history::directory()?,&id)).await.map_err(|e|e.to_string()).and_then(|r|r);
                    if let Some(status)=status.upgrade(){status.set_text(&result.map(|_|"Historial borrado; las copias de recuperación no se han modificado.".to_owned()).unwrap_or_else(|e|e));}
                } else {
                    let picker=gtk::FileDialog::builder().title("Exportar historial completo").initial_name("chat.jsonl").build();
                    if let Ok(file)=picker.save_future(Some(&window)).await {
                        let Some(path)=file.path() else{if let Some(s)=status.upgrade(){s.set_text("Selecciona un destino local nuevo.");}return;};
                        let _=recorder.flush().await;
                        let result=nexuterm::runtime::runtime().spawn_blocking(move ||chat_history::export(&chat_history::directory()?,&id,&path)).await.map_err(|e|e.to_string()).and_then(|r|r);
                        if let Some(status)=status.upgrade(){status.set_text(&result.map(|_|"Historial exportado sin reemplazar archivos existentes.".to_owned()).unwrap_or_else(|e|e));}
                    }
                }
            });
        });
    }
    let (choices, selector, status) = (
        choices.downgrade(),
        selector.downgrade(),
        status.downgrade(),
    );
    let current = recorder.clone();
    let initializing = loading.clone();
    glib::spawn_future_local(async move {
        let _ = current.flush().await;
        let result = nexuterm::runtime::runtime()
            .spawn_blocking(|| chat_history::list(&chat_history::directory()?))
            .await
            .map_err(|e| e.to_string())
            .and_then(|r| r);
        if let (Some(choices), Some(selector), Some(status)) =
            (choices.upgrade(), selector.upgrade(), status.upgrade())
        {
            match result {
                Ok(entries) => {
                    let selected = entries.iter().position(|e| e.id == current.id).unwrap_or(0);
                    let labels: Vec<String> = entries
                        .iter()
                        .map(|e| {
                            let date = glib::DateTime::from_unix_local(e.saved_at as i64)
                                .ok()
                                .and_then(|d| d.format("%d/%m %H:%M").ok())
                                .map(|s| s.to_string())
                                .unwrap_or_default();
                            format!(
                                "{date} · {} · {}",
                                e.title.chars().take(60).collect::<String>(),
                                e.id.chars().take(8).collect::<String>()
                            )
                        })
                        .collect();
                    *ids.borrow_mut() = entries.iter().map(|e| e.id.clone()).collect();
                    initializing.set(true);
                    choices.splice(
                        0,
                        choices.n_items(),
                        &labels.iter().map(String::as_str).collect::<Vec<_>>(),
                    );
                    selector.set_selected(selected as u32);
                    initializing.set(false);
                    load();
                }
                Err(error) => status.set_text(&error),
            }
        }
    });
}
