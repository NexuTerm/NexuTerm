#![cfg(feature = "gui")]
use nexuterm::{
    codex::{Client, Event},
    config::AiAgent,
};
use serde_json::{Value, json};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};
struct Fixture(PathBuf);
impl Fixture {
    fn new(unauthenticated: bool) -> Self {
        let dir = std::env::temp_dir().join(format!("nexuterm-codex-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let script = dir.join("codex");
        std::fs::write(
            &script,
            format!(
                "#!/usr/bin/python3\n{}",
                include_str!("../lab/codex_fixture.py")
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        if unauthenticated {
            std::fs::write(dir.join("unauthenticated"), "").unwrap();
        }
        Self(dir)
    }
    fn start(&self) -> (Client, tokio::sync::mpsc::Receiver<Event>) {
        Client::start(
            AiAgent {
                program: self.0.join("codex").to_string_lossy().into(),
                directory: self.0.to_string_lossy().into(),
                runtime_root: Some(self.0.join("data")),
                ..Default::default()
            },
            false,
        )
    }
    fn requests(&self) -> Vec<Value> {
        std::fs::read_to_string(self.0.join("protocol.jsonl"))
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
async fn event(rx: &mut tokio::sync::mpsc::Receiver<Event>) -> Event {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .expect("server closed");
        if !matches!(
            event,
            Event::Capabilities { .. }
                | Event::Account(_)
                | Event::RateLimits(_)
                | Event::ContextUsage { .. }
                | Event::TokenUsage { .. }
        ) {
            return event;
        }
    }
}

#[test]
fn native_protocol_streams_reuses_thread_denies_approval_and_interrupts() {
    let fixture = Fixture::new(false);
    nexuterm::runtime::runtime().block_on(async {
        let (client, mut rx) = fixture.start();
        assert!(matches!(event(&mut rx).await, Event::Ready(true)));
        for _ in 0..2 {
            assert!(client.send("hola".into()));
            let mut text = String::new();
            loop {
                match event(&mut rx).await {
                    Event::Delta { text: delta, .. } => text.push_str(&delta),
                    Event::Finished(s) => {
                        assert_eq!(s, "completed");
                        break;
                    }
                    Event::Error(e) => panic!("{e}"),
                    _ => {}
                }
            }
            assert_eq!(text, "Respuesta gráfica de Codex.");
        }
        assert!(client.send("approval".into()));
        loop {
            match event(&mut rx).await {
                Event::Request { id, method, .. } => {
                    assert_eq!(method, "item/commandExecution/requestApproval");
                    client.reply(id, json!({"decision":"decline"}));
                }
                Event::Finished(_) => break,
                Event::Error(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert!(client.send("interrupt".into()));
        tokio::time::sleep(Duration::from_millis(100)).await;
        client.interrupt();
        loop {
            if let Event::Finished(s) = event(&mut rx).await {
                assert_eq!(s, "interrupted");
                break;
            }
        }
        client.close();
        let requests = fixture.requests();
        assert_eq!(
            requests
                .iter()
                .filter(|r| r["method"] == "thread/start")
                .count(),
            1
        );
        assert_eq!(
            requests
                .iter()
                .filter(|r| r["method"] == "turn/start")
                .count(),
            4
        );
        assert_eq!(
            requests
                .iter()
                .find(|r| r["method"] == "thread/start")
                .unwrap()["params"]["sandbox"],
            "read-only"
        );
    });
}
#[test]
fn native_login_uses_official_url_and_rechecks_account() {
    let fixture = Fixture::new(true);
    nexuterm::runtime::runtime().block_on(async {
        let (client, mut rx) = fixture.start();
        client.login();
        let mut url = false;
        loop {
            match event(&mut rx).await {
                Event::LoginUrl(value) => {
                    assert_eq!(value, "https://example.invalid/login");
                    url = true;
                }
                Event::Ready(true) => break,
                Event::Error(e) => panic!("{e}"),
                _ => {}
            }
        }
        assert!(url);
        assert!(!client.busy());
        client.close();
        assert_eq!(
            fixture
                .requests()
                .iter()
                .filter(|r| r["method"] == "account/read")
                .count(),
            2
        );
    });
}
#[test]
fn malformed_server_output_closes_without_accepting_requests() {
    let fixture = Fixture::new(false);
    std::fs::write(
        fixture.0.join("codex"),
        "#!/usr/bin/python3\nprint('invalid json',flush=True)\n",
    )
    .unwrap();
    nexuterm::runtime::runtime().block_on(async {
        let (client, mut rx) = fixture.start();
        assert!(matches!(event(&mut rx).await, Event::Error(_)));
        assert!(matches!(event(&mut rx).await, Event::Closed));
        assert!(!client.busy());
    });
}

#[test]
fn history_models_permissions_and_reasoning_are_real_protocol_requests() {
    let fixture = Fixture::new(false);
    nexuterm::runtime::runtime().block_on(async {
        let (client, mut rx) = fixture.start();
        let mut history = false;
        let mut models = false;
        while !history || !models {
            match event(&mut rx).await {
                Event::History(data) => {
                    assert_eq!(data[0]["id"], "old-thread");
                    history = true;
                }
                Event::Models(data) => {
                    assert_eq!(data[0]["model"], "fixture-model");
                    models = true;
                }
                Event::Error(e) => panic!("{e}"),
                _ => {}
            }
        }
        client.permissions(true);
        assert!(client.resume("old-thread".into()));
        loop {
            if let Event::Resumed(thread) = event(&mut rx).await {
                assert_eq!(thread["turns"][0]["items"][1]["text"], "Respuesta anterior");
                break;
            }
        }
        assert!(client.send_with_options(
            "Contexto elegido".into(),
            "fixture-model".into(),
            "high".into()
        ));
        loop {
            if let Event::Finished(_) = event(&mut rx).await {
                break;
            }
        }
        client.close();
        let requests = fixture.requests();
        let resume = requests
            .iter()
            .find(|r| r["method"] == "thread/resume")
            .unwrap();
        assert_eq!(resume["params"]["sandbox"], "workspace-write");
        let turn = requests
            .iter()
            .find(|r| r["method"] == "turn/start")
            .unwrap();
        assert_eq!(turn["params"]["model"], "fixture-model");
        assert_eq!(turn["params"]["effort"], "high");
    });
}

#[tokio::test]
async fn account_and_real_quota_windows_are_read_and_updated() {
    let fixture = Fixture::new(false);
    let (client, mut rx) = fixture.start();
    let mut account = false;
    let mut quota = false;
    while !account || !quota {
        match tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap()
        {
            Event::Account(user) => {
                assert_eq!(user.as_deref(), Some("fixture@example.invalid"));
                account = true;
            }
            Event::RateLimits(value) => {
                let windows = nexuterm::ai_usage::windows(&value, "fixture-model");
                assert_eq!(windows[0].remaining, 75.0);
                assert_eq!(windows[1].remaining, 40.0);
                quota = true;
            }
            Event::Error(error) => panic!("{error}"),
            _ => {}
        }
    }
    // Readiness and usage responses are independent asynchronous events.
    while client.busy() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(client.send("usage".into()));
    let mut context = false;
    let mut updated = false;
    while !context || !updated {
        match tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap()
        {
            Event::ContextUsage { used, size } => {
                assert_eq!((used, size), (1024, 4096));
                context = true;
            }
            Event::RateLimits(value) => {
                let windows = nexuterm::ai_usage::windows(&value, "fixture-model");
                assert_eq!(windows[0].remaining, 74.0);
                updated = true;
            }
            Event::Error(error) => panic!("{error}"),
            _ => {}
        }
    }
    client.close();
    assert!(
        fixture
            .requests()
            .iter()
            .any(|r| r["method"] == "account/rateLimits/read")
    );
}

#[tokio::test]
async fn unavailable_quota_does_not_break_the_conversation() {
    let fixture = Fixture::new(false);
    std::fs::write(fixture.0.join("no-rate-limits"), "").unwrap();
    let (client, mut rx) = fixture.start();
    let mut ready = false;
    let mut unknown = false;
    while !ready || !unknown {
        match tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap()
        {
            Event::Ready(value) => {
                assert!(value);
                ready = true;
            }
            Event::RateLimits(value) => {
                assert!(value.is_null());
                unknown = true;
            }
            Event::Error(error) => panic!("{error}"),
            _ => {}
        }
    }
    assert!(client.send("hello".into()));
    loop {
        match event(&mut rx).await {
            Event::Finished(status) => {
                assert_eq!(status, "completed");
                break;
            }
            Event::Error(error) => panic!("{error}"),
            _ => {}
        }
    }
    client.close();
}
