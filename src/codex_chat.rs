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
    pub open: Rc<dyn Fn(AiAgent, bool)>,
    pub close: Rc<dyn Fn()>,
    pub history: gtk::Box,
    pub models: Rc<RefCell<Vec<Value>>>,
    pub login: gtk::Button,
}
#[derive(Clone)]
struct UsageView {
    account: gtk::Label,
    quota: gtk::Label,
    context: gtk::Label,
    tokens: gtk::Label,
    snapshot: Rc<RefCell<Value>>,
}
impl UsageView {
    fn new(parent: &gtk::Box) -> Self {
        let view = Self { account: gtk::Label::new(Some("Usuario: no disponible")), quota: gtk::Label::new(Some("Cuota restante: no disponible")), context: gtk::Label::new(None), tokens: gtk::Label::new(None), snapshot: Rc::new(RefCell::new(Value::Null)) };
        view.account.set_widget_name("ai-account"); view.quota.set_widget_name("ai-quota");
        for label in [&view.account, &view.quota, &view.context, &view.tokens] {
            label.set_xalign(0.0); label.set_wrap(true); label.add_css_class("caption"); label.add_css_class("dim-label"); parent.append(label);
        }
        view.context.set_visible(false); view.tokens.set_visible(false); view
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
            let mut detail = String::from("Cuota restante de la cuenta comunicada por el proveedor; puede estar compartida entre modelos.");
            for w in windows { if let Some(date) = w.resets_at.and_then(|time| glib::DateTime::from_unix_local(time).ok()).and_then(|date| date.format("%d/%m %H:%M").ok()) { detail.push_str(&format!("\n{} se restablece: {date}", w.label)); } }
            self.quota.set_tooltip_text(Some(&detail));
        }
    }
}

fn message(list: &gtk::Box, title: &str, content: &str, tool: bool) -> gtk::Label {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
    card.add_css_class("card");
    card.set_margin_top(4);
    card.set_margin_bottom(4);
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("heading");
    heading.set_xalign(0.0);
    heading.set_margin_top(10);
    heading.set_margin_start(12);
    let text = gtk::Label::new(Some(&content.chars().take(65536).collect::<String>()));
    text.set_wrap(true);
    text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    text.set_selectable(true);
    text.set_xalign(0.0);
    text.set_yalign(0.0);
    text.set_margin_start(12);
    text.set_margin_end(12);
    text.set_margin_bottom(12);
    if tool {
        text.add_css_class("monospace");
    }
    card.append(&heading);
    card.append(&text);
    list.append(&card);
    while list.observe_children().n_items() > 128 {
        if let Some(child) = list.first_child() {
            list.remove(&child);
        }
    }
    text
}
fn append(label: &gtk::Label, text: &str) {
    let current = label.text();
    let remaining = 65536_usize.saturating_sub(current.chars().count());
    if remaining > 0 {
        label.set_text(&format!(
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
            "$ {}\n\n{}",
            item["command"].as_str().unwrap_or(""),
            item["aggregatedOutput"].as_str().unwrap_or("En curso…")
        ),
        "fileChange" => file_changes(item),
        "webSearch" => item["query"]
            .as_str()
            .unwrap_or("Búsqueda web en curso…")
            .to_owned(),
        "mcpToolCall" => format!(
            "{} / {}\n{}\n{}",
            item["server"].as_str().unwrap_or("MCP"),
            item["tool"].as_str().unwrap_or(""),
            serde_json::to_string_pretty(&item["arguments"]).unwrap_or_default(),
            serde_json::to_string_pretty(&item["result"]).unwrap_or_default()
        ),
        "collabAgentToolCall" => format!(
            "{} · {}",
            item["tool"].as_str().unwrap_or("Subagente"),
            item["status"].as_str().unwrap_or("En curso")
        ),
        _ => String::new(),
    }
}
fn close_dialogs(dialogs: &Dialogs) {
    for (_, (window, done)) in dialogs.borrow_mut().drain() {
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
    params: Value,
    item: Option<&Value>,
    dialogs: &Dialogs,
) {
    let approval = matches!(
        method,
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
    );
    let question = method == "item/tool/requestUserInput";
    if !approval && !question {
        client.reject_unknown(id);
        return;
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
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .title(if question {
            "Codex necesita tu respuesta"
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
            "Revisa la operación completa antes de autorizarla. La aprobación se aplica solo a esta solicitud.",
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
        } else {
            format!(
                "Cambios solicitados\n{}\n\nRaíz de escritura solicitada: {}\n{}",
                item.map(file_changes).unwrap_or_default(),
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
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
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
    let weak = window.downgrade();
    reject.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    window.connect_close_request(move |_| {
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
    if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("codex-approval") {
        let weak = window.downgrade();
        let reject = reject.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
            if let Some(window) = weak.upgrade() {
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
                if let Some(reject) = reject.upgrade() {
                    reject.emit_clicked();
                    eprintln!("NEXUTERM_SMOKE_CODEX_APPROVAL_REJECTED");
                }
            }
        });
    }
}

pub(super) fn build(state: &Rc<State>, process: &Rc<ai_panel::Process>) -> Chat {
    let login_button = gtk::Button::with_label("Conectar / iniciar sesión");
    login_button.add_css_class("flat");
    login_button.set_halign(gtk::Align::End);
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let history = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let heading = gtk::Label::new(Some("Chats recientes"));
    heading.set_xalign(0.0);
    heading.add_css_class("dim-label");
    history.append(&heading);
    widget.append(&history);
    let recent = gtk::Box::new(gtk::Orientation::Vertical, 0);
    history.append(&recent);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 12);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list)
        .build();
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&scroll));
    overlay.set_vexpand(true);
    let welcome = gtk::Box::new(gtk::Orientation::Vertical, 12);
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
    widget.append(&overlay);
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
                            chips.set_visible(true);
                            let chip = gtk::Button::with_label(&format!(
                                "{}  ×",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                            chip.set_tooltip_text(Some(&path.to_string_lossy()));
                            chip.add_css_class("flat");
                            let paths = paths.clone();
                            let row = chips.downgrade();
                            chip.connect_clicked(move |button| {
                                paths.borrow_mut().retain(|p| p != &path);
                                if let Some(row) = row.upgrade() {
                                    row.remove(button);
                                    row.set_visible(!paths.borrow().is_empty());
                                }
                            });
                            chips.append(&chip);
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
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    footer.append(&gtk::Image::from_icon_name("computer-symbolic"));
    let mode =
        gtk::DropDown::from_strings(&["Local · solo lectura", "Local · solicitar aprobación"]);
    mode.add_css_class("flat");
    mode.set_tooltip_text(Some("Permisos para la nueva conversación"));
    footer.append(&mode);
    widget.append(&footer);
    let usage = UsageView::new(&widget);
    let browser = gtk::LinkButton::with_label("about:blank", "Abrir inicio de sesión de ChatGPT");
    browser.set_visible(false);
    widget.append(&browser);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let spinner = gtk::Spinner::new();
    let info = gtk::Label::new(Some("Chat de IA"));
    info.set_wrap(true);
    info.set_hexpand(true);
    info.set_xalign(0.0);
    info.add_css_class("dim-label");
    info.add_css_class("caption");
    row.append(&spinner);
    row.append(&info);
    widget.append(&row);
    let current = Rc::new(RefCell::new(None::<(String, Rc<Client>)>));
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
    let open: Rc<dyn Fn(AiAgent, bool)> = {
        let usage = usage.clone();
        let welcome = welcome.downgrade();
        let recent = recent.downgrade();
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
        let effort = effort.downgrade();
        Rc::new(move |mut agent, login| {
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
            let (client, mut events) = Client::start(agent.clone(), mode.selected() == 1);
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
            let mut buffers: HashMap<String, gtk::Label> = HashMap::new();
            let mut items: HashMap<String, Value> = HashMap::new();
            let welcome = welcome.clone();
            let agent_name = agent.name.clone();
            let api_profile = agent.kind.is_api();
            let codex_profile = agent.is_codex();
            let configured_models = agent.models.clone();
            let preferred_model = agent.model.clone();
            let login_button = login_button.clone();
            let reasoning_selector = effort.clone();
            let usage = usage.clone();
            let mut ready = false;
            let mut provider_authenticated = false;
            let recent = recent.clone();
            let models = models.clone();
            let model = model.clone();
            let model_names = model_names.clone();
            let mode = mode.downgrade();
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
                    let event = match events.try_recv() {
                        Ok(event) => event,
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(_) => return glib::ControlFlow::Break,
                    };
                    match event {
                        Event::Authenticated(value) => {
                            provider_authenticated = value;
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
                                usage.context.set_text(&format!("Contexto restante: {remaining:.0}%"));
                                usage.context.set_tooltip_text(Some(&format!("{used} de {size} tokens de contexto. No es la cuota de la cuenta.")));
                                usage.context.set_visible(true);
                            } else { usage.context.set_visible(false); }
                        }
                        Event::TokenUsage {input,output} => {
                            usage.tokens.set_text(&format!("Última petición: {input} tokens de entrada · {output} de salida")); usage.tokens.set_visible(true);
                        }
                        Event::Capabilities {
                            tools,
                            history: has_history,
                            reasoning,
                        } => {
                            if let Some(mode) = mode.upgrade() {
                                mode.set_sensitive(tools);
                                mode.set_tooltip_text(Some(if tools{"Permisos del agente"}else{"Este proveedor API solo intercambia texto; las extensiones se conservan en el perfil."}));
                            }
                            if let Some(recent) = recent.upgrade()
                                && let Some(parent) = recent.parent() {
                                    parent.set_visible(has_history);
                                }
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
                                }
                            }
                        }
                        Event::History(threads) => {
                            if let Some(recent) = recent.upgrade() {
                                while let Some(child) = recent.first_child() {
                                    recent.remove(&child);
                                }
                                if threads.is_empty() {
                                    let label = gtk::Label::new(Some(
                                        "Todavía no hay chats en esta carpeta",
                                    ));
                                    label.set_xalign(0.0);
                                    label.add_css_class("dim-label");
                                    label.add_css_class("caption");
                                    recent.append(&label);
                                }
                                for thread in threads.into_iter().take(3) {
                                    let title = thread["name"]
                                        .as_str()
                                        .filter(|s| !s.is_empty())
                                        .or(thread["preview"].as_str())
                                        .unwrap_or("Conversación");
                                    let button = gtk::Button::with_label(title);
                                    button.add_css_class("flat");
                                    if let Some(label) = button.child().and_downcast::<gtk::Label>()
                                    {
                                        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                                        label.set_xalign(0.0);
                                    }
                                    let c = client.clone();
                                    let id = thread["id"].as_str().unwrap_or("").to_owned();
                                    let mode = mode.clone();
                                    button.connect_clicked(move |_| {
                                        if let Some(mode) = mode.upgrade() {
                                            c.permissions(mode.selected() == 1);
                                        }
                                        c.resume(id.clone());
                                    });
                                    recent.append(&button);
                                }
                            }
                        }
                        Event::Resumed(thread) => {
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
                                                message(
                                                    &list,
                                                    "Codex",
                                                    item["text"].as_str().unwrap_or(""),
                                                    false,
                                                );
                                            } else if item["type"] == "userMessage"
                                                && let Some(content) = item["content"].as_array() {
                                                    for text in content {
                                                        if let Some(text) = text["text"].as_str() {
                                                            message(&list, "Tú", text, false);
                                                        }
                                                    }
                                                }
                                        }
                                    }
                                }
                            }
                            info.set_text("Conversación recuperada.");
                            if let Some(mode) = mode.upgrade() {
                                mode.set_sensitive(false);
                            }
                        }
                        Event::Ready(authenticated) => {
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
                            info.set_text(if ready {
                                "Agente listo. Escribe tu mensaje."
                            } else {
                                "Conecta o autentica este agente para conversar."
                            });
                            browser.set_visible(false);
                            browser.set_uri("about:blank");
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
                                client.interrupt();
                                continue;
                            }
                            let buffer = buffers
                                .entry(item)
                                .or_insert_with(|| message(&list, &agent_name, "", false));
                            append(buffer, &text);
                        }
                        Event::Item(item) => {
                            let kind = item["type"].as_str().unwrap_or("");
                            if kind == "requestResolved" {
                                if let Some((window, done)) =
                                    dialogs.borrow_mut().remove(&item["requestId"].to_string())
                                {
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
                                client.interrupt();
                                continue;
                            }
                            if kind == "agentMessage" {
                                let text = item["text"].as_str().unwrap_or("");
                                if !text.is_empty() {
                                    let b = buffers
                                        .entry(id.clone())
                                        .or_insert_with(|| message(&list, &agent_name, "", false));
                                    b.set_text(&text.chars().take(65536).collect::<String>());
                                }
                            } else if matches!(
                                kind,
                                "commandExecution"
                                    | "fileChange"
                                    | "mcpToolCall"
                                    | "webSearch"
                                    | "collabAgentToolCall"
                            ) {
                                if buffers.len() >= 128 && !buffers.contains_key(&id) {
                                    client.interrupt();
                                    continue;
                                }
                                let b = buffers.entry(id.clone()).or_insert_with(|| {
                                    message(
                                        &list,
                                        match kind {
                                            "commandExecution" => "Comando",
                                            "fileChange" => "Cambios en archivos",
                                            "mcpToolCall" => "Herramienta MCP",
                                            "collabAgentToolCall" => "Subagente",
                                            _ => "Búsqueda",
                                        },
                                        "",
                                        true,
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
                                );
                            } else {
                                client.reject_unknown(id);
                            }
                        }
                        Event::Finished(status) => {
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
                                "completed" => "Respuesta completada.",
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
                                eprintln!("NEXUTERM_SMOKE_CODEX_NATIVE_OK");
                            }
                        }
                        Event::Error(error) => {
                            close_dialogs(&dialogs);
                            message(&list, "Codex · error", &error, false);
                            info.set_text(&error);
                        }
                        Event::Closed => {
                            if let Some(button) = login_button.upgrade() { button.set_visible(!api_profile); }
                            close_dialogs(&dialogs);
                            send.set_sensitive(false);
                            spinner.stop();
                            return glib::ControlFlow::Break;
                        }
                    }
                }
                if let Some(welcome) = welcome.upgrade() {
                    welcome.set_visible(list.first_child().is_none());
                }
                send.set_sensitive(ready && !client.busy());
                if client.busy() {
                    spinner.start();
                } else {
                    spinner.stop();
                }
                if let Some(input) = input.upgrade() {
                    input.set_sensitive(!client.busy());
                }
                glib::ControlFlow::Continue
            });
        })
    };
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
    send.connect_clicked(move |_| {
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
        let attached=paths.borrow().iter().map(|p|p.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect::<Vec<_>>().join(", ");
        if let Some(mode)=mode_weak.upgrade(){client.permissions(mode.selected()==1);}
        if client.send_with_options(prompt,selected_model,selected_effort) {
            if let Some(mode)=mode_weak.upgrade(){mode.set_sensitive(false);}
            paths.borrow_mut().clear();if let Some(chips)=chips.upgrade(){while let Some(child)=chips.first_child(){chips.remove(&child);}chips.set_visible(false);}
            message(&list, "Tú", &if attached.is_empty(){text.clone()}else{format!("{text}\n\nContexto: {attached}")}, false);
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
    adjustment.connect_upper_notify(|a| a.set_value((a.upper() - a.page_size()).max(0.0)));
    #[cfg(debug_assertions)]
    if matches!(
        std::env::var("NEXUTERM_SMOKE_AI").as_deref(),
        Ok("codex" | "codex-approval" | "acp" | "api" | "shutdown" | "switch")
    ) {
        let input = composer.downgrade();
        let send = send.downgrade();
        let paths = context.clone();
        let context_row = context_row.downgrade();
        let effort = effort.downgrade();
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
                    input.buffer().set_text(
                        if std::env::var("NEXUTERM_SMOKE_AI").as_deref() == Ok("codex-approval") {
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
    let close: Rc<dyn Fn()> = Rc::new(move || {
        generation.set(generation.get() + 1);
        close_dialogs(&dialogs);
        if let Some((_, client)) = current.borrow_mut().take() { client.close(); }
    });
    Chat {
        models: catalog,
        close,
        widget,
        open,
        history,
        login: login_button,
    }
}
