//! Real curl transport against disposable loopback HTTP fixtures. No provider calls.
use nexuterm::{
    ai::{self, Message},
    ai_config::AiKind,
    config::AiAgent,
};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use zeroize::Zeroizing;

fn read_request(stream: &mut std::net::TcpStream) -> (String, Value) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut request = Vec::new();
    let header_end;
    loop {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        assert!(n > 0);
        request.extend_from_slice(&buffer[..n]);
        if let Some(end) = request.windows(4).position(|s| s == b"\r\n\r\n") {
            header_end = end + 4;
            break;
        }
        assert!(request.len() < 65536);
    }
    let headers = String::from_utf8(request[..header_end].to_vec()).unwrap();
    let length: usize = headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|s| s.trim().parse().unwrap())
        })
        .unwrap();
    while request.len() < header_end + length {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).unwrap();
        assert!(n > 0);
        request.extend_from_slice(&buffer[..n]);
    }
    (
        headers,
        serde_json::from_slice(&request[header_end..header_end + length]).unwrap(),
    )
}
fn agent(port: u16, kind: AiKind) -> AiAgent {
    AiAgent {
        id: "fixture".into(),
        name: "Fixture".into(),
        kind,
        endpoint: format!("http://127.0.0.1:{port}/v1"),
        model: "fixture-model".into(),
        ..AiAgent::default()
    }
}
fn history() -> Vec<Message> {
    vec![Message {
        role: "user".into(),
        content: "Texto ficticio con Unicode: áéí\n\"$()\"".into(),
    }]
}

#[tokio::test]
async fn actual_transport_preserves_context_and_parses_both_api_formats() {
    for kind in [AiKind::Responses, AiKind::ChatCompletions] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let (headers, body) = read_request(&mut stream);
            assert!(headers.contains("Authorization: Bearer fixture-key\r\n"));
            assert_eq!(body["model"], "fixture-model");
            let response = if kind == AiKind::Responses {
                assert!(headers.starts_with("POST /v1/responses HTTP/"));
                assert_eq!(body["store"], false);
                assert_eq!(body["input"].as_array().unwrap().len(),2);
                assert_eq!(body["input"][0]["role"],"system");
                assert_eq!(body["input"][1]["role"],"user");
                assert_eq!(body["input"][1]["content"], history()[0].content);
                json!({"status":"completed","output":[{"type":"message","content":[{"type":"output_text","text":"Respuesta ficticia"}]}]})
            } else {
                assert!(headers.starts_with("POST /v1/chat/completions HTTP/"));
                assert_eq!(body["messages"][0]["role"], "system");
                assert_eq!(body["messages"][1]["content"], history()[0].content);
                json!({"choices":[{"message":{"content":"Respuesta ficticia"}}]})
            }.to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
        });
        let answer = ai::complete(
            agent(port, kind),
            Zeroizing::new("fixture-key".into()),
            history(),
        )
        .await
        .unwrap();
        assert_eq!(answer, "Respuesta ficticia");
        server.join().unwrap();
    }
}

#[tokio::test]
async fn errors_do_not_echo_provider_body_or_credentials() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        let body = r#"{"error":{"message":"sensitive-fixture-body"}}"#;
        write!(
            stream,
            "HTTP/1.1 401 Unauthorized\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .unwrap();
    });
    let error = ai::complete(
        agent(port, AiKind::Responses),
        Zeroizing::new("fixture-key".into()),
        history(),
    )
    .await
    .unwrap_err();
    assert!(!error.contains("sensitive-fixture-body"));
    assert!(!error.contains("fixture-key"));
    server.join().unwrap();
}

#[tokio::test]
async fn cancelling_the_task_closes_the_transport_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let started = Arc::new(AtomicBool::new(false));
    let ready = started.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        ready.store(true, Ordering::SeqCst);
        let mut byte = [0];
        assert_eq!(
            stream.read(&mut byte).unwrap(),
            0,
            "aborting must terminate curl and close its socket"
        );
    });
    let request = tokio::spawn(ai::complete(
        agent(port, AiKind::Responses),
        Zeroizing::new(String::new()),
        history(),
    ));
    for _ in 0..100 {
        if started.load(Ordering::SeqCst) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(started.load(Ordering::SeqCst));
    request.abort();
    assert!(request.await.unwrap_err().is_cancelled());
    server.join().unwrap();
}

#[cfg(feature = "gui")]
#[test]
fn completed_api_context_rehydrates_without_network_or_tool_replay() {
    use nexuterm::codex::{Client, Event};
    for kind in [AiKind::Responses, AiKind::ChatCompletions] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = count.clone();
        let server = thread::spawn(move || {
            for index in 1..=2 {
                let (mut stream, _) = listener.accept().unwrap();
                let (_, body) = read_request(&mut stream);
                let messages = body[if kind == AiKind::Responses {
                    "input"
                } else {
                    "messages"
                }]
                .as_array()
                .unwrap();
                assert_eq!(
                    messages.last().unwrap()["content"],
                    if index == 1 { "primero" } else { "segundo" }
                );
                if index == 2 {
                    if kind == AiKind::Responses {
                        assert_eq!(messages.len(), 3);
                        assert_eq!(messages[0]["content"], "primero");
                        assert_eq!(messages[1]["content"], "respuesta-1");
                    } else {
                        assert_eq!(messages.len(), 4);
                        assert_eq!(messages[0]["role"], "system");
                        assert_eq!(messages[1]["content"], "primero");
                        assert_eq!(messages[2]["content"], "respuesta-1");
                    }
                }
                counted.fetch_add(1, Ordering::Relaxed);
                let answer = format!("respuesta-{index}");
                let value = if kind == AiKind::Responses {
                    json!({"status":"completed","output":[{"type":"message","content":[{"type":"output_text","text":answer}]}]})
                } else {
                    json!({"choices":[{"message":{"content":answer}}]})
                };
                let data = value.to_string();
                write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",data.len(),data).unwrap();
            }
        });
        let profile = agent(port, kind);
        nexuterm::runtime::runtime().block_on(async {
            async fn event(rx: &mut tokio::sync::mpsc::Receiver<Event>) -> Event {
                tokio::time::timeout(Duration::from_secs(8), rx.recv())
                    .await
                    .unwrap()
                    .unwrap()
            }
            let (client, mut rx) = Client::start(profile.clone(), false);
            client.credentials(Ok(Zeroizing::new("fixture-key".into())));
            loop {
                if let Event::Ready(true) = event(&mut rx).await {
                    break;
                }
            }
            assert!(client.send("primero".into()));
            let mut checkpoint = None;
            loop {
                match event(&mut rx).await {
                    Event::Continuation(value) => checkpoint = Some(value),
                    Event::Finished(_) => break,
                    Event::Error(e) => panic!("{e}"),
                    _ => {}
                }
            }
            client.close();
            let encoded = serde_json::to_vec(&checkpoint.unwrap()).unwrap();
            assert!(!String::from_utf8_lossy(&encoded).contains("fixture-key"));
            let checkpoint = serde_json::from_slice(&encoded).unwrap();
            let (client, mut rx) = Client::start_recovery(profile, false);
            client.credentials(Ok(Zeroizing::new("fixture-key".into())));
            loop {
                if let Event::Ready(true) = event(&mut rx).await {
                    break;
                }
            }
            assert!(client.restore(checkpoint));
            loop {
                match event(&mut rx).await {
                    Event::Continued => break,
                    Event::Error(e) => panic!("{e}"),
                    _ => {}
                }
            }
            assert_eq!(
                count.load(Ordering::Relaxed),
                1,
                "restoring context must not contact the provider"
            );
            assert!(client.send("segundo".into()));
            loop {
                match event(&mut rx).await {
                    Event::Finished(_) => break,
                    Event::Error(e) => panic!("{e}"),
                    _ => {}
                }
            }
            client.close();
        });
        server.join().unwrap();
    }
}
