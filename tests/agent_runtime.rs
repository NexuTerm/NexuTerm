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
