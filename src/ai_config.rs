//! Profile-scoped OpenCode extensions. No external configuration files are changed.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiKind {
    #[default]
    Local,
    OpenCode,
    Codex,
    Acp,
    Responses,
    ChatCompletions,
}
impl AiKind {
    pub fn is_api(self) -> bool {
        matches!(self, Self::Responses | Self::ChatCompletions)
    }
}

pub fn validate_url(value: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "URL inválida")?;
    let local = match url.host() {
        Some(url::Host::Domain("localhost")) => true,
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if url.host().is_none()
        || !(url.scheme() == "https" || (url.scheme() == "http" && local))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || value.chars().any(|c| c.is_control())
    {
        return Err(
            "Usa HTTPS, o HTTP en loopback, sin credenciales, parámetros ni fragmento".into(),
        );
    }
    Ok(url)
}

fn enabled() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum McpServer {
    Local {
        command: Vec<String>,
        #[serde(default = "enabled")]
        enabled: bool,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        environment: BTreeMap<String, String>,
    },
    Remote {
        url: String,
        #[serde(default = "enabled")]
        enabled: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        oauth: Option<bool>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        headers: BTreeMap<String, String>,
    },
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    #[default]
    Primary,
    Subagent,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgentDefinition {
    pub description: String,
    pub mode: AgentMode,
    pub prompt: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub model: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Skills {
    pub paths: Vec<String>,
}
impl Skills {
    fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpenCodeConfig {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp: BTreeMap<String, McpServer>,
    #[serde(skip_serializing_if = "Skills::is_empty")]
    pub skills: Skills,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub agent: BTreeMap<String, AgentDefinition>,
}
impl OpenCodeConfig {
    pub fn is_empty(&self) -> bool {
        self.mcp.is_empty() && self.skills.paths.is_empty() && self.agent.is_empty()
    }
    pub fn validate(&self) -> Result<(), String> {
        fn valid_name(name: &str) -> bool {
            !name.is_empty()
                && name.len() <= 80
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        }
        if self.mcp.len() > 32 || self.agent.len() > 32 || self.skills.paths.len() > 32 {
            return Err("Máximo 32 servidores, agentes y rutas de skills por perfil".into());
        }
        for (name, server) in &self.mcp {
            if !valid_name(name) {
                return Err("Nombre MCP inválido: usa letras, números, guion o guion bajo".into());
            }
            match server {
                McpServer::Local {
                    command,
                    environment,
                    ..
                } => {
                    if command.is_empty()
                        || command[0].trim().is_empty()
                        || command.iter().any(|arg| arg.contains('\0'))
                        || environment.iter().any(|(k, v)| {
                            k.is_empty() || k.contains(['\0', '=']) || v.contains('\0')
                        })
                    {
                        return Err("Comando o entorno MCP local inválido".into());
                    }
                }
                McpServer::Remote { url, headers, .. } => {
                    validate_url(url)?;
                    if headers.iter().any(|(k, v)| {
                        k.is_empty()
                            || k.contains(['\r', '\n', '\0'])
                            || v.contains(['\r', '\n', '\0'])
                    }) {
                        return Err("Cabeceras MCP inválidas".into());
                    }
                }
            }
        }
        for path in &self.skills.paths {
            if !Path::new(path).is_absolute() || path.contains('\0') {
                return Err("Las rutas de skills deben ser absolutas".into());
            }
        }
        for (name, agent) in &self.agent {
            if !valid_name(name)
                || agent.description.trim().is_empty()
                || agent.prompt.trim().is_empty()
            {
                return Err(
                    "Cada agente o subagente necesita nombre válido, descripción e instrucciones"
                        .into(),
                );
            }
        }
        let size = serde_json::to_vec(self)
            .map_err(|_| "Extensiones inválidas")?
            .len();
        if size > 64 * 1024 {
            return Err("Extensiones demasiado grandes (máximo 64 KiB)".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_security() {
        for url in [
            "https://example.invalid/v1",
            "http://127.0.0.1:11434/v1",
            "http://[::1]:8080/v1",
        ] {
            assert!(validate_url(url).is_ok(), "{url}");
        }
        for url in [
            "http://remote.invalid/v1",
            "https://user:secret@example.invalid",
            "file:///tmp/key",
            "https://example.invalid?key=x",
            "https://example.invalid/#key",
        ] {
            assert!(validate_url(url).is_err(), "{url}");
        }
    }
    #[test]
    fn extensions_match_native_shape_and_validate() {
        assert_eq!(
            serde_json::to_value(OpenCodeConfig::default()).unwrap(),
            serde_json::json!({})
        );
        let config: OpenCodeConfig = serde_json::from_str(r#"{"mcp":{"demo":{"type":"local","command":["tool","arg with spaces"],"enabled":false}},"skills":{"paths":["/tmp/skills"]},"agent":{"review":{"description":"Review","mode":"subagent","prompt":"Review code"}}}"#).unwrap();
        config.validate().unwrap();
        assert_eq!(
            serde_json::to_value(&config).unwrap()["agent"]["review"]["mode"],
            "subagent"
        );
        let invalid: OpenCodeConfig =
            serde_json::from_str(r#"{"mcp":{"demo":{"type":"local","command":[]}}}"#).unwrap();
        assert!(invalid.validate().is_err());
        assert!(serde_json::from_str::<OpenCodeConfig>(r#"{"unexpected":true}"#).is_err());
    }
}
