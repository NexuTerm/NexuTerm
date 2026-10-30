//! A private, bounded MCP bridge to the running GUI. Read/write approval follows the explicit policy of its owning chat.
use serde_json::{Value, json};
use std::io::Read as _;
use std::{
    io::{BufRead, Write},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    sync::{Semaphore, mpsc, oneshot},
};
pub struct Request {
    pub agent: String,
    pub name: String,
    pub arguments: Value,
    pub response: oneshot::Sender<Value>,
}
pub struct Broker {
    directory: tempfile::TempDir,
    abort: tokio::task::AbortHandle,
    pub full_access: Arc<std::sync::atomic::AtomicBool>,
}
impl Broker {
    pub fn start() -> Result<(Self, mpsc::Receiver<Request>), String> {
        let directory = tempfile::Builder::new()
            .prefix("nx-ai-")
            .tempdir()
            .map_err(|_| "No se pudo crear el puente de terminales")?;
        let listener = std::os::unix::net::UnixListener::bind(directory.path().join("mcp"))
            .map_err(|_| "No se pudo abrir el puente privado")?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "Puente no disponible")?;
        let (tx, rx) = mpsc::channel::<Request>(8);
        let task = crate::runtime::runtime().spawn(async move {
            let listener = tokio::net::UnixListener::from_std(listener).unwrap();
            let slots = Arc::new(Semaphore::new(8));
            while let Ok((stream, _)) = listener.accept().await {
                let Ok(slot) = slots.clone().try_acquire_owned() else {
                    drop(stream);
                    continue;
                };
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _slot = slot;
                    let operation = async {
                        let (read, mut write) = stream.into_split();
                        let mut reader = tokio::io::BufReader::new(read).take(65537);
                        let mut line = Vec::new();
                        reader.read_until(b'\n', &mut line).await.ok()?;
                        if line.len() > 65536 {
                            return None;
                        }
                        let value: Value = serde_json::from_slice(&line).ok()?;
                        let (response, answer) = oneshot::channel();
                        tx.send(Request {
                            agent: value["agent"]
                                .as_str()
                                .unwrap_or("Agente")
                                .chars()
                                .take(100)
                                .collect(),
                            name: value["name"].as_str()?.into(),
                            arguments: value["arguments"].clone(),
                            response,
                        })
                        .await
                        .ok()?;
                        let result = answer.await.ok()?;
                        let bytes = serde_json::to_vec(&result).ok()?;
                        write.write_all(&bytes).await.ok()?;
                        Some(())
                    };
                    let _ =
                        tokio::time::timeout(std::time::Duration::from_secs(180), operation).await;
                });
            }
        });
        Ok((
            Self {
                directory,
                abort: task.abort_handle(),
                full_access: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            },
            rx,
        ))
    }
    pub fn path(&self) -> PathBuf {
        self.directory.path().join("mcp")
    }
}
impl Drop for Broker {
    fn drop(&mut self) {
        self.abort.abort();
    }
}
pub fn tools() -> Vec<Value> {
    vec![
        json!({"name":"supervised_targets","description":"List exact targets for separate supervised local/SSH processes. Does not execute or connect.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"supervised_execute","description":"Run a separate non-interactive process after GUI approval of command, target and absolute directory, or explicit full access for this chat. Does not type into a terminal. Defaults to 60 seconds, captures up to 4 MiB, returns confirmed exit code or unknown outcome. Never retry unknown results automatically. Replies contain bounded output tails; local history is enabled by default.","inputSchema":{"type":"object","properties":{"target_id":{"type":"string"},"directory":{"type":"string"},"command":{"type":"string"},"timeout_seconds":{"type":"integer","minimum":1,"maximum":120},"keep_history":{"type":"boolean"}},"required":["target_id","directory","command"]}}),
        json!({"name":"terminal_list","description":"List open local and remote NexuTerm terminals. IDs identify the exact target.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"terminal_read","description":"Read a terminal's visible text with user approval or explicit full access for this chat. Never assume the active terminal is local.","inputSchema":{"type":"object","properties":{"terminal_id":{"type":"string"}},"required":["terminal_id"]}}),
        json!({"name":"terminal_execute","description":"Send a command to the selected existing local or remote terminal, with explicit user approval or full access for this chat. It runs in that terminal's current session, not a separate shell. Returns dispatch acknowledgement; read the terminal afterwards. Never infer success or exit status from dispatch.","inputSchema":{"type":"object","properties":{"terminal_id":{"type":"string"},"command":{"type":"string"}},"required":["terminal_id","command"]}}),
    ]
}
pub fn result(text: String, error: bool) -> Value {
    json!({"content":[{"type":"text","text":text}],"isError":error})
}
pub async fn call(
    socket: &str,
    agent: &str,
    name: &str,
    arguments: Value,
) -> Result<Value, String> {
    let mut stream = tokio::net::UnixStream::connect(socket)
        .await
        .map_err(|_| "La ventana de terminales no está disponible")?;
    let mut bytes = serde_json::to_vec(&json!({"agent":agent,"name":name,"arguments":arguments}))
        .map_err(|_| "Solicitud inválida")?;
    if bytes.len() > 65536 {
        return Err("Solicitud de terminal demasiado grande".into());
    }
    bytes.push(b'\n');
    stream
        .write_all(&bytes)
        .await
        .map_err(|_| "No se pudo contactar con la ventana")?;
    let mut output = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(180),
        (&mut stream).take(131073).read_to_end(&mut output),
    )
    .await
    .map_err(|_| "La aprobación de terminal agotó su tiempo")?
    .map_err(|_| "No se pudo leer la terminal")?;
    if output.len() > 131072 {
        return Err("Respuesta de terminal demasiado grande".into());
    }
    serde_json::from_slice(&output).map_err(|_| "Respuesta de terminal inválida".into())
}
pub fn stdio(socket: &str, agent: &str) -> i32 {
    let input = std::io::stdin();
    let mut input = input.lock();
    let mut output = std::io::stdout().lock();
    loop {
        let mut bytes = Vec::new();
        match (&mut input).take(1048577).read_until(b'\n', &mut bytes) {
            Ok(0) | Err(_) => break,
            Ok(_) if bytes.len() > 1048576 => return 1,
            _ => {}
        }
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
            return 1;
        };
        let Some(id) = value.get("id") else {
            continue;
        };
        let method = value["method"].as_str().unwrap_or("");
        let response = match method {
            "initialize" => {
                json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":value["params"]["protocolVersion"].as_str().unwrap_or("2024-11-05"),"capabilities":{"tools":{}},"serverInfo":{"name":"NexuTerm terminals","version":env!("CARGO_PKG_VERSION")}}})
            }
            "tools/list" => json!({"jsonrpc":"2.0","id":id,"result":{"tools":tools()}}),
            "tools/call" => {
                let name = value["params"]["name"].as_str().unwrap_or("");
                let result = crate::runtime::runtime()
                    .block_on(call(
                        socket,
                        agent,
                        name,
                        value["params"]["arguments"].clone(),
                    ))
                    .unwrap_or_else(|e| result(e, true));
                json!({"jsonrpc":"2.0","id":id,"result":result})
            }
            "ping" => json!({"jsonrpc":"2.0","id":id,"result":{}}),
            _ => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Unknown method"}})
            }
        };
        if serde_json::to_writer(&mut output, &response).is_err()
            || output.write_all(b"\n").is_err()
            || output.flush().is_err()
        {
            return 1;
        }
    }
    0
}
