//! Opt-in live connection probe: minimal inference, no terminal tools or project access.
use nexuterm::{
    ai_config::AiKind,
    codex::{Client, Event},
    config::{self, AiAgent},
};
use serde_json::json;
use std::time::Duration;
async fn probe(mut agent: AiAgent, models: usize) {
    let workspace = tempfile::tempdir().unwrap();
    agent.directory = workspace.path().to_string_lossy().into();
    agent.terminal_access = false;
    agent.wait_credentials = false;
    let (client, mut events) = Client::start(agent.clone(), false);
    client.credentials(Ok(zeroize::Zeroizing::new(String::new())));
    let result = tokio::time::timeout(Duration::from_secs(140), async {
        let mut catalog = Vec::new();
        let mut selected = Vec::new();
        let mut completed = 0;
        let mut received = false;
        loop {
            match events.recv().await {
                Some(Event::Models(values)) => catalog = values,
                Some(Event::Ready(false)) => {
                    println!("{}: SIN SESIÓN AUTENTICADA", agent.name);
                    return;
                }
                Some(Event::Ready(true)) => {
                    println!("{}: CONECTADO; catálogo={}", agent.name, catalog.len());
                    selected = if agent.kind == AiKind::OpenCode {
                        catalog
                            .iter()
                            .filter_map(|v| v["model"].as_str())
                            .filter(|id| {
                                id.starts_with("opencode/")
                                    && (id.ends_with("-free") || *id == "opencode/big-pickle")
                            })
                            .take(models)
                            .map(String::from)
                            .collect()
                    } else {
                        vec![agent.model.clone()]
                    };
                    if selected.is_empty() {
                        println!("{}: SIN MODELO GRATUITO", agent.name);
                        return;
                    }
                    client.send_with_options(
                        "Responde únicamente OK. No uses herramientas ni leas archivos.".into(),
                        selected[0].clone(),
                        String::new(),
                    );
                }
                Some(Event::Delta { text, .. }) => received |= !text.trim().is_empty(),
                Some(Event::Request { id, .. }) => client.reply(id, json!({"decision":"decline"})),
                Some(Event::Error(error)) => {
                    println!("{}: ERROR {error}", agent.name);
                    return;
                }
                Some(Event::Finished(status)) => {
                    println!(
                        "{} / {}: estado={status}; texto_recibido={received}",
                        agent.name,
                        selected
                            .get(completed)
                            .map(String::as_str)
                            .unwrap_or("predeterminado")
                    );
                    completed += 1;
                    if completed == selected.len() {
                        return;
                    }
                    received = false;
                    client.send_with_options(
                        "Responde únicamente OK. No uses herramientas.".into(),
                        selected[completed].clone(),
                        String::new(),
                    );
                }
                Some(Event::Closed) | None => {
                    println!("{}: CONEXIÓN CERRADA", agent.name);
                    return;
                }
                _ => {}
            }
        }
    })
    .await;
    if result.is_err() {
        client.interrupt();
        println!("{}: TIEMPO AGOTADO (140 s)", agent.name);
    }
    client.close();
}
#[tokio::main]
async fn main() {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config")
        })
        .join("nexuterm/sessions.json");
    if let Ok(config) = config::load(&path) {
        for agent in config.ai_agents {
            if agent.is_codex() {
                probe(agent, 1).await;
            }
        }
    }
    let private = tempfile::tempdir().unwrap();
    probe(
        AiAgent {
            id: "live-connection-probe".into(),
            name: "OpenCode gratuito".into(),
            kind: AiKind::OpenCode,
            program: "opencode".into(),
            runtime_root: Some(private.path().into()),
            ..Default::default()
        },
        2,
    )
    .await;
    probe(
        AiAgent {
            id: "ollama-live-probe".into(),
            name: "Ollama local".into(),
            kind: AiKind::ChatCompletions,
            endpoint: "http://127.0.0.1:11434/v1".into(),
            model: "qwen2.5-coder:1.5b-base".into(),
            max_output_tokens: 16,
            timeout_seconds: 60,
            ..Default::default()
        },
        1,
    )
    .await;
}
