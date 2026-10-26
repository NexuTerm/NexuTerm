//! Shared graphical agent palette; no agent TUI is embedded here.
use super::*;
use nexuterm::{ai_config::AiKind, config::AiAgent};
pub(super) struct Process {
    pub(super) bridge: Option<Rc<nexuterm::terminal_mcp::Broker>>,
    native: RefCell<Option<Rc<nexuterm::codex::Client>>>,
    children: RefCell<Vec<Rc<Process>>>,
}
impl Process {
    pub(super) fn set_native(&self, client: Rc<nexuterm::codex::Client>) {
        if let Some(old) = self.native.borrow_mut().replace(client) {
            old.close();
        }
    }
    pub(super) fn stop(&self) {
        if let Some(client) = self.native.borrow().as_ref() {
            client.interrupt();
        }
    }
    pub(super) fn shutdown(&self) {
        for child in self.children.borrow().iter() {
            child.shutdown();
        }
        if let Some(client) = self.native.borrow_mut().take() {
            client.close();
        }
    }
    fn current_busy(&self) -> bool {
        self.native.borrow().as_ref().is_some_and(|c| c.busy())
    }
    pub(super) fn busy(&self) -> bool {
        self.current_busy() || self.children.borrow().iter().any(|c| c.busy())
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.shutdown();
    }
}
struct Conversation {
    page: gtk::Box,
    chat: super::codex_chat::Chat,
    process: Rc<Process>,
    label: gtk::Label,
    agent_id: RefCell<Option<String>>,
}
fn conversation(state: &Rc<State>, process: Rc<Process>) -> Rc<Conversation> {
    let chat = super::codex_chat::build(state, &process);
    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.append(&chat.widget);
    page.append(&chat.login);
    Rc::new(Conversation {
        page,
        chat,
        process,
        label: gtk::Label::new(Some("Nuevo chat")),
        agent_id: RefCell::new(None),
    })
}

pub(super) fn install(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    app: &gtk::Application,
    side: &gtk::Box,
) {
    side.set_width_request(420);
    for class in ["ai-panel", "nexuterm-section", "nexuterm-panel"] {
        side.add_css_class(class);
    }
    side.set_margin_top(2);
    side.set_margin_end(0);
    side.set_margin_bottom(8);
    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let title = gtk::Label::new(Some("Chat"));
    title.add_css_class("heading");
    heading.append(&title);
    let names = gtk::StringList::new(&[]);
    let order = Rc::new(RefCell::new(Vec::<usize>::new()));
    let sections = Rc::new(RefCell::new(Vec::<Option<&str>>::new()));
    let selector = gtk::DropDown::builder()
        .model(&names)
        .enable_search(true)
        .hexpand(true)
        .build();
    let factory = gtk::SignalListItemFactory::new();
    let section_names = sections.clone();
    let list_names = names.clone();
    factory.connect_bind(move |_, item| {
        let item = item.downcast_ref::<gtk::ListItem>().unwrap();
        let row = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let position = (0..list_names.n_items()).find(|&i| list_names.item(i) == item.item());
        if let Some(section) =
            position.and_then(|i| section_names.borrow().get(i as usize).copied().flatten())
        {
            if position != Some(0) {
                row.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            }
            let label = gtk::Label::new(Some(section));
            label.set_xalign(0.0);
            label.add_css_class("caption");
            label.add_css_class("dim-label");
            label.set_can_target(false);
            row.append(&label);
        }
        let label = gtk::Label::new(
            item.item()
                .and_downcast::<gtk::StringObject>()
                .as_ref()
                .map(|s| s.string())
                .as_deref(),
        );
        label.set_xalign(0.0);
        row.append(&label);
        item.set_child(Some(&row));
    });
    selector.set_list_factory(Some(&factory));
    selector.add_css_class("flat");
    selector.set_tooltip_text(Some(
        "Agente configurado; sus modelos aparecen en el editor",
    ));
    selector.set_widget_name("ai-agent-selector");
    heading.append(&selector);
    let history = icon_button(
        "document-open-recent-symbolic",
        "Mostrar u ocultar chats recientes",
    );
    let start = icon_button("list-add-symbolic", "Nuevo chat con otro modelo o agente");
    start.set_widget_name("ai-new-chat");
    let stop = icon_button("media-playback-stop-symbolic", "Detener respuesta");
    let settings = icon_button("emblem-system-symbolic", "Configurar agentes de IA");
    let close = icon_button("window-close-symbolic", "Ocultar agentes de IA");
    for button in [&history, &start, &stop, &settings, &close] {
        heading.append(button);
    }
    side.append(&heading);
    let info = gtk::Label::new(None);
    info.set_wrap(true);
    info.set_xalign(0.0);
    info.add_css_class("caption");
    info.set_visible(false);
    info.connect_label_notify(|l| l.set_visible(true));
    side.append(&info);
    let process = Rc::new(Process {
        bridge: super::ai_terminals::install(state),
        native: RefCell::new(None),
        children: RefCell::new(Vec::new()),
    });
    *state.ai_process.borrow_mut() = Some(process.clone());
    let notebook = gtk::Notebook::new();
    notebook.set_widget_name("ai-conversations");
    notebook.set_scrollable(true);
    notebook.set_show_border(false);
    notebook.set_vexpand(true);
    let initial = conversation(state, process.clone());
    notebook.append_page(&initial.page, Some(&initial.label));
    let active = Rc::new(RefCell::new(initial.clone()));
    let conversations = Rc::new(RefCell::new(vec![initial.clone()]));
    let login = initial.chat.login.clone();
    side.append(&notebook);
    let current = active.clone();
    history.connect_clicked(move |_| {
        let history = current.borrow().chat.history.clone();
        history.set_visible(!history.is_visible());
    });
    let refreshing = Rc::new(Cell::new(false));
    let known = Rc::new(RefCell::new(Vec::<String>::new()));
    let refreshed = Rc::new(Cell::new(false));
    let refresh: Rc<dyn Fn()> = {
        let names = names.clone();
        let s = Rc::downgrade(state);
        let selector = selector.downgrade();
        let refreshing = refreshing.clone();
        let known = known.clone();
        let refreshed = refreshed.clone();
        let order = order.clone();
        let sections = sections.clone();
        Rc::new(move || {
            let (Some(s), Some(selector)) = (s.upgrade(), selector.upgrade()) else {
                return;
            };
            refreshing.set(true);
            let selected = order
                .borrow()
                .get(selector.selected() as usize)
                .copied()
                .unwrap_or(0);
            let config = s.config.borrow();
            let fresh = config
                .ai_agents
                .iter()
                .position(|a| !known.borrow().contains(&a.id));
            let had_profiles = refreshed.replace(true);
            *known.borrow_mut() = config.ai_agents.iter().map(|a| a.id.clone()).collect();
            let mut indices: Vec<usize> = (0..config.ai_agents.len()).collect();
            indices.sort_by_key(|&i| !is_local(&config.ai_agents[i]));
            *sections.borrow_mut() = indices
                .iter()
                .enumerate()
                .map(|(n, &i)| {
                    let local = is_local(&config.ai_agents[i]);
                    (n == 0 || local != is_local(&config.ai_agents[indices[n - 1]]))
                        .then_some(if local { "Locales" } else { "Web" })
                })
                .collect();
            let labels: Vec<&str> = indices
                .iter()
                .map(|&i| config.ai_agents[i].name.as_str())
                .collect();
            *order.borrow_mut() = indices;
            names.splice(0, names.n_items(), &labels);
            if !labels.is_empty() {
                selector.set_selected(
                    order
                        .borrow()
                        .iter()
                        .position(|&i| i == selected)
                        .unwrap_or(0) as u32,
                );
            }
            refreshing.set(false);
            if had_profiles
                && let Some(index) = fresh.and_then(|i| order.borrow().iter().position(|&n| n == i))
            {
                if selector.selected() == index as u32 {
                    selector.notify("selected");
                } else {
                    selector.set_selected(index as u32);
                }
            }
        })
    };
    refresh();
    let last = Rc::new(Cell::new(selector.selected()));
    let launch: Rc<dyn Fn(bool)> = {
        let s = Rc::downgrade(state);
        let selector = selector.downgrade();
        let notice = info.downgrade();
        let active = Rc::downgrade(&active);
        let last = last.clone();
        let order = order.clone();
        Rc::new(move |login| {
            let (Some(s), Some(selector), Some(info)) =
                (s.upgrade(), selector.upgrade(), notice.upgrade())
            else {
                return;
            };
            let Some(active) = active.upgrade() else {
                return;
            };
            let current = active.borrow().clone();
            if current.process.current_busy() {
                info.set_text("Detén la respuesta actual antes de cambiar de agente.");
                if selector.selected() != last.get() {
                    selector.set_selected(last.get());
                }
                return;
            }
            let agent = s
                .config
                .borrow()
                .ai_agents
                .get(
                    order
                        .borrow()
                        .get(selector.selected() as usize)
                        .copied()
                        .unwrap_or(usize::MAX),
                )
                .cloned();
            let Some(agent) = agent else {
                info.set_text("Añade un agente desde los ajustes o inicia sesión en Codex.");
                return;
            };
            if let Err(error) = agent.validate() {
                info.set_text(&error);
                return;
            }
            last.set(selector.selected());
            info.set_visible(false);
            current.label.set_text(&agent.name);
            *current.agent_id.borrow_mut() = Some(agent.id.clone());
            (current.chat.open)(agent, login);
        })
    };
    let action = gio::SimpleAction::new("ai-agents", None);
    let panel = side.downgrade();
    action.connect_activate(move |_, _| {
        if let Some(side) = panel.upgrade() {
            side.set_visible(!side.is_visible());
        }
    });
    window.add_action(&action);
    app.set_accels_for_action("win.ai-agents", &["<Control><Shift>i"]);
    let panel = side.downgrade();
    close.connect_clicked(move |_| {
        if let Some(side) = panel.upgrade() {
            side.set_visible(false);
        }
    });
    let current = active.clone();
    stop.connect_clicked(move |_| current.borrow().process.stop());
    let current = active.clone();
    let chats = conversations.clone();
    let order_switch = order.clone();
    let choice = selector.downgrade();
    let s = Rc::downgrade(state);
    let changing = refreshing.clone();
    let last_chat = last.clone();
    notebook.connect_switch_page(move |_, page, _| {
        let found = chats
            .borrow()
            .iter()
            .find(|c| c.page.upcast_ref::<gtk::Widget>() == page)
            .cloned();
        if let Some(chat) = found {
            *current.borrow_mut() = chat.clone();
            if let (Some(s), Some(choice), Some(id)) = (
                s.upgrade(),
                choice.upgrade(),
                chat.agent_id.borrow().as_ref(),
            )
                && let Some(index) = order_switch.borrow().iter().position(|&i| {
                    s.config
                        .borrow()
                        .ai_agents
                        .get(i)
                        .is_some_and(|a| &a.id == id)
                }) {
                    changing.set(true);
                    choice.set_selected(index as u32);
                    changing.set(false);
                    last_chat.set(index as u32);
            }
        }
    });
    let go = launch.clone();
    let s = Rc::downgrade(state);
    let w = window.downgrade();
    let parent = notebook.downgrade();
    let current = active.clone();
    let chats = conversations.clone();
    let root = process.clone();
    let choices = names.clone();
    let factory = factory.clone();
    let order_new = order.clone();
    let selected_new = selector.downgrade();
    let notice = info.downgrade();
    start.connect_clicked(move |_| {
        if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
            && std::env::var("NEXUTERM_SMOKE_AI").as_deref() != Ok("new-chat")
        {
            go(false);
            return;
        }
        let (Some(state), Some(window), Some(notebook), Some(selected)) = (
            s.upgrade(),
            w.upgrade(),
            parent.upgrade(),
            selected_new.upgrade(),
        ) else {
            return;
        };
        if chats.borrow().len() >= 8 {
            if let Some(info) = notice.upgrade() {
                info.set_text("Cierra un chat antes de abrir otro (máximo 8).");
            }
            return;
        }
        let dialog = gtk::Window::builder()
            .title("Nuevo chat")
            .transient_for(&window)
            .modal(true)
            .destroy_with_parent(true)
            .default_width(400)
            .build();
        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content.set_margin_start(16);
        content.set_margin_end(16);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.append(&gtk::Label::new(Some(
            "Elige el agente y el modelo del nuevo chat",
        )));
        let agents = gtk::DropDown::builder()
            .model(&choices)
            .enable_search(true)
            .build();
        agents.set_widget_name("ai-new-chat-agent");
        agents.set_list_factory(Some(&factory));
        agents.set_selected(selected.selected());
        let models = gtk::DropDown::new(None::<gtk::StringList>, None::<gtk::Expression>);
        models.set_enable_search(true);
        let model_ids = Rc::new(RefCell::new(Vec::<String>::new()));
        let refresh_models: Rc<dyn Fn()> = {
            let s = Rc::downgrade(&state);
            let a = agents.downgrade();
            let m = models.downgrade();
            let ids = model_ids.clone();
            let order = order_new.clone();
            let chats = chats.clone();
            Rc::new(move || {
                if let (Some(s), Some(a), Some(m)) = (s.upgrade(), a.upgrade(), m.upgrade()) {
                    let agent = order
                        .borrow()
                        .get(a.selected() as usize)
                        .and_then(|&i| s.config.borrow().ai_agents.get(i).cloned());
                    let mut values = agent.as_ref().map(|a| a.models.clone()).unwrap_or_default();
                    if let Some(a) = agent {
                        if !a.model.is_empty() && !values.contains(&a.model) {
                            values.insert(0, a.model.clone());
                        }
                        for chat in chats
                            .borrow()
                            .iter()
                            .filter(|c| c.agent_id.borrow().as_ref() == Some(&a.id))
                        {
                            for model in chat
                                .chat
                                .models
                                .borrow()
                                .iter()
                                .filter_map(|v| v["model"].as_str())
                            {
                                if !values.iter().any(|v| v == model) {
                                    values.push(model.to_owned());
                                }
                            }
                        }
                    }
                    values.retain(|v| !v.is_empty());
                    let labels: Vec<&str> = if values.is_empty() {
                        vec!["Modelo predeterminado"]
                    } else {
                        values.iter().map(String::as_str).collect()
                    };
                    m.set_model(Some(&gtk::StringList::new(&labels)));
                    m.set_selected(0);
                    *ids.borrow_mut() = values;
                }
            })
        };
        refresh_models();
        let r = refresh_models.clone();
        agents.connect_selected_notify(move |_| r());
        content.append(&agents);
        content.append(&models);
        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let cancel = gtk::Button::with_label("Cancelar");
        let open = gtk::Button::with_label("Abrir chat");
        open.set_sensitive(choices.n_items() > 0);
        open.add_css_class("suggested-action");
        open.set_widget_name("ai-open-chat");
        buttons.append(&cancel);
        buttons.append(&open);
        content.append(&buttons);
        let d = dialog.downgrade();
        cancel.connect_clicked(move |_| {
            if let Some(d) = d.upgrade() {
                d.close();
            }
        });
        let s = Rc::downgrade(&state);
        let n = notebook.downgrade();
        let d = dialog.downgrade();
        let root = root.clone();
        let chats = chats.clone();
        let current = current.clone();
        let order = order_new.clone();
        let go = go.clone();
        let a = agents.downgrade();
        let m = models.downgrade();
        #[cfg(debug_assertions)]
        let smoke_context = (current.clone(), chats.clone(), root.clone());
        open.connect_clicked(move |open| {
            let (Some(state), Some(notebook), Some(dialog), Some(a), Some(m)) = (
                s.upgrade(),
                n.upgrade(),
                d.upgrade(),
                a.upgrade(),
                m.upgrade(),
            ) else {
                return;
            };
            let Some(mut agent) = order
                .borrow()
                .get(a.selected() as usize)
                .and_then(|&i| state.config.borrow().ai_agents.get(i).cloned())
            else {
                return;
            };
            if let Some(id) = model_ids.borrow().get(m.selected() as usize) {
                agent.model = id.clone();
            }
            if let Err(error) = agent.validate() {
                open.set_tooltip_text(Some(&error));
                return;
            }
            let process = Rc::new(Process {
                bridge: root.bridge.clone(),
                native: RefCell::new(None),
                children: RefCell::new(Vec::new()),
            });
            let chat = conversation(&state, process.clone());
            chat.label
                .set_text(&if agent.model.is_empty() || agent.name == agent.model {
                    agent.name.clone()
                } else {
                    format!("{} · {}", agent.name, agent.model)
                });
            *chat.agent_id.borrow_mut() = Some(agent.id.clone());
            let label = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            label.append(&chat.label);
            let close = icon_button("window-close-symbolic", "Cerrar este chat");
            label.append(&close);
            let n = notebook.downgrade();
            let page = chat.page.downgrade();
            let chats_close = chats.clone();
            let root_close = root.clone();
            let p = process.clone();
            let cleanup = chat.chat.close.clone();
            close.connect_clicked(move |_| {
                if let (Some(n), Some(page)) = (n.upgrade(), page.upgrade()) {
                    cleanup();
                    p.shutdown();
                    chats_close.borrow_mut().retain(|c| c.page != page);
                    root_close
                        .children
                        .borrow_mut()
                        .retain(|c| !Rc::ptr_eq(c, &p));
                    if let Some(index) = n.page_num(&page) {
                        n.remove_page(Some(index));
                    }
                }
            });
            let go = go.clone();
            chat.chat.login.connect_clicked(move |_| go(true));
            root.children.borrow_mut().push(process);
            chats.borrow_mut().push(chat.clone());
            notebook.append_page(&chat.page, Some(&label));
            *current.borrow_mut() = chat.clone();
            notebook.set_current_page(notebook.page_num(&chat.page));
            (chat.chat.open)(agent, false);
            dialog.close();
        });
        dialog.set_child(Some(&content));
        dialog.present();
        #[cfg(debug_assertions)]
        if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("new-chat") {
            let (current, chats, root) = smoke_context;
            find_named(current.borrow().page.upcast_ref(), "codex-composer")
                .unwrap()
                .downcast::<gtk::TextView>()
                .unwrap()
                .buffer()
                .set_text("Borrador Codex conservado");
            agents.set_selected(1);
            models.set_selected(1);
            let open = open.downgrade();
            let chats = chats.clone();
            let notebook = notebook.downgrade();
            let root = root.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
                open.upgrade().unwrap().emit_clicked();
                let chats = chats.clone();
                let notebook = notebook.clone();
                let root = root.clone();
                let sent = Cell::new(false);
                let tries = Cell::new(0);
                glib::timeout_add_local(std::time::Duration::from_millis(70), move || {
                    tries.set(tries.get() + 1);
                    assert!(tries.get() < 50, "new chat did not respond");
                    let n = notebook.upgrade().unwrap();
                    assert_eq!(n.n_pages(), 2);
                    let pages = chats.borrow();
                    let first = pages[0].clone();
                    let second = pages[1].clone();
                    drop(pages);
                    let input = find_named(second.page.upcast_ref(), "codex-composer")
                        .unwrap()
                        .downcast::<gtk::TextView>()
                        .unwrap();
                    let send = find_named(second.page.upcast_ref(), "codex-send")
                        .unwrap()
                        .downcast::<gtk::Button>()
                        .unwrap();
                    if !sent.get() && send.is_sensitive() {
                        input.buffer().set_text("Hola desde otro agente");
                        send.emit_clicked();
                        sent.set(true);
                        return glib::ControlFlow::Continue;
                    }
                    if !sent.get()
                        || !send.is_sensitive()
                        || !has_text(second.page.upcast_ref(), "Respuesta ACP gráfica.")
                    {
                        return glib::ControlFlow::Continue;
                    }
                    let original = find_named(first.page.upcast_ref(), "codex-composer")
                        .unwrap()
                        .downcast::<gtk::TextView>()
                        .unwrap()
                        .buffer();
                    assert_eq!(
                        original.text(&original.start_iter(), &original.end_iter(), false),
                        "Borrador Codex conservado"
                    );
                    assert!(root.native.borrow().as_ref().unwrap().alive());
                    n.set_current_page(Some(0));
                    n.set_current_page(Some(1));
                    let tab = n.tab_label(&second.page).unwrap();
                    let close = tab.last_child().unwrap().downcast::<gtk::Button>().unwrap();
                    close.emit_clicked();
                    assert_eq!(n.n_pages(), 1);
                    assert!(root.children.borrow().is_empty());
                    assert!(root.native.borrow().as_ref().unwrap().alive());
                    eprintln!("NEXUTERM_SMOKE_NEW_CHAT_OK");
                    glib::ControlFlow::Break
                });
            });
        }
    });
    let go = launch.clone();
    selector.connect_selected_notify(move |_| {
        if !refreshing.get() {
            go(false);
        }
    });
    let s = Rc::downgrade(state);
    let w = window.downgrade();
    let selected = selector.downgrade();
    let r = refresh.clone();
    let settings_order = order.clone();
    settings.set_sensitive(state.config_writable);
    settings.connect_clicked(move |_| {
        if let (Some(s), Some(w), Some(selected)) = (s.upgrade(), w.upgrade(), selected.upgrade()) {
            let index = settings_order
                .borrow()
                .get(selected.selected() as usize)
                .copied()
                .unwrap_or(usize::MAX);
            if matches!(
                std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
                Ok("settings" | "templates" | "settings-api")
            ) {
                configure(&s, &w, index, r.clone());
            } else {
                settings::manage(&s, &w, index, r.clone());
            }
        }
    });
    let s = Rc::downgrade(state);
    let selected = selector.downgrade();
    let r = refresh.clone();
    let go = launch.clone();
    login.connect_clicked(move |_| {
        let (Some(s), Some(selected)) = (s.upgrade(), selected.upgrade()) else {
            return;
        };
        if s.config.borrow().ai_agents.is_empty() && s.config_writable {
            let mut config = s.config.borrow().clone();
            config.ai_agents.push(AiAgent {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Codex".into(),
                kind: AiKind::Codex,
                program: "codex".into(),
                ..Default::default()
            });
            if config::save(&s.path, &config).is_ok() {
                *s.config.borrow_mut() = config;
                r();
                selected.set_selected(0);
            }
        }
        go(true);
    });
    let first = Cell::new(true);
    let go = launch.clone();
    side.connect_visible_notify(move |side| {
        if side.is_visible()
            && first.replace(false)
            && std::env::var_os("NEXUTERM_SMOKE_TEST").is_none()
        {
            go(false);
        }
    });
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && let Ok(mode) = std::env::var("NEXUTERM_SMOKE_AI")
    {
        if matches!(mode.as_str(), "credentials" | "keyring") {
            super::ai_credentials::smoke(state, mode == "keyring");
        }
        if mode == "switch" {
            let selected = selector.downgrade();
            glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
                if let Some(selector) = selected.upgrade() {
                    assert_eq!(selector.model().unwrap().n_items(), 2);
                    selector.set_selected(1);
                }
            });
        }
        if mode == "new-chat" {
            let go = launch.clone();
            let button = start.downgrade();
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                go(false);
            });
            glib::timeout_add_local_once(std::time::Duration::from_millis(800), move || {
                button.upgrade().unwrap().emit_clicked();
            });
        }
        let panel = side.downgrade();
        let start = start.downgrade();
        let settings = settings.downgrade();
        let login = login.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            if let Some(panel) = panel.upgrade() {
                panel.set_visible(true);
            }
            if mode == "login" {
                if let Some(button) = login.upgrade() {
                    button.emit_clicked();
                }
            } else if matches!(
                mode.as_str(),
                "settings" | "templates" | "groups" | "settings-api"
            ) {
                if let Some(button) = settings.upgrade() {
                    button.emit_clicked();
                }
                eprintln!("NEXUTERM_SMOKE_AI_SETTINGS_OPEN");
            } else if !matches!(mode.as_str(), "credentials" | "keyring" | "new-chat")
                && let Some(button) = start.upgrade()
            {
                button.emit_clicked();
            }
        });
    }
    app.connect_shutdown(move |_| process.shutdown());
}
#[path = "ai_settings.rs"]
mod settings;
use settings::configure;

/// The connector process is local for every provider; classify by the inference endpoint.
fn is_local(agent: &AiAgent) -> bool {
    if !agent.kind.is_api() {
        return false;
    }
    url::Url::parse(&agent.endpoint)
        .ok()
        .is_some_and(|url| match url.host() {
            Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            None => false,
        })
}

#[cfg(debug_assertions)]
fn find_named(widget: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if widget.widget_name() == name {
        return Some(widget.clone());
    }
    let mut child = widget.first_child();
    while let Some(w) = child {
        if let Some(found) = find_named(&w, name) {
            return Some(found);
        }
        child = w.next_sibling();
    }
    None
}

#[cfg(debug_assertions)]
fn has_text(widget: &gtk::Widget, text: &str) -> bool {
    if widget
        .downcast_ref::<gtk::Label>()
        .is_some_and(|label| label.text() == text)
    {
        return true;
    }
    let mut child = widget.first_child();
    while let Some(w) = child {
        if has_text(&w, text) {
            return true;
        }
        child = w.next_sibling();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn groups_use_inference_location_not_connector_executable() {
        for endpoint in [
            "http://127.0.0.1:11434/v1",
            "http://localhost:11434/v1",
            "http://[::1]:11434/v1",
        ] {
            assert!(is_local(&AiAgent {
                kind: AiKind::ChatCompletions,
                endpoint: endpoint.into(),
                ..Default::default()
            }));
        }
        assert!(!is_local(&AiAgent {
            kind: AiKind::Codex,
            program: "codex".into(),
            ..Default::default()
        }));
        assert!(!is_local(&AiAgent {
            kind: AiKind::Responses,
            endpoint: "https://api.openai.com/v1".into(),
            ..Default::default()
        }));
    }
}
