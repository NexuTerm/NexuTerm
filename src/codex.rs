//! Codex app-server client: bounded JSON lines over private stdin/stdout pipes.
use crate::config::AiAgent;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};

const MAX_FRAME: usize = 1024 * 1024;
#[derive(Debug)]
pub enum Event {
    Account(Option<String>),
    RateLimits(Value),
    ContextUsage { used: u64, size: u64 },
    TokenUsage { input: u64, output: u64 },
    Ready(bool),
    Capabilities {
        tools: bool,
        history: bool,
        reasoning: bool,
    },
    History(Vec<Value>),
    Models(Vec<Value>),
    Resumed(Value),
    Status(String),
    Delta {
        item: String,
        text: String,
    },
    Item(Value),
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    LoginUrl(String),
    Finished(String),
    Error(String),
    Closed,
}
pub(crate) enum Command {
    Credentials(Result<zeroize::Zeroizing<String>, String>),
    Send(String, String, String),
    Resume(String),
    Permissions(bool),
    Login,
    Interrupt,
    Reply(Value),
    Shutdown,
}
pub struct Client {
    commands: mpsc::Sender<Command>,
    busy: Arc<AtomicBool>,
    abort: tokio::task::AbortHandle,
}
impl Client {
    pub fn start(agent: AiAgent, writable: bool) -> (Self, mpsc::Receiver<Event>) {
        let (commands, rx) = mpsc::channel(32);
        let (events, out) = mpsc::channel(32);
        let busy = Arc::new(AtomicBool::new(true));
        let working = busy.clone();
        let task = crate::runtime::runtime().spawn(async move {
            let result=if agent.is_codex(){run(agent,writable,rx,events.clone(),working.clone()).await}else{crate::agent_runtime::run(agent,writable,rx,events.clone(),working.clone()).await};
            if result.is_err() {
                let _ = events.send(Event::Error("Se perdió la conexión con el agente. Abre una nueva conversación para reconectar.".into())).await;
            }
            working.store(false, Ordering::Relaxed);
            let _ = events.send(Event::Closed).await;
        });
        (
            Self {
                commands,
                busy,
                abort: task.abort_handle(),
            },
            out,
        )
    }
    pub fn credentials(&self, key: Result<zeroize::Zeroizing<String>, String>) {
        let _ = self.commands.try_send(Command::Credentials(key));
    }
    pub fn alive(&self) -> bool {
        !self.commands.is_closed()
    }
    pub fn busy(&self) -> bool {
        self.busy.load(Ordering::Relaxed)
    }
    pub fn send(&self, text: String) -> bool {
        self.send_with_options(text, String::new(), String::new())
    }
    pub fn send_with_options(&self, text: String, model: String, effort: String) -> bool {
        if self.busy.swap(true, Ordering::Relaxed) {
            return false;
        }
        if self
            .commands
            .try_send(Command::Send(text, model, effort))
            .is_err()
        {
            self.busy.store(false, Ordering::Relaxed);
            return false;
        }
        true
    }
    pub fn permissions(&self, writable: bool) {
        if self
            .commands
            .try_send(Command::Permissions(writable))
            .is_err()
        {
            self.close();
        }
    }
    pub fn resume(&self, id: String) -> bool {
        if self.busy.swap(true, Ordering::Relaxed) {
            return false;
        }
        if self.commands.try_send(Command::Resume(id)).is_err() {
            self.busy.store(false, Ordering::Relaxed);
            return false;
        }
        true
    }
    pub fn login(&self) {
        let _ = self.commands.try_send(Command::Login);
    }
    pub fn interrupt(&self) {
        let _ = self.commands.try_send(Command::Interrupt);
    }
    pub fn reply(&self, id: Value, result: Value) {
        if self
            .commands
            .try_send(Command::Reply(json!({"id":id,"result":result})))
            .is_err()
        {
            self.close();
        }
    }
    pub fn reject_unknown(&self, id: Value) {
        if self
            .commands
            .try_send(Command::Reply(
                json!({"id":id,"error":{"code":-32601,"message":"Unsupported client request"}}),
            ))
            .is_err()
        {
            self.close();
        }
    }
    pub fn close(&self) {
        let _ = self.commands.try_send(Command::Shutdown);
        self.abort.abort();
        self.busy.store(false, Ordering::Relaxed);
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.close();
    }
}

pub(crate) struct Process {
    pub(crate) child: tokio::process::Child,
    pub(crate) reader: tokio::task::JoinHandle<()>,
}
impl Drop for Process {
    fn drop(&mut self) {
        self.reader.abort();
        if let Some(pid) = self.child.id() {
            // The app-server and its tool processes belong to a dedicated group.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
}
pub(crate) async fn write(input: &mut tokio::process::ChildStdin, value: Value) -> Result<(), ()> {
    let mut bytes = serde_json::to_vec(&value).map_err(|_| ())?;
    if bytes.len() > MAX_FRAME {
        return Err(());
    }
    bytes.push(b'\n');
    input.write_all(&bytes).await.map_err(|_| ())
}
pub(crate) async fn request(
    input: &mut tokio::process::ChildStdin,
    pending: &mut HashMap<u64, String>,
    seq: &mut u64,
    method: &str,
    params: Value,
) -> Result<(), ()> {
    *seq += 1;
    pending.insert(*seq, method.into());
    write(input, json!({"id":*seq,"method":method,"params":params})).await
}
pub(crate) async fn read_frames(output: tokio::process::ChildStdout, tx: mpsc::Sender<Value>) {
    let mut reader = BufReader::new(output);
    loop {
        let mut bytes = Vec::new();
        match (&mut reader)
            .take(MAX_FRAME as u64 + 1)
            .read_until(b'\n', &mut bytes)
            .await
        {
            Ok(0) | Err(_) => break,
            Ok(_) if bytes.len() > MAX_FRAME => break,
            Ok(_) => match serde_json::from_slice(&bytes) {
                Ok(value) => {
                    if tx.send(value).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            },
        }
    }
}
async fn start_turn(
    input: &mut tokio::process::ChildStdin,
    pending: &mut HashMap<u64, String>,
    seq: &mut u64,
    thread: &str,
    text: String,
    model: &str,
    effort: &str,
) -> Result<(), ()> {
    let mut params = json!({"threadId":thread,"input":[{"type":"text","text":text}]});
    if !model.is_empty() {
        params["model"] = json!(model);
    }
    if !effort.is_empty() {
        params["effort"] = json!(effort);
    }
    request(input, pending, seq, "turn/start", params).await
}
async fn run(
    agent: AiAgent,
    mut writable: bool,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Event>,
    busy: Arc<AtomicBool>,
) -> Result<(), ()> {
    let directory = if agent.directory.is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| "/".into())
    } else {
        agent.directory.clone()
    };
    let root = crate::agent_profile::prepare(&agent).map_err(|_| ())?;
    let mut command = tokio::process::Command::new(&agent.program);
    crate::agent_profile::isolated(&mut command, &root);
    let mut child = command
        .args(["app-server", "--listen", "stdio://"])
        .current_dir(&directory)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| ())?;
    let mut input = child.stdin.take().ok_or(())?;
    let output = child.stdout.take().ok_or(())?;
    let (frames_tx, mut frames) = mpsc::channel(32);
    let reader = tokio::spawn(read_frames(output, frames_tx));
    let _process = Process { child, reader };
    let mut pending = HashMap::new();
    let mut seq = 0;
    request(&mut input, &mut pending, &mut seq, "initialize", json!({"clientInfo":{"name":"nexuterm","title":"NexuTerm","version":env!("CARGO_PKG_VERSION")}})).await?;
    events
        .send(Event::Capabilities {
            tools: true,
            history: true,
            reasoning: true,
        })
        .await
        .map_err(|_| ())?;
    let mut api_key: Option<zeroize::Zeroizing<String>> = None;
    let mut initialized = false;
    let mut account_known = false;
    let mut authenticated = false;
    let mut chatgpt = false;
    let mut want_login = false;
    let mut thread = String::new();
    let mut turn = String::new();
    let mut login = String::new();
    let mut queued: Option<String> = None;
    let mut selected_model = String::new();
    let mut selected_effort = String::new();
    let mut stopping = false;
    let mut last_activity = tokio::time::Instant::now();
    let mut last_usage_read = tokio::time::Instant::now();
    let mut watchdog = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        tokio::select! {
            command = commands.recv() => {
                last_activity = tokio::time::Instant::now();
                match command {
                Some(Command::Credentials(result)) => {match result{Ok(key)if !key.is_empty()=>{if initialized{request(&mut input,&mut pending,&mut seq,"account/login/start",json!({"type":"apiKey","apiKey":key.as_str()})).await?;}else{api_key=Some(key);}},Err(error)=>{events.send(Event::Error(error)).await.map_err(|_|())?;},_=>{}}},
                Some(Command::Shutdown) | None => break,
                Some(Command::Reply(value)) => write(&mut input, value).await?,
                Some(Command::Login) => {
                    stopping=false;
                    want_login = true;
                    if account_known {
                        if chatgpt { events.send(Event::Status("Codex ya tiene sesión de ChatGPT.".into())).await.map_err(|_| ())?; want_login=false; }
                        else {
                            busy.store(true, Ordering::Relaxed);
                            request(&mut input,&mut pending,&mut seq,"account/login/start",json!({"type":"chatgpt","useHostedLoginSuccessPage":true})).await?;
                        }
                    }
                }
                Some(Command::Interrupt) => {
                    queued = None; want_login=false;
                    stopping = true;
                    if !turn.is_empty() {
                        request(&mut input,&mut pending,&mut seq,"turn/interrupt",json!({"threadId":thread,"turnId":turn})).await?;
                    } else if !login.is_empty() {
                        request(&mut input,&mut pending,&mut seq,"account/login/cancel",json!({"loginId":login})).await?;
                    } else if !pending.values().any(|m|matches!(m.as_str(),"thread/start"|"turn/start"|"account/login/start")) { busy.store(false, Ordering::Relaxed); }
                }
                Some(Command::Permissions(value)) => { if thread.is_empty() { writable=value; } }
                Some(Command::Resume(id)) => {
                    if !authenticated || !initialized { busy.store(false, Ordering::Relaxed); continue; }
                    request(&mut input, &mut pending, &mut seq, "thread/resume", json!({"threadId":id,"cwd":directory,"approvalPolicy":"untrusted","sandbox":if writable {"workspace-write"} else {"read-only"}})).await?;
                }
                Some(Command::Send(text, model, effort)) => {
                    selected_model = model; selected_effort = effort;
                    stopping = false;
                    if !initialized || !authenticated {
                        busy.store(false, Ordering::Relaxed);
                        events.send(Event::Error("Conecta tu cuenta de Codex antes de enviar mensajes.".into())).await.map_err(|_| ())?;
                    } else if thread.is_empty() {
                        queued = Some(text);
                        let mut params = json!({"cwd":directory,"approvalPolicy":"untrusted","sandbox":if writable {"workspace-write"} else {"read-only"}});
                        if !agent.model.is_empty() { params["model"]=json!(agent.model); }
                        if !agent.system_prompt.is_empty() { params["developerInstructions"]=json!(agent.system_prompt); }
                        request(&mut input,&mut pending,&mut seq,"thread/start",params).await?;
                    } else { start_turn(&mut input,&mut pending,&mut seq,&thread,text,&selected_model,&selected_effort).await?; }
                }
            }
            },
            frame = frames.recv() => {
                let Some(frame) = frame else { return Err(()); };
                last_activity = tokio::time::Instant::now();
                if let Some(method) = frame["method"].as_str() {
                    let params = &frame["params"];
                    if frame.get("id").is_some() {
                        events.send(Event::Request{id:frame["id"].clone(),method:method.into(),params:params.clone()}).await.map_err(|_| ())?;
                        continue;
                    }
                    match method {
                        "account/rateLimits/updated" => { events.send(Event::RateLimits(params.clone())).await.map_err(|_|())?; }
                        "account/updated" => { request(&mut input,&mut pending,&mut seq,"account/read",json!({"refreshToken":false})).await?; }
                        "thread/tokenUsage/updated" => {
                            if let (Some(used),Some(size))=(params["tokenUsage"]["last"]["totalTokens"].as_u64(),params["tokenUsage"]["modelContextWindow"].as_u64()) {
                                events.send(Event::ContextUsage{used,size}).await.map_err(|_|())?;
                            }
                        }
                        "item/agentMessage/delta" => {
                            events.send(Event::Delta {item:params["itemId"].as_str().unwrap_or("message").into(),text:params["delta"].as_str().unwrap_or("").into()}).await.map_err(|_| ())?;
                        }
                        "item/started" | "item/completed" => { events.send(Event::Item(params["item"].clone())).await.map_err(|_| ())?; }
                        "turn/started" => {
                            turn = params["turn"]["id"].as_str().unwrap_or("").into();
                            if stopping && !turn.is_empty() { request(&mut input,&mut pending,&mut seq,"turn/interrupt",json!({"threadId":thread,"turnId":turn})).await?; }
                        }
                        "turn/completed" => {
                            turn.clear(); busy.store(false,Ordering::Relaxed);
                            let status=params["turn"]["status"].as_str().unwrap_or("failed");
                            events.send(Event::Finished(status.into())).await.map_err(|_| ())?;
                            if chatgpt && !pending.values().any(|m|m=="account/rateLimits/read") { request(&mut input,&mut pending,&mut seq,"account/rateLimits/read",json!({})).await?; }
                        }
                        "account/login/completed" => {
                            login.clear(); want_login=false;
                            if params["success"] == true {
                                request(&mut input,&mut pending,&mut seq,"account/read",json!({"refreshToken":false})).await?;
                            } else {
                                busy.store(false,Ordering::Relaxed);
                                events.send(Event::Error("Inicio de sesión cancelado o fallido.".into())).await.map_err(|_| ())?;
                            }
                        }
                        "error" => { events.send(Event::Status("Codex ha comunicado un error en el turno; espera su resultado o pulsa Detener.".into())).await.map_err(|_| ())?; }
                        "serverRequest/resolved" => { events.send(Event::Item(json!({"type":"requestResolved","requestId":params["requestId"]}))).await.map_err(|_| ())?; }
                        _ => {}
                    }
                    continue;
                }
                let Some(id)=frame["id"].as_u64() else { continue; };
                let Some(method)=pending.remove(&id) else { continue; };
                if frame.get("error").is_some() {
                    if method == "account/rateLimits/read" { events.send(Event::RateLimits(Value::Null)).await.map_err(|_|())?; continue; }
                    if matches!(method.as_str(),"model/list"|"thread/list") { events.send(Event::Status(format!("No se pudo consultar {}.",method))).await.map_err(|_|())?; continue; }
                    busy.store(false,Ordering::Relaxed); queued=None;
                    events.send(Event::Error(format!("Codex rechazó {method} (código {}). Revisa modelo, cuenta y configuración.",frame["error"]["code"])) ).await.map_err(|_| ())?;
                    if method == "initialize" { return Err(()); }
                    continue;
                }
                let result=&frame["result"];
                match method.as_str() {
                    "initialize" => {
                        initialized=true;
                        write(&mut input,json!({"method":"initialized","params":{}})).await?;
                        request(&mut input,&mut pending,&mut seq,"account/read",json!({"refreshToken":false})).await?;
                        if let Some(key)=api_key.take(){request(&mut input,&mut pending,&mut seq,"account/login/start",json!({"type":"apiKey","apiKey":key.as_str()})).await?;}
                    }
                    "account/rateLimits/read" => { last_usage_read=tokio::time::Instant::now(); events.send(Event::RateLimits(result.clone())).await.map_err(|_|())?; }
                    "account/read" => {
                        account_known=true;
                        chatgpt=result["account"]["type"]=="chatgpt";
                        authenticated=!result["account"].is_null() || result["requiresOpenaiAuth"]==false;
                        busy.store(false,Ordering::Relaxed);
                        events.send(Event::Account(result["account"]["email"].as_str().map(String::from))).await.map_err(|_|())?;
                        if chatgpt { request(&mut input,&mut pending,&mut seq,"account/rateLimits/read",json!({})).await?; }
                        events.send(Event::Ready(authenticated)).await.map_err(|_| ())?;
                        request(&mut input, &mut pending, &mut seq, "thread/list", json!({"limit":12,"cwd":directory,"sortKey":"updated_at"})).await?;
                        request(&mut input, &mut pending, &mut seq, "model/list", json!({})).await?;
                        if want_login {
                            if chatgpt {
                                want_login=false;
                                events.send(Event::Status("Codex ya tiene sesión de ChatGPT. Escribe tu mensaje.".into())).await.map_err(|_| ())?;
                            } else {
                                busy.store(true,Ordering::Relaxed);
                                request(&mut input,&mut pending,&mut seq,"account/login/start",json!({"type":"chatgpt","useHostedLoginSuccessPage":true})).await?;
                            }
                        }
                    }
                    "account/login/start" => {
                        if result["type"]=="apiKey"{request(&mut input,&mut pending,&mut seq,"account/read",json!({"refreshToken":false})).await?;continue;}
                        login=result["loginId"].as_str().unwrap_or("").into();
                        if stopping { request(&mut input,&mut pending,&mut seq,"account/login/cancel",json!({"loginId":login})).await?; }
                        else if let Some(url)=result["authUrl"].as_str() { events.send(Event::LoginUrl(url.into())).await.map_err(|_| ())?; }
                    }
                    "account/login/cancel" => { busy.store(false,Ordering::Relaxed); login.clear(); }
                    "model/list" => {
                        events.send(Event::Models(result["data"].as_array().cloned().unwrap_or_default())).await.map_err(|_| ())?;
                    }
                    "thread/list" => {
                        events.send(Event::History(result["data"].as_array().cloned().unwrap_or_default())).await.map_err(|_| ())?;
                    }
                    "thread/resume" => {
                        thread=result["thread"]["id"].as_str().ok_or(())?.into();
                        busy.store(false, Ordering::Relaxed);
                        events.send(Event::Resumed(result["thread"].clone())).await.map_err(|_| ())?;
                    }
                    "thread/start" => {
                        thread=result["thread"]["id"].as_str().ok_or(())?.into();
                        if let Some(text)=queued.take() { start_turn(&mut input,&mut pending,&mut seq,&thread,text,&selected_model,&selected_effort).await?; } else { busy.store(false,Ordering::Relaxed); }
                    }
                    "turn/start" => {
                        turn=result["turn"]["id"].as_str().unwrap_or("").into();
                        if stopping && !turn.is_empty() { request(&mut input,&mut pending,&mut seq,"turn/interrupt",json!({"threadId":thread,"turnId":turn})).await?; }
                    }
                    _ => {}
                }
            }
            _ = watchdog.tick() => {
                if chatgpt && last_usage_read.elapsed().as_secs() >= 60 && !pending.values().any(|m|m=="account/rateLimits/read") {
                    last_usage_read=tokio::time::Instant::now();
                    request(&mut input,&mut pending,&mut seq,"account/rateLimits/read",json!({})).await?;
                }
                let limit=if !initialized || pending.values().any(|m|m!="account/rateLimits/read") { 30 } else if busy.load(Ordering::Relaxed) { 600 } else { u64::MAX };
                if last_activity.elapsed().as_secs() > limit { return Err(()); }
            }
        }
    }
    Ok(())
}
