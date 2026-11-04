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
    let preparation_started=std::time::Instant::now();
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
    println!("NEXUTERM_BENCHMARK ssh_prepare_ms={:.3}",preparation_started.elapsed().as_secs_f64()*1000.0);
    benchmark_tree(&root,connection.clone()).await;
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
    // Explicit binary images use the same authenticated destination and have a tighter cap.
    let asset=remote.parent().unwrap().join("imagen con espacio.png");
    let image_bytes=b"\x89PNG\r\n\x1a\n\x00\xffbinary-image";
    tokio::fs::write(&asset,image_bytes).await.unwrap();
    assert_eq!(doc.read_image("imagen%20con%20espacio.png").await.unwrap(),image_bytes);
    assert!(doc.read_image("https://example.invalid/no-network").await.is_err());
    let oversized=remote.parent().unwrap().join("large-image.png");
    std::fs::File::create(&oversized).unwrap().set_len(nexuterm::markdown_image::MAX_BYTES as u64+1).unwrap();
    assert!(doc.read_image("large-image.png").await.unwrap_err().contains("4 MiB"));

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
    // Recipes reuse the same authenticated channel. Real disk query; synthetic service tools.
    let disk = nexuterm::recipes::Recipe::Disk {path:job_dir.to_string_lossy().into(),max_used_percent:100}.command().unwrap();
    let job = nexuterm::execution::start(spec(&disk,5),nexuterm::execution::Target::Ssh(connection.clone())).unwrap();
    let result = execution_done(&job).await;
    assert!(matches!(result.exit_code,Some(0 | 2)));
    assert!(String::from_utf8_lossy(&result.stdout).contains("VERIFICACION:"));
    println!("NEXUTERM_BENCHMARK diagnostic_disk_ms={} exit={:?}",result.duration_ms,result.exit_code);
    let bin = job_dir.join("recipe-bin");
    tokio::fs::create_dir(&bin).await.unwrap();
    for (name, script) in [
        ("systemctl", "#!/bin/sh\ncase \"$1\" in show) printf 'LoadState=loaded\\nActiveState=inactive\\n';; is-active) exit 3;; *) exit 9;; esac\n"),
        ("journalctl", "#!/bin/sh\nprintf 'remote recipe journal\\n'\n"),
    ] {
        use std::os::unix::fs::PermissionsExt;
        let path = bin.join(name);
        tokio::fs::write(&path,script).await.unwrap();
        tokio::fs::set_permissions(path,std::fs::Permissions::from_mode(0o700)).await.unwrap();
    }
    for (recipe, code, evidence) in [
        (nexuterm::recipes::Recipe::Service{unit:"demo.service".into()},3,"servicio no activo"),
        (nexuterm::recipes::Recipe::Journal{unit:"demo.service".into(),lines:7,minutes:5},0,"remote recipe journal"),
    ] {
        let command = format!("PATH={}:$PATH\n{}",nexuterm::ssh::quote(&bin.to_string_lossy()),recipe.command().unwrap());
        let job = nexuterm::execution::start(spec(&command,5),nexuterm::execution::Target::Ssh(connection.clone())).unwrap();
        let result = execution_done(&job).await;
        assert_eq!(result.exit_code,Some(code));
        assert!(String::from_utf8_lossy(&result.stdout).contains(evidence));
    }
    assert_eq!(connection.capture("printf recipes_channel_alive").await.unwrap(),"recipes_channel_alive");
    let lost_tunnel = check_tunnels(connection.clone()).await;
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
    wait_tunnel(&lost_tunnel,nexuterm::tunnels::Status::Unknown).await;
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

fn unused_port() -> u16 { std::net::TcpListener::bind(("127.0.0.1",0)).unwrap().local_addr().unwrap().port() }
async fn wait_tunnel(h:&nexuterm::tunnels::Handle,status:nexuterm::tunnels::Status){
    tokio::time::timeout(std::time::Duration::from_secs(12),async{loop{let snap=h.snapshot();if snap.status==status{break;}if !matches!(snap.status,nexuterm::tunnels::Status::Starting|nexuterm::tunnels::Status::Active|nexuterm::tunnels::Status::Stopping){panic!("Expected {status:?}, got {snap:?}");}tokio::time::sleep(std::time::Duration::from_millis(25)).await;}}).await.unwrap();
}
async fn check_tunnels(connection:std::sync::Arc<SshConnection>)->std::sync::Arc<nexuterm::tunnels::Handle>{
 use nexuterm::tunnels::{Handle,Spec,Kind,Status};use tokio::io::{AsyncReadExt,AsyncWriteExt};
 let listener=tokio::net::TcpListener::bind(("127.0.0.1",0)).await.unwrap();let destination=listener.local_addr().unwrap().port();
 let echo=tokio::spawn(async move{loop{let (mut stream,_)=listener.accept().await.unwrap();tokio::spawn(async move{let mut b=[0u8;64];if let Ok(n)=stream.read(&mut b).await{let _=stream.write_all(&b[..n]).await;}});}});
 for kind in [Kind::Local,Kind::Remote,Kind::Socks]{
  let port=unused_port();let spec=Spec{kind,bind:"127.0.0.1".parse().unwrap(),port,host:"127.0.0.1".into(),destination_port:destination};
  let h=Handle::start(spec,connection.clone()).unwrap();wait_tunnel(&h,Status::Active).await;
  let mut stream=tokio::net::TcpStream::connect(("127.0.0.1",port)).await.unwrap();
  if kind==Kind::Socks{
   stream.write_all(&[5,1,0]).await.unwrap();let mut greeting=[0;2];stream.read_exact(&mut greeting).await.unwrap();assert_eq!(greeting,[5,0]);
   let mut request=vec![5,1,0,1,127,0,0,1];request.extend(destination.to_be_bytes());stream.write_all(&request).await.unwrap();let mut head=[0;4];stream.read_exact(&mut head).await.unwrap();assert_eq!(head[1],0);let length=match head[3]{1=>6,4=>18,_=>panic!("Unexpected SOCKS address")};let mut address=vec![0;length];stream.read_exact(&mut address).await.unwrap();
  }
  stream.write_all(b"tunnel-proof").await.unwrap();let mut output=[0;12];stream.read_exact(&mut output).await.unwrap();assert_eq!(&output,b"tunnel-proof");drop(stream);
  h.stop();wait_tunnel(&h,Status::Stopped).await;
  assert!(tokio::net::TcpStream::connect(("127.0.0.1",port)).await.is_err());
 }
 let occupied=std::net::TcpListener::bind(("127.0.0.1",0)).unwrap();let port=occupied.local_addr().unwrap().port();
 let failed=Handle::start(Spec{kind:Kind::Local,bind:"127.0.0.1".parse().unwrap(),port,host:"127.0.0.1".into(),destination_port:destination},connection.clone()).unwrap();wait_tunnel(&failed,Status::Unknown).await;assert!(tokio::net::TcpStream::connect(("127.0.0.1",port)).await.is_ok());
 let lost=Handle::start(Spec{kind:Kind::Local,bind:"127.0.0.1".parse().unwrap(),port:unused_port(),host:"127.0.0.1".into(),destination_port:destination},connection).unwrap();wait_tunnel(&lost,Status::Active).await;echo.abort();lost
}

async fn benchmark_tree(root:&std::path::Path,connection:std::sync::Arc<SshConnection>){
 use nexuterm::{transfer::{Client,Request,Event},transfer_queue::{Journal,Job,Edit,Status}};
 let source=root.join("benchmark-source");tokio::fs::create_dir(&source).await.unwrap();
 let payload=vec![b'x';8192];for directory in 0..4{let folder=source.join(format!("folder-{directory}"));tokio::fs::create_dir(&folder).await.unwrap();for file in 0..8{tokio::fs::write(folder.join(format!("file-{file}.txt")),&payload).await.unwrap();}}
 let mut client=Client::start(connection.clone());let planning=std::time::Instant::now();client.requests.send(Request::PlanUpload{local:source,directory:root.join("remote")}).await.unwrap();
 let plan=match client.events.recv().await.unwrap(){Event::Planned(plan)=>plan,other=>panic!("Plan failed: {other:?}")};let planning_ms=planning.elapsed().as_secs_f64()*1000.0;
 assert_eq!(plan.len(),37);assert!(!root.join("remote/benchmark-source").exists());
 let queue_dir=root.join("benchmark-queue");let scope=nexuterm::transfer_queue::scope(&connection.profile);let queue=Journal::open(&queue_dir,&scope).unwrap();let jobs=plan.into_iter().map(|p|{let mut j=Job::new(p.upload,false,p.local,p.remote);j.mkdir=p.directory;j}).collect();queue.edit(Edit::Add(jobs)).unwrap();
 let copy=std::time::Instant::now();while let Some(job)=queue.edit(Edit::Begin).unwrap().1{client.requests.send(job.request()).await.unwrap();assert!(matches!(client.events.recv().await.unwrap(),Event::Finished(Ok(()))));queue.edit(Edit::Finish{id:job.id,error:None,cancelled:false}).unwrap();}
 let copy_ms=copy.elapsed().as_secs_f64()*1000.0;assert!(queue.snapshot().unwrap().jobs.iter().all(|j|j.status==Status::Done));client.stop();drop(queue);
 let recovery=std::time::Instant::now();let queue=Journal::open(&queue_dir,&scope).unwrap();assert!(queue.edit(Edit::Begin).unwrap().1.is_none());let recovery_ms=recovery.elapsed().as_secs_f64()*1000.0;
 for directory in 0..4{for file in 0..8{assert_eq!(tokio::fs::read(root.join(format!("remote/benchmark-source/folder-{directory}/file-{file}.txt"))).await.unwrap(),payload);}}
 println!("NEXUTERM_BENCHMARK tree_files=32 tree_bytes=262144 planning_ms={planning_ms:.3} copy_ms={copy_ms:.3} queue_recovery_ms={recovery_ms:.3}");
}
