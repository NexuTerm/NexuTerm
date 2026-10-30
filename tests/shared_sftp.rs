//! Explicit opt-in: only the loopback-only container created by lab/lab.py.
use nexuterm::{
    config::{Profile, Protocol},
    ssh::SshConnection,
    transfer::{self, Client, Event, Request},
};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

#[tokio::test]
#[ignore = "requires the local rootless laboratory: python3 lab/lab.py up && python3 lab/lab.py trust"]
async fn shared_socket_navigation_and_transfers() {
    tokio::time::timeout(Duration::from_secs(90), laboratory(Protocol::Ssh))
        .await
        .expect("laboratory timeout");
}
#[tokio::test]
#[ignore = "requires the disposable local laboratory"]
async fn standalone_sftp_navigation_and_transfers() {
    tokio::time::timeout(Duration::from_secs(90), laboratory(Protocol::Sftp))
        .await
        .expect("laboratory timeout");
}
async fn laboratory(protocol: Protocol) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".lab");
    assert!(
        root.join("known_hosts").exists(),
        "Run lab/lab.py trust first"
    );
    let connection = SshConnection::new(Profile {
        id: "lab-only".into(),
        name: "Local lab".into(),
        protocol,
        host: "127.0.0.1".into(),
        port: 22222,
        username: "tester".into(),
        identity: root.join("id_ed25519").to_string_lossy().into(),
        ..Profile::default()
    })
    .unwrap();
    let spec = connection.terminal_command().unwrap();
    let mut command = tokio::process::Command::new(spec.program);
    command
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "IdentitiesOnly=yes",
            "-o",
            "StrictHostKeyChecking=yes",
            "-o",
        ])
        .arg(format!(
            "UserKnownHostsFile={}",
            root.join("known_hosts").display()
        ))
        .args(spec.args);
    if protocol == Protocol::Ssh {
        command.arg("sleep 120");
    }
    let mut master = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    connection.ready().await.unwrap();
    let mut client = Client::start(connection.clone());
    client
        .requests
        .send(Request::Navigate(".".into()))
        .await
        .unwrap();
    let event = client.events.recv().await.unwrap();
    match event {
        Event::Page { directory, .. } => assert_eq!(directory, PathBuf::from("/home/tester")),
        event => panic!("unexpected event {event:?}"),
    }
    let channel = connection.sftp().await.unwrap();
    let remote = PathBuf::from(format!(
        "/home/tester/nexuterm-test-{}",
        uuid::Uuid::new_v4()
    ));
    channel.sftp.fs().create_dir(&remote).await.unwrap();
    let local = tempfile::tempdir().unwrap();
    let filename = "á Unicode ' $ archivo.bin";
    let source = local.path().join(filename);
    let content: Vec<u8> = (0..262_171).map(|i| (i % 251) as u8).collect();
    tokio::fs::write(&source, &content).await.unwrap();
    let cancel = AtomicBool::new(false);
    let stop = AtomicBool::new(false);
    let (progress, _) = tokio::sync::watch::channel(transfer::Progress::default());
    transfer::upload(
        &channel.sftp,
        source.clone(),
        remote.clone(),
        &cancel,
        &stop,
        &progress,
    )
    .await
    .unwrap();
    client
        .requests
        .send(Request::Navigate(remote.clone()))
        .await
        .unwrap();
    match client.events.recv().await.unwrap() {
        Event::Page { entries, .. } => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].name, PathBuf::from(filename));
        }
        event => panic!("unexpected event {event:?}"),
    }
    let target = local.path().join("download.bin");
    transfer::download(
        &channel.sftp,
        remote.join(filename),
        target.clone(),
        &cancel,
        &stop,
        &progress,
    )
    .await
    .unwrap();
    assert_eq!(tokio::fs::read(&target).await.unwrap(), content);
    // Neither remote nor local conflicts may destroy existing content.
    assert!(
        transfer::upload(
            &channel.sftp,
            source.clone(),
            remote.clone(),
            &cancel,
            &stop,
            &progress
        )
        .await
        .is_err()
    );
    assert!(
        transfer::download(
            &channel.sftp,
            remote.join(filename),
            target.clone(),
            &cancel,
            &stop,
            &progress
        )
        .await
        .is_err()
    );
    assert_eq!(tokio::fs::read(&target).await.unwrap(), content);
    cancel.store(true, Ordering::Relaxed);
    let cancelled = local.path().join("cancelled.bin");
    assert!(
        transfer::download(
            &channel.sftp,
            remote.join(filename),
            cancelled.clone(),
            &cancel,
            &stop,
            &progress
        )
        .await
        .unwrap_err()
        .contains("cancelada")
    );
    assert!(!cancelled.exists());
    cancel.store(false, Ordering::Relaxed);
    // Cancelling after bytes have moved must not publish a partial final file.
    assert!(
        channel.sftp.support_hardlink(),
        "Our OpenSSH laboratory must support exclusive publication"
    );
    let large = local.path().join("cancel-during.bin");
    let sparse = std::fs::File::create(&large).unwrap();
    sparse.set_len(16 * 1024 * 1024).unwrap();
    drop(sparse);
    let mut updates = progress.subscribe();
    progress.send_replace(transfer::Progress::default());
    let copying = transfer::upload(
        &channel.sftp,
        large.clone(),
        remote.clone(),
        &cancel,
        &stop,
        &progress,
    );
    let cancelling = async {
        loop {
            if updates.borrow().bytes > 0 {
                cancel.store(true, Ordering::Relaxed);
                break;
            }
            updates.changed().await.unwrap();
        }
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(15), async {
        tokio::join!(copying, cancelling)
    })
    .await
    .unwrap();
    assert!(result.is_err());
    assert!(
        channel
            .sftp
            .fs()
            .symlink_metadata(remote.join("cancel-during.bin"))
            .await
            .is_err()
    );
    cancel.store(false, Ordering::Relaxed);
    transfer::upload(
        &channel.sftp,
        large,
        remote.clone(),
        &cancel,
        &stop,
        &progress,
    )
    .await
    .unwrap();
    let cancelled_download = local.path().join("cancel-during-download.bin");
    progress.send_replace(transfer::Progress::default());
    let mut updates = progress.subscribe();
    let copying = transfer::download(
        &channel.sftp,
        remote.join("cancel-during.bin"),
        cancelled_download.clone(),
        &cancel,
        &stop,
        &progress,
    );
    let cancelling = async {
        loop {
            if updates.borrow().bytes > 0 {
                cancel.store(true, Ordering::Relaxed);
                break;
            }
            updates.changed().await.unwrap();
        }
    };
    let (result, ()) = tokio::time::timeout(Duration::from_secs(15), async {
        tokio::join!(copying, cancelling)
    })
    .await
    .unwrap();
    assert!(result.is_err());
    assert!(!cancelled_download.exists());
    cancel.store(false, Ordering::Relaxed);
    transfer::download(
        &channel.sftp,
        remote.join("cancel-during.bin"),
        cancelled_download.clone(),
        &cancel,
        &stop,
        &progress,
    )
    .await
    .unwrap();
    assert_eq!(
        std::fs::metadata(cancelled_download).unwrap().len(),
        16 * 1024 * 1024
    );
    channel
        .sftp
        .fs()
        .remove_file(remote.join("cancel-during.bin"))
        .await
        .unwrap();
    // More than one page, without accumulating a directory's entire listing in the UI.
    for i in 0..205 {
        channel
            .sftp
            .create(remote.join(format!("page-{i:03}")))
            .await
            .unwrap()
            .close()
            .await
            .unwrap();
    }
    client
        .requests
        .send(Request::Navigate(remote.clone()))
        .await
        .unwrap();
    match client.events.recv().await.unwrap() {
        Event::Page { entries, more, .. } => {
            assert_eq!(entries.len(), transfer::PAGE_SIZE);
            assert!(more);
        }
        event => panic!("unexpected event {event:?}"),
    }
    client.requests.send(Request::Next).await.unwrap();
    match client.events.recv().await.unwrap() {
        Event::Page { entries, more, .. } => {
            assert_eq!(entries.len(), 6);
            assert!(!more);
        }
        event => panic!("unexpected event {event:?}"),
    }
    for i in 0..205 {
        channel
            .sftp
            .fs()
            .remove_file(remote.join(format!("page-{i:03}")))
            .await
            .unwrap();
    }
    channel
        .sftp
        .fs()
        .remove_file(remote.join(filename))
        .await
        .unwrap();
    channel.sftp.fs().remove_dir(&remote).await.unwrap();
    client.stop();
    drop(client);
    channel.close().await;
    master.kill().await.unwrap();
    let _ = master.wait().await;
    // Missing master must fail, never establish an unrelated fallback connection.
    assert!(
        tokio::time::timeout(Duration::from_secs(3), connection.capture("true"))
            .await
            .is_err()
    );
    connection.closed.store(true, Ordering::Relaxed);
}
