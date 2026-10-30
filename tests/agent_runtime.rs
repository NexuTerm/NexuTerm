#![cfg(feature = "gui")]
use nexuterm::{
    ai_config::AiKind,
    codex::{Client, Event},
    config::AiAgent,
};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, time::Duration};
async fn next(rx: &mut tokio::sync::mpsc::Receiver<Event>) -> Event {
    tokio::time::timeout(Duration::from_secs(8), rx.recv())
        .await
        .unwrap()
        .unwrap()
}
#[test]
fn acp_models_stream_permissions_cancel_and_private_profiles() {
    let directory = tempfile::tempdir().unwrap();
    let program = directory.path().join("acp");
    std::fs::write(
        &program,
        format!(
            "#!/usr/bin/python3\n{}",
            include_str!("../lab/acp_fixture.py")
        ),
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut agent = AiAgent {
        id: "first".into(),
        name: "OpenCode".into(),
        kind: AiKind::OpenCode,
        program: program.to_string_lossy().into(),
        directory: directory.path().to_string_lossy().into(),
        runtime_root: Some(directory.path().join("private")),
        ..Default::default()
    };
    agent
        .opencode
        .skills
        .paths
        .push("/tmp/synthetic-skill".into());
    nexuterm::runtime::runtime().block_on(async {
        let (client, mut rx) = Client::start(agent.clone(), true);
        let mut models = false;
        loop {
            match next(&mut rx).await {
                Event::Models(m) => {
                    assert_eq!(m.len(), 2);
                    models = true;
                }
                Event::Ready(true) => break,
                Event::Error(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert!(models);
        assert!(client.send_with_options("hello".into(), "fixture/two".into(), String::new()));
        let mut text = String::new();
        loop {
            match next(&mut rx).await {
                Event::Delta { text: delta, .. } => text.push_str(&delta),
                Event::Finished(s) => {
                    assert_eq!(s, "completed");
                    break;
                }
                Event::Error(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert_eq!(text, "Respuesta ACP gráfica.");
        assert!(client.send("approval".into()));
        loop {
            match next(&mut rx).await {
                Event::Request { id, .. } => client.reply(id, json!({"decision":"decline"})),
                Event::Finished(_) => break,
                Event::Error(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert!(client.send("interrupt".into()));
        tokio::time::sleep(Duration::from_millis(100)).await;
        client.interrupt();
        loop {
            if let Event::Finished(s) = next(&mut rx).await {
                assert_eq!(s, "interrupted");
                break;
            }
        }
        client.close();
    });
    let root = nexuterm::agent_profile::root(&agent);
    let contents = std::fs::read_to_string(root.join("extensions.json")).unwrap();
    assert!(contents.contains("synthetic-skill"));
    agent.id = "second".into();
    agent.opencode = Default::default();
    let second = nexuterm::agent_profile::prepare(&agent).unwrap();
    assert_ne!(root, second);
    assert!(
        !std::fs::read_to_string(second.join("extensions.json"))
            .unwrap()
            .contains("synthetic-skill")
    );
    let trace = std::fs::read_to_string(directory.path().join("acp-protocol.jsonl")).unwrap();
    let trace: Vec<Value> = trace
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert!(trace.iter().any(
        |r| r["method"] == "session/set_config_option" && r["params"]["value"] == "fixture/two"
    ));
}
#[test]
fn private_terminal_bridge_only_forwards_requests_and_returns_gui_decision() {
    nexuterm::runtime::runtime().block_on(async {
        let (broker, mut requests) = nexuterm::terminal_mcp::Broker::start().unwrap();
        let path = broker.path();
        let worker = tokio::spawn(async move {
            let request = requests.recv().await.unwrap();
            assert_eq!(request.name, "terminal_execute");
            assert_eq!(request.arguments["terminal_id"], "remote-fixture");
            request
                .response
                .send(nexuterm::terminal_mcp::result(
                    "El usuario rechazó la operación".into(),
                    true,
                ))
                .unwrap();
        });
        let response = nexuterm::terminal_mcp::call(
            path.to_str().unwrap(),
            "Synthetic agent",
            "terminal_execute",
            json!({"terminal_id":"remote-fixture","command":"printf fixture"}),
        )
        .await
        .unwrap();
        assert_eq!(response["isError"], true);
        worker.await.unwrap();
    });
}

#[test]
fn web_login_keeps_callback_alive_across_permissions_and_duplicate_login() {
    let directory = tempfile::tempdir().unwrap();
    let program = directory.path().join("copilot");
    std::fs::write(
        &program,
        format!(
            r#"#!/usr/bin/python3
import json, os, socket, sys
from pathlib import Path
if sys.argv[1:] == ['login', '--web-flow']:
    listener = socket.socket()
    listener.bind(('127.0.0.1', 0))
    listener.listen(1)
    Path(os.environ['HOME'], 'callback-port').write_text(str(listener.getsockname()[1]))
    print('https://github.com/login/fixture', flush=True)
    connection, _ = listener.accept()
    connection.close()
    listener.close()
    state = Path(os.environ['HOME'], '.copilot')
    state.mkdir(exist_ok=True)
    (state / 'config.json').write_text('// Provider-managed state\n' + json.dumps(dict(lastLoggedInUser=dict(login='fixture-user'))))
    sys.exit(0)
{}
"#,
            include_str!("../lab/acp_fixture.py")
        ),
    )
    .unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let agent = AiAgent {
        id: "login-fixture".into(),
        kind: AiKind::Acp,
        program: program.to_string_lossy().into(),
        args: vec!["--acp".into()],
        directory: directory.path().to_string_lossy().into(),
        runtime_root: Some(directory.path().join("private")),
        ..Default::default()
    };
    let root = nexuterm::agent_profile::root(&agent);
    nexuterm::runtime::runtime().block_on(async {
        let (client, mut rx) = Client::start(agent.clone(), true);
        loop {
            match next(&mut rx).await {
                Event::Ready(true) => break,
                Event::Error(error) => panic!("{error}"),
                _ => {}
            }
        }
        client.login();
        loop {
            match next(&mut rx).await {
                Event::LoginLink(_) => break,
                Event::Error(error) => panic!("{error}"),
                _ => {}
            }
        }
        client.permissions(false);
        client.login();
        tokio::time::sleep(Duration::from_millis(150)).await;
        let port: u16 = std::fs::read_to_string(root.join("callback-port"))
            .unwrap()
            .parse()
            .unwrap();
        let callback = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("Callback must survive unrelated chat commands");
        drop(callback);
        let mut account_seen = false;
        let mut authenticated_seen = false;
        loop {
            match next(&mut rx).await {
                Event::Account(Some(name)) => {
                    assert_eq!(name, "fixture-user");
                    account_seen = true;
                }
                Event::Authenticated(true) => authenticated_seen = true,
                Event::Ready(true) => break,
                Event::Error(error) => panic!("{error}"),
                Event::Status(text) => assert!(!text.contains("cancelado")),
                _ => {}
            }
        }
        assert!(account_seen && authenticated_seen);
        assert!(client.send("approval".into()));
        loop {
            match next(&mut rx).await {
                Event::Status(text) if text.contains("solo lectura") => break,
                Event::Request { .. } => panic!("Read-only change was lost during authentication"),
                Event::Error(error) => panic!("{error}"),
                _ => {}
            }
        }
        client.login();
        loop {
            match next(&mut rx).await {
                Event::LoginLink(_) => break,
                Event::Error(error) => panic!("{error}"),
                _ => {}
            }
        }
        let port: u16 = std::fs::read_to_string(root.join("callback-port"))
            .unwrap()
            .parse()
            .unwrap();
        client.interrupt();
        loop {
            match next(&mut rx).await {
                Event::Status(text) if text.contains("cancelado") => break,
                Event::Error(error) => panic!("{error}"),
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(
            tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .is_err(),
            "Explicit cancellation must close the callback"
        );
        client.close();
        let (client, mut rx) = Client::start(agent, false);
        let mut account_seen = false;
        let mut authenticated_seen = false;
        loop {
            match next(&mut rx).await {
                Event::Account(Some(name)) => {
                    assert_eq!(name, "fixture-user");
                    account_seen = true;
                }
                Event::Authenticated(true) => authenticated_seen = true,
                Event::Ready(true) => break,
                Event::Error(error) => panic!("{error}"),
                _ => {}
            }
        }
        assert!(
            account_seen && authenticated_seen,
            "Saved provider account must be restored"
        );
        client.close();
    });
}

#[test]
fn acp_recovery_honours_capabilities_denies_replay_and_never_creates_a_new_session() {
    for (flag, success, method) in [
        ("support-load", true, "session/load"),
        ("support-resume", true, "session/resume"),
        ("unsupported", false, ""),
        ("load-failure", false, "session/load"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let program = dir.path().join("acp");
        std::fs::write(
            &program,
            format!(
                "#!/usr/bin/python3\n{}",
                include_str!("../lab/acp_fixture.py")
            ),
        )
        .unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        if flag != "unsupported" {
            std::fs::write(
                dir.path().join(if flag == "load-failure" {
                    "support-load"
                } else {
                    flag
                }),
                "",
            )
            .unwrap();
        }
        if flag == "load-failure" {
            std::fs::write(dir.path().join(flag), "").unwrap();
        }
        let agent = AiAgent {
            id: "recovery-fixture".into(),
            kind: AiKind::Acp,
            program: program.to_string_lossy().into(),
            directory: dir.path().to_string_lossy().into(),
            runtime_root: Some(dir.path().join("private")),
            ..Default::default()
        };
        let saved = nexuterm::chat_state::Continuation::Acp {
            destination: nexuterm::chat_state::destination(&agent),
            session_id: "fixture-session".into(),
        };
        nexuterm::runtime::runtime().block_on(async {
            let (client, mut rx) = Client::start_recovery(agent, false);
            loop {
                match next(&mut rx).await {
                    Event::Ready(true) => break,
                    Event::Error(e) => panic!("{e}"),
                    _ => {}
                }
            }
            assert!(client.restore(saved));
            let mut continued = false;
            loop {
                match next(&mut rx).await {
                    Event::Continued => {
                        continued = true;
                        break;
                    }
                    Event::Error(e) => {
                        assert!(!success, "{e}");
                        break;
                    }
                    Event::Delta { .. } | Event::Request { .. } => {
                        panic!("replayed content/tools must not be executed or redisplayed")
                    }
                    _ => {}
                }
            }
            assert_eq!(continued, success);
            if continued {
                assert!(client.send("hello".into()));
                let mut answer = String::new();
                loop {
                    match next(&mut rx).await {
                        Event::Delta { text, .. } => answer.push_str(&text),
                        Event::Finished(s) => {
                            assert_eq!(s, "completed");
                            break;
                        }
                        Event::Error(e) => panic!("{e}"),
                        _ => {}
                    }
                }
                assert_eq!(answer, "Respuesta ACP gráfica.");
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
            client.close();
        });
        let log = std::fs::read_to_string(dir.path().join("acp-protocol.jsonl")).unwrap();
        let requests: Vec<Value> = log
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert!(!requests.iter().any(|v| v["method"] == "session/new"));
        if !method.is_empty() {
            assert_eq!(requests.iter().filter(|v| v["method"] == method).count(), 1);
        }
        if flag == "support-load" {
            assert!(requests.iter().any(|v| v["id"] == "replay-permission"
                && v["result"]["outcome"]["outcome"] == "cancelled"));
        }
    }
}
