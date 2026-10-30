//! Bounded append-only local transcripts. No credentials, approvals or executable jobs.
use crate::workspaces::ChatMessage;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, oneshot};
pub const LIMIT: u64 = 1024 * 1024 * 1024;
const QUEUE_BYTES: usize = 4 * 1024 * 1024;
const BOOTSTRAP_BYTES: usize = 16 * 1024 * 1024;
const RECORD_LIMIT: u64 = 2 * 1024 * 1024;
pub const PAGE_SIZE: usize = 25;
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "event", deny_unknown_fields)]
pub enum Record {
    Header {
        version: u32,
        title: String,
    },
    Message {
        id: String,
        title: String,
        text: String,
        tool: bool,
        response: bool,
    },
    Append {
        id: String,
        text: String,
    },
    Replace {
        id: String,
        text: String,
    },
}
impl Record {
    fn weight(&self) -> usize {
        128 + match self {
            Self::Header { title, .. } => title.len(),
            Self::Message {
                id, title, text, ..
            } => id.len() + title.len() + text.len(),
            Self::Append { id, text } | Self::Replace { id, text } => id.len() + text.len(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        match self {
            Self::Header { version, title } => {
                if *version != 1 || title.len() > 4096 {
                    return Err("Cabecera de historial desconocida".into());
                }
            }
            Self::Message { id, title, .. } => {
                uuid::Uuid::parse_str(id).map_err(|_| "Mensaje inválido")?;
                if title.len() > 4096 {
                    return Err("Título demasiado grande".into());
                }
            }
            Self::Append { id, .. } | Self::Replace { id, .. } => {
                uuid::Uuid::parse_str(id).map_err(|_| "Mensaje inválido")?;
            }
        }
        Ok(())
    }
}
pub fn directory() -> Result<PathBuf, String> {
    Ok(crate::workspaces::Store::path()?
        .parent()
        .and_then(Path::parent)
        .ok_or("Ruta de datos inválida")?
        .join("chat-history"))
}
fn path(dir: &Path, id: &str) -> Result<PathBuf, String> {
    uuid::Uuid::parse_str(id).map_err(|_| "Identificador de historial inválido")?;
    Ok(dir.join(format!("{id}.jsonl")))
}
struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
fn lock(dir: &Path) -> Result<Lock, String> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .map_err(|e| e.to_string())?;
    if !fs::symlink_metadata(dir)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_dir()
    {
        return Err("El historial no es un directorio normal".into());
    }
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(dir.join(".lock"))
        .map_err(|e| e.to_string())?;
    if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(Lock(f))
}
fn append(dir: &Path, id: &str, records: &[Record], only_if_new: bool) -> Result<(), String> {
    let file_path = path(dir, id)?;
    let _lock = lock(dir)?;
    if only_if_new && file_path.exists() {
        return Ok(());
    }
    if dir.join(format!(".{id}.deleted")).exists() {
        return Err("Este historial se eliminó; la captura permanece desactivada".into());
    }
    let mut data = Vec::new();
    for record in records {
        record.validate()?;
        let bytes = serde_json::to_vec(record).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > RECORD_LIMIT {
            return Err("Entrada de historial demasiado grande; captura detenida".into());
        }
        data.extend(bytes);
        data.push(b'\n');
    }
    capacity(dir, &file_path, data.len() as u64, false)?;
    if let Ok(metadata) = fs::symlink_metadata(&file_path) {
        if !metadata.file_type().is_file() {
            return Err("Historial no regular; no se ha modificado".into());
        }
        if metadata.len() > 0 {
            if !matches!(
                record(&mut BufReader::new(
                    File::open(&file_path).map_err(|e| e.to_string())?
                ))?,
                Some(Record::Header { version: 1, .. })
            ) {
                return Err("Formato de historial desconocido; no se ha modificado".into());
            }
            let mut file = File::open(&file_path).map_err(|e| e.to_string())?;
            use std::io::{Seek, SeekFrom};
            file.seek(SeekFrom::End(-1)).map_err(|e| e.to_string())?;
            let mut tail = [0];
            file.read_exact(&mut tail).map_err(|e| e.to_string())?;
            if tail[0] != b'\n' {
                return Err("Historial incompleto; se conserva sin añadir nuevas entradas".into());
            }
        }
    }
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(file_path)
        .map_err(|e| e.to_string())?;
    file.write_all(&data)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
pub fn remove(dir: &Path, id: &str) -> Result<(), String> {
    let target = path(dir, id)?;
    let _lock = lock(dir)?;
    let tombstone = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(dir.join(format!(".{id}.deleted")))
        .map_err(|e| e.to_string())?;
    tombstone.sync_all().map_err(|e| e.to_string())?;
    fs::remove_file(target)
        .or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .map_err(|e| e.to_string())?;
    let state = dir.join(format!("{id}.json"));
    match fs::remove_file(state) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.to_string()),
    }
    File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
fn record(reader: &mut BufReader<File>) -> Result<Option<Record>, String> {
    let mut bytes = Vec::new();
    let n = reader
        .take(RECORD_LIMIT + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(None);
    }
    if n as u64 > RECORD_LIMIT || bytes.last() != Some(&b'\n') {
        return Err("Entrada incompleta o demasiado grande; el archivo se conserva".into());
    }
    let value: Record = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    value.validate()?;
    Ok(Some(value))
}
fn open(dir: &Path, id: &str) -> Result<File, String> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path(dir, id)?)
        .map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Historial no regular".into());
    }
    Ok(file)
}
pub struct Entry {
    pub id: String,
    pub title: String,
    pub saved_at: u64,
}
pub fn list(dir: &Path) -> Result<Vec<Entry>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };
    let mut out = Vec::new();
    for e in entries {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.extension().is_none_or(|v| v != "jsonl") {
            continue;
        }
        if out.len() >= 1000 {
            return Err("Demasiados historiales".into());
        }
        let id = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_owned();
        let title = open(dir, &id)
            .and_then(|f| record(&mut BufReader::new(f)))
            .ok()
            .flatten()
            .and_then(|r| {
                if let Record::Header { title, .. } = r {
                    Some(title)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| format!("Historial {} (revisar)", &id));
        let saved_at = fs::metadata(&p)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        out.push(Entry {
            id,
            title,
            saved_at,
        });
    }
    out.sort_by(|a, b| b.saved_at.cmp(&a.saved_at).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}
// Recent conversations share the transcript quota and deletion tombstones.
const STATE_LIMIT: u64 = 16 * 1024 * 1024;
fn capacity(dir: &Path, target: &Path, bytes: u64, replace: bool) -> Result<(), String> {
    let mut total = 0u64;
    let mut ids = std::collections::HashSet::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        if !p.extension().is_some_and(|e| e == "jsonl" || e == "json") {
            continue;
        }
        if let Some(id) = p.file_stem().and_then(|s| s.to_str()) {
            ids.insert(id.to_owned());
        }
        if !replace || p != target {
            total = total.saturating_add(entry.metadata().map_err(|e| e.to_string())?.len());
        }
    }
    let id = target.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if total.saturating_add(bytes) > LIMIT || (ids.len() >= 1000 && !ids.contains(id)) {
        return Err("Límite local de historial alcanzado (1 GiB/1000 conversaciones); libera espacio. No se borran datos automáticamente".into());
    }
    Ok(())
}
#[derive(Debug)]
pub struct Recent {
    pub id: String,
    pub title: String,
    pub thread: Option<String>,
    pub saved_at: std::time::SystemTime,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecentState {
    version: u32,
    captured_at: u128,
    chat: crate::workspaces::Chat,
}
/// Capture order before dispatching disk work, so a late worker cannot overwrite newer state.
pub struct RecentSave(RecentState);
impl RecentSave {
    pub fn new(chat: crate::workspaces::Chat) -> Self {
        Self(RecentState {
            version: 1,
            captured_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            chat,
        })
    }
    pub fn save(self, dir: &Path) -> Result<(), String> {
        let chat = &self.0.chat;
        if chat.agent_id.is_none() || (chat.messages.is_empty() && chat.draft.is_empty()) {
            return Ok(());
        }
        crate::workspaces::Chats {
            tabs: vec![chat.clone()],
            ..Default::default()
        }
        .validate()?;
        let target = dir.join(format!("{}.json", chat.id));
        let data = serde_json::to_vec(&self.0).map_err(|e| e.to_string())?;
        if data.len() as u64 > STATE_LIMIT {
            return Err("Conversación demasiado grande para recientes (16 MiB)".into());
        }
        let _lock = lock(dir)?;
        if dir.join(format!(".{}.deleted", chat.id)).exists() {
            return Ok(());
        }
        if let Ok(old) = read_recent(dir, &chat.id)
            && old.captured_at > self.0.captured_at
        {
            return Ok(());
        }
        capacity(dir, &target, data.len() as u64, true)?;
        let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
        file.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        file.write_all(&data)
            .and_then(|_| file.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        file.persist(target).map_err(|e| e.to_string())?;
        File::open(dir)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
}
fn read_recent(dir: &Path, id: &str) -> Result<RecentState, String> {
    path(dir, id)?;
    if dir.join(format!(".{id}.deleted")).exists() {
        return Err("Conversación eliminada".into());
    }
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(dir.join(format!("{id}.json")))
        .map_err(|e| e.to_string())?;
    let meta = f.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > STATE_LIMIT {
        return Err("Copia de conversación inválida".into());
    }
    let state: RecentState =
        serde_json::from_reader(f.take(STATE_LIMIT + 1)).map_err(|e| e.to_string())?;
    if state.version != 1 {
        return Err("Versión de recientes desconocida".into());
    }
    let chat = &state.chat;
    if chat.id != id {
        return Err("La copia no corresponde al chat".into());
    }
    crate::workspaces::Chats {
        tabs: vec![chat.clone()],
        ..Default::default()
    }
    .validate()?;
    Ok(state)
}
pub fn load_recent(dir: &Path, id: &str) -> Result<crate::workspaces::Chat, String> {
    Ok(read_recent(dir, id)?.chat)
}
pub fn recent(dir: &Path, agent: &str) -> Result<Vec<Recent>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };
    let mut out = Vec::new();
    let mut count = 0;
    for e in entries {
        let e = e.map_err(|e| e.to_string())?;
        let p = e.path();
        if !p.extension().is_some_and(|e| e == "json") {
            continue;
        }
        count += 1;
        if count > 1000 {
            return Err("Demasiadas conversaciones recientes".into());
        }
        let Some(id) = p.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        // A damaged entry is preserved and does not prevent opening other chats.
        if let Ok(chat) = load_recent(dir, id)
            && chat.agent_id.as_deref() == Some(agent)
        {
            out.push(Recent {
                id: chat.id,
                title: chat.title,
                thread: chat.codex_thread,
                saved_at: e
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH),
            });
        }
    }
    out.sort_by(|a, b| b.saved_at.cmp(&a.saved_at).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

pub struct Page {
    pub messages: Vec<ChatMessage>,
    pub total: usize,
}
pub fn page(dir: &Path, id: &str, offset: usize) -> Result<Page, String> {
    let file = open(dir, id)?;
    let mut reader = BufReader::new(file);
    let mut messages = Vec::new();
    let mut ids = std::collections::HashMap::new();
    let mut total = 0;
    fn bounded(text: &str) -> String {
        text.chars().take(65536).collect()
    }
    while let Some(r) = record(&mut reader)? {
        match r {
            Record::Header { .. } => {}
            Record::Message {
                id,
                title,
                text,
                tool,
                response,
            } => {
                if total >= offset && messages.len() < PAGE_SIZE {
                    ids.insert(id, messages.len());
                    messages.push(ChatMessage {
                        title,
                        text: bounded(&text),
                        tool,
                        response,
                    });
                }
                total += 1;
            }
            Record::Append { id, text } => {
                if let Some(i) = ids.get(&id) {
                    let m: &mut ChatMessage = &mut messages[*i];
                    let left = 65536usize.saturating_sub(m.text.chars().count());
                    m.text.extend(text.chars().take(left));
                }
            }
            Record::Replace { id, text } => {
                if let Some(i) = ids.get(&id) {
                    messages[*i].text = bounded(&text);
                }
            }
        }
    }
    Ok(Page { messages, total })
}
pub fn export(dir: &Path, id: &str, destination: &Path) -> Result<(), String> {
    let _lock = lock(dir)?;
    let mut input = open(dir, id)?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(destination)
        .map_err(|e| e.to_string())?;
    let length = input.metadata().map_err(|e| e.to_string())?.len();
    std::io::copy(
        &mut std::io::Read::by_ref(&mut input).take(length),
        &mut output,
    )
    .and_then(|_| output.sync_all())
    .map_err(|e| e.to_string())
}
enum Work {
    Records(Record, usize),
    Bootstrap(Vec<Record>, usize),
    Flush(oneshot::Sender<Result<(), String>>),
}
pub struct Recorder {
    pub id: String,
    tx: mpsc::Sender<Work>,
    error: Arc<Mutex<Option<String>>>,
    enabled: std::cell::Cell<bool>,
    pending_bytes: Arc<std::sync::atomic::AtomicUsize>,
    bootstrapping: Arc<std::sync::atomic::AtomicBool>,
}
impl Recorder {
    pub fn new(id: String) -> Self {
        Self::at(id, directory())
    }
    fn at(id: String, root: Result<PathBuf, String>) -> Self {
        let (tx, mut rx) = mpsc::channel::<Work>(64);
        let error = Arc::new(Mutex::new(None));
        let errors = error.clone();
        let identifier = id.clone();
        let pending_bytes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let bytes = pending_bytes.clone();
        let bootstrapping = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let bootstrap_active = bootstrapping.clone();
        crate::runtime::runtime().spawn(async move {
            let mut pending = None;
            let mut failed = false;
            while let Some(work) = match pending.take() {
                Some(w) => Some(w),
                None => rx.recv().await,
            } {
                match work {
                    Work::Records(r, size) => {
                        let mut total = size;
                        let mut records = vec![r];
                        while records.len() < 8 {
                            match rx.try_recv() {
                                Ok(Work::Records(r, size)) => {
                                    total += size;
                                    records.push(r);
                                }
                                Ok(w) => {
                                    pending = Some(w);
                                    break;
                                }
                                Err(_) => break,
                            }
                        }
                        if !failed {
                            let id = identifier.clone();
                            let root = root.clone();
                            let result = crate::runtime::runtime()
                                .spawn_blocking(move || append(&root?, &id, &records, false))
                                .await
                                .map_err(|e| e.to_string())
                                .and_then(|r| r);
                            if let Err(e) = result {
                                failed = true;
                                *errors.lock().unwrap() = Some(e);
                            }
                        }
                        bytes.fetch_sub(total, std::sync::atomic::Ordering::Relaxed);
                    }
                    Work::Bootstrap(records, size) => {
                        if !failed {
                            let id = identifier.clone();
                            let root = root.clone();
                            let result = crate::runtime::runtime()
                                .spawn_blocking(move || append(&root?, &id, &records, true))
                                .await
                                .map_err(|e| e.to_string())
                                .and_then(|r| r);
                            if let Err(e) = result {
                                failed = true;
                                *errors.lock().unwrap() = Some(e);
                            }
                        }
                        bytes.fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
                        bootstrap_active.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                    Work::Flush(reply) => {
                        let result = errors.lock().unwrap().clone().map_or(Ok(()), Err);
                        let _ = reply.send(result);
                    }
                }
            }
        });
        Self {
            id,
            tx,
            error,
            enabled: std::cell::Cell::new(true),
            pending_bytes,
            bootstrapping,
        }
    }
    pub fn bootstrap(&self, title: String, messages: &[ChatMessage]) {
        if !self.enabled.get() {
            return;
        }
        let mut records = vec![Record::Header { version: 1, title }];
        records.extend(messages.iter().map(|m| Record::Message {
            id: uuid::Uuid::new_v4().to_string(),
            title: m.title.clone(),
            text: m.text.clone(),
            tool: m.tool,
            response: m.response,
        }));
        let size = records.iter().map(Record::weight).sum();
        if size > BOOTSTRAP_BYTES {
            *self.error.lock().unwrap() = Some(
                "La importación de historial supera 16 MiB; la copia de recuperación se conserva"
                    .into(),
            );
            return;
        }
        self.bootstrapping
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.pending_bytes
            .fetch_add(size, std::sync::atomic::Ordering::Relaxed);
        if self.tx.try_send(Work::Bootstrap(records, size)).is_err() {
            self.pending_bytes
                .fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
            self.bootstrapping
                .store(false, std::sync::atomic::Ordering::Relaxed);
            *self.error.lock().unwrap() = Some("No se pudo iniciar el historial local".into());
        }
    }
    pub fn ready(&self) -> bool {
        !self.enabled.get()
            || self.error().is_some()
            || (!self
                .bootstrapping
                .load(std::sync::atomic::Ordering::Relaxed)
                && self
                    .pending_bytes
                    .load(std::sync::atomic::Ordering::Relaxed)
                    < QUEUE_BYTES / 2
                && self.tx.capacity() >= 4)
    }
    pub fn enabled(&self) -> bool {
        self.enabled.get() && self.error().is_none()
    }
    pub fn set_enabled(&self, value: bool) {
        let previous = self.enabled.replace(value);
        if !previous && value && self.error().is_none() {
            self.record(Record::Header {
                version: 1,
                title: "Captura reactivada; el intervalo excluido no se ha registrado".into(),
            });
        }
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }
    pub fn record(&self, record: Record) {
        if !self.enabled.get() || self.error().is_some() {
            return;
        }
        let size = record.weight();
        let limit = if self
            .bootstrapping
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            BOOTSTRAP_BYTES + 65536
        } else {
            QUEUE_BYTES
        };
        let previous = self
            .pending_bytes
            .fetch_add(size, std::sync::atomic::Ordering::Relaxed);
        if previous.saturating_add(size) > limit {
            self.pending_bytes
                .fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
            *self.error.lock().unwrap() =
                Some("Presupuesto de memoria del historial alcanzado; captura detenida".into());
            return;
        }
        if self.tx.try_send(Work::Records(record, size)).is_err() {
            self.pending_bytes
                .fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
            *self.error.lock().unwrap() = Some(
                "La cola de historial se llenó; captura detenida para evitar pérdidas silenciosas"
                    .into(),
            );
        }
    }
    pub async fn flush(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Work::Flush(tx))
            .await
            .map_err(|_| "Captura de historial cerrada")?;
        rx.await.map_err(|_| "Captura de historial cerrada")?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_history_survives_beyond_visible_cards_and_exports_without_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let mut records = vec![Record::Header {
            version: 1,
            title: "Fixture".into(),
        }];
        let first = uuid::Uuid::new_v4().to_string();
        records.push(Record::Message {
            id: first.clone(),
            title: "Agente".into(),
            text: "á".repeat(70000),
            tool: false,
            response: true,
        });
        records.push(Record::Append {
            id: first,
            text: "final conservado".into(),
        });
        for _ in 0..150 {
            records.push(Record::Message {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Tú".into(),
                text: "Mensaje".into(),
                tool: false,
                response: false,
            });
        }
        append(temp.path(), &id, &records, false).unwrap();
        let first = page(temp.path(), &id, 0).unwrap();
        assert_eq!(first.total, 151);
        assert_eq!(first.messages.len(), PAGE_SIZE);
        assert_eq!(first.messages[0].text.chars().count(), 65536);
        assert_eq!(page(temp.path(), &id, 150).unwrap().messages.len(), 1);
        let target = temp.path().join("export.jsonl");
        export(temp.path(), &id, &target).unwrap();
        assert!(
            String::from_utf8(fs::read(&target).unwrap())
                .unwrap()
                .contains("final conservado")
        );
        assert!(export(temp.path(), &id, &target).is_err());
        assert_eq!(
            fs::metadata(path(temp.path(), &id).unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        remove(temp.path(), &id).unwrap();
        assert!(append(temp.path(), &id, &records, false).is_err());
        assert!(!path(temp.path(), &id).unwrap().exists());
    }
    #[test]
    fn quota_and_torn_records_preserve_existing_data() {
        let temp = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let r = vec![Record::Header {
            version: 1,
            title: "Fixture".into(),
        }];
        append(temp.path(), &id, &r, false).unwrap();
        let file = path(temp.path(), &id).unwrap();
        fs::write(&file, b"torn").unwrap();
        assert!(append(temp.path(), &id, &r, false).is_err());
        assert_eq!(fs::read(&file).unwrap(), b"torn");
        fs::write(&file, b"\n").unwrap();
        File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_len(LIMIT)
            .unwrap();
        assert!(append(temp.path(), &uuid::Uuid::new_v4().to_string(), &r, false).is_err());
        assert_eq!(fs::metadata(file).unwrap().len(), LIMIT);
    }
    #[test]
    fn recorder_flush_exclusion_and_deleted_logs_are_durable() {
        let temp = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let recorder = Recorder::at(id.clone(), Ok(temp.path().to_owned()));
        recorder.record(Record::Header {
            version: 1,
            title: "Fixture".into(),
        });
        recorder.record(Record::Message {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Tú".into(),
            text: "Conservar".into(),
            tool: false,
            response: false,
        });
        recorder.set_enabled(false);
        recorder.record(Record::Message {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Tú".into(),
            text: "No conservar".into(),
            tool: false,
            response: false,
        });
        crate::runtime::runtime()
            .block_on(recorder.flush())
            .unwrap();
        assert_eq!(page(temp.path(), &id, 0).unwrap().total, 1);
        remove(temp.path(), &id).unwrap();
        recorder.set_enabled(true);
        recorder.record(Record::Header {
            version: 1,
            title: "No recrear".into(),
        });
        assert!(
            crate::runtime::runtime()
                .block_on(recorder.flush())
                .is_err()
        );
        assert!(!path(temp.path(), &id).unwrap().exists());
    }
    #[test]
    fn slow_disk_applies_backpressure_and_flush_keeps_all_fragments() {
        let temp = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let held = lock(temp.path()).unwrap();
        let recorder = Recorder::at(id.clone(), Ok(temp.path().to_owned()));
        let message = uuid::Uuid::new_v4().to_string();
        recorder.record(Record::Header {
            version: 1,
            title: "Fixture".into(),
        });
        recorder.record(Record::Message {
            id: message.clone(),
            title: "Agente".into(),
            text: "x".repeat(1024 * 1024),
            tool: false,
            response: true,
        });
        recorder.record(Record::Append {
            id: message,
            text: "y".repeat(1024 * 1024),
        });
        assert!(!recorder.ready());
        assert!(recorder.error().is_none());
        drop(held);
        crate::runtime::runtime()
            .block_on(recorder.flush())
            .unwrap();
        assert!(recorder.ready());
        let exported = temp.path().join("full.jsonl");
        export(temp.path(), &id, &exported).unwrap();
        let records: Vec<Record> = fs::read_to_string(exported)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(matches!(&records[1],Record::Message{text,..} if text.len()==1024*1024));
        assert!(matches!(&records[2],Record::Append{text,..} if text.len()==1024*1024));
    }
}

#[cfg(test)]
mod recent_tests {
    use super::*;
    fn chat(agent: &str) -> crate::workspaces::Chat {
        serde_json::from_value(serde_json::json!({
            "id":uuid::Uuid::new_v4().to_string(),"title":format!("Chat {agent}"),
            "agent_id":agent,"model":"fixture","directory":"/tmp",
            "codex_thread":null,"messages":[],"draft":"Borrador pendiente"
        }))
        .unwrap()
    }
    #[test]
    fn recents_keep_agent_state_private_and_do_not_resurrect_deleted_chats() {
        let dir = tempfile::tempdir().unwrap();
        let codex = chat("codex");
        let github = chat("github");
        RecentSave::new(codex.clone()).save(dir.path()).unwrap();
        RecentSave::new(github.clone()).save(dir.path()).unwrap();
        assert_eq!(
            recent(dir.path(), "codex")
                .unwrap()
                .iter()
                .map(|r| &r.id)
                .collect::<Vec<_>>(),
            vec![&codex.id]
        );
        assert_eq!(
            recent(dir.path(), "github")
                .unwrap()
                .iter()
                .map(|r| &r.id)
                .collect::<Vec<_>>(),
            vec![&github.id]
        );
        assert!(recent(dir.path(), "other").unwrap().is_empty());
        assert_eq!(load_recent(dir.path(), &codex.id).unwrap(), codex);
        assert_eq!(
            fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(dir.path().join(format!("{}.json", codex.id)))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        remove(dir.path(), &codex.id).unwrap();
        RecentSave::new(codex.clone()).save(dir.path()).unwrap();
        assert!(load_recent(dir.path(), &codex.id).is_err());
        assert!(recent(dir.path(), "codex").unwrap().is_empty());
        assert_eq!(recent(dir.path(), "github").unwrap().len(), 1);
    }
    #[test]
    fn late_writes_corruption_and_capture_preference_are_handled() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = chat("codex");
        let old = RecentSave::new(c.clone());
        c.draft = "Texto más reciente".into();
        RecentSave::new(c.clone()).save(dir.path()).unwrap();
        old.save(dir.path()).unwrap();
        assert_eq!(
            load_recent(dir.path(), &c.id).unwrap().draft,
            "Texto más reciente"
        );
        let target = dir.path().join(format!("{}.json", c.id));
        fs::write(&target, b"broken").unwrap();
        assert!(load_recent(dir.path(), &c.id).is_err());
        assert!(recent(dir.path(), "codex").unwrap().is_empty());
        assert_eq!(fs::read(&target).unwrap(), b"broken");
        fs::remove_file(&target).unwrap();
        std::os::unix::fs::symlink("/dev/zero", &target).unwrap();
        assert!(load_recent(dir.path(), &c.id).is_err());
        let mut disabled = chat("codex");
        disabled.capture_history = false;
        RecentSave::new(disabled.clone()).save(dir.path()).unwrap();
        assert!(
            !load_recent(dir.path(), &disabled.id)
                .unwrap()
                .capture_history
        );
        assert!(!dir.path().join(format!("{}.jsonl", disabled.id)).exists());
    }
}
