//! Run only through lab/check_remote_editor.py: isolated loopback SSH daemon and keys.
use nexuterm::{
    config::{Profile, Protocol},
    remote_edit::{EDIT_LIMIT, RemoteDocument},
    ssh::SshConnection,
};
use std::{path::PathBuf, process::Stdio, sync::atomic::Ordering};
#[tokio::test]
#[ignore = "requires lab/check_remote_editor.py and its disposable loopback SSH server"]
async fn remote_editor_atomic_save_recovery_conflict_and_limits() {
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
    let mut client = nexuterm::transfer::Client::start(connection.clone());
    let directory = root.join("remote");
    for (name, folder) in [("created-dir", true), ("created-file.txt", false)] {
        client
            .requests
            .send(nexuterm::transfer::Request::Create {
                directory: directory.clone(),
                name: name.into(),
                folder,
            })
            .await
            .unwrap();
        assert!(matches!(
            client.events.recv().await.unwrap(),
            nexuterm::transfer::Event::Finished(Ok(()))
        ));
        assert_eq!(directory.join(name).is_dir(), folder);
    }
    tokio::fs::write(directory.join("created-file.txt"), "keep original")
        .await
        .unwrap();
    client
        .requests
        .send(nexuterm::transfer::Request::Create {
            directory: directory.clone(),
            name: "created-file.txt".into(),
            folder: false,
        })
        .await
        .unwrap();
    assert!(matches!(
        client.events.recv().await.unwrap(),
        nexuterm::transfer::Event::Finished(Err(_))
    ));
    assert_eq!(
        tokio::fs::read_to_string(directory.join("created-file.txt"))
            .await
            .unwrap(),
        "keep original"
    );
    client
        .requests
        .send(nexuterm::transfer::Request::Navigate(directory))
        .await
        .unwrap();
    match client.events.recv().await.unwrap() {
        nexuterm::transfer::Event::Page { entries, .. } => {
            assert!(
                entries
                    .iter()
                    .find(|e| e.name == std::path::Path::new("created-dir"))
                    .unwrap()
                    .permissions
                    .is_some()
            );
            assert!(
                entries
                    .iter()
                    .find(|e| e.name == std::path::Path::new("created-file.txt"))
                    .unwrap()
                    .permissions
                    .is_some()
            );
        }
        other => panic!("Unexpected listing: {other:?}"),
    }
    client.stop();

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
    // Recovery belongs to persistent app data, never to the SSH TempDir.
    let draft = nexuterm::recovery::Draft {
        version: 1,
        id: uuid::Uuid::new_v4().to_string(),
        source: doc.recovery_source(),
        text: "Cambios remotos sin guardar\n".into(),
        base_text: Some(original.into()),
    };
    let journal = nexuterm::recovery::Journal::new(&root.join("recovery"), &draft.id).unwrap();
    journal.advance(1);
    assert!(journal.write(1, &draft).unwrap());
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
    let original_inode = {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(&remote).unwrap().ino()
    };
    for value in [2, 3] {
        let text = format!("def value():\n    return {value}\n");
        tokio::fs::write(&local, &text).await.unwrap();
        doc.save(text.clone()).await.unwrap();
        if value == 2 {
            use std::os::unix::fs::MetadataExt;
            assert_ne!(
                std::fs::metadata(&remote).unwrap().ino(),
                original_inode,
                "Publish must replace the inode, not truncate it"
            );
        }
        assert_eq!(tokio::fs::read_to_string(&remote).await.unwrap(), text);
        assert_eq!(
            std::fs::metadata(&remote).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
    assert!(!std::fs::read_dir(root.join("remote")).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".nexuterm-edit-")
    }));
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
    // Three-version review rejects stale content and metadata, and never publishes by itself.
    let review_path=root.join("remote/review.txt");
    tokio::fs::write(&review_path,"base\n").await.unwrap();
    tokio::fs::set_permissions(&review_path,std::fs::Permissions::from_mode(0o2640)).await.unwrap();
    let review_doc=RemoteDocument::open(connection.clone(),review_path.clone()).await.unwrap();
    review_doc.save("base\n".into()).await.unwrap();
    assert_eq!(std::fs::metadata(&review_path).unwrap().permissions().mode()&0o7777,0o2640);
    tokio::fs::write(review_doc.local_path(),"local\n").await.unwrap();
    tokio::fs::write(&review_path,"server\n").await.unwrap();
    assert!(review_doc.save("local\n".into()).await.unwrap_err().contains("Conflicto"));
    let comparison=review_doc.comparison().await.unwrap();
    assert_eq!(comparison.base.as_deref(),Some("base\n"));assert_eq!(comparison.remote,"server\n");
    tokio::fs::write(&review_path,"server newer\n").await.unwrap();
    assert!(review_doc.accept_comparison(&comparison).await.unwrap_err().contains("volvió a cambiar"));
    let comparison=review_doc.comparison().await.unwrap();
    review_doc.accept_comparison(&comparison).await.unwrap();
    assert_eq!(tokio::fs::read_to_string(&review_path).await.unwrap(),"server newer\n");
    tokio::fs::set_permissions(&review_path,std::fs::Permissions::from_mode(0o600)).await.unwrap();
    assert!(review_doc.save("local\n".into()).await.unwrap_err().contains("metadatos"));
    let comparison=review_doc.comparison().await.unwrap();
    review_doc.accept_comparison(&comparison).await.unwrap();
    review_doc.save("local\n".into()).await.unwrap();
    assert_eq!(std::fs::metadata(&review_path).unwrap().permissions().mode()&0o7777,0o600);
    assert_eq!(review_doc.comparison().await.unwrap().base.as_deref(),Some("local\n"));
    // Recovery with a changed origin remains guarded until an explicit review.
    let preview=RemoteDocument::recovery_preview(connection.clone(),&draft.source,draft.base_text.clone()).await.unwrap();
    assert!(preview.requires_review());
    assert!(preview.save("anything".into()).await.unwrap_err().contains("Conflicto"));
    let comparison=preview.comparison().await.unwrap();
    assert_eq!(comparison.base.as_deref(),Some(original));
    preview.accept_comparison(&comparison).await.unwrap();assert!(!preview.requires_review());
    drop(preview);
    let recovered =
        nexuterm::recovery::read(&root.join("recovery").join(format!("{}.json", draft.id)))
            .unwrap();
    assert_eq!(recovered.text, draft.text);
    assert_eq!(recovered.source, draft.source);
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
    let recovered_path = root.join("remote/recovered.py");
    tokio::fs::write(&recovered_path, original).await.unwrap();
    let first = RemoteDocument::open(connection.clone(), recovered_path.clone())
        .await
        .unwrap();
    let origin = first.recovery_source();
    let old_copy = first.local_path().to_path_buf();
    drop(first);
    assert!(!old_copy.exists());
    let resumed = RemoteDocument::recover(connection.clone(), &origin)
        .await
        .unwrap();
    assert_eq!(
        tokio::fs::read_to_string(&recovered_path).await.unwrap(),
        original,
        "Binding must not write to the original"
    );
    let changes = "def value():\n    return 100\n";
    tokio::fs::write(resumed.local_path(), changes)
        .await
        .unwrap();
    resumed.save(changes.into()).await.unwrap();
    let updated = resumed.recovery_source();
    assert_ne!(
        updated, origin,
        "A later draft must remember the newly saved baseline"
    );
    drop(resumed);
    assert_eq!(
        tokio::fs::read_to_string(&recovered_path).await.unwrap(),
        changes
    );
    assert!(
        RemoteDocument::recover(connection.clone(), &origin)
            .await
            .is_err(),
        "Stale baseline must be refused"
    );
    let mut legacy = updated.clone();
    if let nexuterm::recovery::Source::Remote { fingerprint, .. } = &mut legacy {
        *fingerprint = None;
    }
    assert!(
        RemoteDocument::recover(connection.clone(), &legacy)
            .await
            .is_err()
    );
    let mut wrong_target = updated.clone();
    if let nexuterm::recovery::Source::Remote { destination, .. } = &mut wrong_target {
        *destination = "0".repeat(64);
    }
    assert!(
        RemoteDocument::recover(connection.clone(), &wrong_target)
            .await
            .is_err()
    );
    let readonly = root.join("remote/readonly");
    tokio::fs::create_dir(&readonly).await.unwrap();
    let protected = readonly.join("file.txt");
    tokio::fs::write(&protected, "unchanged").await.unwrap();
    let protected_doc = RemoteDocument::open(connection.clone(), protected.clone())
        .await
        .unwrap();
    tokio::fs::write(protected_doc.local_path(), "must not overwrite")
        .await
        .unwrap();
    tokio::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o500))
        .await
        .unwrap();
    let failed = protected_doc.save("must not overwrite".into()).await;
    tokio::fs::set_permissions(&readonly, std::fs::Permissions::from_mode(0o700))
        .await
        .unwrap();
    assert!(failed.is_err());
    assert_eq!(
        tokio::fs::read_to_string(&protected).await.unwrap(),
        "unchanged"
    );
    assert_eq!(
        tokio::fs::read_to_string(protected_doc.local_path())
            .await
            .unwrap(),
        "must not overwrite"
    );
    let doc = RemoteDocument::open(connection.clone(), remote.clone())
        .await
        .unwrap();
    // Supervised SSH runs in its own channel with explicit cwd and no terminal injection.
    let job_dir=root.join("remote/space ' supervised");tokio::fs::create_dir(&job_dir).await.unwrap();
    let spec=|command:&str,timeout_secs:u64|nexuterm::execution::Spec{command:command.into(),directory:job_dir.clone(),timeout_secs,keep_history:false};
    let job=nexuterm::execution::start(spec("printf '%s' \"$PWD\"; printf 'remote error' >&2; exit 13",5),nexuterm::execution::Target::Ssh(connection.clone())).unwrap();
    let result=execution_done(&job).await;assert_eq!(result.stdout,job_dir.to_string_lossy().as_bytes());assert_eq!(result.stderr,b"remote error");assert_eq!(result.exit_code,Some(13));assert_eq!(result.status,nexuterm::execution::Status::Completed);
    let job=nexuterm::execution::start(spec("sleep 2",1),nexuterm::execution::Target::Ssh(connection.clone())).unwrap();let result=execution_done(&job).await;assert_eq!(result.status,nexuterm::execution::Status::Unknown);assert_eq!(result.exit_code,None);
    let job=nexuterm::execution::start(spec("sleep 2",5),nexuterm::execution::Target::Ssh(connection.clone())).unwrap();
    for _ in 0..100{if job.snapshot().status==nexuterm::execution::Status::Running{break;}tokio::time::sleep(std::time::Duration::from_millis(10)).await;}job.cancel();assert_eq!(execution_done(&job).await.status,nexuterm::execution::Status::Unknown);
    assert_eq!(connection.capture("printf channel_alive").await.unwrap(),"channel_alive");
    let job=nexuterm::execution::start(spec("exit 255",5),nexuterm::execution::Target::Ssh(connection.clone())).unwrap();assert_eq!(execution_done(&job).await.status,nexuterm::execution::Status::Unknown);
    connection.closed.store(true, Ordering::Relaxed);
    assert!(doc.save("other".into()).await.is_err());
    assert!(
        tokio::fs::read_to_string(&remote)
            .await
            .unwrap()
            .contains("9")
    );
    // Interrupt a real upload after observing partial data in its private temporary.
    connection.closed.store(false, Ordering::Relaxed);
    let interrupted = root.join("remote/interrupted.txt");
    tokio::fs::write(&interrupted, "original completo")
        .await
        .unwrap();
    let interrupted_doc = RemoteDocument::open(connection.clone(), interrupted.clone())
        .await
        .unwrap();
    let payload = "z".repeat(EDIT_LIMIT);
    tokio::fs::write(interrupted_doc.local_path(), &payload)
        .await
        .unwrap();
    let saving = interrupted_doc.clone();
    let upload = tokio::spawn(async move { saving.save(payload).await });
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let partial = std::fs::read_dir(root.join("remote"))
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".nexuterm-edit-")
                })
                .filter_map(|entry| std::fs::metadata(entry.path().join("content")).ok())
                .any(|metadata| metadata.len() > 0 && metadata.len() < EDIT_LIMIT as u64);
            if partial {
                break;
            }
            assert!(
                !upload.is_finished(),
                "Upload completed before interruption could be exercised"
            );
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("Expected a partial temporary upload");
    master.kill().await.unwrap();
    assert!(upload.await.unwrap().is_err());
    let original_after = tokio::fs::read_to_string(&interrupted).await.unwrap();
    assert!(
        original_after == "original completo" || original_after == "z".repeat(EDIT_LIMIT),
        "The published path must never contain partial data"
    );
    assert_eq!(
        tokio::fs::metadata(interrupted_doc.local_path())
            .await
            .unwrap()
            .len(),
        EDIT_LIMIT as u64
    );
}

async fn execution_done(handle:&nexuterm::execution::Handle)->nexuterm::execution::Record{
    tokio::time::timeout(std::time::Duration::from_secs(8),async{while !handle.finished(){tokio::time::sleep(std::time::Duration::from_millis(10)).await;}}).await.unwrap();handle.snapshot()
}
