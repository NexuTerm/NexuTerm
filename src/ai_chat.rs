//! Native, text-only API chat. No terminal or remote content is collected automatically.
use super::*;
use nexuterm::{
    ai::{self, Message},
    config::AiAgent,
};

pub(super) struct Chat {
    pub widget: gtk::Box,
    pub new_conversation: Rc<dyn Fn(AiAgent)>,
}
fn bubble(list: &gtk::Box, title: &str, text: &str) -> gtk::Label {
    let group = gtk::Box::new(gtk::Orientation::Vertical, 4);
    group.set_margin_top(8);
    group.set_margin_bottom(8);
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("heading");
    heading.set_xalign(0.0);
    let label = gtk::Label::new(Some(text));
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_selectable(true);
    crate::context_menu::message(&label);
    label.set_xalign(0.0);
    group.append(&heading);
    group.append(&label);
    list.append(&group);
    // Failed exchanges are not in API history, but their visible widgets must also be bounded.
    while list.observe_children().n_items() > 130 {
        if let Some(child) = list.first_child() {
            list.remove(&child);
        } else {
            break;
        }
    }
    label
}
pub(super) fn build(state: &Rc<State>, process: &Rc<ai_panel::Process>) -> Chat {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let conversation = gtk::Box::new(gtk::Orientation::Vertical, 6);
    crate::context_menu::conversation(&conversation);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&conversation)
        .build();
    widget.append(&scroll);
    let composer = gtk::TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .accepts_tab(false)
        .top_margin(8)
        .bottom_margin(8)
        .left_margin(8)
        .right_margin(8)
        .build();
    composer.update_property(&[gtk::accessible::Property::Label("Mensaje al agente de IA")]);
    let input_scroll = gtk::ScrolledWindow::builder()
        .min_content_height(80)
        .max_content_height(160)
        .propagate_natural_height(true)
        .child(&composer)
        .build();
    input_scroll.add_css_class("nexuterm-section");
    input_scroll.add_css_class("ai-composer");
    widget.append(&input_scroll);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let spinner = gtk::Spinner::new();
    let info = gtk::Label::new(Some("Escribe un mensaje. Ctrl+Intro para enviar."));
    info.set_wrap(true);
    info.set_hexpand(true);
    info.set_xalign(0.0);
    let send = gtk::Button::with_label("Enviar");
    send.add_css_class("suggested-action");
    row.append(&spinner);
    row.append(&info);
    row.append(&send);
    widget.append(&row);
    let current: Rc<RefCell<Option<AiAgent>>> = Rc::new(RefCell::new(None));
    let history = Rc::new(RefCell::new(Vec::<Message>::new()));
    let new_conversation: Rc<dyn Fn(AiAgent)> = {
        let list = conversation.downgrade();
        let input = composer.downgrade();
        let current = current.clone();
        let history = history.clone();
        let info = info.downgrade();
        Rc::new(move |agent| {
            if let (Some(list), Some(input), Some(info)) =
                (list.upgrade(), input.upgrade(), info.upgrade())
            {
                while let Some(child) = list.first_child() {
                    list.remove(&child);
                }
                history.borrow_mut().clear();
                input.buffer().set_text("");
                info.set_text(&format!("{} · {}", agent.name, agent.model));
                bubble(
                    &list,
                    &agent.name,
                    &format!(
                        "Modelo: {}\nServidor: {}\nSolo se envían los mensajes escritos en este chat.",
                        agent.model, agent.endpoint
                    ),
                );
                *current.borrow_mut() = Some(agent);
                input.grab_focus();
            }
        })
    };
    let input = composer.downgrade();
    let list = conversation.downgrade();
    let notice = info.downgrade();
    let button = send.downgrade();
    let spinning = spinner.downgrade();
    let s = Rc::downgrade(state);
    let p = process.clone();
    let send_history = history.clone();
    send.connect_clicked(move |_| {
        let (Some(input), Some(list), Some(info), Some(button), Some(spinner), Some(state)) = (
            input.upgrade(),
            list.upgrade(),
            notice.upgrade(),
            button.upgrade(),
            spinning.upgrade(),
            s.upgrade(),
        ) else {
            return;
        };
        if p.busy() {
            info.set_text("Espera o detén la respuesta actual.");
            return;
        }
        let Some(agent) = current.borrow().clone() else {
            info.set_text("Inicia una conversación con un perfil API.");
            return;
        };
        let buffer = input.buffer();
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), false)
            .to_string();
        if text.trim().is_empty() {
            return;
        }
        if text.len() > 16384 {
            info.set_text("Máximo 16 KiB por mensaje.");
            return;
        }
        let mut messages = send_history.borrow().clone();
        messages.push(Message {
            role: "user".into(),
            content: text.clone(),
        });
        if let Err(error) = ai::request_body(&agent, &messages) {
            info.set_text(&error);
            return;
        }
        let cancel = p.begin_api();
        input.set_sensitive(false);
        button.set_sensitive(false);
        spinner.start();
        buffer.set_text("");
        bubble(&list, "Tú", &text);
        let reply = bubble(&list, &agent.name, "Esperando respuesta…");
        let reply = reply.downgrade();
        let input = input.downgrade();
        let button = button.downgrade();
        let spinner = spinner.downgrade();
        let info = info.downgrade();
        let p = p.clone();
        let history = send_history.clone();
        glib::spawn_future_local(async move {
            let result = async {
                let key = super::ai_credentials::lookup(&state, &agent).await?;
                if cancel.is_cancelled() {
                    return Err("Respuesta cancelada".to_owned());
                }
                let timeout = agent.timeout_seconds as u64 + 5;
                let request_messages = messages.clone();
                let mut worker = nexuterm::runtime::runtime().spawn(async move {
                    tokio::time::timeout(
                        std::time::Duration::from_secs(timeout),
                        ai::complete(agent, key, request_messages),
                    )
                    .await
                    .map_err(|_| "Se agotó el tiempo de espera de la API".to_owned())?
                });
                p.track_api(worker.abort_handle());
                loop {
                    if cancel.is_cancelled() {
                        worker.abort();
                        let _ = worker.await;
                        return Err("Respuesta cancelada".into());
                    }
                    if worker.is_finished() {
                        break;
                    }
                    glib::timeout_future(std::time::Duration::from_millis(50)).await;
                }
                (&mut worker)
                    .await
                    .map_err(|_| "La solicitud API no pudo finalizar".to_owned())?
            }
            .await;
            p.finish_api();
            if let Some(spinner) = spinner.upgrade() {
                spinner.stop();
            }
            if let Some(button) = button.upgrade() {
                button.set_sensitive(true);
            }
            let input = input.upgrade();
            if let Some(input) = &input {
                input.set_sensitive(true);
            }
            match result {
                Ok(answer) => {
                    if let Some(reply) = reply.upgrade() {
                        reply.set_text(&answer);
                    }
                    messages.push(Message {
                        role: "assistant".into(),
                        content: answer,
                    });
                    *history.borrow_mut() = messages;
                    if let Some(info) = info.upgrade() {
                        info.set_text("Respuesta recibida.");
                    }
                }
                Err(error) => {
                    if let Some(reply) = reply.upgrade() {
                        reply.set_text(&error);
                    }
                    if let Some(input) = &input {
                        input.buffer().set_text(&text);
                    }
                    if let Some(info) = info.upgrade() {
                        info.set_text(
                            "No se añadió este intercambio al contexto. Puedes reintentar.",
                        );
                    }
                }
            }
        });
    });
    let send_weak = send.downgrade();
    let keys = gtk::EventControllerKey::new();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if key == gtk::gdk::Key::Return && modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK)
        {
            if let Some(send) = send_weak.upgrade() {
                if send.is_sensitive() {
                    send.emit_clicked();
                }
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    composer.add_controller(keys);
    let adjustment = scroll.vadjustment();
    adjustment.connect_upper_notify(move |adjustment| {
        adjustment.set_value((adjustment.upper() - adjustment.page_size()).max(0.0));
    });
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && matches!(
            std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
            Ok("api" | "shutdown")
        )
    {
        let first_input = composer.downgrade();
        let first_send = send.downgrade();
        // The panel selects the fixture profile before this timer runs.
        glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
            if let (Some(input), Some(send)) = (first_input.upgrade(), first_send.upgrade()) {
                input.buffer().set_text("primer-mensaje");
                send.emit_clicked();
            }
        });
        if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("api") {
            let input = composer.downgrade();
            let send = send.downgrade();
            let attempts = Cell::new(0);
            let second = Cell::new(false);
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                attempts.set(attempts.get() + 1);
                assert!(attempts.get() < 25, "API chat did not complete two turns");
                if history.borrow().len() == 2 && !second.replace(true) {
                    if let (Some(input), Some(send)) = (input.upgrade(), send.upgrade()) {
                        input.buffer().set_text("segundo-mensaje");
                        send.emit_clicked();
                    }
                }
                if history.borrow().len() == 4 {
                    assert_eq!(history.borrow()[3].content, "fixture-2");
                    eprintln!("NEXUTERM_SMOKE_AI_API_OK");
                    return glib::ControlFlow::Break;
                }
                glib::ControlFlow::Continue
            });
        }
    }
    Chat {
        widget,
        new_conversation,
    }
}
