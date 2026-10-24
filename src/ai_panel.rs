//! Shared graphical agent palette; no agent TUI is embedded here.
use super::*;
use nexuterm::{ai_config::AiKind, config::AiAgent};
pub(super) struct Process {
    pub(super) bridge: Option<Rc<nexuterm::terminal_mcp::Broker>>,
    native: RefCell<Option<Rc<nexuterm::codex::Client>>>,
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
        if let Some(client) = self.native.borrow_mut().take() {
            client.close();
        }
    }
    pub(super) fn busy(&self) -> bool {
        self.native.borrow().as_ref().is_some_and(|c| c.busy())
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.shutdown();
    }
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
    let start = icon_button("document-edit-symbolic", "Nueva conversación");
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
    });
    *state.ai_process.borrow_mut() = Some(process.clone());
    let chat = super::codex_chat::build(state, &process);
    side.append(&chat.widget);
    let login = chat.login.clone();
    side.append(&login);
    let recent = chat.history.downgrade();
    history.connect_clicked(move |_| {
        if let Some(recent) = recent.upgrade() {
            recent.set_visible(!recent.is_visible());
        }
    });
    let refreshing = Rc::new(Cell::new(false));
    let known = Rc::new(RefCell::new(Vec::<String>::new()));
    let refreshed = Rc::new(Cell::new(false));
    let refresh: Rc<dyn Fn()> = {
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
        let p = process.clone();
        let open = chat.open.clone();
        let last = last.clone();
        let order = order.clone();
        Rc::new(move |login| {
            let (Some(s), Some(selector), Some(info)) =
                (s.upgrade(), selector.upgrade(), notice.upgrade())
            else {
                return;
            };
            if p.busy() {
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
            open(agent, login);
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
    let p = process.clone();
    stop.connect_clicked(move |_| p.stop());
    let go = launch.clone();
    start.connect_clicked(move |_| go(false));
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
            } else if !matches!(mode.as_str(), "credentials" | "keyring")
                && let Some(button) = start.upgrade() {
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
