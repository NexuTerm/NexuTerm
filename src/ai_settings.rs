const CREDENTIAL_ENV: &[&str] = &[
    "",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "DEEPSEEK_API_KEY",
    "OPENCODE_API_KEY",
    "COPILOT_GITHUB_TOKEN",
    "GH_TOKEN",
];
use super::*;
use adw::prelude::ActionRowExt;
use nexuterm::ai_config::{AgentDefinition, AgentMode, McpServer, OpenCodeConfig};

const ENGINES: &[&str] = &[
    "ACP personalizado",
    "OpenCode",
    "OpenAI Responses",
    "Chat Completions",
    "Codex",
    "Agente ACP",
];
fn set_engines(
    widget: &gtk::DropDown,
    allowed: &Rc<RefCell<Vec<u32>>>,
    values: Vec<u32>,
    selected: u32,
) {
    let labels: Vec<&str> = values.iter().map(|&i| ENGINES[i as usize]).collect();
    let position = values.iter().position(|&i| i == selected).unwrap_or(0);
    *allowed.borrow_mut() = values;
    widget.set_model(Some(&gtk::StringList::new(&labels)));
    widget.set_selected(position as u32);
}
fn engine(widget: &gtk::DropDown, allowed: &Rc<RefCell<Vec<u32>>>) -> u32 {
    allowed
        .borrow()
        .get(widget.selected() as usize)
        .copied()
        .unwrap_or(0)
}
fn permitted_engines(agent: &AiAgent) -> Vec<u32> {
    if agent.is_codex() {
        return vec![4];
    }
    match agent.kind {
        AiKind::Local => vec![0],
        AiKind::OpenCode => vec![1],
        AiKind::Codex => vec![4],
        AiKind::Acp => vec![5],
        AiKind::Responses => vec![2],
        AiKind::ChatCompletions => vec![3],
    }
}

fn credential_options(widget: &gtk::DropDown, program: &str) {
    let previous = widget
        .selected_item()
        .and_downcast::<gtk::StringObject>()
        .map(|s| s.string().to_string())
        .unwrap_or_default();
    let name = std::path::Path::new(program)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let values: &[&str] = match name {
        "codex" => &["", "OPENAI_API_KEY"],
        "claude" | "claude-agent-acp" => &["", "ANTHROPIC_API_KEY"],
        "gemini" => &["", "GEMINI_API_KEY"],
        "copilot" => &["", "COPILOT_GITHUB_TOKEN", "GH_TOKEN"],
        "opencode" => &["", "OPENCODE_API_KEY"],
        _ => CREDENTIAL_ENV,
    };
    widget.set_model(Some(&gtk::StringList::new(values)));
    widget.set_selected(values.iter().position(|v| *v == previous).unwrap_or(0) as u32);
}

fn text(buffer: &gtk::TextBuffer) -> String {
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string()
}
fn editor(value: &str, height: i32) -> (gtk::ScrolledWindow, gtk::TextBuffer) {
    let view = gtk::TextView::builder()
        .monospace(true)
        .wrap_mode(gtk::WrapMode::WordChar)
        .build();
    view.buffer().set_text(value);
    let buffer = view.buffer();
    let scroll = gtk::ScrolledWindow::builder()
        .min_content_height(height)
        .vexpand(true)
        .child(&view)
        .build();
    (scroll, buffer)
}
fn margins(widget: &impl IsA<gtk::Widget>) {
    widget.set_margin_start(12);
    widget.set_margin_end(12);
    widget.set_margin_top(12);
    widget.set_margin_bottom(12);
}

fn page(tabs: &gtk::Notebook, widget: &impl IsA<gtk::Widget>, title: &str) {
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(widget)
        .build();
    tabs.append_page(&scroll, Some(&gtk::Label::new(Some(title))));
}

pub(super) fn configure(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    index: usize,
    refresh: Rc<dyn Fn()>,
) {
    if !state.config_writable {
        return;
    }
    let dialog = gtk::Window::builder()
        .title("Configurar agentes de IA")
        .transient_for(window)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(680)
        .default_height(680)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    margins(&root);
    let tabs = gtk::Notebook::new();
    tabs.set_vexpand(true);
    root.append(&tabs);
    let old = state.config.borrow().ai_agents.get(index).cloned();
    let agent = old.clone().unwrap_or_default();
    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(12)
        .build();
    margins(&grid);
    let presets = gtk::DropDown::from_strings(&[
        "Personalizado",
        "Codex",
        "Claude Code",
        "OpenCode",
        "OpenAI API",
        "API compatible",
        "Ollama local",
        "Gemini API",
        "DeepSeek API",
        "Agente ACP",
        "Gemini CLI (ACP)",
        "GitHub Copilot",
    ]);
    grid.attach(&gtk::Label::new(Some("Plantilla")), 0, 0, 1, 1);
    grid.attach(&presets, 1, 0, 1, 1);
    let name = entry(&grid, 1, "Nombre", &agent.name);
    name.set_widget_name("ai-settings-name");
    let allowed = Rc::new(RefCell::new(Vec::new()));
    let kind = gtk::DropDown::new(None::<gtk::StringList>, None::<gtk::Expression>);
    let values = if old.is_some() {
        permitted_engines(&agent)
    } else {
        (0..6).collect()
    };
    let initial = if agent.is_codex() {
        4
    } else {
        match agent.kind {
            AiKind::Local => 0,
            AiKind::OpenCode => 1,
            AiKind::Responses => 2,
            AiKind::ChatCompletions => 3,
            AiKind::Codex => 4,
            AiKind::Acp => 5,
        }
    };
    set_engines(&kind, &allowed, values, initial);
    grid.attach(&gtk::Label::new(Some("Motor")), 0, 2, 1, 1);
    grid.attach(&kind, 1, 2, 1, 1);
    let program = entry(&grid, 3, "Ejecutable", &agent.program);
    let args = entry(
        &grid,
        4,
        "Argumentos (lista JSON)",
        &serde_json::to_string(&agent.args).unwrap(),
    );
    let directory = entry(&grid, 5, "Carpeta de trabajo", &agent.directory);
    directory.set_placeholder_text(Some("Vacío: carpeta personal"));
    let model = entry(&grid, 6, "Modelo", &agent.model);
    model.set_placeholder_text(Some("API: identificador; OpenCode: proveedor/modelo"));
    let free = gtk::Button::with_label("Actualizar modelos gratuitos de OpenCode");
    grid.attach(&free, 0, 7, 2, 1);
    let free_names = gtk::StringList::new(&[]);
    let free_selector = gtk::DropDown::builder()
        .model(&free_names)
        .enable_search(true)
        .build();
    free_selector.set_tooltip_text(Some("Selecciona un modelo gratuito para OpenCode CLI"));
    grid.attach(&free_selector, 0, 8, 2, 1);
    let (models_view, model_list) = editor(&agent.models.join("\n"), 90);
    model_list.set_text(&agent.models.join("\n"));
    grid.attach(
        &gtk::Label::new(Some("Modelos de este agente (uno por línea)")),
        0,
        11,
        2,
        1,
    );
    grid.attach(&models_view, 0, 12, 2, 1);
    let terminal_access = gtk::CheckButton::with_label(
        "Acceso a terminales abiertas (cada lectura o comando requiere aprobación)",
    );
    terminal_access.set_active(agent.terminal_access);
    grid.attach(&terminal_access, 0, 13, 2, 1);
    let note = gtk::Label::new(Some(
        "Todos los agentes se abren en el mismo chat gráfico. Los agentes ACP necesitan un ejecutable compatible (Claude Code: claude-agent-acp). Los modelos gratuitos son promociones sujetas a disponibilidad y condiciones del proveedor. En OpenCode usa /models para ver tu catálogo y el selector de permisos para elegir lectura o edición. No se cambia a un modelo de pago si falla el seleccionado.",
    ));
    note.set_wrap(true);
    note.set_max_width_chars(65);
    grid.attach(&note, 0, 9, 2, 1);
    let dictation = gtk::Button::with_label("Configurar dictado local…");
    grid.attach(&dictation, 0, 10, 2, 1);
    let dictation_state = Rc::downgrade(state);
    dictation.connect_clicked(move |_| {
        if let Some(state) = dictation_state.upgrade() {
            super::super::ai_dictation::configure(&state);
        }
    });
    page(&tabs, &grid, "General");

    let api = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(12)
        .build();
    margins(&api);
    let endpoint = entry(&api, 0, "URL base de API", &agent.endpoint);
    endpoint.set_placeholder_text(Some("https://api.openai.com/v1"));
    let key = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .placeholder_text("Vacío conserva la clave del mismo servidor")
        .build();
    api.attach(&gtk::Label::new(Some("Clave API")), 0, 1, 1, 1);
    api.attach(&key, 1, 1, 1, 1);
    let remember = gtk::CheckButton::with_label("Guardar clave en el llavero del sistema");
    remember.set_active(agent.secret_ref.is_some());
    api.attach(&remember, 0, 2, 2, 1);
    let clear = gtk::CheckButton::with_label("Eliminar clave guardada y temporal");
    api.attach(&clear, 0, 3, 2, 1);
    let tokens = gtk::SpinButton::with_range(1.0, 32768.0, 256.0);
    tokens.set_value(agent.max_output_tokens as f64);
    api.attach(
        &gtk::Label::new(Some("Máximo de tokens de salida")),
        0,
        4,
        1,
        1,
    );
    api.attach(&tokens, 1, 4, 1, 1);
    let timeout = gtk::SpinButton::with_range(5.0, 600.0, 5.0);
    timeout.set_value(agent.timeout_seconds as f64);
    api.attach(
        &gtk::Label::new(Some("Espera máxima (segundos)")),
        0,
        5,
        1,
        1,
    );
    api.attach(&timeout, 1, 5, 1, 1);
    api.attach(
        &gtk::Label::new(Some("Instrucciones del sistema")),
        0,
        6,
        2,
        1,
    );
    let (prompt_view, prompt) = editor(&agent.system_prompt, 120);
    api.attach(&prompt_view, 0, 7, 2, 1);
    let api_note = gtk::Label::new(Some(
        "La clave queda en memoria si no marcas el llavero; no se guarda en JSON. Cambiar la URL exige introducir la clave de nuevo. HTTP solo se admite en loopback. Las herramientas de terminal dependen de las capacidades del modelo; esta API no ejecuta extensiones MCP externas.",
    ));
    api_note.set_wrap(true);
    api_note.set_max_width_chars(65);
    api.attach(&api_note, 0, 8, 2, 1);
    let credential_env = gtk::DropDown::from_strings(CREDENTIAL_ENV);
    credential_env.set_selected(
        CREDENTIAL_ENV
            .iter()
            .position(|v| *v == agent.credential_env)
            .unwrap_or(0) as u32,
    );
    api.attach(
        &gtk::Label::new(Some("Credencial del agente nativo (opcional)")),
        0,
        9,
        2,
        1,
    );
    api.attach(&credential_env, 0, 10, 2, 1);
    page(&tabs, &api, "Conexión y credenciales");

    let extensions = gtk::Box::new(gtk::Orientation::Vertical, 8);
    margins(&extensions);
    let ext_note = gtk::Label::new(Some(
        "MCP, skills y subagentes exclusivos de este agente. Codex y OpenCode los aplican en su entorno privado. Otros agentes ACP aplican los MCP mediante su protocolo; la API de texto conserva la configuración, pero no ejecuta extensiones externas. Añade una plantilla y edita su JSON; guarda y abre una conversación nueva para aplicarla. Los MCP se añaden desactivados. Para secretos usa referencias {env:NOMBRE}, nunca claves literales. Las skills son carpetas con SKILL.md y no se instalan automáticamente.",
    ));
    ext_note.set_wrap(true);
    ext_note.set_max_width_chars(65);
    extensions.append(&ext_note);
    let snippets = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    extensions.append(&snippets);
    let (ext_view, ext_buffer) =
        editor(&serde_json::to_string_pretty(&agent.opencode).unwrap(), 240);
    extensions.append(&ext_view);
    ext_view.child().unwrap().set_widget_name("ai-extensions");
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.set_max_width_chars(75);
    let extra_extension_buttons = Rc::new(RefCell::new(Vec::new()));
    for (label, template) in [
        ("MCP local", 0),
        ("MCP remoto", 1),
        ("Skill", 2),
        ("Agente", 3),
        ("Subagente", 4),
    ] {
        let button = gtk::Button::with_label(label);
        if template >= 2 {
            extra_extension_buttons
                .borrow_mut()
                .push(button.downgrade());
        }
        snippets.append(&button);
        let buffer = ext_buffer.downgrade();
        let notice = error.downgrade();
        button.connect_clicked(move |_| {
            let (Some(buffer), Some(error)) = (buffer.upgrade(), notice.upgrade()) else {
                return;
            };
            let Ok(mut config) = serde_json::from_str::<OpenCodeConfig>(&text(&buffer)) else {
                error.set_text("Corrige el JSON antes de insertar una plantilla.");
                return;
            };
            match template {
                0 | 1 => {
                    let mut i = 1;
                    while config.mcp.contains_key(&format!("servidor-{i}")) {
                        i += 1;
                    }
                    let server = if template == 0 {
                        McpServer::Local {
                            command: vec!["/ruta/al/servidor-mcp".into()],
                            enabled: false,
                            environment: Default::default(),
                        }
                    } else {
                        McpServer::Remote {
                            url: "https://example.invalid/mcp".into(),
                            enabled: false,
                            oauth: None,
                            headers: Default::default(),
                        }
                    };
                    config.mcp.insert(format!("servidor-{i}"), server);
                }
                2 => config.skills.paths.push("/ruta/a/carpeta-de-skills".into()),
                _ => {
                    let mut i = 1;
                    while config.agent.contains_key(&format!("agente-{i}")) {
                        i += 1;
                    }
                    config.agent.insert(
                        format!("agente-{i}"),
                        AgentDefinition {
                            description: "Describe la especialidad del agente".into(),
                            mode: if template == 4 {
                                AgentMode::Subagent
                            } else {
                                AgentMode::Primary
                            },
                            prompt: "Escribe las instrucciones del agente".into(),
                            model: String::new(),
                        },
                    );
                }
            }
            buffer.set_text(&serde_json::to_string_pretty(&config).unwrap());
            error.set_text("");
        });
    }
    page(&tabs, &extensions, "MCP, skills y agentes");
    root.append(&error);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let new = gtk::Button::with_label("+ Añadir modelo");
    let delete = gtk::Button::with_label("Eliminar");
    delete.set_sensitive(old.is_some());
    let save = gtk::Button::with_label("Guardar");
    save.add_css_class("suggested-action");
    save.set_widget_name("ai-settings-save");
    buttons.append(&new);
    buttons.append(&delete);
    buttons.append(&save);
    root.append(&buttons);
    dialog.set_child(Some(&root));
    let target = Rc::new(RefCell::new(old));

    let fields: Vec<gtk::Widget> = [&program, &args, &directory]
        .map(|e| e.clone().upcast())
        .into();
    let model_weak = model.downgrade();
    let api_weak = api.downgrade();
    let endpoint_weak = endpoint.downgrade();
    let tokens_weak = tokens.downgrade();
    let timeout_weak = timeout.downgrade();
    let credential_weak = credential_env.downgrade();
    let api_grid = api.downgrade();
    let general_grid = grid.downgrade();
    let ext_weak = extensions.downgrade();
    let free_weak = free.downgrade();
    let list_weak = free_selector.downgrade();
    let fields: Vec<_> = fields.iter().map(|w| w.downgrade()).collect();
    let args_weak = args.downgrade();
    let credential_program = program.downgrade();
    let extra_buttons = extra_extension_buttons.clone();
    let update: Rc<dyn Fn(u32)> = Rc::new(move |selected| {
        let is_api = matches!(selected, 2 | 3);
        let native_skills = matches!(selected, 1 | 4)
            || credential_program.upgrade().is_some_and(|p| {
                std::path::Path::new(p.text().as_str())
                    .file_name()
                    .is_some_and(|n| n == "claude" || n == "claude-agent-acp" || n == "gemini")
            });
        for button in extra_buttons.borrow().iter() {
            if let Some(button) = button.upgrade() {
                button.set_visible(native_skills);
            }
        }
        for field in &fields {
            if let Some(field) = field.upgrade() {
                field.set_sensitive(!is_api);
            }
        }
        if let Some(args) = args_weak.upgrade() {
            args.set_sensitive(!matches!(selected, 2..=4));
        }
        if let Some(model) = model_weak.upgrade() {
            model.set_sensitive(selected != 0);
        }
        if let Some(api) = api_weak.upgrade() {
            api.set_sensitive(true);
        }
        for field in [&endpoint_weak] {
            if let Some(field) = field.upgrade() {
                field.set_sensitive(is_api);
            }
        }
        if let Some(tokens) = tokens_weak.upgrade() {
            tokens.set_sensitive(is_api);
        }
        if let Some(timeout) = timeout_weak.upgrade() {
            timeout.set_sensitive(is_api);
        }
        if let Some(credential) = credential_weak.upgrade() {
            credential.set_visible(!is_api);
            if let Some(program) = credential_program.upgrade() {
                credential_options(&credential, &program.text());
            }
        }
        if let Some(grid) = api_grid.upgrade() {
            for (row, visible) in [
                (0, is_api),
                (4, is_api),
                (5, is_api),
                (9, !is_api),
                (10, !is_api),
            ] {
                for col in 0..2 {
                    if let Some(widget) = grid.child_at(col, row) {
                        widget.set_visible(visible);
                    }
                }
            }
        }
        if let Some(grid) = general_grid.upgrade() {
            for (row, visible) in [
                (3, !is_api),
                (4, !is_api && selected != 4),
                (5, !is_api),
                (7, selected == 1),
                (8, selected == 1),
            ] {
                for col in 0..2 {
                    if let Some(widget) = grid.child_at(col, row) {
                        widget.set_visible(visible);
                    }
                }
            }
        }
        if let Some(ext) = ext_weak.upgrade() {
            ext.set_sensitive(!is_api);
            ext.set_visible(!is_api);
            if let Some(page) = ext.parent() {
                page.set_visible(!is_api);
            }
        }
        if let Some(free) = free_weak.upgrade() {
            free.set_sensitive(selected == 1);
        }
        if let Some(list) = list_weak.upgrade() {
            list.set_sensitive(selected == 1);
        }
    });
    update(engine(&kind, &allowed));
    let u = update.clone();
    let choices = allowed.clone();
    kind.connect_selected_notify(move |kind| u(engine(kind, &choices)));
    let cred = credential_env.downgrade();
    let program_update = update.clone();
    let program_kind = kind.downgrade();
    let program_choices = allowed.clone();
    program.connect_changed(move |program| {
        if let Some(cred) = cred.upgrade() {
            credential_options(&cred, &program.text());
        }
        if let Some(kind) = program_kind.upgrade() {
            program_update(engine(&kind, &program_choices));
        }
    });
    let n = name.downgrade();
    let p = program.downgrade();
    let a = args.downgrade();
    let k = kind.downgrade();
    let e = endpoint.downgrade();
    let m = model.downgrade();
    let preset_target = target.clone();
    let preset_delete = delete.downgrade();
    let preset_key = key.downgrade();
    let preset_extensions = ext_buffer.downgrade();
    let preset_models = model_list.downgrade();
    let preset_prompt = prompt.downgrade();
    let preset_save = save.downgrade();
    let preset_update = update.clone();
    let preset_allowed = allowed.clone();
    presets.connect_selected_notify(move |preset| {
        let (Some(n), Some(p), Some(a), Some(k), Some(e), Some(m)) = (
            n.upgrade(),
            p.upgrade(),
            a.upgrade(),
            k.upgrade(),
            e.upgrade(),
            m.upgrade(),
        ) else {
            return;
        };
        let (label, executable, engine, endpoint) = match preset.selected() {
            1 => ("Codex", "codex", 4, ""),
            2 => ("Claude Code", "claude-agent-acp", 5, ""),
            3 => ("OpenCode", "opencode", 1, ""),
            4 => ("OpenAI", "", 2, "https://api.openai.com/v1"),
            5 => ("API compatible", "", 3, ""),
            6 => ("Ollama", "", 3, "http://127.0.0.1:11434/v1"),
            7 => (
                "Gemini",
                "",
                3,
                "https://generativelanguage.googleapis.com/v1beta/openai",
            ),
            8 => ("DeepSeek", "", 3, "https://api.deepseek.com/v1"),
            9 => ("Agente ACP", "", 5, ""),
            10 => ("Gemini CLI", "gemini", 5, ""),
            11 => ("GitHub Copilot", "copilot", 5, ""),
            _ => return,
        };
        // A template creates a separate connection; it never overwrites the selected agent.
        *preset_target.borrow_mut() = None;
        if let Some(delete) = preset_delete.upgrade() {
            delete.set_sensitive(false);
        }
        if let Some(key) = preset_key.upgrade() {
            key.set_text("");
        }
        if let Some(extensions) = preset_extensions.upgrade() {
            extensions.set_text("{}");
        }
        if let Some(models) = preset_models.upgrade() {
            models.set_text("");
        }
        if let Some(prompt) = preset_prompt.upgrade() {
            prompt.set_text("");
        }
        if let Some(save) = preset_save.upgrade() {
            save.set_label("Añadir y abrir chat");
        }
        n.set_text(label);
        p.set_text(executable);
        a.set_text(if preset.selected() == 10 {
            r#"["--experimental-acp"]"#
        } else if preset.selected() == 11 {
            r#"["--acp"]"#
        } else {
            "[]"
        });
        set_engines(
            &k,
            &preset_allowed,
            if preset.selected() == 5 {
                vec![2, 3]
            } else {
                vec![engine]
            },
            engine,
        );
        preset_update(engine);
        e.set_text(endpoint);
        m.set_text("");
    });
    let updating_models = Rc::new(Cell::new(false));
    let updating = updating_models.clone();
    let m = model.downgrade();
    let names = free_names.clone();
    free_selector.connect_selected_notify(move |selector| {
        if updating.get() {
            return;
        }
        if let (Some(model), Some(value)) = (m.upgrade(), names.string(selector.selected())) {
            model.set_text(&value);
        }
    });
    let list_weak = free_selector.downgrade();
    let names = free_names;
    let notice = error.downgrade();
    let weak = dialog.downgrade();
    free.connect_clicked(move |button| {
        button.set_sensitive(false);
        let button = button.downgrade(); let notice = notice.clone(); let dialog = weak.clone(); let names = names.clone();
        let updating = updating_models.clone();
        let selector = list_weak.clone();
        let mut worker = nexuterm::runtime::runtime().spawn(nexuterm::ai::free_models());
        // Abort directly on dialog closure as well as in the UI polling loop.
        if let Some(window) = dialog.upgrade() {
            let abort = worker.abort_handle();
            window.connect_close_request(move |_| { abort.abort(); glib::Propagation::Proceed });
        }
        glib::spawn_future_local(async move {
            while !worker.is_finished() {
                if !dialog.upgrade().is_some_and(|d| d.is_visible()) { worker.abort(); let _ = worker.await; return; }
                glib::timeout_future(std::time::Duration::from_millis(50)).await;
            }
            if let Some(button) = button.upgrade() { button.set_sensitive(true); }
            if let Some(notice) = notice.upgrade() {
                match (&mut worker).await {
                    Ok(Ok(models)) => {
                        let labels: Vec<&str> = models.iter().map(String::as_str).collect();
                        updating.set(true);
                        names.splice(0, names.n_items(), &labels);
                        if let Some(selector) = selector.upgrade() { selector.set_selected(gtk::INVALID_LIST_POSITION); }
                        updating.set(false);
                        notice.set_text("Catálogo actualizado. La gratuidad y disponibilidad dependen de OpenCode; no se ha enviado ningún prompt.");
                    }
                    Ok(Err(error)) => notice.set_text(&error), Err(_) => notice.set_text("No se pudo consultar el catálogo de OpenCode"),
                }
            }
        });
    });
    let target_new = target.clone();
    let n = name.downgrade();
    let p = program.downgrade();
    let a = args.downgrade();
    let d = directory.downgrade();
    let m = model.downgrade();
    let e = endpoint.downgrade();
    let del = delete.downgrade();
    let k = key.downgrade();
    let ext = ext_buffer.downgrade();
    let prompt_new = prompt.downgrade();
    let clear_new = clear.downgrade();
    let remember_new = remember.downgrade();
    let models_new = model_list.downgrade();
    let new_kind = kind.downgrade();
    let new_choices = allowed.clone();
    let new_presets = presets.downgrade();
    let new_update = update.clone();
    new.connect_clicked(move |_| {
        if let Some(presets) = new_presets.upgrade() {
            presets.set_selected(0);
        }
        if let Some(kind) = new_kind.upgrade() {
            set_engines(&kind, &new_choices, (0..6).collect(), 0);
        }
        if let Some(models) = models_new.upgrade() {
            models.set_text("");
        }
        new_update(0);
        *target_new.borrow_mut() = None;
        for entry in [&n, &p, &d, &m, &e] {
            if let Some(entry) = entry.upgrade() {
                entry.set_text("");
            }
        }
        if let Some(a) = a.upgrade() {
            a.set_text("[]");
        }
        if let Some(del) = del.upgrade() {
            del.set_sensitive(false);
        }
        if let Some(key) = k.upgrade() {
            key.set_text("");
        }
        if let Some(ext) = ext.upgrade() {
            ext.set_text(&serde_json::to_string_pretty(&OpenCodeConfig::default()).unwrap());
        }
        if let Some(prompt) = prompt_new.upgrade() {
            prompt.set_text("");
        }
        if let Some(clear) = clear_new.upgrade() {
            clear.set_active(false);
        }
        if let Some(remember) = remember_new.upgrade() {
            remember.set_active(false);
        }
    });
    let capability_generation = Rc::new(Cell::new(0u64));
    let capability_refresh: Rc<dyn Fn()> = {
        let generation = capability_generation.clone();
        let model = model.downgrade();
        let endpoint = endpoint.downgrade();
        let terminal = terminal_access.downgrade();
        let dialog = dialog.downgrade();
        Rc::new(move || {
            generation.set(generation.get() + 1);
            let epoch = generation.get();
            let (Some(model), Some(endpoint), Some(terminal)) =
                (model.upgrade(), endpoint.upgrade(), terminal.upgrade())
            else {
                return;
            };
            terminal.set_sensitive(true);
            terminal.set_tooltip_text(Some(
                "La disponibilidad depende de las herramientas admitidas por el modelo.",
            ));
            let model = model.text().to_string();
            let endpoint = endpoint.text().to_string();
            let terminal = terminal.downgrade();
            let dialog = dialog.clone();
            let generation = generation.clone();
            glib::spawn_future_local(async move {
                glib::timeout_future(std::time::Duration::from_millis(350)).await;
                if generation.get() != epoch || !dialog.upgrade().is_some_and(|w| w.is_visible()) {
                    return;
                }
                let mut worker = nexuterm::runtime::runtime()
                    .spawn(nexuterm::ai::local_model_tools(endpoint, model));
                while !worker.is_finished() {
                    if generation.get() != epoch
                        || !dialog.upgrade().is_some_and(|w| w.is_visible())
                    {
                        worker.abort();
                        return;
                    }
                    glib::timeout_future(std::time::Duration::from_millis(50)).await;
                }
                if generation.get() != epoch {
                    return;
                }
                if let (Ok(Ok(Some(supported))), Some(terminal)) =
                    ((&mut worker).await, terminal.upgrade())
                {
                    terminal.set_sensitive(supported);
                    if !supported {
                        terminal.set_active(false);
                        terminal.set_tooltip_text(Some(
                            "Este modelo no anuncia soporte de herramientas en Ollama.",
                        ));
                    }
                }
            });
        })
    };
    let r = capability_refresh.clone();
    model.connect_changed(move |_| r());
    let r = capability_refresh.clone();
    endpoint.connect_changed(move |_| r());
    capability_refresh();
    let s = Rc::downgrade(state);
    let weak = dialog.downgrade();
    #[cfg(debug_assertions)]
    let capability_fields = (
        kind.downgrade(),
        program.downgrade(),
        extensions.downgrade(),
        credential_env.downgrade(),
        endpoint.downgrade(),
        tokens.downgrade(),
    );
    let commit: Rc<dyn Fn(bool)> = Rc::new(move |remove| {
        let (Some(state), Some(dialog)) = (s.upgrade(), weak.upgrade()) else {
            return;
        };
        let old = target.borrow().clone();
        if remove {
            let Some(old) = old else {
                return;
            };
            let mut config = state.config.borrow().clone();
            config.ai_agents.retain(|a| a.id != old.id);
            if let Err(e) = config::save(&state.path, &config) {
                error.set_text(&e);
                return;
            }
            *state.config.borrow_mut() = config;
            state
                .ai_keys
                .borrow_mut()
                .retain(|(id, _), _| id != &old.id);
            refresh();
            dialog.close();
            glib::spawn_future_local(async move {
                if let Err(e) = super::super::ai_credentials::forget(&old).await {
                    state.status.set_text(&e);
                }
            });
            return;
        }
        let engine = match engine(&kind, &allowed) {
            1 => AiKind::OpenCode,
            2 => AiKind::Responses,
            3 => AiKind::ChatCompletions,
            4 => AiKind::Codex,
            5 => AiKind::Acp,
            _ => AiKind::Local,
        };
        let args = if engine.is_api() || engine == AiKind::Codex {
            Vec::new()
        } else {
            let Ok(args) = serde_json::from_str::<Vec<String>>(&args.text()) else {
                error.set_text("Argumentos inválidos: utiliza una lista JSON de textos.");
                return;
            };
            args
        };
        let opencode = match serde_json::from_str::<OpenCodeConfig>(&text(&ext_buffer)) {
            Ok(config) => config,
            Err(e) => {
                error.set_text(&format!("Extensiones inválidas: {e}"));
                return;
            }
        };
        let agent = AiAgent {
            id: old
                .as_ref()
                .map(|a| a.id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: name.text().trim().into(),
            kind: engine,
            program: program.text().trim().into(),
            args,
            directory: directory.text().trim().into(),
            endpoint: endpoint.text().trim().trim_end_matches('/').into(),
            model: model.text().trim().into(),
            terminal_access: terminal_access.is_active(),
            terminal_socket: None,
            runtime_root: None,
            credential_env: credential_env
                .selected_item()
                .and_downcast::<gtk::StringObject>()
                .map(|s| s.string().to_string())
                .unwrap_or_default(),
            wait_credentials: false,
            models: text(&model_list)
                .lines()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            system_prompt: text(&prompt),
            max_output_tokens: tokens.value_as_int() as u32,
            timeout_seconds: timeout.value_as_int() as u32,
            secret_ref: None,
            opencode,
        };
        if let Err(e) = agent.validate() {
            error.set_text(&e);
            return;
        }
        let password = zeroize::Zeroizing::new(key.text().to_string());
        let remember = remember.is_active();
        let clear = clear.is_active();
        let error = error.downgrade();
        let key = key.downgrade();
        let refresh = refresh.clone();
        dialog.set_sensitive(false);
        glib::spawn_future_local(async move {
            let result =
                super::super::ai_credentials::save(&state, agent, old, password, remember, clear)
                    .await;
            dialog.set_sensitive(true);
            match result {
                Ok(()) => {
                    if let Some(key) = key.upgrade() {
                        key.set_text("");
                    }
                    refresh();
                    dialog.close();
                }
                Err(e) => {
                    if let Some(error) = error.upgrade() {
                        error.set_text(&e);
                    }
                }
            }
        });
    });
    let save_commit = commit.clone();
    save.connect_clicked(move |_| save_commit(false));
    let weak = dialog.downgrade();
    delete.connect_clicked(move |_| {
        let weak = weak.clone();
        let commit = commit.clone();
        glib::spawn_future_local(async move {
            if let Some(dialog) = weak.upgrade() {
                let confirm = gtk::AlertDialog::builder()
                    .message("¿Eliminar el agente configurado?")
                    .buttons(["Cancelar", "Eliminar"])
                    .cancel_button(0)
                    .default_button(0)
                    .build();
                if confirm.choose_future(Some(&dialog)).await == Ok(1) {
                    commit(true);
                }
            }
        });
    });
    dialog.present();
    #[cfg(debug_assertions)]
    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("settings-api") {
        let (kind, program, ext, credential, endpoint, tokens) = &capability_fields;
        assert_eq!(kind.upgrade().unwrap().model().unwrap().n_items(), 1);
        assert!(!gtk::prelude::WidgetExt::is_visible(
            &program.upgrade().unwrap()
        ));
        assert!(!ext.upgrade().unwrap().is_visible());
        assert!(!credential.upgrade().unwrap().is_visible());
        assert!(gtk::prelude::WidgetExt::is_visible(
            &endpoint.upgrade().unwrap()
        ));
        assert!(tokens.upgrade().unwrap().is_visible());
        save.emit_clicked();
        eprintln!("NEXUTERM_SMOKE_CAPABILITIES_OK");
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && matches!(
            std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
            Ok("settings" | "templates")
        )
    {
        if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("templates") {
            presets.set_selected(3);
        }
        smoke(&dialog);
    }
}

#[cfg(debug_assertions)]
fn smoke(dialog: &gtk::Window) {
    fn find(widget: &gtk::Widget, predicate: &dyn Fn(&gtk::Widget) -> bool) -> Option<gtk::Widget> {
        if predicate(widget) {
            return Some(widget.clone());
        }
        let mut child = widget.first_child();
        while let Some(current) = child {
            if let Some(found) = find(&current, predicate) {
                return Some(found);
            }
            child = current.next_sibling();
        }
        None
    }
    let weak = dialog.downgrade();
    glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
        let dialog = weak.upgrade().unwrap();
        let name = find(dialog.upcast_ref(), &|w| {
            w.widget_name() == "ai-settings-name"
        })
        .unwrap()
        .downcast::<gtk::Entry>()
        .unwrap();
        name.set_text("OpenCode comprobado");
        for label in ["MCP local", "Skill", "Agente", "Subagente"] {
            let button = find(dialog.upcast_ref(), &|w| {
                w.downcast_ref::<gtk::Button>()
                    .is_some_and(|b| b.label().as_deref() == Some(label))
            })
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
            button.emit_clicked();
        }
        let notebook = find(dialog.upcast_ref(), &|w| w.is::<gtk::Notebook>())
            .unwrap()
            .downcast::<gtk::Notebook>()
            .unwrap();
        notebook.set_current_page(Some(2));
        let weak = dialog.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
            let dialog = weak.upgrade().unwrap();
            if let Some(path) = std::env::var_os("NEXUTERM_AI_SETTINGS_SCREENSHOT") {
                let paintable = gtk::WidgetPaintable::new(Some(&dialog));
                let snapshot = gtk::Snapshot::new();
                paintable.snapshot(&snapshot, dialog.width() as f64, dialog.height() as f64);
                if let (Some(node), Some(renderer)) = (snapshot.to_node(), dialog.renderer()) {
                    renderer
                        .render_texture(&node, None)
                        .save_to_png(path)
                        .unwrap();
                }
            }
            find(dialog.upcast_ref(), &|w| {
                w.widget_name() == "ai-settings-save"
            })
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        });
    });
}

/// Overview of configured connections; editing stays in the existing profile form.
pub(super) fn manage(
    state: &Rc<State>,
    window: &gtk::ApplicationWindow,
    selected: usize,
    refresh: Rc<dyn Fn()>,
) {
    let dialog = gtk::Window::builder()
        .title("Modelos configurados")
        .transient_for(window)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(560)
        .default_height(560)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    margins(&content);
    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let title = gtk::Label::new(Some("Modelos configurados"));
    title.add_css_class("title-3");
    title.set_hexpand(true);
    title.set_xalign(0.0);
    heading.append(&title);
    let add = gtk::Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Añadir modelo")
        .build();
    add.set_widget_name("ai-add-model");
    add.update_property(&[gtk::accessible::Property::Label("Añadir modelo")]);
    heading.append(&add);
    content.append(&heading);
    let groups = gtk::Box::new(gtk::Orientation::Vertical, 12);
    for (local, label) in [(true, "Locales"), (false, "Web")] {
        let title = gtk::Label::new(Some(label));
        title.set_xalign(0.0);
        title.add_css_class("heading");
        groups.append(&title);
        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::None);
        list.add_css_class("boxed-list");
        let config = state.config.borrow();
        let mut count = 0;
        for (index, agent) in config
            .ai_agents
            .iter()
            .enumerate()
            .filter(|(_, a)| is_local(a) == local)
        {
            count += 1;
            let row = adw::ActionRow::builder()
                .title(&agent.name)
                .subtitle(if agent.model.is_empty() {
                    "Modelo predeterminado"
                } else {
                    &agent.model
                })
                .activatable(true)
                .build();
            if index == selected {
                row.add_css_class("accent");
            }
            row.add_suffix(&gtk::Image::from_icon_name("go-next-symbolic"));
            let s = Rc::downgrade(state);
            let w = window.downgrade();
            let d = dialog.downgrade();
            let r = refresh.clone();
            row.connect_activated(move |_| {
                if let (Some(s), Some(w), Some(d)) = (s.upgrade(), w.upgrade(), d.upgrade()) {
                    d.close();
                    configure(&s, &w, index, r.clone());
                }
            });
            list.append(&row);
        }
        if count == 0 {
            let empty = gtk::Label::new(Some("No hay modelos configurados"));
            margins(&empty);
            list.append(&empty);
        }
        groups.append(&list);
    }
    content.append(
        &gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&groups)
            .build(),
    );
    let s = Rc::downgrade(state);
    let w = window.downgrade();
    let d = dialog.downgrade();
    add.connect_clicked(move |_| {
        if let (Some(s), Some(w), Some(d)) = (s.upgrade(), w.upgrade(), d.upgrade()) {
            d.close();
            configure(&s, &w, usize::MAX, refresh.clone());
        }
    });
    dialog.set_child(Some(&content));
    dialog.present();
    #[cfg(debug_assertions)]
    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("groups") {
        assert!(state.config.borrow().ai_agents.iter().any(is_local));
        assert!(state.config.borrow().ai_agents.iter().any(|a| !is_local(a)));
        let add = add.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
            let add = add.upgrade().unwrap();
            assert_eq!(add.tooltip_text().as_deref(), Some("Añadir modelo"));
            add.emit_clicked();
            eprintln!("NEXUTERM_SMOKE_MODEL_GROUPS_OK");
        });
    }
}

#[cfg(test)]
mod capability_tests {
    use super::*;
    #[test]
    fn engine_choices_match_the_configured_connector() {
        for (kind, expected) in [
            (AiKind::Codex, vec![4]),
            (AiKind::Acp, vec![5]),
            (AiKind::OpenCode, vec![1]),
            (AiKind::ChatCompletions, vec![3]),
        ] {
            assert_eq!(
                permitted_engines(&AiAgent {
                    kind,
                    ..Default::default()
                }),
                expected
            );
        }
        assert_eq!(
            permitted_engines(&AiAgent {
                kind: AiKind::Responses,
                endpoint: "https://api.openai.com/v1".into(),
                ..Default::default()
            }),
            vec![2]
        );
        assert_eq!(
            permitted_engines(&AiAgent {
                kind: AiKind::ChatCompletions,
                endpoint: "http://127.0.0.1:11434/v1".into(),
                ..Default::default()
            }),
            vec![3]
        );
    }
}
