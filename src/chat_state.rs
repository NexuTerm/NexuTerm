//! Portable conversation references and completed API context; never pending operations.
use crate::config::AiAgent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Continuation {
    Api {
        destination: String,
        messages: Vec<Value>,
    },
    Acp {
        destination: String,
        session_id: String,
    },
}
impl Continuation {
    pub fn validate(&self) -> Result<(), String> {
        let destination = match self {
            Self::Api {
                destination,
                messages,
            } => {
                validate_api(messages)?;
                destination
            }
            Self::Acp {
                destination,
                session_id,
            } => {
                if session_id.is_empty()
                    || session_id.len() > 4096
                    || session_id.chars().any(char::is_control)
                {
                    return Err("Identificador ACP inválido".into());
                }
                destination
            }
        };
        if destination.len() != 64 || !destination.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Destino de conversación inválido".into());
        }
        Ok(())
    }
    pub fn matches(&self, agent: &AiAgent) -> bool {
        let expected = destination(agent);
        match self {
            Self::Api { destination, .. } => agent.kind.is_api() && destination == &expected,
            Self::Acp { destination, .. } => {
                !agent.kind.is_api() && !agent.is_codex() && destination == &expected
            }
        }
    }
}
pub fn destination(agent: &AiAgent) -> String {
    let value = serde_json::json!([
        agent.id,
        agent.kind,
        agent.program,
        agent.args,
        agent.endpoint
    ]);
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}
pub fn validate_api(messages: &[Value]) -> Result<(), String> {
    if messages.is_empty()
        || messages.len() > 129
        || serde_json::to_vec(messages)
            .map_err(|e| e.to_string())?
            .len()
            > 2 * 1024 * 1024
    {
        return Err(
            "Contexto API inválido o superior al límite de recuperación de 2 MiB/129 entradas"
                .into(),
        );
    }
    if messages.first().and_then(|m| m["role"].as_str()) != Some("user")
        || messages.last().and_then(|m| m["role"].as_str()) != Some("assistant")
    {
        return Err("Solo se reanudan intercambios API completados".into());
    }
    for message in messages {
        if !message.is_object()
            || !(matches!(
                message["role"].as_str(),
                Some("user" | "assistant" | "tool")
            ) || matches!(
                message["type"].as_str(),
                Some("function_call" | "function_call_output" | "reasoning" | "message")
            ))
        {
            return Err("Entrada de contexto API desconocida".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_context_and_destination_are_required() {
        let mut agent = AiAgent {
            kind: crate::ai_config::AiKind::Responses,
            endpoint: "https://fixture.invalid/v1".into(),
            ..Default::default()
        };
        let state = Continuation::Api {
            destination: destination(&agent),
            messages: vec![
                serde_json::json!({"role":"user","content":"Hola"}),
                serde_json::json!({"role":"assistant","content":"Respuesta"}),
            ],
        };
        state.validate().unwrap();
        assert!(state.matches(&agent));
        agent.endpoint = "https://other.invalid/v1".into();
        assert!(!state.matches(&agent));
        let unfinished = Continuation::Api {
            destination: destination(&agent),
            messages: vec![serde_json::json!({"role":"user","content":"No completado"})],
        };
        assert!(unfinished.validate().is_err());
    }
}
