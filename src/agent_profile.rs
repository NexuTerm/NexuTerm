//! Agent-owned runtime configuration. User/global CLI files are never changed.
use crate::{ai_config::McpServer, config::AiAgent};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};
pub fn root(agent: &AiAgent) -> PathBuf {
    if let Some(root) = agent.runtime_root.as_ref().filter(|p| p.is_absolute()) {
        return root.join(format!("{:x}", Sha256::digest(agent.id.as_bytes())));
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        });
    base.join("nexuterm/agents")
        .join(format!("{:x}", Sha256::digest(agent.id.as_bytes())))
}
fn private_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| "No se pudo crear configuración privada")?;
    file.write_all(bytes)
        .map_err(|_| "No se pudo escribir configuración privada")?;
    std::fs::rename(&temporary, path).map_err(|_| "No se pudo guardar configuración privada".into())
}
fn toml(value: &Value) -> String {
    match value {
        Value::Object(map) => format!(
            "{{{}}}",
            map.iter()
                .map(|(k, v)| format!("{} = {}", json!(k), toml(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(toml).collect::<Vec<_>>().join(", ")
        ),
        _ => value.to_string(),
    }
}
pub fn mcp(agent: &AiAgent) -> Value {
    let mut servers = serde_json::Map::new();
    for (name, server) in &agent.opencode.mcp {
        match server {
            McpServer::Local {
                command,
                enabled: true,
                environment,
            } => {
                servers.insert(
                    name.clone(),
                    json!({"command":command[0],"args":command[1..],"env":environment}),
                );
            }
            McpServer::Remote {
                url,
                enabled: true,
                headers,
                ..
            } => {
                servers.insert(name.clone(), json!({"url":url,"http_headers":headers}));
            }
            _ => {}
        }
    }
    if agent.terminal_access
        && let (Some(socket), Ok(exe)) = (&agent.terminal_socket, std::env::current_exe()) {
            servers.insert(
                "nexuterm-terminals".into(),
                json!({"command":exe,"args":["--terminal-mcp",socket,&agent.name]}),
            );
        }
    Value::Object(servers)
}
fn skill_folders(agent: &AiAgent) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for path in &agent.opencode.skills.paths {
        let path = PathBuf::from(path);
        if path.join("SKILL.md").is_file() {
            result.push(path);
        } else if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten().take(128) {
                if entry.path().join("SKILL.md").is_file() {
                    result.push(entry.path());
                }
            }
        }
    }
    result.truncate(128);
    result
}
pub fn prepare(agent: &AiAgent) -> Result<PathBuf, String> {
    let root = root(agent);
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&root)
        .map_err(|_| "No se pudo crear el almacenamiento privado del agente")?;
    // The directories are private to this profile, including native SDK discovery paths.
    for target in [root.join("claude/skills"), root.join(".gemini/skills")] {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&target)
            .map_err(|_| "No se pudo preparar skills privadas")?;
        // Remove only links created in this generated directory; never touch source skills.
        for entry in std::fs::read_dir(&target)
            .map_err(|_| "No se pudieron leer skills privadas")?
            .flatten()
        {
            if entry.file_type().is_ok_and(|t| t.is_symlink()) {
                std::fs::remove_file(entry.path())
                    .map_err(|_| "No se pudo actualizar una skill privada")?;
            }
        }
        for source in skill_folders(agent) {
            let name = format!(
                "skill-{:x}",
                Sha256::digest(source.to_string_lossy().as_bytes())
            );
            std::os::unix::fs::symlink(&source, target.join(name))
                .map_err(|_| "No se pudo preparar una skill privada")?;
        }
    }
    for directory in [root.join("claude/agents"), root.join(".gemini/agents")] {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)
            .map_err(|_| "No se pudieron preparar subagentes privados")?;
        for entry in std::fs::read_dir(&directory)
            .map_err(|_| "No se pudieron actualizar subagentes privados")?
            .flatten()
        {
            if entry.file_name().to_string_lossy().starts_with("nexuterm-")
                && entry.path().extension().is_some_and(|e| e == "md")
            {
                std::fs::remove_file(entry.path())
                    .map_err(|_| "No se pudo actualizar un subagente privado")?;
            }
        }
        for (name, definition) in &agent.opencode.agent {
            let body = format!(
                "---\nname: {}\ndescription: {}\n{}---\n\n{}\n",
                json!(name),
                json!(definition.description),
                if definition.model.is_empty() {
                    String::new()
                } else {
                    format!("model: {}\n", json!(definition.model))
                },
                definition.prompt
            );
            private_write(
                &directory.join(format!("nexuterm-{name}.md")),
                body.as_bytes(),
            )?;
        }
    }
    let mut config = json!({"mcp_servers":mcp(agent),"skills":{"config":skill_folders(agent).iter().map(|p|json!({"path":p,"enabled":true})).collect::<Vec<_>>()}});
    let mut roles = serde_json::Map::new();
    for (name, role) in &agent.opencode.agent {
        let file = root.join(format!("role-{name}.toml"));
        let mut layer = format!("developer_instructions = {}\n", json!(role.prompt));
        if !role.model.is_empty() {
            layer.push_str(&format!("model = {}\n", json!(role.model)));
        }
        private_write(&file, layer.as_bytes())?;
        roles.insert(
            name.clone(),
            json!({"description":role.description,"config_file":file}),
        );
    }
    if !roles.is_empty() {
        config["agents"] = Value::Object(roles);
    }
    let contents = config
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| format!("{key} = {}\n", toml(value)))
        .collect::<String>();
    private_write(&root.join("config.toml"), contents.as_bytes())?;
    private_write(
        &root.join("extensions.json"),
        &serde_json::to_vec_pretty(&agent.opencode).map_err(|_| "Configuración inválida")?,
    )?;
    Ok(root)
}
pub fn isolated(command: &mut tokio::process::Command, root: &Path) {
    for name in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
        "DEEPSEEK_API_KEY",
        "OPENCODE_API_KEY",
        "COPILOT_GITHUB_TOKEN",
        "GH_TOKEN",
        "GITHUB_TOKEN",
        "OPENCODE_CONFIG",
        "OPENCODE_CONFIG_DIR",
    ] {
        command.env_remove(name);
    }
    command
        .env("HOME", root)
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("CODEX_HOME", root)
        .env("COPILOT_HOME", root.join(".copilot"))
        .env("CLAUDE_CONFIG_DIR", root.join("claude"))
        .env("GEMINI_CLI_HOME", root);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_distinct_paths_and_safe_toml() {
        let a = AiAgent {
            id: "../one".into(),
            ..Default::default()
        };
        let b = AiAgent {
            id: "two".into(),
            ..Default::default()
        };
        assert_ne!(root(&a), root(&b));
        assert_eq!(root(&a), root(&a));
        assert!(!root(&a).to_string_lossy().contains("../one"));
        assert_eq!(
            toml(&json!({"x":"hello\nworld"})),
            "{\"x\" = \"hello\\nworld\"}"
        );
    }
}
