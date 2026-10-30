//! Bounded SFTP operations on the authenticated terminal's OpenSSH control socket.
use crate::ssh::{SftpChannel, SshConnection, join_remote};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use openssh_sftp_client::{Sftp, fs::ReadDir};
use std::{
    path::PathBuf,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::{mpsc, watch},
};

pub const PAGE_SIZE: usize = 200;
const BLOCK: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: PathBuf,
    pub directory: bool,
    pub regular: bool,
    pub size: Option<u64>,
    pub permissions: Option<u32>,
}
#[derive(Debug)]
pub enum Request {
    Create {
        directory: PathBuf,
        name: PathBuf,
        folder: bool,
    },
    Navigate(PathBuf),
    Next,
    CreateLocalDirectory(PathBuf),
    PlanUpload {
        local: PathBuf,
        directory: PathBuf,
    },
    PlanDownload {
        remote: PathBuf,
        local: PathBuf,
    },
    Upload {
        local: PathBuf,
        directory: PathBuf,
    },
    Download {
        remote: PathBuf,
        local: PathBuf,
    },
}
#[derive(Debug, Clone)]
pub struct Planned {
    pub upload: bool,
    pub directory: bool,
    pub local: PathBuf,
    pub remote: PathBuf,
}
#[derive(Debug)]
pub enum Event {
    Planned(Vec<Planned>),
    Page {
        directory: PathBuf,
        entries: Vec<Entry>,
        more: bool,
    },
    Finished(Result<(), String>),
    Error(String),
}
#[derive(Debug, Default, Clone)]
pub struct Progress {
    pub bytes: u64,
    pub total: Option<u64>,
}

pub struct Client {
    pub requests: mpsc::Sender<Request>,
    pub events: mpsc::Receiver<Event>,
    pub progress: watch::Receiver<Progress>,
    pub cancel: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
}
pub struct WorkerChannels {
    pub requests: mpsc::Receiver<Request>,
    pub events: mpsc::Sender<Event>,
    pub progress: watch::Sender<Progress>,
    pub cancel: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
}
impl Client {
    pub fn channels() -> (Self, WorkerChannels) {
        let (requests, receiver) = mpsc::channel(4);
        let (sender, events) = mpsc::channel(4);
        let (progress_sender, progress) = watch::channel(Progress::default());
        let cancel = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let worker = WorkerChannels {
            requests: receiver,
            events: sender,
            progress: progress_sender,
            cancel: cancel.clone(),
            stop: stop.clone(),
        };
        (
            Self {
                requests,
                events,
                progress,
                cancel,
                stop,
            },
            worker,
        )
    }
    pub fn start(connection: Arc<SshConnection>) -> Self {
        let (client, channels) = Self::channels();
        crate::runtime::runtime().spawn(worker(
            connection,
            channels.requests,
            channels.events,
            channels.progress,
            channels.cancel,
            channels.stop,
        ));
        client
    }
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.stop.store(true, Ordering::Relaxed);
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn shutdown(channel: SftpChannel) {
    let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
}
async fn worker(
    connection: Arc<SshConnection>,
    mut requests: mpsc::Receiver<Request>,
    events: mpsc::Sender<Event>,
    progress: watch::Sender<Progress>,
    cancel: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
) {
    let mut channel: Option<SftpChannel> = None;
    let mut cursor: Option<Pin<Box<ReadDir>>> = None;
    let mut directory = PathBuf::from(".");
    loop {
        if stop.load(Ordering::Relaxed) || connection.closed.load(Ordering::Relaxed) {
            break;
        }
        let request = tokio::select! {
            r = requests.recv() => match r {Some(r) => r, None => break},
            _ = tokio::time::sleep(Duration::from_millis(250)) => continue,
        };
        if channel.is_none() {
            // Dropping the UI cancels authentication polling without waiting a minute.
            let connect = connection.sftp();
            tokio::pin!(connect);
            let result = loop {
                tokio::select! {
                    result = &mut connect => break Some(result),
                    _ = tokio::time::sleep(Duration::from_millis(250)) => if stop.load(Ordering::Relaxed) {break None;},
                }
            };
            match result {
                Some(Ok(c)) => channel = Some(c),
                Some(Err(e)) => {
                    if events.send(Event::Error(e)).await.is_err() {
                        break;
                    }
                    continue;
                }
                None => break,
            }
        }
        let sftp = &channel.as_ref().expect("SFTP connected").sftp;
        let result: Result<Event, String> = async {
            match request {
                Request::Navigate(path) => {
                    let mut fs = sftp.fs();
                    let path = tokio::time::timeout(Duration::from_secs(30), fs.canonicalize(path))
                        .await
                        .map_err(|_| "SFTP: tiempo de espera agotado")?
                        .map_err(|e| e.to_string())?;
                    let dir = tokio::time::timeout(Duration::from_secs(30), fs.open_dir(&path))
                        .await
                        .map_err(|_| "SFTP: tiempo de espera agotado")?
                        .map_err(|e| e.to_string())?;
                    cursor = Some(Box::pin(dir.read_dir()));
                    directory = path;
                    page(&mut cursor, &directory).await
                }
                Request::Create {
                    directory,
                    name,
                    folder,
                } => {
                    let path = join_remote(&directory, &name)?;
                    let operation = async {
                        if folder {
                            sftp.fs()
                                .create_dir(path)
                                .await
                                .map_err(|e| e.to_string())?;
                        } else {
                            sftp.options()
                                .write(true)
                                .create_new(true)
                                .open(path)
                                .await
                                .map_err(|e| e.to_string())?
                                .close()
                                .await
                                .map_err(|e| e.to_string())?;
                        }
                        Ok(())
                    };
                    Ok(Event::Finished(
                        tokio::time::timeout(Duration::from_secs(30), operation)
                            .await
                            .map_err(|_| "Creación remota: tiempo agotado".to_owned())
                            .and_then(|r| r),
                    ))
                }
                Request::CreateLocalDirectory(path) => Ok(Event::Finished(
                    tokio::fs::create_dir(&path).await.map_err(|e| {
                        format!(
                            "No se creó la carpeta local (no se fusionan destinos existentes): {e}"
                        )
                    }),
                )),
                Request::PlanUpload { local, directory } => Ok(Event::Planned(plan_upload(
                    local, directory, &cancel, &stop,
                )?)),
                Request::PlanDownload { remote, local } => Ok(Event::Planned(
                    plan_download(sftp, remote, local, &cancel, &stop).await?,
                )),
                Request::Next => page(&mut cursor, &directory).await,
                Request::Upload { local, directory } => Ok(Event::Finished(
                    upload(sftp, local, directory, &cancel, &stop, &progress).await,
                )),
                Request::Download { remote, local } => Ok(Event::Finished(
                    download(sftp, remote, local, &cancel, &stop, &progress).await,
                )),
            }
        }
        .await;
        let event = match result {
            Ok(event) => event,
            Err(e) => Event::Error(e),
        };
        if matches!(event, Event::Error(_)) {
            cursor = None;
            if let Some(old) = channel.take() {
                shutdown(old).await;
            }
        }
        if events.send(event).await.is_err() {
            break;
        }
    }
    drop(cursor);
    if let Some(channel) = channel {
        shutdown(channel).await;
    }
}

async fn page(
    cursor: &mut Option<Pin<Box<ReadDir>>>,
    directory: &std::path::Path,
) -> Result<Event, String> {
    let mut entries = Vec::with_capacity(PAGE_SIZE);
    let mut more = false;
    if let Some(cursor) = cursor {
        while entries.len() < PAGE_SIZE {
            let next = tokio::time::timeout(Duration::from_secs(30), cursor.try_next())
                .await
                .map_err(|_| "SFTP: tiempo de espera agotado")?
                .map_err(|e| e.to_string())?;
            let Some(entry) = next else {
                break;
            };
            if entry.filename() == std::path::Path::new(".")
                || entry.filename() == std::path::Path::new("..")
            {
                continue;
            }
            // Reject names returned by a server that could escape the displayed directory.
            join_remote(directory, entry.filename())?;
            entries.push(Entry {
                name: entry.filename().into(),
                directory: entry.file_type().is_some_and(|f| f.is_dir()),
                regular: entry.file_type().is_some_and(|f| f.is_file()),
                size: entry.metadata().len(),
                permissions: entry.metadata().permissions().map(|p| p.as_raw().bits()),
            });
        }
        more = entries.len() == PAGE_SIZE;
    }
    entries.sort_by(|a, b| b.directory.cmp(&a.directory).then(a.name.cmp(&b.name)));
    Ok(Event::Page {
        directory: directory.into(),
        entries,
        more,
    })
}
const TREE_LIMIT: usize = 512;
/// Plan first, copy only after the GUI has durably stored every operation.
pub fn plan_upload(
    local: PathBuf,
    directory: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
) -> Result<Vec<Planned>, String> {
    let remote = join_remote(
        &directory,
        std::path::Path::new(local.file_name().ok_or("Carpeta sin nombre")?),
    )?;
    let mut stack = vec![(local, remote, 0usize)];
    let mut plan = Vec::new();
    while let Some((local, remote, depth)) = stack.pop() {
        check_cancel(cancel, stop)?;
        if depth > 32 || plan.len() + stack.len() >= TREE_LIMIT {
            return Err("El árbol supera 512 elementos o 32 niveles".into());
        }
        let metadata = std::fs::symlink_metadata(&local).map_err(|e| e.to_string())?;
        if depth == 0 && !metadata.is_dir() {
            return Err("La carpeta local cambió de tipo".into());
        }
        if !metadata.is_dir() && !metadata.is_file() {
            return Err(
                "El árbol contiene enlaces o archivos especiales; no se ha transferido".into(),
            );
        }
        plan.push(Planned {
            upload: true,
            directory: metadata.is_dir(),
            local: local.clone(),
            remote: remote.clone(),
        });
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&local).map_err(|e| e.to_string())? {
                check_cancel(cancel, stop)?;
                let entry = entry.map_err(|e| e.to_string())?;
                if plan.len() + stack.len() >= TREE_LIMIT {
                    return Err("El árbol supera 512 elementos".into());
                }
                stack.push((
                    entry.path(),
                    join_remote(&remote, std::path::Path::new(&entry.file_name()))?,
                    depth + 1,
                ));
            }
        }
    }
    Ok(plan)
}
async fn plan_download(
    sftp: &Sftp,
    remote: PathBuf,
    local: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
) -> Result<Vec<Planned>, String> {
    let mut stack = vec![(remote, local, 0usize)];
    let mut plan = Vec::new();
    while let Some((remote, local, depth)) = stack.pop() {
        check_cancel(cancel, stop)?;
        if depth > 32 || plan.len() + stack.len() >= TREE_LIMIT {
            return Err("El árbol supera 512 elementos o 32 niveles".into());
        }
        let metadata = remote_operation(sftp.fs().symlink_metadata(&remote), cancel, stop).await?;
        let kind = metadata
            .file_type()
            .ok_or("El servidor no informa del tipo del archivo")?;
        if depth == 0 && !kind.is_dir() {
            return Err("La carpeta remota cambió de tipo".into());
        }
        if !kind.is_dir() && !kind.is_file() {
            return Err(
                "El árbol contiene enlaces o archivos especiales; no se ha transferido".into(),
            );
        }
        plan.push(Planned {
            upload: false,
            directory: kind.is_dir(),
            local: local.clone(),
            remote: remote.clone(),
        });
        if kind.is_dir() {
            let mut entries = Box::pin(
                remote_operation(sftp.fs().open_dir(&remote), cancel, stop)
                    .await?
                    .read_dir(),
            );
            loop {
                check_cancel(cancel, stop)?;
                let entry = remote_operation(entries.try_next(), cancel, stop).await?;
                let Some(entry) = entry else {
                    break;
                };
                let name = entry.filename();
                if name == std::path::Path::new(".") || name == std::path::Path::new("..") {
                    continue;
                }
                let child = join_remote(&remote, name)?;
                if plan.len() + stack.len() >= TREE_LIMIT {
                    return Err("El árbol supera 512 elementos".into());
                }
                stack.push((child, local.join(name), depth + 1));
            }
        }
    }
    Ok(plan)
}

async fn remote_operation<T, E: std::fmt::Display>(
    future: impl std::future::Future<Output = Result<T, E>>,
    cancel: &AtomicBool,
    stop: &AtomicBool,
) -> Result<T, String> {
    check_cancel(cancel, stop)?;
    let cancelled = async {
        while !cancel.load(Ordering::Relaxed) && !stop.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    };
    tokio::select! {
        result=tokio::time::timeout(Duration::from_secs(30),future)=>result.map_err(|_|"SFTP: operación sin respuesta".to_string())?.map_err(|e|e.to_string()),
        _=cancelled=>Err("Transferencia cancelada; revisa el destino si se perdió la confirmación.".into()),
    }
}
fn check_cancel(cancel: &AtomicBool, stop: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::Relaxed) || stop.load(Ordering::Relaxed) {
        Err("Transferencia cancelada; puede quedar un archivo parcial en el destino.".into())
    } else {
        Ok(())
    }
}
pub async fn upload(
    sftp: &Sftp,
    local: PathBuf,
    directory: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
    progress: &watch::Sender<Progress>,
) -> Result<(), String> {
    if !sftp.support_hardlink() {
        return upload_file(sftp, local, directory, cancel, stop, progress).await;
    }
    check_cancel(cancel, stop)?;
    let name = local.file_name().ok_or("Archivo sin nombre")?.to_owned();
    let temporary = directory.join(format!(".nexuterm-{}.part", uuid::Uuid::new_v4()));
    let mut private = openssh_sftp_client::metadata::Permissions::new();
    private
        .set_read_by_owner(true)
        .set_write_by_owner(true)
        .set_execute_by_owner(true);
    remote_operation(
        sftp.fs()
            .dir_builder()
            .permissions(private)
            .create(&temporary),
        cancel,
        stop,
    )
    .await?;
    let result = async {
        upload_file(sftp, local, temporary.clone(), cancel, stop, progress).await?;
        check_cancel(cancel, stop)?;
        remote_operation(
            sftp.fs()
                .hard_link(temporary.join(&name), directory.join(&name)),
            cancel,
            stop,
        )
        .await
        .map_err(|e| {
            format!("No se publicó o confirmó el archivo (no se sobrescriben destinos): {e}")
        })
    }
    .await;
    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        sftp.fs().remove_file(temporary.join(&name)),
    )
    .await;
    let cleanup = tokio::time::timeout(Duration::from_secs(5), sftp.fs().remove_dir(&temporary))
        .await
        .map_err(|_| format!("No se confirmó la limpieza de {}", temporary.display()))
        .and_then(|r| {
            r.map_err(|e| {
                format!(
                    "No se retiró el temporal privado {}: {e}",
                    temporary.display()
                )
            })
        });
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(format!("{error}. {cleanup}")),
        (result, cleanup) => result.and(cleanup),
    }
}
pub async fn download(
    sftp: &Sftp,
    remote: PathBuf,
    local: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
    progress: &watch::Sender<Progress>,
) -> Result<(), String> {
    check_cancel(cancel, stop)?;
    let parent = local.parent().ok_or("Destino local inválido")?;
    let temporary = tempfile::Builder::new()
        .prefix(".nexuterm-")
        .tempdir_in(parent)
        .map_err(|e| e.to_string())?;
    let partial = temporary.path().join("content");
    download_file(sftp, remote, partial.clone(), cancel, stop, progress).await?;
    check_cancel(cancel, stop)?;
    tokio::fs::hard_link(&partial, &local).await.map_err(|e| {
        format!("No se publicó el archivo local (no se sobrescriben destinos): {e}")
    })?;
    Ok(())
}

async fn upload_file(
    sftp: &Sftp,
    local: PathBuf,
    directory: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
    progress: &watch::Sender<Progress>,
) -> Result<(), String> {
    check_cancel(cancel, stop)?;
    let metadata = tokio::fs::symlink_metadata(&local)
        .await
        .map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("Selecciona un archivo regular; no se siguen enlaces simbólicos.".into());
    }
    let remote = join_remote(
        &directory,
        std::path::Path::new(local.file_name().ok_or("Archivo sin nombre")?),
    )?;
    let mut input = tokio::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&local)
        .await
        .map_err(|e| e.to_string())?;
    use std::os::unix::fs::MetadataExt;
    let opened = input.metadata().await.map_err(|e| e.to_string())?;
    if opened.ino() != metadata.ino() || opened.dev() != metadata.dev() || !opened.is_file() {
        return Err("El archivo local cambió al abrirlo".into());
    }
    let mut output = remote_operation(
        sftp.options().write(true).create_new(true).open(remote),
        cancel,
        stop,
    )
    .await
    .map_err(|e| {
        format!("No se pudo crear el destino (no se sobrescriben archivos existentes): {e}")
    })?;
    let mut buffer = vec![0; BLOCK];
    let mut bytes = 0;
    progress.send_replace(Progress {
        bytes,
        total: Some(metadata.len()),
    });
    let result = async {
        loop {
            check_cancel(cancel, stop)?;
            let n = input.read(&mut buffer).await.map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            remote_operation(output.write_all(&buffer[..n]), cancel, stop).await?;
            bytes += n as u64;
            progress.send_replace(Progress {
                bytes,
                total: Some(metadata.len()),
            });
        }
        let current = input.metadata().await.map_err(|e| e.to_string())?;
        if bytes != metadata.len()
            || current.len() != metadata.len()
            || current.modified().ok() != metadata.modified().ok()
        {
            return Err("El archivo local cambió durante la subida".into());
        }
        if sftp.support_fsync() {
            remote_operation(output.sync_all(), cancel, stop).await?;
        }
        Ok(())
    }
    .await;
    let close = tokio::time::timeout(Duration::from_secs(30), output.close())
        .await
        .map_err(|_| "SFTP: cierre sin respuesta")?
        .map_err(|e| e.to_string());
    result.and(close)
}
async fn download_file(
    sftp: &Sftp,
    remote: PathBuf,
    local: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
    progress: &watch::Sender<Progress>,
) -> Result<(), String> {
    check_cancel(cancel, stop)?;
    let info = remote_operation(sftp.fs().symlink_metadata(&remote), cancel, stop).await?;
    if !info.file_type().is_some_and(|f| f.is_file()) {
        return Err("No se siguen enlaces ni se descargan archivos especiales".into());
    }
    let mut input = remote_operation(sftp.open(remote), cancel, stop).await?;
    let metadata = remote_operation(input.metadata(), cancel, stop).await?;
    if !metadata.file_type().is_some_and(|f| f.is_file()) {
        return Err("Solo se descargan archivos regulares.".into());
    }
    let mut output = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(local)
        .await
        .map_err(|e| {
            format!("No se pudo crear el destino (no se sobrescriben archivos existentes): {e}")
        })?;
    let mut bytes = 0;
    progress.send_replace(Progress {
        bytes,
        total: metadata.len(),
    });
    let result = async {
        loop {
            check_cancel(cancel, stop)?;
            // Advance by bytes actually returned, including short SFTP responses.
            remote_operation(input.seek(std::io::SeekFrom::Start(bytes)), cancel, stop).await?;
            let buffer = remote_operation(
                input.read(BLOCK as u32, BytesMut::with_capacity(BLOCK)),
                cancel,
                stop,
            )
            .await?;
            let Some(buffer) = buffer else {
                break;
            };
            if buffer.is_empty() {
                break;
            }
            output.write_all(&buffer).await.map_err(|e| e.to_string())?;
            bytes += buffer.len() as u64;
            progress.send_replace(Progress {
                bytes,
                total: metadata.len(),
            });
        }
        if metadata.len().is_some_and(|expected| expected != bytes) {
            return Err("El tamaño remoto cambió durante la descarga".into());
        }
        output.sync_all().await.map_err(|e| e.to_string())
    }
    .await;
    let close = tokio::time::timeout(Duration::from_secs(30), input.close())
        .await
        .map_err(|_| "SFTP: cierre sin respuesta")?
        .map_err(|e| e.to_string());
    result.and(close)
}

/// Unix access bits, including setuid, setgid and sticky bits; absent metadata stays unknown.
pub fn permission_text(mode: Option<u32>) -> String {
    let Some(mode) = mode else {
        return "—".into();
    };
    let mut text = String::with_capacity(9);
    for (read, write, execute, special, lower, upper) in [
        (0o400, 0o200, 0o100, 0o4000, 's', 'S'),
        (0o040, 0o020, 0o010, 0o2000, 's', 'S'),
        (0o004, 0o002, 0o001, 0o1000, 't', 'T'),
    ] {
        text.push(if mode & read != 0 { 'r' } else { '-' });
        text.push(if mode & write != 0 { 'w' } else { '-' });
        text.push(if mode & special != 0 {
            if mode & execute != 0 { lower } else { upper }
        } else if mode & execute != 0 {
            'x'
        } else {
            '-'
        });
    }
    text
}
pub fn creation_name(value: &str) -> Result<PathBuf, String> {
    if value.trim().is_empty() || value.contains('/') || value.chars().any(char::is_control) {
        return Err("Indica un nombre sin caracteres de control".into());
    }
    let path = PathBuf::from(value);
    join_remote(std::path::Path::new("/"), &path)?;
    Ok(path)
}
#[cfg(test)]
mod browser_tests {
    use super::*;
    #[test]
    fn trees_are_bounded_planned_before_writing_and_reject_links() {
        let root = tempfile::tempdir().unwrap();
        let tree = root.path().join("tree á");
        std::fs::create_dir_all(tree.join("nested/empty")).unwrap();
        std::fs::write(tree.join("nested/file.txt"), b"fixture").unwrap();
        let cancel = AtomicBool::new(false);
        let stop = AtomicBool::new(false);
        let plan = plan_upload(tree.clone(), "/remote".into(), &cancel, &stop).unwrap();
        assert!(plan[0].directory);
        assert_eq!(plan.len(), 4);
        assert!(plan.iter().any(|p| p.remote
            == std::path::Path::new("/remote/tree á/nested/empty")
            && p.directory));
        std::os::unix::fs::symlink("nested/file.txt", tree.join("link")).unwrap();
        assert!(plan_upload(tree.clone(), "/remote".into(), &cancel, &stop).is_err());
        std::fs::remove_file(tree.join("link")).unwrap();
        cancel.store(true, Ordering::Relaxed);
        assert!(plan_upload(tree.clone(), "/remote".into(), &cancel, &stop).is_err());
        cancel.store(false, Ordering::Relaxed);
        for i in 0..513 {
            std::fs::write(tree.join(format!("file-{i}")), b"").unwrap();
        }
        assert!(plan_upload(tree, "/remote".into(), &cancel, &stop).is_err());
    }
    #[test]
    fn unix_permissions_and_creation_names() {
        assert_eq!(permission_text(Some(0o100644)), "rw-r--r--");
        assert_eq!(permission_text(Some(0o040750)), "rwxr-x---");
        assert_eq!(permission_text(Some(0o7754)), "rwsr-sr-T");
        assert_eq!(permission_text(None), "—");
        for name in ["", "  ", ".", "..", "../outside", "/absolute", "a/b", "a\n"] {
            assert!(creation_name(name).is_err(), "{name:?}");
        }
        assert_eq!(
            creation_name("archivo á ' nuevo.py").unwrap(),
            PathBuf::from("archivo á ' nuevo.py")
        );
    }
}
