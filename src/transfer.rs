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
}
#[derive(Debug)]
pub enum Request {
    Navigate(PathBuf),
    Next,
    Upload { local: PathBuf, directory: PathBuf },
    Download { remote: PathBuf, local: PathBuf },
}
#[derive(Debug)]
pub enum Event {
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
impl Client {
    pub fn start(connection: Arc<SshConnection>) -> Self {
        let (requests, receiver) = mpsc::channel(4);
        let (sender, events) = mpsc::channel(4);
        let (progress_sender, progress) = watch::channel(Progress::default());
        let cancel = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        crate::runtime::runtime().spawn(worker(
            connection,
            receiver,
            sender,
            progress_sender,
            cancel.clone(),
            stop.clone(),
        ));
        Self {
            requests,
            events,
            progress,
            cancel,
            stop,
        }
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
    let mut input = tokio::fs::File::open(local)
        .await
        .map_err(|e| e.to_string())?;
    let mut output = sftp
        .options()
        .write(true)
        .create_new(true)
        .open(remote)
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
            tokio::time::timeout(Duration::from_secs(30), output.write_all(&buffer[..n]))
                .await
                .map_err(|_| "SFTP: escritura sin respuesta")?
                .map_err(|e| e.to_string())?;
            bytes += n as u64;
            progress.send_replace(Progress {
                bytes,
                total: Some(metadata.len()),
            });
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
pub async fn download(
    sftp: &Sftp,
    remote: PathBuf,
    local: PathBuf,
    cancel: &AtomicBool,
    stop: &AtomicBool,
    progress: &watch::Sender<Progress>,
) -> Result<(), String> {
    check_cancel(cancel, stop)?;
    let mut input = sftp.open(remote).await.map_err(|e| e.to_string())?;
    let metadata = input.metadata().await.map_err(|e| e.to_string())?;
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
            input
                .seek(std::io::SeekFrom::Start(bytes))
                .await
                .map_err(|e| e.to_string())?;
            let buffer = tokio::time::timeout(
                Duration::from_secs(30),
                input.read(BLOCK as u32, BytesMut::with_capacity(BLOCK)),
            )
            .await
            .map_err(|_| "SFTP: lectura sin respuesta")?
            .map_err(|e| e.to_string())?;
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
        output.flush().await.map_err(|e| e.to_string())
    }
    .await;
    let close = tokio::time::timeout(Duration::from_secs(30), input.close())
        .await
        .map_err(|_| "SFTP: cierre sin respuesta")?
        .map_err(|e| e.to_string());
    result.and(close)
}
