//! Native API and ACP adapters sharing the graphical chat event stream.
use crate::{
    ai::{self},
    codex::{Command, Event, Process, read_frames, write},
    config::AiAgent,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::mpsc;
async fn request(
    input: &mut tokio::process::ChildStdin,
    pending: &mut HashMap<u64, String>,
    seq: &mut u64,
    method: &str,
    params: Value,
) -> Result<(), ()> {
    *seq += 1;
    pending.insert(*seq, method.into());
    write(
        input,
        json!({"jsonrpc":"2.0","id":*seq,"method":method,"params":params}),
    )
    .await
}
async fn emit(tx: &mpsc::Sender<Event>, event: Event) -> Result<(), ()> {
    tx.send(event).await.map_err(|_| ())
}
fn configured_models(agent: &AiAgent) -> Vec<Value> {
    let mut ids = agent.models.clone();
    if !agent.model.is_empty() && !ids.contains(&agent.model) {
        ids.insert(0, agent.model.clone());
    }
    ids.into_iter()
        .map(|id| json!({"model":id,"displayName":id,"supportedReasoningEfforts":[]}))
        .collect()
}
pub(crate) async fn run(
    mut agent: AiAgent,
    writable: bool,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Event>,
    busy: Arc<AtomicBool>,
) -> Result<(), ()> {
    if agent.kind == crate::ai_config::AiKind::Local {
        let name = std::path::Path::new(&agent.program)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned();
        match name.as_str() {
            "claude" => {
                agent.program = "claude-agent-acp".into();
                agent.args.clear();
            }
            "gemini" => agent.args = vec!["--experimental-acp".into()],
            "copilot" => agent.args = vec!["--acp".into()],
            "opencode" => agent.kind = crate::ai_config::AiKind::OpenCode,
            _ => {}
        }
    }
    if agent.kind.is_api() {
        api(agent, commands, events, busy).await
    } else {
        let key = if agent.wait_credentials {
            loop {
                match commands.recv().await {
                    Some(Command::Credentials(Ok(key))) => break key,
                    Some(Command::Credentials(Err(error))) => {
                        emit(&events, Event::Error(error)).await?;
                        return Ok(());
                    }
                    Some(Command::Shutdown) | None => return Ok(()),
                    _ => {}
                }
            }
        } else {
            zeroize::Zeroizing::new(String::new())
        };
        acp(agent, writable, commands, events, busy, key).await
    }
}
async fn api(
    mut agent: AiAgent,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Event>,
    busy: Arc<AtomicBool>,
) -> Result<(), ()> {
    emit(
        &events,
        Event::Capabilities {
            tools: agent.terminal_access && agent.terminal_socket.is_some(),
            history: false,
            reasoning: false,
        },
    )
    .await?;
    emit(&events, Event::Models(configured_models(&agent))).await?;
    let mut key = zeroize::Zeroizing::new(String::new());
    let mut history = Vec::<Value>::new();
    let mut loaded = false;
    loop {
        match commands.recv().await {
            Some(Command::Credentials(result)) => match result {
                Ok(value) => {
                    key = value;
                    if agent.model.is_empty() && agent.models.is_empty() {
                        match ai::provider_models(&agent, &key).await {
                            Ok(models) => {
                                agent.models = models
                                    .iter()
                                    .filter_map(|m| m["model"].as_str().map(String::from))
                                    .collect();
                                emit(&events, Event::Models(models)).await?;
                            }
                            Err(error) => {
                                busy.store(false, Ordering::Relaxed);
                                emit(&events, Event::Error(error)).await?;
                                continue;
                            }
                        }
                    }
                    loaded = true;
                    busy.store(false, Ordering::Relaxed);
                    emit(&events, Event::Ready(true)).await?;
                }
                Err(error) => {
                    busy.store(false, Ordering::Relaxed);
                    emit(&events, Event::Error(error)).await?;
                }
            },
            Some(Command::Send(text, model, _)) => {
                if !loaded {
                    busy.store(false, Ordering::Relaxed);
                    continue;
                }
                if !model.is_empty() {
                    agent.model = model;
                } else if agent.model.is_empty() {
                    agent.model = agent.models.first().cloned().unwrap_or_default();
                }
                let operation = api_turn(&agent, &key, history.clone(), text, &events);
                tokio::pin!(operation);
                let result = loop {
                    tokio::select! {result=&mut operation=>break Some(result),command=commands.recv()=>match command{Some(Command::Interrupt)=>break None,Some(Command::Shutdown)|None=>return Ok(()),_=>{}}}
                };
                busy.store(false, Ordering::Relaxed);
                match result {
                    Some(Ok((messages, answer))) => {
                        emit(
                            &events,
                            Event::Delta {
                                item: uuid::Uuid::new_v4().to_string(),
                                text: answer.clone(),
                            },
                        )
                        .await?;
                        history = messages;
                        emit(&events, Event::Finished("completed".into())).await?;
                    }
                    Some(Err(error)) => {
                        emit(&events, Event::Error(error)).await?;
                        emit(&events, Event::Finished("failed".into())).await?;
                    }
                    None => emit(&events, Event::Finished("interrupted".into())).await?,
                }
            }
            Some(Command::Shutdown) | None => return Ok(()),
            Some(Command::Login) => {
                emit(
                    &events,
                    Event::Status("Configura la clave API de este agente en sus ajustes.".into()),
                )
                .await?
            }
            _ => {}
        }
    }
}
async fn api_turn(
    agent: &AiAgent,
    key: &zeroize::Zeroizing<String>,
    mut history: Vec<Value>,
    text: String,
    events: &mpsc::Sender<Event>,
) -> Result<(Vec<Value>, String), String> {
    history.push(json!({"role":"user","content":text}));
    let mut answer = String::new();
    for _ in 0..12 {
        let response = ai::exchange(agent, key, &history).await?;
        let responses = agent.kind == crate::ai_config::AiKind::Responses;
        let calls: Vec<Value> = if responses {
            response["output"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|item| item["type"] == "function_call")
                .cloned()
                .collect()
        } else {
            response["choices"][0]["message"]["tool_calls"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        };
        if calls.is_empty() {
            let text = ai::parse_response(agent.kind, &response)?;
            answer.push_str(&text);
            history.push(json!({"role":"assistant","content":answer}));
            return Ok((history, answer));
        }
        if calls.len() > 8 {
            return Err("Demasiadas solicitudes de herramientas".into());
        }
        if responses {
            if let Some(items) = response["output"].as_array() {
                history.extend(items.clone());
            }
        } else {
            history.push(response["choices"][0]["message"].clone());
        }
        for call in calls {
            let (name, args, id) = if responses {
                (
                    call["name"].as_str().unwrap_or(""),
                    call["arguments"].as_str().unwrap_or("{}"),
                    call["call_id"].as_str().unwrap_or(""),
                )
            } else {
                (
                    call["function"]["name"].as_str().unwrap_or(""),
                    call["function"]["arguments"].as_str().unwrap_or("{}"),
                    call["id"].as_str().unwrap_or(""),
                )
            };
            if !agent.terminal_access {
                return Err("El acceso a terminales está desactivado para este agente".into());
            }
            let socket = agent
                .terminal_socket
                .as_deref()
                .ok_or("El puente de terminales no está disponible")?;
            emit(
                events,
                Event::Status(format!(
                    "{name}: esperando respuesta de la ventana de terminales…"
                )),
            )
            .await
            .map_err(|_| "Chat cerrado")?;
            let arguments =
                serde_json::from_str(args).map_err(|_| "Argumentos de herramienta inválidos")?;
            let result = crate::terminal_mcp::call(socket, &agent.name, name, arguments)
                .await
                .unwrap_or_else(|error| crate::terminal_mcp::result(error, true));
            let output = result.to_string();
            history.push(if responses {
                json!({"type":"function_call_output","call_id":id,"output":output})
            } else {
                json!({"role":"tool","tool_call_id":id,"content":output})
            });
        }
    }
    Err(
        "Límite de llamadas a herramientas alcanzado; revisa la conversación antes de continuar"
            .into(),
    )
}
fn collect_options(options: &Value, out: &mut Vec<Value>) {
    if let Some(values) = options.as_array() {
        for option in values {
            if let Some(id) = option["value"].as_str() {
                out.push(json!({"model":id,"displayName":option["name"].as_str().unwrap_or(id),"supportedReasoningEfforts":[]}));
            } else {
                collect_options(&option["options"], out);
            }
        }
    }
}
async fn acp(
    agent: AiAgent,
    mut writable: bool,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Event>,
    busy: Arc<AtomicBool>,
    key: zeroize::Zeroizing<String>,
) -> Result<(), ()> {
    let root = crate::agent_profile::prepare(&agent).map_err(|_| ())?;
    let directory = if agent.directory.is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| "/".into())
    } else {
        agent.directory.clone()
    };
    let mut command = tokio::process::Command::new(&agent.program);
    crate::agent_profile::isolated(&mut command, &root);
    if !key.is_empty() && !agent.credential_env.is_empty() {
        command.env(&agent.credential_env, key.as_str());
    }
    if agent.kind == crate::ai_config::AiKind::OpenCode
        || std::path::Path::new(&agent.program)
            .file_name()
            .is_some_and(|p| p == "opencode")
    {
        command.arg("acp");
        let mut config = serde_json::to_value(&agent.opencode).map_err(|_| ())?;
        config["permission"] = json!("ask");
        command
            .env("OPENCODE_DISABLE_PROJECT_CONFIG", "true")
            .env("OPENCODE_CONFIG_CONTENT", config.to_string());
    } else {
        command.args(&agent.args);
    }
    // Claude ACP respects its isolated config directory; explicit MCP configuration comes from session/new.
    let mut child = command
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
    let (tx, mut frames) = mpsc::channel(32);
    let reader = tokio::spawn(read_frames(output, tx));
    let _process = Process { child, reader };
    let mut pending = HashMap::new();
    let mut seq = 0;
    request(&mut input,&mut pending,&mut seq,"initialize",json!({"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"NexuTerm","version":env!("CARGO_PKG_VERSION")}})).await?;
    let mut want_login = false;
    let mut session = String::new();
    let mut model_option = String::new();
    let mut model_current = String::new();
    let mut modes = Vec::<String>::new();
    let mut auth = Vec::<Value>::new();
    let mut permission_requests = HashMap::<String, Value>::new();
    let mut queued: Option<(String, String)> = None;
    let mut item = 0_u64;
    let mut stopping = false;
    let mut activity = tokio::time::Instant::now();
    let mut watchdog = tokio::time::interval(std::time::Duration::from_secs(1));
    let mut mcp=agent.opencode.mcp.iter().filter_map(|(name,s)|match s {
  crate::ai_config::McpServer::Local{command,enabled:true,environment}=>Some(json!({"name":name,"command":command[0],"args":command[1..],"env":environment.iter().map(|(name,value)|json!({"name":name,"value":value})).collect::<Vec<_>>()})),
  crate::ai_config::McpServer::Remote{url,enabled:true,headers,..}=>Some(json!({"name":name,"type":"http","url":url,"headers":headers.iter().map(|(name,value)|json!({"name":name,"value":value})).collect::<Vec<_>>()})),_=>None}).collect::<Vec<_>>();
    if agent.terminal_access
        && let (Some(socket), Ok(exe)) = (&agent.terminal_socket, std::env::current_exe()) {
            mcp.push(json!({"name":"nexuterm-terminals","command":exe,"args":["--terminal-mcp",socket,&agent.name],"env":[]}));
        }
    emit(
        &events,
        Event::Capabilities {
            tools: true,
            history: false,
            reasoning: false,
        },
    )
    .await?;
    loop {
        tokio::select! {
         command=commands.recv()=>{activity=tokio::time::Instant::now();match command{
          Some(Command::Shutdown)|None=>return Ok(()),Some(Command::Credentials(_))=>{},
          Some(Command::Permissions(value))=>writable=value,
          Some(Command::Login)=>{if pending.values().any(|m|m=="initialize") && web_login_command(&agent).is_none(){want_login=true;continue;}
                    if let Some((program,args))=web_login_command(&agent){
            busy.store(true,Ordering::Relaxed);
            let login=web_login(&program,&args,&root,&events);tokio::pin!(login);
            let result=tokio::select!{result=&mut login=>Some(result),command=commands.recv()=>match command{Some(Command::Interrupt)=>None,Some(Command::Shutdown)|None=>return Ok(()),_=>None}};
            busy.store(false,Ordering::Relaxed);match result{Some(Ok(()))=>{if !pending.values().any(|m|m=="initialize"){request(&mut input,&mut pending,&mut seq,"session/new",json!({"cwd":directory,"mcpServers":mcp})).await?;}},Some(Err(error))=>emit(&events,Event::Error(error)).await?,None=>emit(&events,Event::Status("Inicio de sesión cancelado.".into())).await?}
          }else if let Some(method)=auth.first().and_then(|a|a["id"].as_str()){busy.store(true,Ordering::Relaxed);request(&mut input,&mut pending,&mut seq,"authenticate",json!({"methodId":method})).await?;}else{emit(&events,Event::Status("Este agente no ofrece inicio de sesión web por ACP. Configura sus credenciales en el perfil.".into())).await?;}},
          Some(Command::Interrupt)=>{stopping=true;write(&mut input,json!({"jsonrpc":"2.0","method":"session/cancel","params":{"sessionId":session}})).await?;},
          Some(Command::Reply(reply))=>{let id=reply["id"].clone();if let Some(params)=permission_requests.remove(&id.to_string()){
           let accept=reply["result"]["decision"]=="accept";let option=params["options"].as_array().and_then(|o|o.iter().find(|o|o["kind"]==if accept{"allow_once"}else{"reject_once"})).and_then(|o|o["optionId"].as_str());
           let outcome=option.map(|id|json!({"outcome":"selected","optionId":id})).unwrap_or_else(||json!({"outcome":"cancelled"}));write(&mut input,json!({"jsonrpc":"2.0","id":id,"result":{"outcome":outcome}})).await?;
          }},
          Some(Command::Send(text,model,_))=>{stopping=false;queued=Some((text,model.clone()));
           if session.is_empty(){busy.store(false,Ordering::Relaxed);queued=None;emit(&events,Event::Error("El agente no tiene una sesión preparada. Revisa la autenticación.".into())).await?;}
           else if !model.is_empty()&&model!=model_current{
            let (method,params)=if model_option.is_empty(){("session/set_model",json!({"sessionId":session,"modelId":model}))}else{("session/set_config_option",json!({"sessionId":session,"configId":model_option,"value":model}))};request(&mut input,&mut pending,&mut seq,method,params).await?;
           }else{let mode=if writable{"build"}else{"plan"};if modes.iter().any(|v|v==mode){request(&mut input,&mut pending,&mut seq,"session/set_mode",json!({"sessionId":session,"modeId":mode})).await?;}else{send_prompt(&mut input,&mut pending,&mut seq,&session,&agent,&mut queued,&mut item).await?;}}
          },Some(Command::Resume(_))=>{}
         }},
         frame=frames.recv()=>{let Some(frame)=frame else{return Err(());};activity=tokio::time::Instant::now();
          if let Some(method)=frame["method"].as_str(){let params=&frame["params"];
           if frame.get("id").is_some(){let id=frame["id"].clone();if method=="session/request_permission"{
            if permission_requests.len()>=8{return Err(());}
            permission_requests.insert(id.to_string(),params.clone());let operation=serde_json::to_string_pretty(&params["toolCall"]).unwrap_or_default();
            if !writable{write(&mut input,json!({"jsonrpc":"2.0","id":id,"result":{"outcome":{"outcome":"cancelled"}}})).await?;permission_requests.remove(&id.to_string());emit(&events,Event::Status("Herramienta rechazada: conversación de solo lectura.".into())).await?;}
            else{emit(&events,Event::Request{id,method:"item/commandExecution/requestApproval".into(),params:json!({"command":operation,"cwd":directory,"reason":"Solicitud de herramienta del agente ACP; revisa su entrada completa."})}).await?;}
           }else{write(&mut input,json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Unsupported client capability"}})).await?;}continue;}
           if method=="session/update"{let update=&params["update"];match update["sessionUpdate"].as_str().unwrap_or(""){
            "agent_message_chunk"=>{if let Some(text)=update["content"]["text"].as_str(){emit(&events,Event::Delta{item:format!("acp-{item}"),text:text.into()}).await?;}},
            "tool_call"|"tool_call_update"=>{emit(&events,Event::Item(json!({"type":"commandExecution","id":update["toolCallId"],"command":update["title"].as_str().unwrap_or("Herramienta"),"aggregatedOutput":update["content"].as_array().map(|items|items.iter().filter_map(|i|i["content"]["text"].as_str()).collect::<Vec<_>>().join("\n")).unwrap_or_default()}))).await?;},_=>{}
           }}continue;
          }
          let Some(id)=frame["id"].as_u64()else{continue;};let Some(method)=pending.remove(&id)else{continue;};
          // Internal pipeline marker is translated before writing, never sent to the server.
          if frame.get("error").is_some(){busy.store(false,Ordering::Relaxed);queued=None;emit(&events,Event::Error(format!("El agente rechazó {method} ({}). Revisa cuenta, modelo y capacidades.",frame["error"]["code"]))).await?;if method=="initialize"{return Err(());}continue;}
          let result=&frame["result"];
          match method.as_str(){
           "initialize"=>{auth=result["authMethods"].as_array().cloned().unwrap_or_default();if want_login&&!auth.is_empty(){want_login=false;request(&mut input,&mut pending,&mut seq,"authenticate",json!({"methodId":auth[0]["id"]})).await?;}else{request(&mut input,&mut pending,&mut seq,"session/new",json!({"cwd":directory,"mcpServers":mcp})).await?;}},
           "authenticate"=>{request(&mut input,&mut pending,&mut seq,"session/new",json!({"cwd":directory,"mcpServers":mcp})).await?;},
           "session/new"=>{session=result["sessionId"].as_str().ok_or(())?.into();
            let mut catalog=Vec::new();if let Some(options)=result["configOptions"].as_array(){for option in options{match option["category"].as_str().unwrap_or(""){
             "model"=>{model_option=option["id"].as_str().unwrap_or("model").into();model_current=option["currentValue"].as_str().unwrap_or("").into();collect_options(&option["options"],&mut catalog);},
             "mode"=>{if let Some(options)=option["options"].as_array(){modes=options.iter().filter_map(|v|v["value"].as_str().map(String::from)).collect();}},_=>{}
            }}}
            if catalog.is_empty()&& let Some(models)=result["models"]["availableModels"].as_array(){for m in models{if let Some(id)=m["modelId"].as_str(){catalog.push(json!({"model":id,"displayName":m["name"].as_str().unwrap_or(id),"supportedReasoningEfforts":[]}));}}}
            for model in configured_models(&agent){if !catalog.iter().any(|v|v["model"]==model["model"]){catalog.push(model);}}
            emit(&events,Event::Models(catalog)).await?;busy.store(false,Ordering::Relaxed);emit(&events,Event::Ready(true)).await?;
           },
           "session/set_model"|"session/set_config_option"=>{
            if let Some((_,model))=&queued{model_current=model.clone();}
            let mode=if writable{"build"}else{"plan"};if modes.iter().any(|v|v==mode){request(&mut input,&mut pending,&mut seq,"session/set_mode",json!({"sessionId":session,"modeId":mode})).await?;}else{send_prompt(&mut input,&mut pending,&mut seq,&session,&agent,&mut queued,&mut item).await?;}
           },
           "session/set_mode"=>send_prompt(&mut input,&mut pending,&mut seq,&session,&agent,&mut queued,&mut item).await?,
           "session/prompt"=>{busy.store(false,Ordering::Relaxed);emit(&events,Event::Finished(if stopping||result["stopReason"]=="cancelled"{"interrupted"}else{"completed"}.into())).await?;},_=>{}
          }
         },
         _=watchdog.tick()=>{if (busy.load(Ordering::Relaxed)||!pending.is_empty())&&activity.elapsed().as_secs()>600{return Err(());}}
        }
    }
}
fn web_login_command(agent: &AiAgent) -> Option<(String, Vec<String>)> {
    let name = std::path::Path::new(&agent.program).file_name()?.to_str()?;
    match name {
        "copilot" => Some((
            agent.program.clone(),
            vec!["login".into(), "--web-flow".into()],
        )),
        "claude-agent-acp" => Some(("claude".into(), vec!["auth".into(), "login".into()])),
        _ => None,
    }
}
async fn web_login(
    program: &str,
    args: &[String],
    root: &std::path::Path,
    events: &mpsc::Sender<Event>,
) -> Result<(), String> {
    use tokio::io::AsyncBufReadExt;
    let mut command = tokio::process::Command::new(program);
    crate::agent_profile::isolated(&mut command, root);
    let mut child = command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| {
            "No se pudo iniciar la autenticación web. Instala el CLI del agente.".to_owned()
        })?;
    let output = child
        .stdout
        .take()
        .ok_or("No se pudo consultar el acceso web")?;
    let mut lines = tokio::io::BufReader::new(output).lines();
    let operation = async {
        let mut total = 0;
        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|_| "No se pudo leer el acceso web")?
        {
            total += line.len();
            if total > 65536 {
                return Err("La autenticación devolvió demasiados datos".into());
            }
            for word in line.split_whitespace() {
                if word.starts_with("https://") && url::Url::parse(word).is_ok() {
                    emit(events, Event::LoginUrl(word.into()))
                        .await
                        .map_err(|_| "Chat cerrado")?;
                }
            }
        }
        if child
            .wait()
            .await
            .map_err(|_| "No se pudo finalizar la autenticación")?
            .success()
        {
            Ok(())
        } else {
            Err("El agente no pudo completar el inicio de sesión web.".into())
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(300), operation)
        .await
        .map_err(|_| "Se agotó el tiempo de autenticación web")?
}
async fn send_prompt(
    input: &mut tokio::process::ChildStdin,
    pending: &mut HashMap<u64, String>,
    seq: &mut u64,
    session: &str,
    agent: &AiAgent,
    queued: &mut Option<(String, String)>,
    item: &mut u64,
) -> Result<(), ()> {
    if let Some((text, _)) = queued.take() {
        *item += 1;
        let text = if *item == 1 && !agent.system_prompt.is_empty() {
            format!("{}\n\n{text}", agent.system_prompt)
        } else {
            text
        };
        request(
            input,
            pending,
            seq,
            "session/prompt",
            json!({"sessionId":session,"prompt":[{"type":"text","text":text}]}),
        )
        .await?;
    }
    Ok(())
}
