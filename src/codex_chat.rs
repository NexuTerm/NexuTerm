#![allow(deprecated)] // Keep compatibility with the existing native chooser API.
//! Graphical Codex client. Codex owns inference, tools, authentication and history.
use super::*;
use nexuterm::{
    codex::{Client, Event},
    config::AiAgent,
};
use serde_json::{Value, json};
use std::collections::HashMap;

type Dialogs = Rc<RefCell<HashMap<String, (glib::WeakRef<gtk::Window>, Rc<Cell<bool>>)>>>;
#[derive(Clone)]
pub(super) struct Chat {
    pub widget: gtk::Box,
    pub archive: Rc<nexuterm::chat_history::Recorder>,
    pub snapshot: Rc<dyn Fn() -> nexuterm::workspaces::Chat>,
    pub restore: Rc<dyn Fn(&nexuterm::workspaces::Chat)>,
    pub open: Rc<dyn Fn(AiAgent, bool)>,
    pub close: Rc<dyn Fn()>,
    pub show_recent: Rc<dyn Fn()>,
    pub select_recent: Rc<RefCell<Option<Rc<dyn Fn(String)>>>>,
    pub activate: Rc<dyn Fn()>,
    pub models: Rc<RefCell<Vec<Value>>>,
    pub login: gtk::Button,
}
#[derive(Clone)]
struct UsageView {
    account: gtk::Label,
    account_row: gtk::Box,
    details_row: gtk::Box,
    quota: gtk::Label,
    context: gtk::Label,
    tokens: gtk::Label,
    snapshot: Rc<RefCell<Value>>,
}
impl UsageView {
    fn new(parent: &gtk::Box) -> Self {
        let view = Self {
            account_row: gtk::Box::new(gtk::Orientation::Horizontal, 6),
            details_row: gtk::Box::new(gtk::Orientation::Horizontal, 6),
            account: gtk::Label::new(Some("Usuario: no disponible")),
            quota: gtk::Label::new(Some("Cuota restante: no disponible")),
            context: gtk::Label::new(None),
            tokens: gtk::Label::new(None),
            snapshot: Rc::new(RefCell::new(Value::Null)),
        };
        view.account_row.set_widget_name("ai-footer-account");
        view.details_row.set_widget_name("ai-footer-details");
        view.account.set_widget_name("ai-account");
        view.quota.set_widget_name("ai-quota");
        view.context.set_widget_name("ai-context");
        view.tokens.set_widget_name("ai-tokens");
        view.account.set_hexpand(true);
        view.quota.set_max_width_chars(30);
        for label in [&view.account, &view.quota, &view.context, &view.tokens] {
            label.set_xalign(0.0);
            label.set_wrap(false);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_selectable(true);
            label.add_css_class("caption");
            label.add_css_class("dim-label");
        }
        view.account.connect_label_notify(|label|label.set_tooltip_text(Some(&label.text())));
        view.account.set_tooltip_text(Some(&view.account.text()));
        view.account_row.append(&view.account);
        view.account_row.append(&view.quota);
        view.details_row.append(&view.context);
        view.details_row.append(&view.tokens);
        parent.append(&view.account_row);
        parent.append(&view.details_row);
        view.context.set_visible(false);
        view.tokens.set_visible(false);
        view
    }
    fn reset(&self) {
        self.account.set_text("Usuario: no disponible"); self.context.set_visible(false); self.tokens.set_visible(false);
        *self.snapshot.borrow_mut() = Value::Null; self.render("");
    }
    fn render(&self, model: &str) {
        let windows = nexuterm::ai_usage::windows(&self.snapshot.borrow(), model);
        if windows.is_empty() {
            self.quota.set_text("Cuota restante: no disponible");
            self.quota.set_tooltip_text(Some("El proveedor no ha comunicado su cuota restante. No se estima a partir de los tokens usados."));
        } else {
            self.quota.set_text(&windows.iter().map(|w| format!("{}: {:.0}%", w.label, w.remaining)).collect::<Vec<_>>().join(" · "));
            let mut detail = format!("{}\nCuota restante de la cuenta comunicada por el proveedor; puede estar compartida entre modelos.",self.quota.text());
            for w in windows { if let Some(date) = w.resets_at.and_then(|time| glib::DateTime::from_unix_local(time).ok()).and_then(|date| date.format("%d/%m %H:%M").ok()) { detail.push_str(&format!("\n{} se restablece: {date}", w.label)); } }
            self.quota.set_tooltip_text(Some(&detail));
        }
    }
}

fn attachment_chip(row: &gtk::Box, paths: &Rc<RefCell<Vec<PathBuf>>>, path: PathBuf) {
    row.set_visible(true);
    let chip = gtk::Button::with_label(&format!(
        "{}  ×",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    chip.set_tooltip_text(Some(&path.to_string_lossy()));
    chip.add_css_class("flat");
    let paths = paths.clone();
    let weak = row.downgrade();
    chip.connect_clicked(move |button| {
        paths.borrow_mut().retain(|p| p != &path);
        if let Some(row) = weak.upgrade() {
            row.remove(button);
            row.set_visible(!paths.borrow().is_empty());
        }
    });
    row.append(&chip);
}
fn selected_reasoning(models: &[Value], model: usize, effort: u32) -> String {
    if effort == 0 {
        return String::new();
    }
    models
        .get(model)
        .and_then(|v| v["supportedReasoningEfforts"].as_array())
        .and_then(|v| v.get(effort as usize - 1))
        .and_then(|v| v["reasoningEffort"].as_str())
        .unwrap_or("")
        .to_owned()
}
#[derive(Clone)]
struct MessageLabel {
    label: gtk::Label,
    id: String,
    archive: Option<Rc<nexuterm::chat_history::Recorder>>,
}
impl std::ops::Deref for MessageLabel {
    type Target = gtk::Label;
    fn deref(&self) -> &gtk::Label {
        &self.label
    }
}
impl MessageLabel {
    fn set_text(&self, text: &str) {
        if let Some(a) = &self.archive {
            a.record(nexuterm::chat_history::Record::Replace {
                id: self.id.clone(),
                text: text.into(),
            });
        }
        self.label
            .set_text(&text.chars().take(65536).collect::<String>());
    }
}
fn message(
    list: &gtk::Box,
    archive: Option<&Rc<nexuterm::chat_history::Recorder>>,
    title: &str,
    content: &str,
    tool: bool,
    copy_response: bool,
) -> MessageLabel {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("card");
    if tool {
        card.add_css_class("workspace-tool");
    }
    if copy_response {
        card.add_css_class("workspace-response");
    }
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("heading");
    heading.set_widget_name("workspace-message-title");
    heading.set_xalign(0.0);
    heading.set_margin_top(10);
    heading.set_margin_start(12);
    let text = gtk::Label::new(Some(&content.chars().take(65536).collect::<String>()));
    text.set_widget_name("workspace-message-text");
    text.set_wrap(true);
    text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    text.set_selectable(true);
    crate::context_menu::message(&text);
    text.set_xalign(0.0);
    text.set_yalign(0.0);
    text.set_margin_start(12);
    text.set_margin_end(12);
    text.set_margin_bottom(12);
    if tool {
        text.add_css_class("monospace");
    }
    if copy_response {
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        heading.set_hexpand(true);
        header.append(&heading);
        let copy = icon_button("edit-copy-symbolic", "Copiar respuesta · Markdown original");
        copy.set_widget_name("ai-copy-response");
        copy.add_css_class("ai-copy-response");
        copy.set_valign(gtk::Align::Center);
        copy.set_margin_end(6);
        let source = text.downgrade();
        copy.connect_clicked(move |button| {
            if let Some(source) = source.upgrade() {
                button.clipboard().set_text(&source.text());
            }
        });
        header.append(&copy);
        card.append(&header);
    } else {
        card.append(&heading);
    }
    card.append(&text);
    list.append(&card);
    while list.observe_children().n_items() > 128 {
        if let Some(child) = list.first_child() {
            list.remove(&child);
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    if let Some(a) = archive {
        a.record(nexuterm::chat_history::Record::Message {
            id: id.clone(),
            title: title.into(),
            text: content.into(),
            tool,
            response: copy_response,
        });
    }
    MessageLabel {
        label: text,
        id,
        archive: archive.cloned(),
    }
}
fn append(label: &MessageLabel, text: &str) {
    if let Some(a) = &label.archive {
        a.record(nexuterm::chat_history::Record::Append {
            id: label.id.clone(),
            text: text.into(),
        });
    }
    let current = label.text();
    let remaining = 65536_usize.saturating_sub(current.chars().count());
    if remaining > 0 {
        label.label.set_text(&format!(
            "{current}{}",
            text.chars().take(remaining).collect::<String>()
        ));
    }
}
fn effort_label(value: &str) -> &str {
    match value {
        "minimal" => "Mínimo",
        "low" => "Ligero",
        "medium" => "Medio",
        "high" => "Alto",
        "xhigh" => "Máximo",
        _ => value,
    }
}
fn file_changes(item: &Value) -> String {
    item["changes"]
        .as_array()
        .map(|changes| {
            changes
                .iter()
                .map(|change| {
                    format!(
                        "{}\n{}",
                        change["path"].as_str().unwrap_or("Archivo"),
                        change["diff"].as_str().unwrap_or("(Sin diff disponible)")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .unwrap_or_default()
}
fn tool_text(item: &Value) -> String {
    match item["type"].as_str().unwrap_or("") {
        "commandExecution" => format!(
            "{}\n\nResultado:\n{}",
            item["command"].as_str().unwrap_or(""),
            item["aggregatedOutput"].as_str().unwrap_or("En curso…")
        ),
        "fileChange" => file_changes(item),
        "webSearch" => item["query"]
            .as_str()
            .unwrap_or("Búsqueda web en curso…")
            .to_owned(),
        "mcpToolCall" => format!(
            "Solicitud:\n{}\n\nResultado:\n{}",
            readable_input(&item["arguments"]),
            readable_output(&item["result"])
        ),
        "agentToolCall" => format!(
            "Solicitud:\n{}\n\nResultado:\n{}",
            readable_input(&item["input"]),
            item["output"].as_str().unwrap_or("En curso…")
        ),
        "collabAgentToolCall" => format!(
            "Estado: {}",
            item["status"].as_str().unwrap_or("En curso")
        ),
        _ => String::new(),
    }
}
fn readable_input(value: &Value) -> String {
    let Some(fields) = value.as_object() else {
        return readable_output(value);
    };
    fields
        .iter()
        .map(|(key, value)| {
            let label = match key.as_str() {
                "command" => "Comando",
                "path" | "file_path" => "Archivo",
                "paths" => "Archivos",
                "query" => "Consulta",
                "pattern" => "Patrón",
                "url" => "Dirección",
                "text" => "Texto",
                _ => key,
            };
            format!("{label}: {}", readable_output(value))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn readable_output(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        _ => serde_json::to_string_pretty(value).unwrap_or_default(),
    }
}
fn tool_title(item: &Value) -> String {
    match item["type"].as_str().unwrap_or("") {
        "commandExecution" => "Ejecución de comando".into(),
        "fileChange" => "Cambios en archivos".into(),
        "mcpToolCall" => format!(
            "{} · {}",
            item["server"].as_str().filter(|s| !s.is_empty()).unwrap_or("Integración"),
            item["tool"].as_str().filter(|s| !s.is_empty()).unwrap_or("Herramienta")
        ),
        "agentToolCall" => {
            let title = item["title"]
                .as_str()
                .unwrap_or("")
                .trim()
                .trim_start_matches('$')
                .trim();
            if !title.is_empty()
                && !matches!(title.to_lowercase().as_str(), "herramienta" | "comando")
            {
                return title.to_owned();
            }
            match item["kind"].as_str().unwrap_or("other") {
                "read" => "Lectura de archivos".into(),
                "edit" => "Edición de archivos".into(),
                "delete" => "Eliminación de archivos".into(),
                "move" => "Movimiento de archivos".into(),
                "search" => "Búsqueda".into(),
                "think" => "Análisis".into(),
                "fetch" => "Consulta web".into(),
                "execute" => "Ejecución de comando".into(),
                _ => "Actividad del agente".into(),
            }
        }
        "collabAgentToolCall" => item["tool"]
            .as_str()
            .map(|s| s.trim().trim_start_matches('$').trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("Trabajo delegado")
            .to_owned(),
        _ => "Búsqueda web".into(),
    }
}
#[cfg(test)]
mod presentation_tests {
    use super::*;

    #[test]
    fn tool_activity_uses_human_labels_without_shell_prompt_artifacts() {
        let command = json!({
            "type": "commandExecution",
            "command": "cargo test",
            "aggregatedOutput": "Todas las pruebas pasaron"
        });
        assert_eq!(tool_title(&command), "Ejecución de comando");
        let command_text = tool_text(&command);
        assert!(command_text.starts_with("cargo test"));
        assert!(!command_text.starts_with("$"));
        assert!(command_text.contains("Resultado:\nTodas las pruebas pasaron"));

        let search = json!({
            "type": "agentToolCall",
            "title": "$ Herramienta",
            "kind": "search",
            "input": {"query": "GTK"},
            "output": "Se encontraron resultados"
        });
        assert_eq!(tool_title(&search), "Búsqueda");
        assert_eq!(
            tool_text(&search),
            "Solicitud:\nConsulta: GTK\n\nResultado:\nSe encontraron resultados"
        );
    }
}
fn close_dialogs(dialogs: &Dialogs) {
    let pending = dialogs.borrow_mut().drain().map(|(_, value)| value).collect::<Vec<_>>();
    for (window, done) in pending {
        done.set(true);
        if let Some(window) = window.upgrade() {
            window.close();
        }
    }
}
fn request_dialog(
    parent: &gtk::ApplicationWindow,
    client: Rc<Client>,
    id: Value,
    method: &str,
    mut params: Value,
    item: Option<&Value>,
    dialogs: &Dialogs,
    workspace: &str,
    grants: &Rc<RefCell<nexuterm::approval_grants::Grants>>,
) {
    let approval = matches!(
        method,
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
    );
    let tool_approval = method == "item/tool/requestApproval";
    let question = method == "item/tool/requestUserInput";
    if !approval && !tool_approval && !question {
        client.reject_unknown(id);
        return;
    }
    if method == "item/fileChange/requestApproval" && !params["cwd"].is_string() {
        params["cwd"] = json!(workspace);
    }
    let mut detail = json!({"request":params});
    if let Some(item) = item {
        detail["item"] = item.clone();
    }
    let preview = serde_json::to_string_pretty(&detail).unwrap_or_default();
    let missing_command = method == "item/commandExecution/requestApproval"
        && params["command"]
            .as_str()
            .or_else(|| item.and_then(|i| i["command"].as_str()))
            .is_none_or(str::is_empty);
    let missing_changes = method == "item/fileChange/requestApproval"
        && item
            .and_then(|i| i["changes"].as_array())
            .is_none_or(|changes| {
                changes.is_empty() || changes.iter().any(|c| !c["diff"].is_string())
            });
    if missing_changes
        || missing_command
        || preview.len() > 65536
        || (method == "item/fileChange/requestApproval" && item.is_none())
    {
        client.reply(id, json!({"decision":"decline"}));
        return;
    }
    if (approval || tool_approval) && client.full_access() {
        client.reply(id, json!({"decision":"accept"}));
        return;
    }
    let key = approval.then(|| nexuterm::approval_grants::operation(method, &params, item)).flatten();
    if key.as_ref().is_some_and(|key| grants.borrow().allows(key)) {
        client.reply(id, json!({"decision":"accept"}));
        return;
    }
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(question)
        .destroy_with_parent(true)
        .title(if question {
            "Codex necesita tu respuesta"
        } else if tool_approval {
            "Aprobación de herramienta"
        } else {
            "Aprobación de Codex"
        })
        .default_width(640)
        .default_height(440)
        .build();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.set_margin_top(16);
    root.set_margin_bottom(16);
    root.set_margin_start(16);
    root.set_margin_end(16);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let mut fields = Vec::new();
    if question {
        let questions = params["questions"].as_array().cloned().unwrap_or_default();
        if questions.len() > 16 {
            client.reject_unknown(id);
            return;
        }
        for q in questions {
            let mut label = q["question"].as_str().unwrap_or("Pregunta").to_owned();
            if let Some(options) = q["options"].as_array() {
                for option in options {
                    label.push_str(&format!(
                        "\n• {}: {}",
                        option["label"].as_str().unwrap_or(""),
                        option["description"].as_str().unwrap_or("")
                    ));
                }
            }
            let l = gtk::Label::new(Some(&label));
            l.set_wrap(true);
            l.set_xalign(0.0);
            body.append(&l);
            let entry = gtk::Entry::new();
            entry.set_visibility(q["isSecret"] != true);
            body.append(&entry);
            fields.push((q["id"].as_str().unwrap_or("").to_owned(), entry));
        }
    } else {
        let label = gtk::Label::new(Some(
            "Revisa la operación completa. Los permisos recordados se aplican solo a esta misma operación, carpeta y agente.",
        ));
        label.set_wrap(true);
        body.append(&label);
        let view = gtk::TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .monospace(true)
            .wrap_mode(gtk::WrapMode::WordChar)
            .build();
        let operation = if method == "item/commandExecution/requestApproval" {
            format!(
                "Comando\n{}\n\nCarpeta\n{}\n\n{}",
                params["command"]
                    .as_str()
                    .or_else(|| item.and_then(|i| i["command"].as_str()))
                    .unwrap_or(""),
                params["cwd"]
                    .as_str()
                    .or_else(|| item.and_then(|i| i["cwd"].as_str()))
                    .unwrap_or("No especificada"),
                params["reason"].as_str().unwrap_or("")
            )
        } else if tool_approval {
            format!(
                "Herramienta solicitada\n{}\n\nCarpeta: {}\n{}",
                serde_json::to_string_pretty(&params["toolCall"]).unwrap_or_default(),
                params["cwd"].as_str().unwrap_or("No especificada"),
                params["reason"].as_str().unwrap_or("")
            )
        } else {
            format!(
                "Cambios solicitados\n{}\n\nCarpeta: {}\nRaíz de escritura solicitada: {}\n{}",
                item.map(file_changes).unwrap_or_default(),
                workspace,
                params["grantRoot"].as_str().unwrap_or("sin ampliación"),
                params["reason"].as_str().unwrap_or("")
            )
        };
        let operation = if !params["networkApprovalContext"].is_null() {
            format!(
                "{operation}\n\nAcceso de red solicitado\n{}",
                serde_json::to_string_pretty(&params["networkApprovalContext"]).unwrap_or_default()
            )
        } else {
            operation
        };
        view.buffer().set_text(&operation);
        body.append(&view);
    }
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&body)
        .build();
    root.append(&scroll);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    buttons.add_css_class("ai-approval-actions");
    buttons.set_halign(gtk::Align::End);
    let reject = gtk::Button::with_label("Rechazar");
    let accept = gtk::Button::with_label(if question {
        "Responder"
    } else {
        "Permitir una vez"
    });
    accept.add_css_class("suggested-action");
    buttons.append(&reject);
    buttons.append(&accept);
    root.append(&buttons);
    let grant_error = gtk::Label::new(None);
    grant_error.set_wrap(true);
    grant_error.set_visible(false);
    grant_error.connect_label_notify(|label|label.set_visible(!label.text().is_empty()));
    root.append(&grant_error);
    window.set_child(Some(&root));
    let done = Rc::new(Cell::new(false));
    dialogs
        .borrow_mut()
        .insert(id.to_string(), (window.downgrade(), done.clone()));
    let weak = window.downgrade();
    let finished = done.clone();
    let c = client.clone();
    let request = id.clone();
    accept.connect_clicked(move |_| {
        if finished.get() {
            return;
        }
        let result = if question {
            let mut answers = serde_json::Map::new();
            for (key, entry) in &fields {
                if entry.text().trim().is_empty() {
                    entry.grab_focus();
                    return;
                }
                answers.insert(key.clone(), json!({"answers":[entry.text().as_str()]}));
            }
            json!({"answers":answers})
        } else {
            json!({"decision":"accept"})
        };
        finished.set(true);
        c.reply(request.clone(), result);
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    if approval {
        for (label, always) in [("Permitir en esta sesión", false), ("Permitir siempre", true)] {
            let button = gtk::Button::with_label(label);
            button.set_sensitive(key.is_some());
            button.set_tooltip_text(Some("Solo esta operación exacta, en esta carpeta y para este agente"));
            buttons.append(&button);
            let done = done.clone(); let client = client.clone(); let id = id.clone();
            let weak = window.downgrade(); let grants = grants.clone(); let key = key.clone();
            let error = grant_error.clone();
            button.connect_clicked(move |_| {
                if done.get() { return; }
                let Some(key) = key.clone() else { return; };
                if let Err(message) = grants.borrow_mut().remember(key, always) { error.set_text(&message); return; }
                done.set(true);
                client.reply(id.clone(), json!({"decision":"accept"}));
                if let Some(window) = weak.upgrade() { window.close(); }
            });
        }
    }
    let weak = window.downgrade();
    reject.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    let tracked = Rc::downgrade(dialogs);
    let request_key = id.to_string();
    window.connect_close_request(move |window| {
        window.set_modal(false);
        if let Some(dialogs) = tracked.upgrade() { dialogs.borrow_mut().remove(&request_key); }
        if !done.replace(true) {
            if question {
                client.reply(id.clone(), json!({"answers":{}}));
                client.interrupt();
            } else {
                client.reply(id.clone(), json!({"decision":"decline"}));
            }
        }
        glib::Propagation::Proceed
    });
    window.present();
    #[cfg(debug_assertions)]
    if matches!(std::env::var("NEXUTERM_SMOKE_AI").as_deref(), Ok("codex-approval" | "approval-repeat" | "approval-session" | "approval-always")) {
        let weak = window.downgrade();
        let reject = reject.downgrade();
        let accept = accept.downgrade();
        let actions = buttons.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
            if let Some(window) = weak.upgrade() {
                let actions = actions.upgrade().unwrap();
                assert_eq!(actions.orientation(), gtk::Orientation::Horizontal);
                assert!(actions.height() <= 28, "Approval buttons must stay compact");
                let mut count = 0; let mut child = actions.first_child();
                while let Some(button) = child { count += 1; child = button.next_sibling(); }
                assert_eq!(count, if approval {4} else {2});
                eprintln!("NEXUTERM_SMOKE_APPROVAL_COMPACT_OK");
                if let Some(path) = std::env::var_os("NEXUTERM_SCREENSHOT") {
                    let path = std::path::PathBuf::from(path)
                        .with_file_name("ai-codex-approval-dialog.png");
                    let paintable = gtk::WidgetPaintable::new(Some(&window));
                    let snapshot = gtk::Snapshot::new();
                    paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
                    if let (Some(node), Some(renderer)) = (snapshot.to_node(), window.renderer()) {
                        renderer
                            .render_texture(&node, None)
                            .save_to_png(path)
                            .unwrap();
                    }
                }
                match std::env::var("NEXUTERM_SMOKE_AI").as_deref() {
                    Ok("approval-repeat") => {
                        assert!(!window.is_modal());
                        accept.upgrade().unwrap().emit_clicked();
                        eprintln!("NEXUTERM_SMOKE_CODEX_APPROVAL_ACCEPTED");
                    }
                    Ok(mode @ ("approval-session" | "approval-always")) => {
                        fn choose(widget: &gtk::Widget, label: &str) -> bool {
                            if let Some(button) = widget.downcast_ref::<gtk::Button>()
                                && button.label().is_some_and(|text|text == label) { button.emit_clicked(); return true; }
                            let mut child = widget.first_child();
                            while let Some(widget) = child { child=widget.next_sibling(); if choose(&widget,label) { return true; } }
                            false
                        }
                        assert!(choose(window.upcast_ref(), if mode == "approval-session" {"Permitir en esta sesión"} else {"Permitir siempre"}));
                        eprintln!("NEXUTERM_SMOKE_CODEX_APPROVAL_REMEMBERED");
                    }
                    _ => if let Some(reject) = reject.upgrade() {
                        reject.emit_clicked();
                        eprintln!("NEXUTERM_SMOKE_CODEX_APPROVAL_REJECTED");
                    }
                }
            }
        });
    }
}

pub(super) fn build(state: &Rc<State>, process: &Rc<ai_panel::Process>, identifier:&str) -> Chat {
    let archive=Rc::new(nexuterm::chat_history::Recorder::new(identifier.into()));
    state.chat_archives.borrow_mut().push(archive.clone());
    let login_button = gtk::Button::with_label("Iniciar sesión");
    login_button.add_css_class("flat");
    login_button.add_css_class("ai-auth-control");
    login_button.set_halign(gtk::Align::End);
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let select_recent = Rc::new(RefCell::new(None::<Rc<dyn Fn(String)>>));
    let pages = gtk::Stack::new();
    pages.set_widget_name("ai-chat-pages");
    pages.set_vexpand(true);
    widget.append(&pages);
    let history = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let heading = gtk::Label::new(Some("Chats recientes"));
    heading.set_xalign(0.0);
    heading.add_css_class("dim-label");
    history.append(&heading);
    pages.add_named(&history, Some("recent"));
    let hint=gtk::Label::new(Some("Elige una conversación o escribe para iniciar un chat nuevo."));
    hint.set_xalign(0.0);hint.set_wrap(true);hint.add_css_class("dim-label");history.append(&hint);
    let known_threads=Rc::new(RefCell::new(std::collections::HashSet::<String>::new()));
    let local_recent = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let recent = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let recent_scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    let recent_body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    recent_body.append(&local_recent);
    recent_body.append(&recent);
    recent_scroll.set_child(Some(&recent_body));
    history.append(&recent_scroll);
    let history_more = gtk::Button::with_label("Cargar más chats");
    history_more.set_widget_name("workspace-more-recent-chats");
    history_more.set_halign(gtk::Align::Start);
    history_more.set_visible(false);
    history.append(&history_more);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 12);
    crate::context_menu::conversation(&list);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .overlay_scrolling(false)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list)
        .build();
    let overlay = gtk::Overlay::new();
    scroll.set_widget_name("workspace-chat-scroll");
    overlay.set_child(Some(&scroll));
    overlay.set_vexpand(true);
    let welcome = gtk::Box::new(gtk::Orientation::Vertical, 12);
    welcome.set_widget_name("codex-welcome");
    welcome.set_halign(gtk::Align::Center);
    welcome.set_valign(gtk::Align::Center);
    let logo = gtk::Image::from_icon_name("utilities-terminal-symbolic");
    logo.set_pixel_size(40);
    logo.add_css_class("dim-label");
    welcome.append(&logo);
    let hint = gtk::Label::new(Some("¿Qué quieres hacer?"));
    hint.add_css_class("dim-label");
    welcome.append(&hint);
    overlay.add_overlay(&welcome);
    pages.add_named(&overlay, Some("conversation"));
    pages.set_visible_child_name("recent");
    let retry=gtk::Button::with_label("Reanudar conversación");retry.set_widget_name("workspace-resume-chat");retry.set_halign(gtk::Align::Start);retry.set_visible(false);widget.append(&retry);
    let editor = gtk::Box::new(gtk::Orientation::Vertical, 4);
    editor.add_css_class("ai-composer");
    editor.add_css_class("nexuterm-section");
    let composer = gtk::TextView::builder()
        .wrap_mode(gtk::WrapMode::WordChar)
        .accepts_tab(false)
        .left_margin(12)
        .right_margin(12)
        .top_margin(12)
        .bottom_margin(8)
        .build();
    composer.update_property(&[gtk::accessible::Property::Label("Dime qué quieres hacer")]);
    composer.set_widget_name("codex-composer");
    let input = gtk::ScrolledWindow::builder()
        .min_content_height(64)
        .max_content_height(160)
        .propagate_natural_height(true)
        .child(&composer)
        .build();
    let placeholder = gtk::Label::new(Some("Dime qué quieres hacer"));
    placeholder.set_halign(gtk::Align::Start);
    placeholder.set_valign(gtk::Align::Start);
    placeholder.set_margin_start(12);
    placeholder.set_margin_top(12);
    placeholder.add_css_class("dim-label");
    placeholder.set_can_target(false);
    let input_overlay = gtk::Overlay::new();
    input_overlay.set_child(Some(&input));
    input_overlay.add_overlay(&placeholder);
    editor.append(&input_overlay);
    let weak = placeholder.downgrade();
    composer.buffer().connect_changed(move |b| {
        if let Some(p) = weak.upgrade() {
            p.set_visible(b.char_count() == 0);
        }
    });
    let context_row = gtk::Box::new(gtk::Orientation::Vertical, 2);
    context_row.set_visible(false);
    editor.append(&context_row);
    let context = Rc::new(RefCell::new(Vec::<std::path::PathBuf>::new()));
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    toolbar.set_margin_start(6);
    toolbar.set_margin_end(6);
    toolbar.set_margin_bottom(6);
    let add = gtk::MenuButton::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("Añadir contexto: archivos o carpetas")
        .build();
    add.add_css_class("flat");
    let popover = gtk::Popover::new();
    let additions = gtk::Box::new(gtk::Orientation::Vertical, 4);
    for (label, action) in [
        ("Archivos…", gtk::FileChooserAction::Open),
        ("Carpetas…", gtk::FileChooserAction::SelectFolder),
    ] {
        let button = gtk::Button::new();
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        content.append(&gtk::Image::from_icon_name(
            if action == gtk::FileChooserAction::Open {
                "mail-attachment-symbolic"
            } else {
                "folder-symbolic"
            },
        ));
        content.append(&gtk::Label::new(Some(label)));
        button.set_child(Some(&content));
        button.add_css_class("flat");
        additions.append(&button);
        let s = Rc::downgrade(state);
        let paths = context.clone();
        let chips = context_row.downgrade();
        let pop = popover.downgrade();
        button.connect_clicked(move |_| {
            if let Some(pop) = pop.upgrade() {
                pop.popdown();
            }
            let Some(parent) = s.upgrade().and_then(|s| s.window.upgrade()) else {
                return;
            };
            let chooser = gtk::FileChooserNative::new(
                Some("Añadir contexto a Codex"),
                Some(&parent),
                action,
                Some("Añadir"),
                Some("Cancelar"),
            );
            chooser.set_select_multiple(true);
            let paths = paths.clone();
            let chips = chips.clone();
            chooser.connect_response(move |chooser, response| {
                if response == gtk::ResponseType::Accept {
                    for index in 0..chooser.files().n_items() {
                        let Some(path) = chooser
                            .files()
                            .item(index)
                            .and_downcast::<gio::File>()
                            .and_then(|file| file.path())
                            .and_then(|p| p.canonicalize().ok())
                        else {
                            continue;
                        };
                        if paths.borrow().len() >= 20 || paths.borrow().contains(&path) {
                            continue;
                        }
                        paths.borrow_mut().push(path.clone());
                        if let Some(chips) = chips.upgrade() {
                            attachment_chip(&chips,&paths,path);
                        }
                    }
                }
                chooser.destroy();
            });
            chooser.show();
        });
    }
    let note = gtk::Label::new(Some(
        "Codex recibirá las rutas seleccionadas.
Puede solicitar permiso para leerlas.",
    ));
    note.add_css_class("caption");
    additions.append(&note);
    popover.set_child(Some(&additions));
    add.set_popover(Some(&popover));
    toolbar.append(&add);
    let model_names = gtk::StringList::new(&["Modelo predeterminado"]);
    let model = gtk::DropDown::builder()
        .model(&model_names)
        .enable_search(true)
        .hexpand(true)
        .build();
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        if let Some(item) = item.downcast_ref::<gtk::ListItem>() {
            let label = gtk::Label::new(None);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(18);
            label.set_xalign(0.0);
            item.set_child(Some(&label));
        }
    });
    factory.connect_bind(|_, item| {
        if let Some(item) = item.downcast_ref::<gtk::ListItem>()
            && let (Some(label), Some(value)) = (
                item.child().and_downcast::<gtk::Label>(),
                item.item().and_downcast::<gtk::StringObject>(),
            ) {
                label.set_text(&value.string());
            }
    });
    model.set_factory(Some(&factory));
    model.add_css_class("flat");
    model.set_tooltip_text(Some("Modelo del agente seleccionado"));
    let models: Rc<RefCell<Vec<Value>>> = Rc::new(RefCell::new(Vec::new()));
    toolbar.append(&model);
    let mode = gtk::DropDown::from_strings(&["Solo lectura", "Pedir permiso", "Acceso completo"]);
    mode.set_widget_name("ai-permissions");
    mode.add_css_class("flat");
    mode.set_tooltip_text(Some("Acceso completo autoriza comandos, archivos y herramientas de este chat sin pedir aprobación. No concede privilegios del sistema."));
    toolbar.append(&mode);

    let effort = gtk::DropDown::from_strings(&["Auto"]);
    effort.add_css_class("flat");
    effort.set_tooltip_text(Some("Nivel de razonamiento"));
    toolbar.append(&effort);
    let m = models.clone();
    let e = effort.downgrade();
    model.connect_selected_notify(move |selector| {
        if let Some(e) = e.upgrade() {
            let values = m.borrow();
            let values = values
                .get(selector.selected() as usize)
                .and_then(|v| v["supportedReasoningEfforts"].as_array());
            let names = gtk::StringList::new(&["Auto"]);
            if let Some(values) = values {
                for v in values {
                    if let Some(value) = v["reasoningEffort"].as_str() {
                        names.append(effort_label(value));
                    }
                }
            }
            e.set_model(Some(&names));
            e.set_selected(0);
        }
    });
    let voice = super::ai_dictation::button(state, &composer);
    toolbar.append(&voice);
    let send = icon_button("go-up-symbolic", "Enviar mensaje · Ctrl+Intro");
    send.add_css_class("circular");
    send.add_css_class("suggested-action");
    send.set_sensitive(false);
    send.set_widget_name("codex-send");
    toolbar.append(&send);
    editor.append(&toolbar);
    widget.append(&editor);
    let usage = UsageView::new(&widget);
    let browser = gtk::LinkButton::with_label("about:blank", "Acceso web");
    browser.set_tooltip_text(Some("Abrir el enlace de inicio de sesión del proveedor"));
    browser.add_css_class("ai-auth-control");
    browser.set_visible(false);
    let row = usage.details_row.clone();
    let spinner = gtk::Spinner::new();
    let info = gtk::Label::new(Some("Chat de IA"));
    info.set_wrap(false);
    info.set_ellipsize(gtk::pango::EllipsizeMode::End);
    info.set_selectable(true);
    info.set_widget_name("ai-status");
    info.set_hexpand(true);
    info.set_xalign(0.0);
    info.add_css_class("dim-label");
    info.add_css_class("caption");
    info.connect_label_notify(|label| { label.set_visible(!label.text().is_empty()); label.set_tooltip_text(Some(&label.text())); });
    row.append(&spinner);
    row.append(&info);
    row.append(&browser);
    {
        let archive=Rc::downgrade(&archive);let status=info.downgrade();let last=RefCell::new(None::<String>);
        glib::timeout_add_local(std::time::Duration::from_millis(250),move ||{let (Some(archive),Some(status))=(archive.upgrade(),status.upgrade()) else{return glib::ControlFlow::Break;};if let Some(error)=archive.error() && last.borrow().as_ref()!=Some(&error){status.set_text(&format!("Historial detenido: {error}"));*last.borrow_mut()=Some(error);}glib::ControlFlow::Continue});
    }
    let current = Rc::new(RefCell::new(None::<(String, Rc<Client>)>));
    {
        let current = current.clone();
        let process = process.clone();
        mode.connect_selected_notify(move |selector| {
            let full = selector.selected() == 2;
            if let Some(bridge) = &process.bridge {
                bridge.full_access.store(full, std::sync::atomic::Ordering::Relaxed);
            }
            if let Some((_, client)) = current.borrow().as_ref() {
                client.set_full_access(full);
                client.permissions(selector.selected() != 0);
            }
            if full { selector.add_css_class("error"); } else { selector.remove_css_class("error"); }
        });
    }

    let logout = gtk::LinkButton::with_label("about:blank", "Cerrar sesión");
    logout.set_widget_name("ai-logout");
    logout.set_halign(gtk::Align::End);
    logout.add_css_class("ai-auth-control");
    logout.set_visible(false);
    usage.account_row.append(&logout);
    usage.account_row.append(&login_button);
    let logout_client = current.clone();
    logout.connect_activate_link(move |_| {
        if let Some((_, client)) = logout_client.borrow().as_ref() { client.logout(); }
        glib::Propagation::Stop
    });
    let dialogs: Dialogs = Rc::new(RefCell::new(HashMap::new()));
    let generation = Rc::new(Cell::new(0_u64));
    let weak = welcome.downgrade();
    let children = list.observe_children();
    children.connect_items_changed(move |items, _, _, _| {
        if let Some(w) = weak.upgrade() {
            w.set_visible(items.n_items() == 0);
        }
    });
    let usage_models = models.clone(); let usage_choice = usage.clone();
    model.connect_selected_notify(move |choice| {
        let models = usage_models.borrow();
        usage_choice.render(models.get(choice.selected() as usize).and_then(|v|v["model"].as_str()).unwrap_or(""));
    });
    let conversation_title=Rc::new(RefCell::new(None::<String>));
    let offline = Rc::new(RefCell::new(None::<nexuterm::workspaces::Chat>));
    let saved_agent = Rc::new(RefCell::new(None::<AiAgent>));
    let session = Rc::new(RefCell::new(None::<String>));
    let continuation = Rc::new(RefCell::new(None::<nexuterm::chat_state::Continuation>));
    let pending_resume = Rc::new(RefCell::new(None::<nexuterm::workspaces::Chat>));
    let awaiting_resume = Rc::new(Cell::new(false));
    let history_handler = Rc::new(RefCell::new(None::<glib::SignalHandlerId>));
    let refresh_recent: Rc<dyn Fn(String)> = {
        let target = local_recent.downgrade();
        let selected = select_recent.clone();
        let native=recent.downgrade();let known=known_threads.clone();
        let revision = Rc::new(Cell::new(0u64));
        Rc::new(move |agent| {
            known.borrow_mut().clear();
            revision.set(revision.get()+1);
            let epoch=revision.get(); let revision=revision.clone();
            let target=target.clone(); let selected=selected.clone();let known=known.clone();let native=native.clone();
            if let Some(target)=target.upgrade(){while let Some(child)=target.first_child(){target.remove(&child);}}
            glib::spawn_future_local(async move {
                let result=nexuterm::runtime::runtime().spawn_blocking(move || {
                    nexuterm::chat_history::recent(&nexuterm::chat_history::directory()?, &agent)
                }).await;
                if revision.get()!=epoch{return;}
                let Some(target)=target.upgrade() else{return;};
                match result.unwrap_or_else(|e|Err(e.to_string())) {
                    Ok(entries) => {
                        *known.borrow_mut()=entries.iter().filter_map(|e|e.thread.clone()).collect();
                        if let Some(native)=native.upgrade(){let mut child=native.first_child();while let Some(row)=child{child=row.next_sibling();if known.borrow().iter().any(|id|row.widget_name()==format!("ai-native-recent-{id}")){native.remove(&row);}}}
                        for entry in entries {
                            let button=gtk::Button::with_label(&entry.title);
                            button.set_widget_name("ai-recent-conversation");
                            button.add_css_class("flat");
                            if let Some(label)=button.child().and_downcast::<gtk::Label>(){label.set_xalign(0.0);label.set_ellipsize(gtk::pango::EllipsizeMode::End);}
                            let selected=selected.clone();
                            button.connect_clicked(move |_| {let callback=selected.borrow().clone();if let Some(callback)=callback{callback(entry.id.clone());}});
                            target.append(&button);
                        }
                    }
                    Err(error) => {let label=gtk::Label::new(Some(&format!("No se pudieron cargar los recientes: {error}")));label.set_wrap(true);target.append(&label);}
                }
            });
        })
    };
    let open: Rc<dyn Fn(AiAgent, bool)> = {
        let conversation_title=conversation_title.clone();
        let known_threads=known_threads.clone();
        let refresh_recent=refresh_recent.clone();
        let pages=pages.downgrade();
        let offline = offline.clone();
        let continuation = continuation.clone();
        let saved_agent = saved_agent.clone();
        let session = session.clone();
        let pending_resume = pending_resume.clone();
        let awaiting_resume = awaiting_resume.clone();
        let usage = usage.clone();
        let archive=archive.clone();
        let retry=retry.downgrade();
        let welcome = welcome.downgrade();
        let history=history.downgrade();
        let recent = recent.downgrade();
        let history_more = history_more.downgrade();
        let history_handler = history_handler.clone();
        let model = model.downgrade();
        let model_names = model_names.clone();
        let models = models.clone();
        let current = current.clone();
        let dialogs = dialogs.clone();
        let generation = generation.clone();
        let list = list.downgrade();
        let input = composer.downgrade();
        let status = info.downgrade();
        let spinner = spinner.downgrade();
        let send = send.downgrade();
        let browser = browser.downgrade();
        let mode = mode.downgrade();
        let p = process.clone();
        let s = Rc::downgrade(state);
        let login_button = login_button.downgrade();
        let logout = logout.downgrade();
        let effort = effort.downgrade();
        Rc::new(move |mut agent, login| {
            let resume = pending_resume.borrow_mut().take();
            if resume.is_none() {
                conversation_title.borrow_mut().take();
                if let Some(directory) = s.upgrade().and_then(|state| agent_workspace(&state)).and_then(|workspace| workspace.local_directory) {
                    agent.directory = directory.to_string_lossy().into_owned();
                }
            } else if let Some(directory) = resume.as_ref().and_then(|c| c.directory.as_ref()) {
                agent.directory = directory.to_string_lossy().into_owned();
            }
            if let Some(link) = logout.upgrade() { link.set_visible(false); }
            if let Some(button) = login_button.upgrade() {
                button.set_visible(!agent.kind.is_api());
            }
            if agent.terminal_access {
                agent.terminal_socket = p
                    .bridge
                    .as_ref()
                    .map(|b| b.path().to_string_lossy().into_owned());
            }
            if login
                && let Some((id, client)) = current.borrow().as_ref()
                    && id == &agent.id && client.alive() {
                        client.login();
                        return;
                    }
            refresh_recent(agent.id.clone());
            if let Some(pages)=pages.upgrade(){pages.set_visible_child_name(if resume.is_some(){"conversation"}else{"recent"});}
            if let Some(retry)=retry.upgrade(){retry.set_visible(resume.is_some());retry.set_sensitive(false);}
            *continuation.borrow_mut()=resume.as_ref().and_then(|c|c.continuation.clone());
            *offline.borrow_mut() = None;
            if agent.directory.is_empty() {
                agent.directory = std::env::var("HOME").unwrap_or_else(|_| "/".into());
            }
            *saved_agent.borrow_mut() = Some(agent.clone());
            archive.record(nexuterm::chat_history::Record::Header{version:1,title:format!("{} · {}",agent.name,agent.model)});
            *session.borrow_mut() = resume.as_ref().and_then(|c|c.codex_thread.clone());
            awaiting_resume.set(resume.is_some());
            let (
                Some(list),
                Some(input),
                Some(info),
                Some(spinner),
                Some(send),
                Some(browser),
                Some(mode),
            ) = (
                list.upgrade(),
                input.upgrade(),
                status.upgrade(),
                spinner.upgrade(),
                send.upgrade(),
                browser.upgrade(),
                mode.upgrade(),
            )
            else {
                return;
            };
            usage.reset();
            close_dialogs(&dialogs);
            if let Some((_, old)) = current.borrow_mut().take() {
                old.close();
            }
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            input.buffer().set_text("");
            browser.set_visible(false);
            browser.set_uri("about:blank");
            info.set_text("Conectando con el agente…");
            spinner.start();
            send.set_sensitive(false);
            mode.set_sensitive(true);
            models.borrow_mut().clear();
            if let Some(model) = model.upgrade() {
                let values = if agent.model.is_empty() {
                    "Modelo predeterminado"
                } else {
                    &agent.model
                };
                model_names.splice(0, model_names.n_items(), &[values]);
                model.set_selected(0);
            }
            agent.wait_credentials = !agent.is_codex() && !agent.kind.is_api();
            mode.set_selected(0);
            if let Some(bridge) = &p.bridge { bridge.full_access.store(false, std::sync::atomic::Ordering::Relaxed); }
            let (client, mut events) = if resume.as_ref().is_some_and(|c|c.continuation.is_some()){Client::start_recovery(agent.clone(),false)}else{Client::start(agent.clone(),mode.selected()!=0)};
            let client = Rc::new(client);
            {
                let c = client.clone();
                let s = s.clone();
                let profile = agent.clone();
                glib::spawn_future_local(async move {
                    if let Some(state) = s.upgrade() {
                        c.credentials(super::ai_credentials::lookup(&state, &profile).await);
                        if login {
                            c.login();
                        }
                    }
                });
            }
            p.set_native(client.clone());
            *current.borrow_mut() = Some((agent.id.clone(), client.clone()));
            generation.set(generation.get() + 1);
            let epoch = generation.get();
            let generations = generation.clone();
            let list = list.downgrade();
            let info = info.downgrade();
            let spinner = spinner.downgrade();
            let send = send.downgrade();
            let browser = browser.downgrade();
            let input = input.downgrade();
            let dialogs = dialogs.clone();
            let s = s.clone();
            let mut buffers: HashMap<String, MessageLabel> = HashMap::new();
            let mut items: HashMap<String, Value> = HashMap::new();
            let mut limit_notified=false;
            let welcome = welcome.clone();
            let agent_name = agent.name.clone();
            let grants = Rc::new(RefCell::new(nexuterm::approval_grants::Grants::new(nexuterm::agent_profile::root(&agent))));
            let grant_workspace = if agent.directory.is_empty() {std::env::var("HOME").unwrap_or_else(|_|"/".into())} else {agent.directory.clone()};
            let api_profile = agent.kind.is_api();
            let codex_profile = agent.is_codex();
            let pages=pages.clone();
            let known_threads=known_threads.clone();
            let configured_models = agent.models.clone();
            let preferred_model = agent.model.clone();
            let login_button = login_button.clone();
            let logout = logout.clone();
            let reasoning_selector = effort.clone();
            let usage = usage.clone();
            let capture_resumed=Rc::new(Cell::new(false));
            let mut ready = false;
            let mut provider_authenticated = false;
            #[cfg(debug_assertions)]
            let mut logout_test_started = false;
            #[cfg(debug_assertions)]
            let mut workspace_turns = 0;
            let history=history.clone();
            let recent = recent.clone();
            let history_more = history_more.clone();
            let models = models.clone();
            let model = model.clone();
            let model_names = model_names.clone();
            let mode = mode.downgrade();
            let session = session.clone();
            let preferred_effort=resume.as_ref().map(|c|c.reasoning.clone()).unwrap_or_default();
            let continuation=continuation.clone();
            let archive=archive.clone();
            let retry=retry.clone();
            let mut resume_to_load = resume;
            let awaiting_resume = awaiting_resume.clone();
            let history_cursor = Rc::new(RefCell::new(None::<String>));
            let history_loading = Rc::new(Cell::new(false));
            if let Some(button) = history_more.upgrade() {
                if let Some(handler) = history_handler.borrow_mut().take() {
                    button.disconnect(handler);
                }
                let client = client.clone();
                let cursor = history_cursor.clone();
                let loading = history_loading.clone();
                let info = info.clone();
                let handler = button.connect_clicked(move |button| {
                    if loading.get() {
                        return;
                    }
                    let Some(cursor) = cursor.borrow().clone() else {
                        return;
                    };
                    if client.history(cursor) {
                        loading.set(true);
                        button.set_sensitive(false);
                    } else if let Some(info) = info.upgrade() {
                        info.set_text("No se pudo solicitar la siguiente página del historial.");
                    }
                });
                *history_handler.borrow_mut() = Some(handler);
            }
            glib::timeout_add_local(std::time::Duration::from_millis(35), move || {
                if generations.get() != epoch {
                    return glib::ControlFlow::Break;
                }
                let (Some(list), Some(info), Some(spinner), Some(send), Some(browser)) = (
                    list.upgrade(),
                    info.upgrade(),
                    spinner.upgrade(),
                    send.upgrade(),
                    browser.upgrade(),
                ) else {
                    close_dialogs(&dialogs);
                    client.close();
                    return glib::ControlFlow::Break;
                };
                for _ in 0..64 {
                    if !archive.ready(){break;}
                    let event = match events.try_recv() {
                        Ok(event) => event,
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(_) => return glib::ControlFlow::Break,
                    };
                    match event {
                        Event::Authenticated(value) => {
                            provider_authenticated = value;
                            if !value { grants.borrow_mut().clear_session(); }
                            if let Some(button) = login_button.upgrade() {
                                button.set_visible(!(api_profile || provider_authenticated || codex_profile && ready));
                                #[cfg(debug_assertions)]
                                if value && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some() {
                                    assert!(!button.is_visible());
                                    eprintln!("NEXUTERM_SMOKE_PROVIDER_AUTH_HIDDEN");
                                }
                            }
                        }
                        Event::Account(name) => {
                            usage.reset();
                            usage.account.set_text(&name.map(|name| format!("Usuario: {name}")).unwrap_or_else(|| "Usuario: no disponible".into()));
                            *usage.snapshot.borrow_mut() = Value::Null; usage.render("");
                        }
                        Event::RateLimits(value) => {
                            *usage.snapshot.borrow_mut() = value;
                            let selected = model.upgrade().map(|m|m.selected()).unwrap_or(0);
                            usage.render(models.borrow().get(selected as usize).and_then(|m|m["model"].as_str()).unwrap_or(""));
                        }
                        Event::ContextUsage {used,size} => {
                            if let Some(remaining) = nexuterm::ai_usage::context_remaining(used,size) {
                                usage.context.set_text(&format!("Contexto: {remaining:.0}%"));
                                usage.context.set_tooltip_text(Some(&format!("{used} de {size} tokens de contexto. No es la cuota de la cuenta.")));
                                usage.context.set_visible(true);
                            } else { usage.context.set_visible(false); }
                        }
                        Event::TokenUsage {input,output} => {
                            usage.tokens.set_text(&format!("Tokens: {input} ↑ · {output} ↓")); usage.tokens.set_tooltip_text(Some(&format!("Última petición: {input} tokens de entrada · {output} de salida"))); usage.tokens.set_visible(true);
                        }
                        Event::Capabilities {
                            tools,
                            history: has_history,
                            reasoning,
                        } => {
                            if let Some(mode) = mode.upgrade() {
                                mode.set_sensitive(true);
                                let _ = tools;
                                mode.set_tooltip_text(Some("Acceso completo autoriza las herramientas disponibles sin pedir aprobación; no concede privilegios del sistema ni añade capacidades al proveedor."));
                            }
                            let _ = has_history; // Local recents are available for every agent.
                            if let Some(selector) = reasoning_selector.upgrade() {
                                selector.set_visible(reasoning);
                            }
                        }
                        Event::Models(mut values) => {
                            for id in &configured_models {
                                if !values.iter().any(|v| v["model"].as_str() == Some(id)) {
                                    values.push(json!({"model":id,"displayName":id,"supportedReasoningEfforts":[]}));
                                }
                            }
                            let labels: Vec<&str> = values
                                .iter()
                                .map(|v| {
                                    v["displayName"]
                                        .as_str()
                                        .or(v["model"].as_str())
                                        .unwrap_or("Modelo")
                                })
                                .collect();
                            if !labels.is_empty() {
                                model_names.splice(0, model_names.n_items(), &labels);
                                let selected = values
                                    .iter()
                                    .position(|v| v["model"].as_str() == Some(&preferred_model))
                                    .or_else(|| values.iter().position(|v| v["isDefault"] == true))
                                    .unwrap_or(0);
                                *models.borrow_mut() = values;
                                if let Some(model) = model.upgrade() {
                                    model.set_selected(selected as u32);
                                    model.notify("selected");
                                    if let Some(choice)=reasoning_selector.upgrade(){
                                        let index=models.borrow().get(selected).and_then(|v|v["supportedReasoningEfforts"].as_array()).and_then(|v|v.iter().position(|v|v["reasoningEffort"].as_str()==Some(preferred_effort.as_str()))).map(|v|v as u32+1).unwrap_or(0);
                                        choice.set_selected(index);
                                        if !preferred_effort.is_empty() && index==0 {info.set_text("El proveedor no ofrece el razonamiento guardado; se usará Auto.");}
                                    }
                                }
                            }
                        }
                        Event::History(threads, cursor) => {
                            if let Some(recent) = recent.upgrade() {
                                let append_page = history_loading.replace(false);
                                if !append_page {
                                    while let Some(child) = recent.first_child() {
                                        recent.remove(&child);
                                    }
                                }
                                for thread in threads {
                                    let Some(thread_id)=thread["id"].as_str().filter(|id|!id.is_empty()) else{continue;};
                                    if known_threads.borrow().contains(thread_id){continue;}
                                    let title = thread["name"]
                                        .as_str()
                                        .filter(|s| !s.is_empty())
                                        .or(thread["preview"].as_str())
                                        .unwrap_or("Conversación");
                                    let button = gtk::Button::with_label(title);
                                    button.set_widget_name(&format!("ai-native-recent-{thread_id}"));
                                    button.add_css_class("flat");
                                    if let Some(label) = button.child().and_downcast::<gtk::Label>()
                                    {
                                        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                                        label.set_xalign(0.0);
                                    }
                                    let c = client.clone();
                                    let id = thread["id"].as_str().unwrap_or("").to_owned();
                                    let directory = thread["cwd"].as_str().map(str::to_owned);
                                    let mode = mode.clone();
                                    let pages=pages.clone();
                                    let capture_resumed=capture_resumed.clone();
                                    button.connect_clicked(move |_| {
                                        if let Some(mode) = mode.upgrade() {
                                            c.permissions(mode.selected() != 0);
                                        }
                                        if let Some(pages)=pages.upgrade(){pages.set_visible_child_name("conversation");}
                                        capture_resumed.set(true);
                                        c.resume_in_directory(id.clone(), directory.clone());
                                    });
                                    recent.append(&button);
                                }
                            }
                            *history_cursor.borrow_mut() = cursor;
                            if let Some(button) = history_more.upgrade() {
                                button.set_sensitive(true);
                                button.set_visible(history_cursor.borrow().is_some());
                            }
                        }
                        Event::HistoryFailed(error) => {
                            history_loading.set(false);
                            if let Some(button) = history_more.upgrade() {
                                button.set_sensitive(true);
                            }
                            info.set_text(&error);
                        }
                        Event::Continuation(value)=>{*continuation.borrow_mut()=Some(value);}
                        Event::Continued=>{awaiting_resume.set(false);if let Some(retry)=retry.upgrade(){retry.set_visible(false);}info.set_text("");}
                        Event::Session(id) => { *session.borrow_mut() = Some(id); }
                        Event::Resumed(thread) => {
                            let resumed_archive=capture_resumed.replace(false).then_some(&archive);
                            awaiting_resume.set(false);
                            if let Some(retry)=retry.upgrade(){retry.set_visible(false);}
                            while let Some(child) = list.first_child() {
                                list.remove(&child);
                            }
                            buffers.clear();
                            items.clear();
                            if let Some(turns) = thread["turns"].as_array() {
                                for turn in turns {
                                    if let Some(items) = turn["items"].as_array() {
                                        for item in items {
                                            if item["type"] == "agentMessage" {
                                                message(&list, resumed_archive,
                                                    &agent_name,
                                                    item["text"].as_str().unwrap_or(""),
                                                    false,
                                                    true,
                                                );
                                            } else if item["type"] == "userMessage"
                                                && let Some(content) = item["content"].as_array() {
                                                    for text in content {
                                                        if let Some(text) = text["text"].as_str() {
                                                            message(&list, resumed_archive, "Tú", text, false, false);
                                                        }
                                                    }
                                                }
                                        }
                                    }
                                }
                            }
                            info.set_text("");
                            if let Some(mode) = mode.upgrade() {
                                mode.set_sensitive(true);
                            }
                        }
                        Event::Ready(authenticated) => {
                            if !authenticated { grants.borrow_mut().clear_session(); }
                            if let Some(link) = logout.upgrade() {
                                link.set_visible(codex_profile && authenticated);
                            }
                            if let Some(button) = login_button.upgrade() {
                                button
                                    .set_visible(!(api_profile || provider_authenticated || codex_profile && authenticated));
                                #[cfg(debug_assertions)]
                                if authenticated
                                    && codex_profile
                                    && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                                {
                                    assert!(!button.is_visible());
                                    eprintln!("NEXUTERM_SMOKE_AUTH_BUTTON_HIDDEN");
                                }
                            }
                            ready = authenticated;
                            if ready && let Some(saved) = resume_to_load.take() {
                                if let Some(value)=saved.continuation {client.restore(value);}else if let Some(id) = saved.codex_thread { client.resume(id); }
                            }
                            info.set_text(if ready {
                                ""
                            } else {
                                "Conecta o autentica este agente para conversar."
                            });
                            browser.set_visible(false);
                            browser.set_uri("about:blank");
                            #[cfg(debug_assertions)]
                            if codex_profile && std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("logout") {
                                if authenticated && !logout_test_started {
                                    assert!(info.text().is_empty() && !info.is_visible());
                                    let link = logout.upgrade().expect("Logout link");
                                    assert!(link.is_visible());
                                    logout_test_started = true;
                                    link.emit_by_name::<bool>("activate-link", &[]);
                                } else if !authenticated && logout_test_started {
                                    assert!(login_button.upgrade().unwrap().is_visible());
                                    assert!(!logout.upgrade().unwrap().is_visible());
                                    assert_eq!(usage.account.text().as_str(), "Usuario: no disponible");
                                    eprintln!("NEXUTERM_SMOKE_LOGOUT_OK");
                                }
                            }
                        }
                        Event::Status(text) => {
                            #[cfg(debug_assertions)]
                            if text.contains("ya tiene sesión")
                                && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                            {
                                eprintln!("NEXUTERM_SMOKE_CODEX_CACHED_OK");
                            }
                            info.set_text(&text);
                        }
                        event @ (Event::LoginUrl(_) | Event::LoginLink(_)) => {
                            let automatic = matches!(&event, Event::LoginUrl(_));
                            let (Event::LoginUrl(url) | Event::LoginLink(url)) = event else { unreachable!() };
                            if url::Url::parse(&url)
                                .is_ok_and(|u| u.scheme() == "https" && u.host_str().is_some())
                            {
                                browser.set_uri(&url);
                                browser.set_visible(true);
                                if !automatic || gio::AppInfo::launch_default_for_uri(
                                    &url,
                                    None::<&gio::AppLaunchContext>,
                                )
                                .is_err()
                                {
                                    info.set_text(
                                        "Pulsa el enlace para completar el inicio de sesión.",
                                    );
                                } else {
                                    info.set_text("Completa el inicio de sesión en el navegador.");
                                }
                            } else {
                                info.set_text("Codex devolvió una URL de autenticación inválida.");
                                client.interrupt();
                            }
                        }
                        Event::Delta { item, text } => {
                            if buffers.len() >= 128 && !buffers.contains_key(&item) {
                                if !limit_notified {message(&list,Some(&archive),"Interrupción","Se alcanzó el límite de 128 elementos de un turno. Se conserva lo capturado; consulta el historial del proveedor para lo que no llegó a recibirse.",false,false);limit_notified=true;}
                                client.interrupt();
                                continue;
                            }
                            let buffer = buffers
                                .entry(item)
                                .or_insert_with(|| message(&list, Some(&archive), &agent_name, "", false, true));
                            append(buffer, &text);
                        }
                        Event::Item(item) => {
                            let kind = item["type"].as_str().unwrap_or("");
                            if kind == "requestResolved" {
                                let resolved = dialogs.borrow_mut().remove(&item["requestId"].to_string());
                                if let Some((window, done)) = resolved {
                                    done.set(true);
                                    if let Some(window) = window.upgrade() {
                                        window.close();
                                    }
                                }
                                continue;
                            }
                            let id = item["id"].as_str().unwrap_or("").to_owned();
                            if id.is_empty() {
                                continue;
                            }
                            if buffers.len() >= 128 && !buffers.contains_key(&id) {
                                if !limit_notified {message(&list,Some(&archive),"Interrupción","Se alcanzó el límite de 128 elementos de un turno. Se conserva lo capturado; consulta el historial del proveedor para lo que no llegó a recibirse.",false,false);limit_notified=true;}
                                client.interrupt();
                                continue;
                            }
                            if kind == "agentMessage" {
                                let text = item["text"].as_str().unwrap_or("");
                                if !text.is_empty() {
                                    let b = buffers
                                        .entry(id.clone())
                                        .or_insert_with(|| message(&list, Some(&archive), &agent_name, "", false, true));
                                    b.set_text(text);
                                }
                            } else if matches!(
                                kind,
                                "commandExecution"
                                    | "fileChange"
                                    | "mcpToolCall"
                                    | "agentToolCall"
                                    | "webSearch"
                                    | "collabAgentToolCall"
                            ) {
                                if buffers.len() >= 128 && !buffers.contains_key(&id) {
                                    client.interrupt();
                                    continue;
                                }
                                let b = buffers.entry(id.clone()).or_insert_with(|| {
                                    message(&list, Some(&archive),
                                        &tool_title(&item),
                                        "",
                                        true,
                                        false,
                                    )
                                });
                                let text = tool_text(&item);
                                b.set_text(&text.chars().take(65536).collect::<String>());
                            }
                            if (items.len() < 128 || items.contains_key(&id))
                                && serde_json::to_vec(&item).is_ok_and(|v| v.len() <= 65536)
                            {
                                items.insert(id, item);
                            }
                        }
                        Event::Request { id, method, params } => {
                            if !matches!(
                                method.as_str(),
                                "item/commandExecution/requestApproval"
                                    | "item/fileChange/requestApproval"
                                    | "item/tool/requestApproval"
                                    | "item/tool/requestUserInput"
                            ) {
                                info.set_text("El agente solicitó una operación que este chat todavía no admite; se ha rechazado.");
                            }
                            if dialogs.borrow().len() >= 8 {
                                client.reject_unknown(id);
                                continue;
                            }
                            if let Some(window) = s.upgrade().and_then(|s| s.window.upgrade()) {
                                let detail = items.get(params["itemId"].as_str().unwrap_or(""));
                                request_dialog(
                                    &window,
                                    client.clone(),
                                    id,
                                    &method,
                                    params,
                                    detail,
                                    &dialogs,
                                    &grant_workspace,
                                    &grants,
                                );
                            } else {
                                client.reject_unknown(id);
                            }
                        }
                        Event::Finished(status) => {
                            #[cfg(debug_assertions)]
                            if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("full-access") {
                                assert_eq!(status, "completed");
                                assert!(client.full_access());
                                assert!(dialogs.borrow().is_empty());
                                let selector = mode.upgrade().unwrap();
                                assert_eq!(selector.selected(), 2);
                                selector.set_selected(0);
                                assert!(!client.full_access());
                                eprintln!("NEXUTERM_SMOKE_FULL_ACCESS_OK");
                            }

                            if let Some(mode) = mode.upgrade() { mode.set_sensitive(true); }
                            #[cfg(debug_assertions)]
                            if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("copy") && status == "completed" {
                                let label = buffers.values().find(|label|label.text().starts_with("# Informe")).expect("Markdown response");
                                let expected = label.text().to_string();
                                let card = label.parent().unwrap();
                                let header = card.first_child().unwrap();
                                let copy = header.last_child().unwrap().downcast::<gtk::Button>().unwrap();
                                assert_eq!(copy.widget_name().as_str(), "ai-copy-response");
                                let scroll = copy.ancestor(gtk::ScrolledWindow::static_type()).unwrap().downcast::<gtk::ScrolledWindow>().unwrap();
                                glib::spawn_future_local(async move {
                                    glib::timeout_future(std::time::Duration::from_millis(100)).await;
                                    assert!(scroll.vadjustment().upper() > scroll.vadjustment().page_size(), "Long response must require scrolling");
                                    scroll.vadjustment().set_value(scroll.vadjustment().lower());
                                    glib::timeout_future(std::time::Duration::from_millis(100)).await;
                                    let bounds = copy.compute_bounds(&scroll).unwrap();
                                    let bar = scroll.vscrollbar().compute_bounds(&scroll).unwrap();
                                    assert!(bounds.x() + bounds.width() <= bar.x(), "Scrollbar covers the copy button: copy={bounds:?}, bar={bar:?}, width={}", scroll.width());
                                    let picked = scroll.pick((bounds.x()+bounds.width()/2.0) as f64, (bounds.y()+bounds.height()/2.0) as f64, gtk::PickFlags::DEFAULT).expect("Copy button must be visible");
                                    assert!(picked == copy || picked.is_ancestor(&copy), "Copy button must receive pointer events");
                                    copy.emit_clicked();
                                    let clipboard = copy.clipboard();
                                    assert_eq!(clipboard.read_text_future().await.unwrap().as_deref(), Some(expected.as_str()));
                                    eprintln!("NEXUTERM_SMOKE_COPY_SCROLL_OK");
                                    eprintln!("NEXUTERM_SMOKE_COPY_MARKDOWN_OK");
                                });
                            }
                            #[cfg(debug_assertions)]
                            if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("workspace") && status == "completed" {
                                workspace_turns += 1;
                                if workspace_turns == 1 {
                                    let state = s.upgrade().unwrap();
                                    let terminal = state.focused_terminal.borrow().upgrade().unwrap();
                                    let next = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join("second project");
                                    terminal.feed_child(format!("{}\n",next.display()).as_bytes());
                                    let input = input.clone(); let send = send.downgrade();
                                    glib::timeout_add_local_once(std::time::Duration::from_millis(200),move || {
                                        input.upgrade().unwrap().buffer().set_text("segundo proyecto");
                                        send.upgrade().unwrap().emit_clicked();
                                    });
                                } else { eprintln!("NEXUTERM_SMOKE_WORKSPACE_OK"); }
                            }
                            #[cfg(debug_assertions)]
                            if matches!(std::env::var("NEXUTERM_SMOKE_AI").as_deref(), Ok("approval-repeat" | "approval-session" | "approval-always")) {
                                assert!(dialogs.borrow().is_empty(), "Resolved approval windows must be removed");
                                eprintln!("NEXUTERM_SMOKE_APPROVALS_FINISHED");
                            }
                            #[cfg(debug_assertions)]
                            if api_profile
                                && std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("api")
                                && status == "completed"
                            {
                                if list.observe_children().n_items() < 4 {
                                    let input = input.clone();
                                    let send = send.downgrade();
                                    glib::timeout_add_local_once(
                                        std::time::Duration::from_millis(100),
                                        move || {
                                            if let (Some(input), Some(send)) =
                                                (input.upgrade(), send.upgrade())
                                            {
                                                input.buffer().set_text("segundo-mensaje");
                                                send.emit_clicked();
                                            }
                                        },
                                    );
                                } else {
                                    eprintln!("NEXUTERM_SMOKE_AI_API_OK");
                                }
                            }

                            close_dialogs(&dialogs);
                            info.set_text(match status.as_str() {
                                "completed" => "",
                                "interrupted" => "Respuesta detenida.",
                                _ => "El turno del agente falló. Puedes volver a intentarlo.",
                            });
                            #[cfg(debug_assertions)]
                            if matches!(
                                std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
                                Ok("acp" | "switch")
                            ) && status == "completed"
                            {
                                eprintln!("NEXUTERM_SMOKE_ACP_OK");
                            }
                            #[cfg(debug_assertions)]
                            if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("codex")
                                && status == "completed"
                            {
                                if let Some(state)=s.upgrade() {
                                    let chats=(state.ai_workspace.borrow().as_ref().unwrap().capture)();
                                    let chat=&chats.tabs[chats.selected];
                                    assert_eq!(chat.codex_thread.as_deref(),Some("fixture-thread"));
                                    assert!(chat.messages.iter().any(|m|m.response && m.text=="Respuesta gráfica de Codex."));
                                    assert!(chat.messages.iter().any(|m|m.title=="Tú" && m.text.contains("Mensaje gráfico de prueba")));
                                    assert!(chat.draft.is_empty());
                                    eprintln!("NEXUTERM_SMOKE_CHAT_SNAPSHOT_OK");
                                }
                                eprintln!("NEXUTERM_SMOKE_CODEX_NATIVE_OK");
                            }
                            buffers.clear();items.clear();limit_notified=false;
                        }
                        Event::Error(error) => {
                            close_dialogs(&dialogs);
                            if !awaiting_resume.get() { message(&list, Some(&archive), "Codex · error", &error, false, false); }
                            info.set_text(&error);
                        }
                        Event::Closed => {
                            if let Some(link) = logout.upgrade() { link.set_visible(false); }
                            if let Some(button) = login_button.upgrade() { button.set_visible(!api_profile); }
                            close_dialogs(&dialogs);
                            send.set_sensitive(false);
                            spinner.stop();
                            if awaiting_resume.get() && let Some(retry)=retry.upgrade(){retry.set_visible(true);retry.set_sensitive(true);retry.set_label("Reintentar reanudación");}
                            return glib::ControlFlow::Break;
                        }
                    }
                }
                if let Some(welcome) = welcome.upgrade() {
                    welcome.set_visible(list.first_child().is_none());
                }
                if let Some(mode) = mode.upgrade() { mode.set_sensitive(!client.busy() && !awaiting_resume.get()); }
                if let Some(link) = logout.upgrade() { link.set_sensitive(!client.busy()); }
                let parked_recent=pages.upgrade().is_some_and(|p|p.visible_child_name().as_deref()==Some("recent")) && list.first_child().is_some();
                if let Some(history)=history.upgrade(){history.set_sensitive(!client.busy() && !parked_recent);}
                send.set_sensitive(ready && !client.busy() && !parked_recent && !awaiting_resume.get() && archive.ready());
                if awaiting_resume.get() && let Some(retry)=retry.upgrade(){retry.set_sensitive(!client.busy());if !client.busy(){retry.set_label("Reintentar reanudación");}}
                spinner.set_visible(client.busy());
                if client.busy() {
                    spinner.start();
                } else {
                    spinner.stop();
                }
                if let Some(input) = input.upgrade() {
                    input.set_sensitive(!client.busy() && !parked_recent);
                }
                glib::ControlFlow::Continue
            });
        })
    };
    let typing_pages=pages.downgrade();
    composer.buffer().connect_changed(move |buffer| {
        if buffer.char_count()>0 && let Some(pages)=typing_pages.upgrade(){pages.set_visible_child_name("conversation");}
    });
    let catalog = models.clone();
    let c = current.clone();
    let input = composer.downgrade();
    let list_weak = list.downgrade();
    let status = info.downgrade();
    let model_choice = model.downgrade();
    let effort_choice = effort.downgrade();
    let mode_weak = mode.downgrade();
    let paths = context.clone();
    let chips = context_row.downgrade();
    let workspace_state = Rc::downgrade(state);
    let snapshot_models = models.clone();
    let resume_guard = awaiting_resume.clone();
    let archive_send=archive.clone();
    send.connect_clicked(move |_| {
        if resume_guard.get() || !archive_send.ready() { return; }
        let (Some(input), Some(list), Some(info)) =
            (input.upgrade(), list_weak.upgrade(), status.upgrade())
        else {
            return;
        };
        let Some((_, client)) = c.borrow().clone() else {
            return;
        };
        let b = input.buffer();
        let text = b.text(&b.start_iter(), &b.end_iter(), false).to_string();
        if text.trim().is_empty() {
            return;
        }
        if text.len() > 16384 {
            info.set_text("Máximo 16 KiB por mensaje.");
            return;
        }
        let chosen=model_choice.upgrade().map(|m|m.selected() as usize).unwrap_or(0);
        let selected_model=models.borrow().get(chosen).and_then(|v|v["model"].as_str()).unwrap_or("").to_owned();
        let effort_index=effort_choice.upgrade().map(|e|e.selected() as usize).unwrap_or(0);
        let selected_effort=if effort_index==0{String::new()}else{models.borrow().get(chosen).and_then(|v|v["supportedReasoningEfforts"].as_array()).and_then(|v|v.get(effort_index-1)).and_then(|v|v["reasoningEffort"].as_str()).unwrap_or("").to_owned()};
        let context_text=paths.borrow().iter().map(|p|serde_json::to_string(&p.to_string_lossy()).unwrap_or_default()).collect::<Vec<_>>().join("\n");
        let prompt=if context_text.is_empty(){text.clone()}else{format!("{text}\n\nRutas de contexto seleccionadas por el usuario (leer solo cuando sea necesario):\n{context_text}")};
        let workspace = workspace_state.upgrade().and_then(|state|agent_workspace(&state));
        let directory = workspace.as_ref().and_then(|workspace|workspace.local_directory.as_ref()).map(|path|path.to_string_lossy().into_owned());
        let prompt = if let Some(workspace) = workspace {
            format!("{prompt}\n\nContexto de trabajo de la terminal activa (actual):\n{}\nUsa esta carpeta como base de trabajo salvo que el usuario indique otra. No elijas otro proyecto por el historial. Ejecuta los comandos desde el cwd indicado, con un argumento cwd o un cd explícito si el motor conserva otra carpeta. Si la terminal es remota, utiliza las herramientas de esa terminal; su ruta no es una carpeta local. Si no se conoce la ruta remota, consulta la terminal con los permisos correspondientes.", workspace.context)
        } else { prompt };
        let attached=paths.borrow().iter().map(|p|p.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect::<Vec<_>>().join(", ");
        if let Some(mode)=mode_weak.upgrade(){client.permissions(mode.selected()!=0);}
        if client.send_in_directory(prompt,selected_model,selected_effort,directory) {
            if let Some(mode)=mode_weak.upgrade(){mode.set_sensitive(false);}
            paths.borrow_mut().clear();if let Some(chips)=chips.upgrade(){while let Some(child)=chips.first_child(){chips.remove(&child);}chips.set_visible(false);}
            message(&list, Some(&archive_send), "Tú", &if attached.is_empty(){text.clone()}else{format!("{text}\n\nContexto: {attached}")}, false, false);
            b.set_text("");
            info.set_text("El agente está trabajando…");
        }
    });
    let send_weak = send.downgrade();
    let keys = gtk::EventControllerKey::new();
    keys.connect_key_pressed(move |_, key, _, modifiers| {
        if key == gtk::gdk::Key::Return && modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK)
        {
            if let Some(send) = send_weak.upgrade()
                && send.is_sensitive() {
                    send.emit_clicked();
                }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    composer.add_controller(keys);
    let adjustment = scroll.vadjustment();
    let follow_bottom=Rc::new(Cell::new(true));
    let pending_scroll=Rc::new(Cell::new(None::<f64>));
    let follow=follow_bottom.clone();
    let pending=pending_scroll.clone();
    adjustment.connect_value_changed(move |a| {
        if pending.get().is_none() {follow.set(a.upper()-a.page_size()-a.value()<=2.0);}
    });
    let follow=follow_bottom.clone();
    let pending=pending_scroll.clone();
    adjustment.connect_upper_notify(move |a| {
        if pending.get().is_none() && follow.get() {a.set_value((a.upper()-a.page_size()).max(0.0));}
    });
    #[cfg(debug_assertions)]
    if matches!(
        std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
        Ok("full-access" | "codex" | "codex-approval" | "approval-repeat" | "approval-session" | "approval-always" | "workspace" | "copy" | "acp" | "api" | "shutdown" | "switch")
    ) {
        let input = composer.downgrade();
        let send = send.downgrade();
        let paths = context.clone();
        let context_row = context_row.downgrade();
        let effort = effort.downgrade();
        let permissions = mode.downgrade();
        let attempts = Cell::new(0);
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            attempts.set(attempts.get() + 1);
            assert!(attempts.get() < 25, "native Codex did not become ready");
            if let (Some(input), Some(send)) = (input.upgrade(), send.upgrade())
                && send.is_sensitive()
                    && (std::env::var("NEXUTERM_SMOKE_AI").as_deref() != Ok("switch")
                        || attempts.get() > 14)
                {
                    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap())
                        .join("context.txt");
                    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("codex") {
                        paths.borrow_mut().push(path);
                    }
                    if let Some(effort) = effort.upgrade() {
                        effort.set_selected(1);
                    }
                    if let Some(row) = context_row.upgrade() {
                        row.set_visible(true);
                    }
                    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("full-access") {
                        let selector = permissions.upgrade().unwrap();
                        assert_eq!(selector.selected(), 0);
                        selector.set_selected(2);
                    }
                    input.buffer().set_text(
                        if matches!(std::env::var("NEXUTERM_SMOKE_AI").as_deref(), Ok("full-access" | "codex-approval" | "approval-repeat" | "approval-session" | "approval-always")) {
                            "approval"
                        } else if matches!(
                            std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
                            Ok("api" | "shutdown")
                        ) {
                            "primer-mensaje"
                        } else {
                            "Mensaje gráfico de prueba"
                        },
                    );
                    send.emit_clicked();
                    return glib::ControlFlow::Break;
                }
            glib::ControlFlow::Continue
        });
    }
    let snapshot: Rc<dyn Fn() -> nexuterm::workspaces::Chat> = {
        let conversation_title=conversation_title.clone();
        let adjustment=adjustment.downgrade();
        let pending_scroll=pending_scroll.clone();
        let list = list.downgrade();
        let input = composer.downgrade();
        let offline = offline.clone();
        let archive = archive.clone();
        let paths = context.clone();
        let effort = effort.downgrade();
        let continuation = continuation.clone();
        let agent = saved_agent.clone();
        let session = session.clone();
        let models = snapshot_models.clone();
        let model = model.downgrade();
        Rc::new(move || {
            fn named(widget: &gtk::Widget, name: &str) -> Option<gtk::Label> {
                if widget.widget_name() == name {
                    return widget.clone().downcast().ok();
                }
                let mut child = widget.first_child();
                while let Some(w) = child {
                    if let Some(v) = named(&w, name) {
                        return Some(v);
                    }
                    child = w.next_sibling();
                }
                None
            }
            let mut messages = Vec::new();
            if let Some(list) = list.upgrade() {
                let mut child = list.first_child();
                while let Some(card) = child {
                    if let (Some(title), Some(text)) = (
                        named(&card, "workspace-message-title"),
                        named(&card, "workspace-message-text"),
                    ) {
                        messages.push(nexuterm::workspaces::ChatMessage {
                            title: title.text().to_string(),
                            text: text.text().to_string(),
                            tool: card.has_css_class("workspace-tool"),
                            response: card.has_css_class("workspace-response"),
                        });
                    }
                    child = card.next_sibling();
                }
            }
            let scroll=pending_scroll.get().or_else(||adjustment.upgrade().map(|a| {
                let range=(a.upper()-a.page_size()-a.lower()).max(0.0);
                if range>0.0 {((a.value()-a.lower())/range).clamp(0.0,1.0)} else {1.0}
            })).filter(|v|*v<0.999);
            let attachments = paths.borrow().clone();
            let reasoning = selected_reasoning(
                &models.borrow(),
                model.upgrade().map(|m| m.selected() as usize).unwrap_or(0),
                effort.upgrade().map(|e| e.selected()).unwrap_or(0),
            );
            if let Some(mut saved) = offline.borrow().clone() {
                saved.scroll = scroll;
                saved.attachments = attachments;
                saved.reasoning = reasoning;
                saved.archive = Some(archive.id.clone());
                saved.capture_history = archive.enabled();
                saved.messages = messages;
                saved.draft = input
                    .upgrade()
                    .map(|v| {
                        let b = v.buffer();
                        b.text(&b.start_iter(), &b.end_iter(), false).to_string()
                    })
                    .unwrap_or_default();
                return saved;
            }
            let agent = agent.borrow();
            let selected_model = model.upgrade().and_then(|m| {
                models
                    .borrow()
                    .get(m.selected() as usize)
                    .and_then(|v| v["model"].as_str())
                    .map(str::to_owned)
            });
            nexuterm::workspaces::Chat {
                scroll,
                attachments,
                reasoning,
                continuation: continuation.borrow().clone(),
                archive: Some(archive.id.clone()),
                capture_history: archive.enabled(),
                id: archive.id.clone(),
                title: conversation_title.borrow().clone().unwrap_or_else(||messages.iter().find(|m|m.title=="Tú").map(|m|m.text.chars().take(100).collect()).unwrap_or_else(||"Nuevo chat".into())),
                agent_id: agent.as_ref().map(|a| a.id.clone()),
                model: selected_model
                    .or_else(|| agent.as_ref().map(|a| a.model.clone()))
                    .unwrap_or_default(),
                directory: agent
                    .as_ref()
                    .map(|a| std::path::PathBuf::from(&a.directory))
                    .filter(|p| p.is_absolute()),
                codex_thread: session.borrow().clone(),
                messages,
                draft: input
                    .upgrade()
                    .map(|v| {
                        let b = v.buffer();
                        b.text(&b.start_iter(), &b.end_iter(), false).to_string()
                    })
                    .unwrap_or_default(),
            }
        })
    };
    let restore: Rc<dyn Fn(&nexuterm::workspaces::Chat)> = {
        let conversation_title=conversation_title.clone();
        let scroll_widget=scroll.downgrade();
        let pending_scroll=pending_scroll.clone();
        let follow_bottom=follow_bottom.clone();
        let list = list.downgrade();
        let welcome = welcome.downgrade();
        let pages = pages.downgrade();
        let input = composer.downgrade();
        let notice = info.downgrade();
        let send = send.downgrade();
        let s = Rc::downgrade(state);
        let open = open.clone();
        let offline = offline.clone();
        let archive = archive.clone();
        let retry = retry.downgrade();
        let paths = context.clone();
        let chips = context_row.downgrade();
        let effort = effort.downgrade();
        let continuation = continuation.clone();
        let model_names = model_names.clone();
        let model = model.downgrade();
        let models = snapshot_models.clone();
        let pending = pending_resume.clone();
        let agent_state = saved_agent.clone();
        let session = session.clone();
        let login = login_button.downgrade();
        Rc::new(move |saved| {
            pending_scroll.set(Some(saved.scroll.unwrap_or(1.0)));
            follow_bottom.set(saved.scroll.unwrap_or(1.0)>=0.999);
            if let Some(scroll)=scroll_widget.upgrade() {
                let pending=pending_scroll.clone();
                let follow=follow_bottom.clone();
                let frames=Cell::new(0);
                scroll.add_tick_callback(move |scroll,_| {
                    // Wait until GTK has allocated the recovered message list.
                    frames.set(frames.get()+1);
                    if frames.get()<3 {return glib::ControlFlow::Continue;}
                    if let Some(position)=pending.get() {
                        let a=scroll.vadjustment();
                        let range=(a.upper()-a.page_size()-a.lower()).max(0.0);
                        a.set_value(a.lower()+range*position);
                        pending.set(None);
                        follow.set(position>=0.999);
                    }
                    glib::ControlFlow::Break
                });
            }
            *conversation_title.borrow_mut()=Some(saved.title.clone());
            *offline.borrow_mut() = Some(saved.clone());
            archive.set_enabled(saved.capture_history);
            archive.bootstrap(saved.title.clone(), &saved.messages);
            *continuation.borrow_mut() = saved.continuation.clone();
            *paths.borrow_mut() = saved.attachments.clone();
            if let Some(row) = chips.upgrade() {
                while let Some(child) = row.first_child() {
                    row.remove(&child);
                }
                row.set_visible(!saved.attachments.is_empty());
                for path in &saved.attachments {
                    attachment_chip(&row, &paths, path.clone());
                }
            }
            if let Some(welcome) = welcome.upgrade() {
                welcome.set_visible(saved.messages.is_empty());
            }
            if let Some(pages) = pages.upgrade() {
                pages.set_visible_child_name("conversation");
            }
            model_names.splice(
                0,
                model_names.n_items(),
                &[if saved.model.is_empty() {
                    "Modelo predeterminado"
                } else {
                    &saved.model
                }],
            );
            *models.borrow_mut() = vec![
                json!({"model":saved.model,"supportedReasoningEfforts":if saved.reasoning.is_empty(){vec![]}else{vec![json!({"reasoningEffort":saved.reasoning})]}}),
            ];
            if let Some(model) = model.upgrade() {
                model.set_selected(0);
                model.notify("selected");
            }
            if let Some(effort) = effort.upgrade() {
                effort.set_selected(if saved.reasoning.is_empty() { 0 } else { 1 });
            }
            let (Some(list), Some(input), Some(info), Some(send)) = (
                list.upgrade(),
                input.upgrade(),
                notice.upgrade(),
                send.upgrade(),
            ) else {
                return;
            };
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            for m in &saved.messages {
                message(&list, None, &m.title, &m.text, m.tool, m.response);
            }
            input.buffer().set_text(&saved.draft);
            send.set_sensitive(false);
            if let Some(login) = login.upgrade() {
                login.set_visible(false);
            }
            *session.borrow_mut() = saved.codex_thread.clone();
            let agent = s.upgrade().and_then(|s| {
                s.config
                    .borrow()
                    .ai_agents
                    .iter()
                    .find(|a| Some(&a.id) == saved.agent_id.as_ref())
                    .cloned()
            });
            *agent_state.borrow_mut() = agent.clone().map(|mut a| {
                a.model = saved.model.clone();
                if let Some(d) = &saved.directory {
                    a.directory = d.to_string_lossy().into_owned();
                }
                a
            });
            info.set_text(if agent.is_none(){"El perfil ya no existe. Copia local del chat; utiliza + para iniciar otra conversación."}else{"Copia local del chat. No se ha conectado ni enviado contenido. Usa + para iniciar otra conversación."});
            if let Some(mut agent) = agent.filter(|a| {
                (a.is_codex() && saved.codex_thread.is_some())
                    || saved.continuation.as_ref().is_some_and(|v| v.matches(a))
            }) {
                let Some(resume) = retry.upgrade() else {
                    return;
                };
                resume.set_label(if agent.is_codex() {
                    "Reanudar conversación de Codex"
                } else if agent.kind.is_api() {
                    "Reanudar conversación API"
                } else {
                    "Reanudar sesión ACP"
                });
                resume.set_visible(true);
                resume.set_sensitive(true);
                agent.model = saved.model.clone();
                let saved = saved.clone();
                let pending = pending.clone();
                let open = open.clone();
                let list = list.downgrade();
                let input = input.downgrade();
                let paths = paths.clone();
                let effort = effort.clone();
                let models = models.clone();
                let model = model.clone();
                let state = s.clone();
                let notice = notice.clone();
                resume.connect_clicked(move |_| {
                    let mut saved = saved.clone();
                    saved.attachments=paths.borrow().clone();
                    saved.reasoning=selected_reasoning(&models.borrow(),model.upgrade().map(|m|m.selected() as usize).unwrap_or(0),effort.upgrade().map(|e|e.selected()).unwrap_or(0));
                    if let Some(input) = input.upgrade() {
                        let b = input.buffer();
                        saved.draft = b.text(&b.start_iter(), &b.end_iter(), false).to_string();
                    }
                    let Some(mut agent)=state.upgrade().and_then(|s|s.config.borrow().ai_agents.iter().find(|a|a.id==agent.id).cloned()) else{if let Some(info)=notice.upgrade(){info.set_text("El perfil ya no existe; la copia local se conserva.");}return;};
                    if let Some(value)=&saved.continuation && !value.matches(&agent){if let Some(info)=notice.upgrade(){info.set_text("El destino del perfil ha cambiado. La copia local se conserva sin reenviar contexto.");}return;}
                    agent.model=saved.model.clone();
                    *pending.borrow_mut() = Some(saved.clone());
                    open(agent, false);
                    // Keep the offline copy until the provider confirms a successful resume.
                    if let Some(list) = list.upgrade() {
                        for m in &saved.messages {
                            message(&list, None, &m.title, &m.text, m.tool, m.response);
                        }
                    }
                    if let Some(input) = input.upgrade() {
                        input.buffer().set_text(&saved.draft);
                    }
                });
            }
        })
    };
    let recent_snapshot=snapshot.clone(); let alive=widget.downgrade();
    let recent_notice=info.downgrade();
    let writing=Rc::new(Cell::new(false)); let last=Rc::new(RefCell::new(None));
    glib::timeout_add_local(std::time::Duration::from_secs(3), move || {
        if alive.upgrade().is_none(){return glib::ControlFlow::Break;}
        let chat=recent_snapshot();
        if writing.get() || last.borrow().as_ref()==Some(&chat){return glib::ControlFlow::Continue;}
        writing.set(true);let writing=writing.clone();let last=last.clone();let notice=recent_notice.clone();
        glib::spawn_future_local(async move {
            let saved=nexuterm::chat_history::RecentSave::new(chat.clone());
            let result=nexuterm::runtime::runtime().spawn_blocking(move ||saved.save(&nexuterm::chat_history::directory()?)).await;
            match result.unwrap_or_else(|e|Err(e.to_string())) {
                Ok(())=>*last.borrow_mut()=Some(chat),
                Err(error)=>if let Some(notice)=notice.upgrade(){notice.set_text(&format!("No se pudo guardar el chat en recientes: {error}"));}
            }
            writing.set(false);
        });
        glib::ControlFlow::Continue
    });
    let close_snapshot=snapshot.clone();
    let closing_archive=archive.clone();let closing_state=Rc::downgrade(state);
    let closing_current=current.clone();
    let closing_bridge=process.bridge.clone();
    let close: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(bridge) = &closing_bridge { bridge.full_access.store(false, std::sync::atomic::Ordering::Relaxed); }
        let saved=nexuterm::chat_history::RecentSave::new(close_snapshot());
        nexuterm::runtime::runtime().spawn_blocking(move ||{let _=nexuterm::chat_history::directory().and_then(|dir|saved.save(&dir));});
        generation.set(generation.get() + 1);
        let archive=closing_archive.clone();let state=closing_state.clone();
        glib::spawn_future_local(async move{let _=archive.flush().await;if let Some(state)=state.upgrade(){state.chat_archives.borrow_mut().retain(|a|!Rc::ptr_eq(a,&archive));}});
        close_dialogs(&dialogs);
        if let Some((_, client)) = closing_current.borrow_mut().take() { client.close(); }
    });
    let show_recent:Rc<dyn Fn()>={let pages=pages.downgrade();let agent=saved_agent.clone();let refresh=refresh_recent.clone();let input=composer.downgrade();let current=current.clone();Rc::new(move ||{if current.borrow().as_ref().is_some_and(|(_,c)|c.busy()) && let Some(input)=input.upgrade(){input.set_sensitive(false);}if let Some(pages)=pages.upgrade(){pages.set_visible_child_name("recent");}if let Some(agent)=agent.borrow().as_ref(){refresh(agent.id.clone());}})};
    let activate:Rc<dyn Fn()>={let retry=retry.downgrade();Rc::new(move ||{if let Some(retry)=retry.upgrade() && retry.is_visible() && retry.is_sensitive(){retry.emit_clicked();}})};
    Chat {
        select_recent,
        activate,
        show_recent,
        archive,
        snapshot,
        restore,
        models: catalog,
        close,
        widget,
        open,
        login: login_button,
    }
}
