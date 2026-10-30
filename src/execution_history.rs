//! Private bounded execution records; recovered jobs are evidence, never instructions to rerun.
use crate::execution::{OUTPUT_LIMIT, Record, Status};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    sync::Arc,
};
pub const LIMIT: u64 = 1024 * 1024 * 1024;
const RESERVED: u64 = (OUTPUT_LIMIT * 4 + 256 * 1024) as u64;
pub fn directory() -> Result<PathBuf, String> {
    Ok(crate::recovery::directory()?
        .parent()
        .ok_or("Ruta de datos inválida")?
        .join("executions"))
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
        .is_dir()
    {
        return Err("El historial no es un directorio normal".into());
    }
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(dir.join(".lock"))
        .map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Bloqueo de historial inválido".into());
    }
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(Lock(file))
}
fn path(dir: &Path, id: &str) -> Result<PathBuf, String> {
    uuid::Uuid::parse_str(id).map_err(|_| "Identificador de ejecución inválido")?;
    Ok(dir.join(format!("{id}.json")))
}
pub struct Journal {
    dir: PathBuf,
    id: String,
    reservation: PathBuf,
    _file: File,
    _slot: Arc<tokio::sync::OwnedSemaphorePermit>,
}
impl Journal {
    pub fn start(
        dir: &Path,
        record: &Record,
        slot: Arc<tokio::sync::OwnedSemaphorePermit>,
    ) -> Result<Self, String> {
        record.validate()?;
        let _lock = lock(dir)?;
        let target = path(dir, &record.id)?;
        if target.exists() {
            return Err("La ejecución ya tiene un registro; no se reemplaza".into());
        }
        let mut totals = std::collections::HashMap::<String, u64>::new();
        let mut other = 0usize;
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let p = entry.path();
            if entry.file_name() == ".lock" {
                continue;
            }
            let metadata = match fs::symlink_metadata(&p) {
                Ok(m) => m,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.to_string()),
            };
            if !metadata.is_file() {
                return Err("El historial contiene una entrada no regular; revísala antes de registrar otra ejecución".into());
            }
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            let id = p
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| uuid::Uuid::parse_str(s).is_ok());
            if !matches!(ext, "json" | "reserve") || id.is_none() {
                other += 1;
                totals.insert(format!("other-{other}"), metadata.len());
                continue;
            }
            let id = id.unwrap();
            if ext == "reserve" {
                let f = match OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                    .open(&p)
                {
                    Ok(f) => f,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(e.to_string()),
                };
                if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                    fs::remove_file(&p).map_err(|e| e.to_string())?;
                    continue;
                }
            }
            let size = metadata.len();
            let total = totals.entry(id.into()).or_default();
            *total = (*total).max(size);
        }
        if totals.len() >= 1000
            || totals
                .values()
                .copied()
                .fold(0u64, u64::saturating_add)
                .saturating_add(RESERVED)
                > LIMIT
        {
            return Err("Límite del historial de ejecuciones (1 GiB/1000 registros). Libera espacio o desmarca conservar historial. No se borran registros automáticamente.".into());
        }
        let reservation = dir.join(format!("{}.reserve", record.id));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&reservation)
            .map_err(|e| e.to_string())?;
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        file.set_len(RESERVED).map_err(|e| e.to_string())?;
        let journal = Self {
            dir: dir.into(),
            id: record.id.clone(),
            reservation,
            _file: file,
            _slot: slot,
        };
        journal.write_locked(record)?;
        Ok(journal)
    }
    fn write_locked(&self, record: &Record) -> Result<(), String> {
        record.validate()?;
        if record.id != self.id {
            return Err("Registro de otro proceso".into());
        }
        let mut file = tempfile::NamedTempFile::new_in(&self.dir).map_err(|e| e.to_string())?;
        {
            let mut writer = std::io::BufWriter::with_capacity(64 * 1024, file.as_file_mut());
            serde_json::to_writer(&mut writer, record).map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;
        }
        if file.as_file().metadata().map_err(|e| e.to_string())?.len() > RESERVED {
            return Err("Registro demasiado grande".into());
        }
        file.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        file.flush()
            .and_then(|_| file.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        file.persist(path(&self.dir, &self.id)?)
            .map_err(|e| e.to_string())?;
        File::open(&self.dir)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
    pub fn write(&self, record: &Record) -> Result<(), String> {
        let _lock = lock(&self.dir)?;
        self.write_locked(record)
    }
}
impl Drop for Journal {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.reservation);
    }
}
fn active(dir: &Path, id: &str) -> bool {
    let Ok(f) = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(dir.join(format!("{id}.reserve")))
    else {
        return false;
    };
    unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) != 0 }
}
pub fn read(dir: &Path, id: &str) -> Result<Record, String> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path(dir, id)?)
        .map_err(|e| e.to_string())?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > RESERVED {
        return Err("Registro inválido o demasiado grande".into());
    }
    let mut record: Record = serde_json::from_reader(std::io::BufReader::with_capacity(
        64 * 1024,
        file.take(RESERVED + 1),
    ))
    .map_err(|e| e.to_string())?;
    record.validate()?;
    if record.id != id {
        return Err("El registro no corresponde a la ejecución".into());
    }
    if !record.finished() && !active(dir, id) {
        record.status = Status::Unknown;
        record.exit_code = None;
        record.error="La aplicación dejó de confirmar este proceso. Su resultado es desconocido; no se reanudará ni repetirá automáticamente.".into();
    }
    Ok(record)
}
pub struct Entry {
    pub id: String,
    pub label: String,
    pub started_at: u64,
}
pub fn list(dir: &Path) -> Result<Vec<Entry>, String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let p = entry.map_err(|e| e.to_string())?.path();
        if p.extension().is_none_or(|e| e != "json") {
            continue;
        }
        if out.len() >= 1000 {
            return Err("Demasiados registros".into());
        }
        if let Some(id) = p.file_stem().and_then(|s| s.to_str()) {
            match read(dir, id) {
                Ok(r) => out.push(Entry {
                    id: id.into(),
                    label: format!(
                        "{} · {}",
                        r.destination.label,
                        r.spec.command.chars().take(80).collect::<String>()
                    ),
                    started_at: r.started_at,
                }),
                Err(_) => out.push(Entry {
                    id: id.into(),
                    label: format!("Registro {id} · no se pudo leer"),
                    started_at: 0,
                }),
            }
        }
    }
    out.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(out)
}
pub fn remove(dir: &Path, id: &str) -> Result<(), String> {
    let target = path(dir, id)?;
    let _lock = lock(dir)?;
    let reservation = dir.join(format!("{id}.reserve"));
    match OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(reservation)
    {
        Ok(f) => {
            if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
                return Err("La ejecución sigue activa; no se elimina su registro".into());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.to_string()),
    }
    fs::remove_file(target).map_err(|e| e.to_string())?;
    File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}

pub fn export(record: &Record, destination: &Path) -> Result<(), String> {
    record.validate()?;
    let parent = destination.parent().ok_or("Destino sin carpeta")?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    write!(file,"NexuTerm · {}\nDestino: {}\nCarpeta: {}\nComando:\n{}\nEstado: {:?} · retorno: {:?} · duración: {} ms\nSalida truncada: {}\n{}\n--- stdout ---\n",record.id,record.destination.label,record.spec.directory.display(),record.spec.command,record.status,record.exit_code,record.duration_ms,record.truncated,record.error).and_then(|_|file.write_all(&record.stdout)).and_then(|_|file.write_all(b"\n--- stderr ---\n")).and_then(|_|file.write_all(&record.stderr)).and_then(|_|file.as_file().sync_all()).map_err(|e|e.to_string())?;
    file.persist_noclobber(destination)
        .map_err(|e| e.to_string())?;
    File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{Destination, Spec};
    fn record() -> Record {
        Record {
            version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            spec: Spec {
                command: "printf fixture".into(),
                directory: "/tmp".into(),
                timeout_secs: 60,
                keep_history: true,
            },
            destination: Destination {
                id: "local".into(),
                label: "Este equipo".into(),
                remote: false,
            },
            started_at: 1,
            duration_ms: 0,
            status: Status::Running,
            exit_code: None,
            stdout: vec![],
            stderr: vec![],
            truncated: false,
            error: String::new(),
            recording_error: String::new(),
        }
    }
    fn slot() -> Arc<tokio::sync::OwnedSemaphorePermit> {
        Arc::new(
            Arc::new(tokio::sync::Semaphore::new(1))
                .try_acquire_owned()
                .unwrap(),
        )
    }
    #[test]
    fn private_records_export_delete_and_unknown_after_crash() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("history");
        let mut r = record();
        let journal = Journal::start(&dir, &r, slot()).unwrap();
        assert_eq!(read(&dir, &r.id).unwrap().status, Status::Running);
        assert!(remove(&dir, &r.id).is_err());
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(path(&dir, &r.id).unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(journal);
        assert_eq!(read(&dir, &r.id).unwrap().status, Status::Unknown);
        assert_eq!(list(&dir).unwrap().len(), 1);
        let file = temp.path().join("export.log");
        r.stdout = vec![0, 255, b'x'];
        r.stderr = b"err".to_vec();
        export(&r, &file).unwrap();
        assert!(
            fs::read(&file)
                .unwrap()
                .windows(3)
                .any(|b| b == [0, 255, b'x'])
        );
        assert!(export(&r, &file).is_err());
        assert_eq!(
            fs::metadata(file).unwrap().permissions().mode() & 0o777,
            0o600
        );
        remove(&dir, &r.id).unwrap();
        assert!(list(&dir).unwrap().is_empty());
    }
    #[test]
    fn maximum_binary_capture_roundtrips_within_reserved_size() {
        let temp = tempfile::tempdir().unwrap();
        let mut r = record();
        r.status = Status::Completed;
        r.exit_code = Some(0);
        r.stdout = vec![255; OUTPUT_LIMIT / 2];
        r.stderr = vec![0; OUTPUT_LIMIT / 2];
        let journal = Journal::start(temp.path(), &r, slot()).unwrap();
        journal.write(&r).unwrap();
        let restored = read(temp.path(), &r.id).unwrap();
        assert_eq!(restored.stdout, r.stdout);
        assert_eq!(restored.stderr, r.stderr);
        assert!(
            fs::metadata(path(temp.path(), &r.id).unwrap())
                .unwrap()
                .len()
                <= RESERVED
        );
    }
    #[test]
    fn quota_corruption_and_symlinks_never_trigger_execution() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path();
        let huge = dir.join(format!("{}.json", uuid::Uuid::new_v4()));
        File::create(&huge).unwrap().set_len(LIMIT).unwrap();
        assert!(Journal::start(dir, &record(), slot()).is_err());
        let orphan = dir.join(".unpublished.tmp");
        fs::rename(&huge, &orphan).unwrap();
        assert!(Journal::start(dir, &record(), slot()).is_err());
        assert!(orphan.exists());
        fs::remove_file(orphan).unwrap();
        let r = record();
        fs::write(path(dir, &r.id).unwrap(), "damaged").unwrap();
        assert!(read(dir, &r.id).is_err());
        assert_eq!(fs::read(path(dir, &r.id).unwrap()).unwrap(), b"damaged");
        fs::remove_file(path(dir, &r.id).unwrap()).unwrap();
        std::os::unix::fs::symlink("/dev/zero", path(dir, &r.id).unwrap()).unwrap();
        assert!(read(dir, &r.id).is_err());
    }
}

#[cfg(test)]
mod concurrent_tests {
    use super::*;
    #[test]
    fn concurrent_writers_and_reservation_release_keep_every_record() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().to_path_buf();
        let threads = (0..4)
            .map(|_| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    for _ in 0..25 {
                        let r = crate::execution::Record {
                            version: 1,
                            id: uuid::Uuid::new_v4().to_string(),
                            spec: crate::execution::Spec {
                                command: "true".into(),
                                directory: "/tmp".into(),
                                timeout_secs: 60,
                                keep_history: true,
                            },
                            destination: crate::execution::Destination {
                                id: "local".into(),
                                label: "Local".into(),
                                remote: false,
                            },
                            started_at: 1,
                            duration_ms: 0,
                            status: Status::Completed,
                            exit_code: Some(0),
                            stdout: vec![],
                            stderr: vec![],
                            truncated: false,
                            error: String::new(),
                            recording_error: String::new(),
                        };
                        let slot = Arc::new(
                            Arc::new(tokio::sync::Semaphore::new(1))
                                .try_acquire_owned()
                                .unwrap(),
                        );
                        let journal = Journal::start(&dir, &r, slot).unwrap();
                        journal.write(&r).unwrap();
                        drop(journal);
                    }
                })
            })
            .collect::<Vec<_>>();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(list(&dir).unwrap().len(), 100);
        assert!(!fs::read_dir(dir).unwrap().any(|e| {
            e.unwrap()
                .path()
                .extension()
                .is_some_and(|e| e == "reserve")
        }));
    }
}
