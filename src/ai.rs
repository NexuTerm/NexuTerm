//! Bounded, cancellable text-only API transport. curl is already a runtime dependency.
use crate::{
    ai_config::{AiKind, validate_url},
    config::AiAgent,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

pub const MAX_CONTEXT: usize = 256 * 1024;
pub const MAX_MESSAGES: usize = 64;
const MAX_RESPONSE: usize = 1024 * 1024;
pub const OPENCODE_MODELS_URL: &str = "https://opencode.ai/zen/v1/models";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

pub fn request_body(agent: &AiAgent, history: &[Message]) -> Result<Value, String> {
    agent.validate()?;
    if agent.model.trim().is_empty() {
        return Err("Selecciona un modelo antes de enviar".into());
    }
    if !agent.kind.is_api() {
        return Err("El perfil no utiliza una API".into());
    }
    if history.is_empty()
        || history.len() > MAX_MESSAGES
        || history.iter().map(|m| m.content.len()).sum::<usize>() + agent.system_prompt.len()
            > MAX_CONTEXT
        || history
            .iter()
            .any(|m| !matches!(m.role.as_str(), "user" | "assistant"))
    {
        return Err("Conversación fuera de límites. Inicia una nueva conversación (máximo 64 mensajes o 256 KiB).".into());
    }
    let mut messages = history.to_vec();
    if !agent.system_prompt.is_empty() {
        messages.insert(
            0,
            Message {
                role: "system".into(),
                content: agent.system_prompt.clone(),
            },
        );
    }
    Ok(match agent.kind {
        AiKind::Responses => json!({
            "model": agent.model, "input": messages,
            "max_output_tokens": agent.max_output_tokens, "store": false, "stream": false,
        }),
        AiKind::ChatCompletions => json!({
            "model": agent.model, "messages": messages,
            "max_tokens": agent.max_output_tokens, "stream": false,
        }),
        _ => unreachable!(),
    })
}

pub fn parse_response(kind: AiKind, value: &Value) -> Result<String, String> {
    let mut text = String::new();
    match kind {
        AiKind::Responses => {
            if matches!(value["status"].as_str(), Some("failed" | "cancelled"))
                || !value["error"].is_null()
            {
                return Err("El proveedor no pudo completar la respuesta".into());
            }
            if let Some(output) = value["output"].as_array() {
                for item in output {
                    if item["type"] != "message" {
                        continue;
                    }
                    if let Some(content) = item["content"].as_array() {
                        for part in content {
                            let field = match part["type"].as_str() {
                                Some("output_text") => "text",
                                Some("refusal") => "refusal",
                                _ => continue,
                            };
                            if let Some(value) = part[field].as_str() {
                                text.push_str(value);
                            }
                        }
                    }
                }
            }
            if value["status"] == "incomplete" && !text.is_empty() {
                text.push_str("\n\n[Respuesta incompleta: se alcanzó un límite del proveedor.]");
            }
        }
        AiKind::ChatCompletions => {
            let choice = &value["choices"][0];
            if let Some(value) = choice["message"]["content"]
                .as_str()
                .or(choice["message"]["refusal"].as_str())
            {
                text.push_str(value);
            }
            if choice["finish_reason"] == "length" && !text.is_empty() {
                text.push_str("\n\n[Respuesta incompleta: se alcanzó el límite de tokens.]");
            }
        }
        _ => return Err("Tipo de respuesta inválido".into()),
    }
    if text.trim().is_empty() {
        return Err(
            "El proveedor no devolvió texto. Este chat no ejecuta llamadas a herramientas.".into(),
        );
    }
    Ok(text)
}

pub fn validate_key(key: &str) -> Result<(), String> {
    if key.len() > 8192 || key.chars().any(|c| c.is_control()) {
        return Err("La clave API contiene caracteres inválidos o es demasiado larga".into());
    }
    Ok(())
}
fn quoted(value: &str) -> Result<String, String> {
    if value.chars().any(|c| matches!(c, '\0' | '\r' | '\n')) {
        return Err("Datos de transporte inválidos".into());
    }
    Ok(format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

async fn http_json(
    url: String,
    key: Zeroizing<String>,
    body: Option<Value>,
    timeout: u32,
) -> Result<Value, String> {
    validate_url(&url)?;
    validate_key(&key)?;
    let mut config = Zeroizing::new(format!("url = {}\n", quoted(&url)?));
    if !key.is_empty() {
        config.push_str(&format!(
            "header = {}\n",
            quoted(&format!("Authorization: Bearer {}", *key))?
        ));
    }
    if let Some(body) = body {
        let data = serde_json::to_string(&body).map_err(|_| "No se pudo preparar la solicitud")?;
        config.push_str("header = \"Content-Type: application/json\"\n");
        config.push_str(&format!("data = {}\n", quoted(&data)?));
    }
    let curl = crate::connection::find_program(&["curl"])
        .ok_or("Instala curl para usar proveedores API")?;
    let mut child = tokio::process::Command::new(curl)
        .args([
            "--disable",
            "--config",
            "-",
            "--silent",
            "--globoff",
            "--proto",
            "=http,https",
            "--connect-timeout",
            "10",
            "--max-time",
            &timeout.to_string(),
            "--max-redirs",
            "0",
            "--write-out",
            "\n%{http_code}",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "No se pudo iniciar el transporte API")?;
    let mut input = child
        .stdin
        .take()
        .ok_or("Sin canal privado de transporte")?;
    input
        .write_all(config.as_bytes())
        .await
        .map_err(|_| "No se pudo enviar la solicitud")?;
    drop(input);
    let mut output = Vec::new();
    child
        .stdout
        .take()
        .ok_or("Sin canal de respuesta")?
        .take(MAX_RESPONSE as u64 + 1)
        .read_to_end(&mut output)
        .await
        .map_err(|_| "No se pudo leer la respuesta")?;
    if output.len() > MAX_RESPONSE {
        return Err("Respuesta demasiado grande (máximo 1 MiB)".into());
    }
    let status = child
        .wait()
        .await
        .map_err(|_| "No se pudo finalizar el transporte")?;
    if !status.success() {
        return Err(if status.code() == Some(28) {
            "Se agotó el tiempo de espera de la API"
        } else {
            "Error de red o TLS. Comprueba la URL y la conexión"
        }
        .into());
    }
    let split = output
        .iter()
        .rposition(|&b| b == b'\n')
        .ok_or("Respuesta HTTP inválida")?;
    let code = std::str::from_utf8(&output[split + 1..]).unwrap_or("");
    if code != "200" {
        return Err(match code {
            "401" | "403" => {
                "Autenticación o acceso denegado. Revisa la clave y los permisos".into()
            }
            "429" => "Límite de uso o cuota agotada. No se ha cambiado de modelo".into(),
            "301" | "302" | "307" | "308" => {
                "La API redirige la solicitud. Configura su URL final".into()
            }
            _ => format!(
                "La API rechazó la solicitud (HTTP {code}). Revisa modelo, endpoint y límites"
            ),
        });
    }
    serde_json::from_slice(&output[..split]).map_err(|_| "La API devolvió JSON inválido".into())
}

pub async fn complete(
    agent: AiAgent,
    key: Zeroizing<String>,
    history: Vec<Message>,
) -> Result<String, String> {
    let body = request_body(&agent, &history)?;
    let suffix = match agent.kind {
        AiKind::Responses => "responses",
        _ => "chat/completions",
    };
    let url = format!("{}/{suffix}", agent.endpoint.trim_end_matches('/'));
    let value = http_json(url, key, Some(body), agent.timeout_seconds).await?;
    parse_response(agent.kind, &value)
}

pub fn parse_free_models(value: &Value) -> Vec<String> {
    let mut models: Vec<String> = value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["id"].as_str())
        .filter(|id| {
            (id.ends_with("-free") || *id == "big-pickle")
                && !id.starts_with("jev-")
                && id.len() <= 128
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.'))
        })
        .map(|id| format!("opencode/{id}"))
        .collect();
    models.sort();
    models.dedup();
    models
}
pub async fn free_models() -> Result<Vec<String>, String> {
    let models = if let Some(program) = crate::connection::find_program(&["opencode"]) {
        let mut child = tokio::process::Command::new(program)
            .args(["models", "opencode", "--refresh", "--pure"])
            .env("OPENCODE_DISABLE_AUTOUPDATE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| "No se pudo iniciar OpenCode para consultar modelos")?;
        let query = async {
            let mut output = Vec::new();
            child
                .stdout
                .take()
                .ok_or("Sin salida de OpenCode")?
                .take(256 * 1024 + 1)
                .read_to_end(&mut output)
                .await
                .map_err(|_| "No se pudo leer el catálogo de OpenCode")?;
            if output.len() > 256 * 1024 {
                return Err("Catálogo de OpenCode demasiado grande".to_owned());
            }
            if !child
                .wait()
                .await
                .map_err(|_| "No se pudo finalizar la consulta")?
                .success()
            {
                return Err("OpenCode no pudo listar modelos. Revisa su instalación y autenticación con opencode models opencode".into());
            }
            let data: Vec<Value> = String::from_utf8_lossy(&output)
                .lines()
                .filter_map(|line| line.trim().strip_prefix("opencode/"))
                .map(|id| json!({"id": id}))
                .collect();
            Ok(parse_free_models(&json!({"data": data})))
        };
        tokio::time::timeout(std::time::Duration::from_secs(25), query)
            .await
            .map_err(|_| "OpenCode tardó demasiado en actualizar el catálogo")??
    } else {
        let value = http_json(
            OPENCODE_MODELS_URL.into(),
            Zeroizing::new(String::new()),
            None,
            20,
        )
        .await?;
        parse_free_models(&value)
    };
    if models.is_empty() {
        return Err("El catálogo no publica modelos de chat marcados como gratuitos".into());
    }
    Ok(models)
}

/// One bounded native API exchange with explicit terminal function tools.
/// Arbitrary MCP servers are never executed by this transport.
#[cfg(feature = "gui")]
pub async fn exchange(
    agent: &AiAgent,
    key: &Zeroizing<String>,
    history: &[Value],
) -> Result<Value, String> {
    agent.validate()?;
    if agent.model.trim().is_empty() {
        return Err("Selecciona un modelo para este agente".into());
    }
    if history.len() > 128
        || serde_json::to_vec(history)
            .map_err(|_| "Contexto inválido")?
            .len()
            > MAX_CONTEXT
    {
        return Err("Contexto máximo alcanzado; inicia otra conversación".into());
    }
    let responses = agent.kind == AiKind::Responses;
    let mut messages = history.to_vec();
    let mut body = if responses {
        json!({"model":agent.model,"input":messages,"store":false,"max_output_tokens":agent.max_output_tokens})
    } else {
        if !agent.system_prompt.is_empty() {
            messages.insert(0, json!({"role":"system","content":agent.system_prompt}));
        }
        json!({"model":agent.model,"messages":messages,"max_tokens":agent.max_output_tokens,"stream":false})
    };
    if responses && !agent.system_prompt.is_empty() {
        body["instructions"] = json!(agent.system_prompt);
    }
    if agent.terminal_access && agent.terminal_socket.is_some() {
        let tools=crate::terminal_mcp::tools().into_iter().map(|t|{let function=json!({"name":t["name"],"description":t["description"],"parameters":t["inputSchema"]});if responses{let mut t=function;t["type"]=json!("function");t}else{json!({"type":"function","function":function})}}).collect::<Vec<_>>();
        body["tools"] = json!(tools);
    }
    http_json(
        format!(
            "{}/{}",
            agent.endpoint.trim_end_matches('/'),
            if responses {
                "responses"
            } else {
                "chat/completions"
            }
        ),
        key.clone(),
        Some(body),
        agent.timeout_seconds,
    )
    .await
}

#[cfg(feature = "gui")]
pub async fn provider_models(
    agent: &AiAgent,
    key: &Zeroizing<String>,
) -> Result<Vec<Value>, String> {
    let value = http_json(
        format!("{}/models", agent.endpoint.trim_end_matches('/')),
        key.clone(),
        None,
        agent.timeout_seconds.min(20),
    )
    .await?;
    let models = value["data"]
        .as_array()
        .into_iter()
        .flatten()
        .take(256)
        .filter_map(|v| v["id"].as_str())
        .filter(|id| !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control))
        .map(|id| json!({"model":id,"displayName":id,"supportedReasoningEfforts":[]}))
        .collect::<Vec<_>>();
    if models.is_empty() {
        Err("El proveedor no publicó modelos. Añádelos en los ajustes de este agente.".into())
    } else {
        Ok(models)
    }
}

/// Read advertised tool support from a local Ollama model, without inference.
#[cfg(feature = "gui")]
pub async fn local_model_tools(endpoint: String, model: String) -> Result<Option<bool>, String> {
    let mut url = crate::ai_config::validate_url(&endpoint)?;
    let local = match url.host() {
        Some(url::Host::Domain("localhost")) => true,
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if !local || url.port_or_known_default() != Some(11434) || model.trim().is_empty() {
        return Ok(None);
    }
    url.set_path("/api/show");
    let value = http_json(
        url.to_string(),
        Zeroizing::new(String::new()),
        Some(json!({"model":model})),
        5,
    )
    .await?;
    Ok(value["capabilities"]
        .as_array()
        .map(|values| values.iter().any(|v| v == "tools")))
}

/// Actual allocated context from local Ollama; model catalogue maxima are not substituted.
#[cfg(feature = "gui")]
pub async fn local_context_limit(endpoint: String, model: String) -> Result<Option<u64>, String> {
    let mut url = crate::ai_config::validate_url(&endpoint)?;
    let local = match url.host() {
        Some(url::Host::Domain("localhost")) => true,
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(), _ => false,
    };
    if !local || url.port_or_known_default() != Some(11434) { return Ok(None); }
    url.set_path("/api/ps");
    let value = http_json(url.to_string(), Zeroizing::new(String::new()), None, 5).await?;
    Ok(value["models"].as_array().into_iter().flatten().find(|m| m["name"].as_str() == Some(&model) || m["model"].as_str() == Some(&model)).and_then(|m|m["context_length"].as_u64()).filter(|&size|size>0))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile(kind: AiKind) -> AiAgent {
        AiAgent {
            id: "test".into(),
            name: "Test".into(),
            kind,
            endpoint: "http://127.0.0.1:11434/v1".into(),
            model: "test-model".into(),
            ..AiAgent::default()
        }
    }
    #[test]
    fn request_formats_and_limits() {
        let history = vec![Message {
            role: "user".into(),
            content: "literal \" $()\ntext".into(),
        }];
        let body = request_body(&profile(AiKind::Responses), &history).unwrap();
        assert_eq!(body["store"], false);
        assert_eq!(body["input"][0]["content"], history[0].content);
        let body = request_body(&profile(AiKind::ChatCompletions), &history).unwrap();
        assert_eq!(body["messages"][0]["role"], "user");
        assert!(
            request_body(
                &profile(AiKind::Responses),
                &vec![history[0].clone(); MAX_MESSAGES + 1]
            )
            .is_err()
        );
        assert!(
            request_body(
                &profile(AiKind::Responses),
                &[Message {
                    role: "user".into(),
                    content: "x".repeat(MAX_CONTEXT + 1)
                }]
            )
            .is_err()
        );
        assert!(validate_key("bad\nkey").is_err());
        assert_eq!(quoted("a\"\\b").unwrap(), "\"a\\\"\\\\b\"");
    }
    #[test]
    fn parses_text_blocks_after_reasoning_and_refusals() {
        let value = json!({"output":[{"type":"reasoning"},{"type":"message","content":[{"type":"output_text","text":"Hello"},{"type":"output_text","text":" world"}]}]});
        assert_eq!(
            parse_response(AiKind::Responses, &value).unwrap(),
            "Hello world"
        );
        assert_eq!(
            parse_response(
                AiKind::ChatCompletions,
                &json!({"choices":[{"message":{"refusal":"Cannot do that"}}]})
            )
            .unwrap(),
            "Cannot do that"
        );
        assert!(parse_response(AiKind::Responses, &json!({"output":[]})).is_err());
    }
    #[test]
    fn free_catalog_excludes_paid_and_non_chat_models() {
        let models = parse_free_models(
            &json!({"data":[{"id":"example-free"},{"id":"paid"},{"id":"jev-1.13-free"},{"id":"big-pickle"},{"id":"example-free"}]}),
        );
        assert_eq!(models, vec!["opencode/big-pickle", "opencode/example-free"]);
    }
}
