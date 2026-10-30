//! Durable transfer intent. Loading never starts a transport or retries a job.
use crate::config::Profile;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    os::unix::{
        fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
const LIMIT: u64 = 4 * 1024 * 1024;
pub const MAX_JOBS: usize = 512;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Running,
    Done,
    Failed,
    Cancelled,
    Interrupted,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: String,
    pub upload: bool,
    pub tree: bool,
    pub mkdir: bool,
    pub local: PathBuf,
    pub remote: PathBuf,
    pub status: Status,
    pub attempts: u32,
    pub error: String,
}
impl Job {
    pub fn new(upload: bool, tree: bool, local: PathBuf, remote: PathBuf) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            upload,
            tree,
            mkdir: false,
            local,
            remote,
            status: Status::Pending,
            attempts: 0,
            error: String::new(),
        }
    }
    pub fn request(&self) -> crate::transfer::Request {
        use crate::transfer::Request;
        if self.mkdir {
            return if self.upload {
                Request::Create {
                    directory: self.remote.parent().unwrap().into(),
                    name: self.remote.file_name().unwrap().into(),
                    folder: true,
                }
            } else {
                Request::CreateLocalDirectory(self.local.clone())
            };
        }
        if self.upload {
            let directory = self.remote.parent().unwrap().to_path_buf();
            if self.tree {
                Request::PlanUpload {
                    local: self.local.clone(),
                    directory,
                }
            } else {
                Request::Upload {
                    local: self.local.clone(),
                    directory,
                }
            }
        } else if self.tree {
            Request::PlanDownload {
                remote: self.remote.clone(),
                local: self.local.clone(),
            }
        } else {
            Request::Download {
                remote: self.remote.clone(),
                local: self.local.clone(),
            }
        }
    }
    fn validate(&self) -> Result<(), String> {
        uuid::Uuid::parse_str(&self.id).map_err(|_| "Identificador de transferencia inválido")?;
        for path in [&self.local, &self.remote] {
            if !path.is_absolute()
                || path.as_os_str().len() > 4096
                || path.to_str().is_none()
                || path.to_string_lossy().chars().any(char::is_control)
                || path
                    .components()
                    .any(|c| matches!(c, std::path::Component::ParentDir))
                || path.file_name().is_none()
            {
                return Err("Ruta de transferencia inválida".into());
            }
        }
        if self.mkdir && self.tree {
            return Err("Tipo de transferencia inválido".into());
        }
        if self.upload && self.local.file_name() != self.remote.file_name() {
            return Err("El nombre del destino de subida no coincide".into());
        }
        if self.error.len() > 4096 {
            return Err("Error de transferencia demasiado grande".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    pub scope: String,
    pub revision: u64,
    pub jobs: Vec<Job>,
}
impl Snapshot {
    fn validate(&self, scope: &str) -> Result<(), String> {
        if self.version != 1 || self.scope != scope || self.jobs.len() > MAX_JOBS {
            return Err("Cola incompatible o demasiado grande; se conserva el archivo".into());
        }
        let mut ids = std::collections::HashSet::new();
        for job in &self.jobs {
            job.validate()?;
            if !ids.insert(&job.id) {
                return Err("Transferencia duplicada".into());
            }
        }
        if self
            .jobs
            .iter()
            .filter(|j| j.status == Status::Running)
            .count()
            > 1
        {
            return Err("Hay varias transferencias activas en el archivo".into());
        }
        Ok(())
    }
}
pub enum Edit {
    Defer(String),
    Expand {
        id: String,
        jobs: Vec<Job>,
    },
    Add(Vec<Job>),
    Begin,
    Finish {
        id: String,
        error: Option<String>,
        cancelled: bool,
    },
    Retry(String),
    Cancel(String),
    Interrupt(String),
    ClearDone,
}
pub struct Journal {
    path: PathBuf,
    scope: String,
    _lock: fs::File,
    state: Mutex<Snapshot>,
    failed: AtomicBool,
}
pub fn scope(profile: &Profile) -> String {
    format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(
                &profile.id,
                profile.protocol,
                &profile.host,
                profile.port,
                &profile.username,
                &profile.domain,
                &profile.jump_host
            ))
            .unwrap()
        )
    )
}
pub fn directory() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
        .ok_or("No se encuentra el directorio de datos")?;
    Ok(base.join("nexuterm/transfer-queues"))
}
impl Journal {
    pub fn open(directory: &Path, scope: &str) -> Result<Self, String> {
        if scope.len() != 64 || !scope.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Destino de cola inválido".into());
        }
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)
            .map_err(|e| e.to_string())?;
        let meta = fs::symlink_metadata(directory).map_err(|e| e.to_string())?;
        if !meta.is_dir() {
            return Err("El directorio de colas no es un directorio real".into());
        }
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(directory.join(format!("{scope}.lock")))
            .map_err(|e| e.to_string())?;
        if !lock.metadata().map_err(|e| e.to_string())?.is_file()
            || unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0
        {
            return Err("La cola está abierta en otra ventana o proceso".into());
        }
        let path = directory.join(format!("{scope}.json"));
        let mut state = match fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(file) => {
                if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                    return Err("Archivo de cola inválido".into());
                }
                let mut bytes = Vec::new();
                file.take(LIMIT + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| e.to_string())?;
                if bytes.len() as u64 > LIMIT {
                    return Err("La cola supera el límite de 4 MiB".into());
                }
                serde_json::from_slice::<Snapshot>(&bytes)
                    .map_err(|e| format!("Cola dañada; se conserva sin modificar: {e}"))?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Snapshot {
                version: 1,
                scope: scope.into(),
                revision: 0,
                jobs: Vec::new(),
            },
            Err(e) => return Err(e.to_string()),
        };
        state.validate(scope)?;
        for job in &mut state.jobs {
            if job.status == Status::Running {
                job.status = Status::Interrupted;
                job.error="La ejecución anterior terminó sin confirmar el resultado. Revisa el destino antes de reintentar.".into();
            }
        }
        state.revision = state.revision.saturating_add(1);
        let journal = Self {
            path,
            scope: scope.into(),
            _lock: lock,
            state: Mutex::new(state),
            failed: AtomicBool::new(false),
        };
        journal.save(&journal.snapshot()?)?;
        Ok(journal)
    }
    fn save(&self, state: &Snapshot) -> Result<(), String> {
        state.validate(&self.scope)?;
        let bytes = serde_json::to_vec(state).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > LIMIT {
            return Err("La cola supera el límite de 4 MiB".into());
        }
        let parent = self.path.parent().unwrap();
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temp.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        temp.write_all(&bytes).map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        temp.persist(&self.path).map_err(|e| e.to_string())?;
        fs::File::open(parent)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        self.state
            .lock()
            .map(|s| s.clone())
            .map_err(|_| "La cola no está disponible".into())
    }
    pub fn edit(&self, edit: Edit) -> Result<(Snapshot, Option<Job>), String> {
        if self.failed.load(Ordering::Relaxed) {
            return Err("La cola debe volver a abrirse tras el error de almacenamiento; no se ejecutarán operaciones.".into());
        }
        let mut current = self
            .state
            .lock()
            .map_err(|_| "La cola no está disponible")?;
        let mut next = current.clone();
        let mut picked = None;
        match edit {
            Edit::Defer(id) => {
                let job = next
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id && j.status == Status::Running)
                    .ok_or("Transferencia no preparada")?;
                job.status = Status::Pending;
                job.attempts = job.attempts.saturating_sub(1);
            }
            Edit::Expand { id, jobs } => {
                let index = next
                    .jobs
                    .iter()
                    .position(|j| j.id == id && j.status == Status::Running && j.tree)
                    .ok_or("Plan de carpeta no encontrado")?;
                if jobs.is_empty() || next.jobs.len() - 1 + jobs.len() > MAX_JOBS {
                    return Err(
                        "La carpeta supera el espacio disponible en la cola (512 elementos)".into(),
                    );
                }
                for job in &jobs {
                    job.validate()?;
                    if job.status != Status::Pending || job.tree {
                        return Err("Plan de carpeta inválido".into());
                    }
                }
                next.jobs.splice(index..=index, jobs);
            }
            Edit::Add(jobs) => {
                if jobs.len() > 128 || next.jobs.len() + jobs.len() > MAX_JOBS {
                    return Err("Máximo 128 elementos por lote y 512 en la cola. Retira los completados para continuar.".into());
                }
                for job in &jobs {
                    job.validate()?;
                    if job.status != Status::Pending {
                        return Err("Nueva transferencia inválida".into());
                    }
                }
                next.jobs.extend(jobs);
            }
            Edit::Begin => {
                if next.jobs.iter().any(|j| j.status == Status::Running) {
                    return Err("La cola ya tiene una operación activa".into());
                }
                if let Some(job) = next.jobs.iter_mut().find(|j| j.status == Status::Pending) {
                    job.status = Status::Running;
                    job.attempts = job.attempts.saturating_add(1);
                    job.error.clear();
                    picked = Some(job.clone());
                }
            }
            Edit::Finish {
                id,
                error,
                cancelled,
            } => {
                let job = next
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id)
                    .ok_or("Transferencia no encontrada")?;
                if job.status != Status::Running {
                    return Err("La transferencia ya no está activa".into());
                }
                job.status = if cancelled {
                    Status::Cancelled
                } else if error.is_some() {
                    Status::Failed
                } else {
                    Status::Done
                };
                job.error = error.unwrap_or_default().chars().take(1000).collect();
            }
            Edit::Retry(id) => {
                let job = next
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id)
                    .ok_or("Transferencia no encontrada")?;
                if !matches!(
                    job.status,
                    Status::Failed | Status::Cancelled | Status::Interrupted
                ) {
                    return Err(
                        "Solo se reintentan transferencias fallidas, canceladas o interrumpidas"
                            .into(),
                    );
                }
                job.status = Status::Pending;
                job.error.clear();
            }
            Edit::Cancel(id) => {
                let job = next
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id)
                    .ok_or("Transferencia no encontrada")?;
                if job.status != Status::Pending {
                    return Err("La transferencia ya no está pendiente".into());
                }
                job.status = Status::Cancelled;
            }
            Edit::Interrupt(id) => {
                if let Some(job) = next
                    .jobs
                    .iter_mut()
                    .find(|j| j.id == id && j.status == Status::Running)
                {
                    job.status = Status::Interrupted;
                    job.error =
                        "Conexión cerrada sin confirmar el resultado; revisa el destino.".into();
                }
            }
            Edit::ClearDone => next.jobs.retain(|j| j.status != Status::Done),
        }
        next.revision = next.revision.saturating_add(1);
        if let Err(error) = self.save(&next) {
            self.failed.store(true, Ordering::Relaxed);
            return Err(format!(
                "{error}. Cola detenida; vuelve a abrir la conexión para revisar su estado."
            ));
        }
        *current = next.clone();
        Ok((next, picked))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn job() -> Job {
        Job::new(true, false, "/tmp/a".into(), "/remote/a".into())
    }
    #[test]
    fn durable_manual_retry_and_crash_recovery() {
        let root = tempfile::tempdir().unwrap();
        let key = "a".repeat(64);
        let j = Journal::open(root.path(), &key).unwrap();
        let id = job();
        j.edit(Edit::Add(vec![id.clone()])).unwrap();
        j.edit(Edit::Cancel(id.id.clone())).unwrap();
        assert_eq!(j.snapshot().unwrap().jobs[0].status, Status::Cancelled);
        j.edit(Edit::Retry(id.id.clone())).unwrap();
        let (_, running) = j.edit(Edit::Begin).unwrap();
        assert_eq!(running.unwrap().attempts, 1);
        j.edit(Edit::Defer(id.id.clone())).unwrap();
        assert_eq!(j.snapshot().unwrap().jobs[0].attempts, 0);
        j.edit(Edit::Begin).unwrap();
        assert!(Journal::open(root.path(), &key).is_err());
        drop(j);
        let j = Journal::open(root.path(), &key).unwrap();
        assert_eq!(j.snapshot().unwrap().jobs[0].status, Status::Interrupted);
        assert!(j.edit(Edit::Begin).unwrap().1.is_none());
        j.edit(Edit::Retry(id.id.clone())).unwrap();
        assert_eq!(j.edit(Edit::Begin).unwrap().1.unwrap().attempts, 2);
        j.edit(Edit::Finish {
            id: id.id,
            error: None,
            cancelled: false,
        })
        .unwrap();
        j.edit(Edit::ClearDone).unwrap();
        assert!(j.snapshot().unwrap().jobs.is_empty());
        assert_eq!(
            fs::metadata(root.path().join(format!("{key}.json")))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    #[test]
    fn folder_expansion_is_durable_and_storage_failure_stops_mutations() {
        let root = tempfile::tempdir().unwrap();
        let key = "c".repeat(64);
        let j = Journal::open(root.path(), &key).unwrap();
        let mut folder = job();
        folder.tree = true;
        j.edit(Edit::Add(vec![folder.clone()])).unwrap();
        j.edit(Edit::Begin).unwrap();
        let mut dir = job();
        dir.mkdir = true;
        j.edit(Edit::Expand {
            id: folder.id,
            jobs: vec![
                dir.clone(),
                Job::new(true, false, "/tmp/a/b".into(), "/remote/a/b".into()),
            ],
        })
        .unwrap();
        assert_eq!(j.snapshot().unwrap().jobs.len(), 2);
        assert!(j.snapshot().unwrap().jobs[0].mkdir);
        let original = fs::read(&j.path).unwrap();
        fs::rename(&j.path, j.path.with_extension("backup")).unwrap();
        fs::create_dir(&j.path).unwrap();
        assert!(j.edit(Edit::Begin).is_err());
        assert!(j.edit(Edit::Add(vec![job()])).is_err());
        assert_eq!(fs::read(j.path.with_extension("backup")).unwrap(), original);
    }
    #[test]
    fn limits_and_destination_binding() {
        let root = tempfile::tempdir().unwrap();
        let key = "d".repeat(64);
        let j = Journal::open(root.path(), &key).unwrap();
        assert!(
            j.edit(Edit::Add((0..129).map(|_| job()).collect()))
                .is_err()
        );
        let mut profile = Profile {
            id: "profile".into(),
            host: "one.example.invalid".into(),
            ..Default::default()
        };
        let first = scope(&profile);
        profile.host = "two.example.invalid".into();
        assert_ne!(scope(&profile), first);
    }
    #[test]
    fn invalid_input_and_corrupt_file_preserve_previous_queue() {
        let root = tempfile::tempdir().unwrap();
        let key = "b".repeat(64);
        let j = Journal::open(root.path(), &key).unwrap();
        j.edit(Edit::Add(vec![job()])).unwrap();
        let before = fs::read(&j.path).unwrap();
        let mut invalid = job();
        invalid.local = "relative".into();
        assert!(j.edit(Edit::Add(vec![invalid])).is_err());
        assert_eq!(before, fs::read(&j.path).unwrap());
        let path = j.path.clone();
        drop(j);
        fs::write(&path, b"broken").unwrap();
        assert!(Journal::open(root.path(), &key).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"broken");
        fs::remove_file(&path).unwrap();
        let pipe = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(pipe.as_ptr(), 0o600) }, 0);
        assert!(Journal::open(root.path(), &key).is_err());
        fs::remove_file(&path).unwrap();
        let original = root.path().join("original");
        fs::write(&original, b"preserve").unwrap();
        std::os::unix::fs::symlink(&original, &path).unwrap();
        assert!(Journal::open(root.path(), &key).is_err());
        assert_eq!(fs::read(original).unwrap(), b"preserve");
    }
}
