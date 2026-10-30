//! Per-window automatic checkpoints, isolated from named workspaces.
use crate::workspaces::{Store, Workspace};
use std::{
    fs::{self, File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub const MAX_SESSIONS: usize = 32;

pub fn directory() -> Result<PathBuf, String> {
    Ok(Store::path()?
        .parent()
        .ok_or("Ruta de espacios inválida")?
        .join("sessions"))
}

pub struct Journal {
    pub id: String,
    pub path: PathBuf,
    lock: File,
    retired: Mutex<bool>,
}
impl Drop for Journal {
    fn drop(&mut self) {
        if fs::symlink_metadata(&self.path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            if let Some(parent) = self.path.parent() {
                let _ = fs::remove_file(parent.join(format!(".{}.lock", self.id)));
            }
        }
        // Release explicitly: a concurrent fork may hold the descriptor until exec.
        unsafe {
            libc::flock(self.lock.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
impl Journal {
    fn acquire(directory: &Path, id: &str) -> Result<Option<Arc<Self>>, String> {
        uuid::Uuid::parse_str(id).map_err(|_| "Identificador de sesión inválido")?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(directory.join(format!(".{id}.lock")))
            .map_err(|e| e.to_string())?;
        lock.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        // Advisory locks survive atomic replacement of the separate snapshot file.
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(None);
            }
            return Err(error.to_string());
        }
        Ok(Some(Arc::new(Self {
            id: id.into(),
            path: directory.join(format!("{id}.json")),
            lock,
            retired: Mutex::new(false),
        })))
    }
    pub fn create(directory: &Path) -> Result<Arc<Self>, String> {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(directory)
            .map_err(|e| e.to_string())?;
        if !fs::symlink_metadata(directory)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_dir()
        {
            return Err("El almacén de sesiones no es un directorio normal".into());
        }
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let count = fs::read_dir(directory)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .count();
        if count >= MAX_SESSIONS {
            return Err("Hay 32 copias de sesión pendientes; recupera o descarta alguna antes de crear otra".into());
        }
        Self::acquire(directory, &uuid::Uuid::new_v4().to_string())?
            .ok_or("La nueva sesión está ocupada".into())
    }
    pub fn read(&self) -> Result<Workspace, String> {
        let metadata = fs::symlink_metadata(&self.path).map_err(|e| e.to_string())?;
        if !metadata.file_type().is_file() {
            return Err("La copia de sesión no es un archivo normal".into());
        }
        let mut store = Store::load(&self.path)?;
        if store.workspaces.len() != 1 || store.workspaces[0].id != self.id {
            return Err("La copia no corresponde a esta sesión".into());
        }
        Ok(store.workspaces.remove(0))
    }
    pub fn save(&self, workspace: Workspace) -> Result<(), String> {
        let retired = self
            .retired
            .lock()
            .map_err(|_| "Estado de sesión no disponible")?;
        if *retired {
            return Err("La copia de sesión ya se ha retirado".into());
        }
        if workspace.id != self.id {
            return Err("Identificador de sesión incorrecto".into());
        }
        match fs::symlink_metadata(&self.path) {
            Ok(_) => {
                self.read()?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        Store {
            version: 1,
            workspaces: vec![workspace],
        }
        .save(&self.path)
    }
    /// Retire only this checkpoint, never its referenced documents or drafts.
    pub fn remove(&self) -> Result<(), String> {
        let mut retired = self
            .retired
            .lock()
            .map_err(|_| "Estado de sesión no disponible")?;
        fs::remove_file(&self.path)
            .or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            })
            .map_err(|e| e.to_string())?;
        *retired = true;
        if let Some(parent) = self.path.parent() {
            File::open(parent)
                .and_then(|f| f.sync_all())
                .map_err(|e| e.to_string())?;
            // The UUID is never reused. Scanners require the JSON to exist before locking.
            let _ = fs::remove_file(parent.join(format!(".{}.lock", self.id)));
        }
        Ok(())
    }
}

pub struct Entry {
    pub journal: Arc<Journal>,
    pub workspace: Workspace,
    pub saved_at: u64,
}

pub fn list(directory: &Path) -> Result<(Vec<Entry>, usize), String> {
    match fs::symlink_metadata(directory) {
        Ok(m) if !m.file_type().is_dir() => {
            return Err("El almacén de sesiones no es un directorio normal".into());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], 0)),
        Err(e) => return Err(e.to_string()),
        _ => {}
    }
    let entries = match fs::read_dir(directory) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((vec![], 0)),
        Err(e) => return Err(e.to_string()),
    };
    let mut found = Vec::new();
    let mut invalid = 0;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().is_none_or(|v| v != "json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|v| v.to_str()) else {
            invalid += 1;
            continue;
        };
        match Journal::acquire(directory, id) {
            Ok(Some(journal)) => match journal.read() {
                Ok(workspace) => {
                    let saved_at = fs::metadata(&journal.path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    found.push(Entry {
                        journal,
                        workspace,
                        saved_at,
                    });
                }
                Err(_) => invalid += 1,
            },
            Ok(None) => {}
            Err(_) => invalid += 1,
        }
        if found.len() + invalid > MAX_SESSIONS {
            return Err(
                "Demasiadas copias de sesión; revisa el almacén sin borrar documentos".into(),
            );
        }
    }
    found.sort_by(|a, b| {
        b.saved_at
            .cmp(&a.saved_at)
            .then_with(|| a.journal.id.cmp(&b.journal.id))
    });
    Ok((found, invalid))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(j: &Journal) -> Workspace {
        Workspace {
            id: j.id.clone(),
            name: "Sesión automática".into(),
            notes: String::new(),
            tabs: vec![],
            selected: 0,
            directory: None,
            sidebar_visible: true,
            browser: None,
            chats: Default::default(),
        }
    }
    #[test]
    fn private_atomic_and_locked_until_owner_exits() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("sessions");
        let j = Journal::create(&dir).unwrap();
        let path = j.path.clone();
        let id = j.id.clone();
        j.save(snapshot(&j)).unwrap();
        assert!(list(&dir).unwrap().0.is_empty());
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        let previous = fs::read(&path).unwrap();
        let mut invalid = snapshot(&j);
        invalid.name.clear();
        assert!(j.save(invalid).is_err());
        assert_eq!(fs::read(&path).unwrap(), previous);
        drop(j);
        let (pending, bad) = list(&dir).unwrap();
        assert_eq!(bad, 0);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].workspace.id, id);
        assert!(list(&dir).unwrap().0.is_empty());
        pending[0].journal.remove().unwrap();
        assert!(!path.exists());
        assert!(
            pending[0]
                .journal
                .save(pending[0].workspace.clone())
                .is_err()
        );
    }
    #[test]
    fn independent_windows_and_corruption_are_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let a = Journal::create(temp.path()).unwrap();
        let b = Journal::create(temp.path()).unwrap();
        a.save(snapshot(&a)).unwrap();
        b.save(snapshot(&b)).unwrap();
        let bad = a.path.clone();
        fs::write(&bad, b"unknown-format").unwrap();
        assert!(a.save(snapshot(&a)).is_err());
        assert_eq!(fs::read(&bad).unwrap(), b"unknown-format");
        drop(a);
        drop(b);
        let (entries, invalid) = list(temp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(invalid, 1);
        let target = temp.path().join("original.txt");
        fs::write(&target, b"original").unwrap();
        let link = temp.path().join(format!("{}.json", uuid::Uuid::new_v4()));
        std::os::unix::fs::symlink(&target, link).unwrap();
        assert_eq!(list(temp.path()).unwrap().1, 2);
        assert_eq!(fs::read(&target).unwrap(), b"original");
    }
    #[test]
    fn pending_limit_never_evicts_previous_checkpoints() {
        let temp = tempfile::tempdir().unwrap();
        let mut paths = Vec::new();
        for _ in 0..MAX_SESSIONS {
            let j = Journal::create(temp.path()).unwrap();
            j.save(snapshot(&j)).unwrap();
            paths.push(j.path.clone());
        }
        assert!(Journal::create(temp.path()).is_err());
        assert!(paths.iter().all(|p| p.exists()));
        let (entries, bad) = list(temp.path()).unwrap();
        assert_eq!(entries.len(), MAX_SESSIONS);
        assert_eq!(bad, 0);
        entries[0].journal.remove().unwrap();
        assert!(Journal::create(temp.path()).is_ok());
    }
}
