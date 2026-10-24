//! Run only through lab/check_remote_editor.py: isolated loopback SSH daemon and keys.
use nexuterm::{
    config::{Profile, Protocol},
    remote_edit::{EDIT_LIMIT, RemoteDocument},
    ssh::SshConnection,
};
use std::{path::PathBuf, process::Stdio, sync::atomic::Ordering};
#[tokio::test]
#[ignore = "requires lab/check_remote_editor.py and its disposable loopback SSH server"]
async fn remote_editor_scp_roundtrip_conflict_and_limits() {
    let root = PathBuf::from(
        std::env::var_os("NEXUTERM_REMOTE_FIXTURE_ROOT").expect("Use lab/check_remote_editor.py"),
    );
    let connection = SshConnection::new(Profile {
        id: "editor-fixture".into(),
        name: "Editor fixture".into(),
        protocol: Protocol::Ssh,
        host: "127.0.0.1".into(),
        port: std::env::var("NEXUTERM_REMOTE_FIXTURE_PORT")
            .unwrap()
            .parse()
            .unwrap(),
        username: std::env::var("NEXUTERM_REMOTE_FIXTURE_USER").unwrap(),
        identity: root.join("client").to_string_lossy().into(),
        ..Profile::default()
    })
    .unwrap();
    let spec = connection.terminal_command().unwrap();
    let mut master = tokio::process::Command::new(spec.program)
        .args([
            "-F",
            "/dev/null",
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
        .args(spec.args)
        .arg("/bin/sleep 90")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    connection.ready().await.unwrap();
    let remote = root.join("remote").join("á ' $(echo unsafe) example.py");
    let original = "def value():\n    return 1\n";
    tokio::fs::write(&remote, original).await.unwrap();
    use std::os::unix::fs::PermissionsExt;
    tokio::fs::set_permissions(&remote, std::fs::Permissions::from_mode(0o640))
        .await
        .unwrap();
    let doc = RemoteDocument::open(connection.clone(), remote.clone())
        .await
        .unwrap();
    let local = doc.local_path().to_path_buf();
    assert_eq!(tokio::fs::read_to_string(&local).await.unwrap(), original);
    assert_eq!(
        std::fs::metadata(&local).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::metadata(local.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    for value in [2, 3] {
        let text = format!("def value():\n    return {value}\n");
        tokio::fs::write(&local, &text).await.unwrap();
        doc.save(text.clone()).await.unwrap();
        assert_eq!(tokio::fs::read_to_string(&remote).await.unwrap(), text);
        assert_eq!(
            std::fs::metadata(&remote).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
    tokio::fs::write(&remote, "def value():\n    return 9\n")
        .await
        .unwrap();
    assert!(
        doc.save("def value():\n    return 3\n".into())
            .await
            .unwrap_err()
            .contains("Conflicto")
    );
    assert!(
        tokio::fs::read_to_string(&remote)
            .await
            .unwrap()
            .contains("9")
    );
    drop(doc);
    assert!(
        !local.exists(),
        "Private temporary copy must disappear when the document closes"
    );
    let binary = root.join("remote/binary");
    tokio::fs::write(&binary, [0, 255]).await.unwrap();
    assert!(
        RemoteDocument::open(connection.clone(), binary)
            .await
            .is_err()
    );
    let huge = root.join("remote/huge");
    tokio::fs::write(&huge, vec![b'x'; EDIT_LIMIT + 1])
        .await
        .unwrap();
    assert!(
        RemoteDocument::open(connection.clone(), huge)
            .await
            .is_err()
    );
    let link = root.join("remote/link");
    std::os::unix::fs::symlink(&remote, &link).unwrap();
    assert!(
        RemoteDocument::open(connection.clone(), link)
            .await
            .is_err()
    );
    let doc = RemoteDocument::open(connection.clone(), remote.clone())
        .await
        .unwrap();
    connection.closed.store(true, Ordering::Relaxed);
    assert!(doc.save("other".into()).await.is_err());
    assert!(
        tokio::fs::read_to_string(&remote)
            .await
            .unwrap()
            .contains("9")
    );
    master.kill().await.unwrap();
}
