//! Durable editor drafts, independent of temporary SSH working copies.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

pub const TEXT_LIMIT: usize = 8 * 1024 * 1024;
// JSON escaping can expand one byte to six bytes; write without another text buffer.
const RECORD_LIMIT: u64 = (TEXT_LIMIT * 6 + 64 * 1024) as u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Source {
    Local {
        path: PathBuf,
        etag: Option<String>,
    },
    Remote {
        profile_id: String,
        host: String,
        destination: String,
        path: PathBuf,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fingerprint: Option<String>,
    },
    Detached {
        title: String,
    },
}
impl Source {
    pub fn title(&self) -> String {
        match self {
            Self::Local { path, .. } => path.display().to_string(),
            Self::Remote { host, path, .. } => format!("{} @ {host}", path.display()),
            Self::Detached { title } => title.clone(),
        }
    }
    pub fn same_document(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Local { path: a, .. }, Self::Local { path: b, .. }) => a == b,
            (
                Self::Remote {
                    profile_id: a,
                    host: ah,
                    destination: ad,
                    path: ap,
                    ..
                },
                Self::Remote {
                    profile_id: b,
                    host: bh,
                    destination: bd,
                    path: bp,
                    ..
                },
            ) => a == b && ah == bh && ad == bd && ap == bp,
            (Self::Detached { title: a }, Self::Detached { title: b }) => a == b,
            _ => false,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        fn valid_path(path: &Path) -> bool {
            path.is_absolute()
                && path.as_os_str().len() <= 8192
                && !path.as_os_str().as_encoded_bytes().contains(&0)
        }
        let metadata_valid = match self {
            Source::Local { path, etag } => {
                valid_path(path)
                    && etag
                        .as_ref()
                        .is_none_or(|s| s.len() <= 8192 && !s.contains('\0'))
            }
            Source::Remote {
                profile_id,
                host,
                destination,
                path,
                fingerprint,
            } => {
                valid_path(path)
                    && profile_id.len() <= 1024
                    && !profile_id.contains('\0')
                    && host.len() <= 1024
                    && !host.contains('\0')
                    && destination.len() == 64
                    && destination.bytes().all(|b| b.is_ascii_hexdigit())
                    && fingerprint.as_ref().is_none_or(|value| {
                        value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
                    })
            }
            Source::Detached { title } => title.len() <= 8192 && !title.contains('\0'),
        };
        if !metadata_valid {
            return Err("Origen del borrador inválido".into());
        }
        Ok(())
    }
    pub fn basename(&self) -> PathBuf {
        match self {
            Self::Local { path, .. } | Self::Remote { path, .. } => {
                path.file_name().unwrap_or_default().into()
            }
            Self::Detached { .. } => "recuperado.txt".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub id: String,
    pub source: Source,
    pub text: String,
}
impl Draft {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || uuid::Uuid::parse_str(&self.id).is_err() {
            return Err("Formato de borrador desconocido".into());
        }
        if self.text.len() > TEXT_LIMIT || self.text.contains('\0') {
            return Err("El borrador debe ser texto UTF-8 de hasta 8 MiB, sin bytes nulos".into());
        }
        self.source.validate()
    }
}

pub fn directory() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|p| PathBuf::from(p).join(".local/share"))
                .filter(|p| p.is_absolute())
        })
        .ok_or("No se encuentra el directorio de datos para recuperar documentos")?;
    Ok(base.join("nexuterm/recovery"))
}

/// Export a copy without replacing an existing destination, including the original.
pub fn export_copy(path: &Path, text: &str) -> Result<(), String> {
    if text.len() > TEXT_LIMIT || text.contains('\0') {
        return Err("La copia supera los límites del editor".into());
    }
    let parent = path.parent().ok_or("Destino sin carpeta")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .as_file()
        .set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    temporary
        .write_all(text.as_bytes())
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary
        .persist_noclobber(path)
        .map_err(|e| e.to_string())?;
    fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}

pub fn read(path: &Path) -> Result<Draft, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > RECORD_LIMIT {
        return Err("Borrador inválido o demasiado grande".into());
    }
    let draft: Draft = serde_json::from_reader(
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(RECORD_LIMIT + 1),
    )
    .map_err(|_| "Borrador dañado o formato desconocido".to_string())?;
    draft.validate()?;
    if path.file_stem().and_then(|s| s.to_str()) != Some(draft.id.as_str()) {
        return Err("Identificador de borrador incoherente".into());
    }
    Ok(draft)
}

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub source: Source,
}

pub fn list(directory: &Path) -> Result<(Vec<Entry>, usize), String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), 0)),
        Err(e) => return Err(e.to_string()),
    };
    let mut result = Vec::new();
    let mut invalid = 0;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|s| s != "json") {
            continue;
        }
        match read(&path) {
            Ok(draft) => result.push(Entry {
                id: draft.id,
                source: draft.source,
            }),
            Err(_) => invalid += 1,
        }
    }
    result.sort_by(|a, b| {
        a.source
            .title()
            .cmp(&b.source.title())
            .then(a.id.cmp(&b.id))
    });
    Ok((result, invalid))
}

/// Serialize publication and deletion; invalidation is immediate on the GTK thread.
/// A late autosave must never recreate a discarded or successfully saved draft.
pub struct Journal {
    path: PathBuf,
    revision: AtomicU64,
    retired: AtomicBool,
    lock: Mutex<()>,
}
impl Journal {
    pub fn new(directory: &Path, id: &str) -> Result<Self, String> {
        uuid::Uuid::parse_str(id).map_err(|_| "Identificador de borrador inválido")?;
        Ok(Self {
            path: directory.join(format!("{id}.json")),
            revision: AtomicU64::new(0),
            retired: AtomicBool::new(false),
            lock: Mutex::new(()),
        })
    }
    pub fn advance(&self, revision: u64) {
        self.revision.fetch_max(revision, Ordering::SeqCst);
    }
    pub fn retire(&self) {
        self.retired.store(true, Ordering::SeqCst);
    }
    pub fn resume(&self) {
        self.retired.store(false, Ordering::SeqCst);
    }
    pub fn write(&self, revision: u64, draft: &Draft) -> Result<bool, String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Error de sincronización del borrador")?;
        if self.retired.load(Ordering::SeqCst) || revision != self.revision.load(Ordering::SeqCst) {
            return Ok(false);
        }
        draft.validate()?;
        if self.path.file_stem().and_then(|s| s.to_str()) != Some(draft.id.as_str()) {
            return Err("Identificador de borrador incoherente".into());
        }
        let parent = self.path.parent().ok_or("Ruta de borrador inválida")?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)
            .map_err(|e| e.to_string())?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut temporary, draft).map_err(|e| e.to_string())?;
        temporary.flush().map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        if self.retired.load(Ordering::SeqCst) || revision != self.revision.load(Ordering::SeqCst) {
            return Ok(false);
        }
        temporary.persist(&self.path).map_err(|e| e.to_string())?;
        fs::File::open(parent)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())?;
        Ok(true)
    }
    pub fn clear(&self, revision: u64) -> Result<(), String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Error de sincronización del borrador")?;
        if !self.retired.load(Ordering::SeqCst) && revision != self.revision.load(Ordering::SeqCst)
        {
            return Ok(());
        }
        match fs::remove_file(&self.path) {
            Ok(()) => fs::File::open(self.path.parent().unwrap())
                .and_then(|f| f.sync_all())
                .map_err(|e| e.to_string()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draft() -> Draft {
        Draft {
            version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            source: Source::Local {
                path: "/tmp/original.txt".into(),
                etag: Some("version-1".into()),
            },
            text: "Cambios áé\n".into(),
        }
    }
    #[test]
    fn atomic_private_roundtrip_without_touching_original() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("recovery");
        let draft = draft();
        let journal = Journal::new(&dir, &draft.id).unwrap();
        journal.advance(1);
        assert!(journal.write(1, &draft).unwrap());
        assert_eq!(read(&journal.path).unwrap().text, draft.text);
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&journal.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        assert_eq!(list(&dir).unwrap().0.len(), 1);
        journal.advance(2);
        journal.clear(2).unwrap();
        assert!(list(&dir).unwrap().0.is_empty());
    }
    #[test]
    fn stale_writes_and_discards_cannot_resurrect_a_draft() {
        let root = tempfile::tempdir().unwrap();
        let draft = draft();
        let journal = Journal::new(root.path(), &draft.id).unwrap();
        journal.advance(2);
        assert!(!journal.write(1, &draft).unwrap());
        assert!(journal.write(2, &draft).unwrap());
        journal.advance(3);
        journal.clear(3).unwrap();
        assert!(!journal.write(2, &draft).unwrap());
        journal.advance(4);
        assert!(journal.write(4, &draft).unwrap());
        journal.retire();
        journal.clear(4).unwrap();
        assert!(!journal.write(4, &draft).unwrap());
        assert!(!journal.path.exists());
    }
    #[test]
    fn malformed_versions_symlinks_and_limits_are_preserved() {
        let root = tempfile::tempdir().unwrap();
        let mut draft = draft();
        let path = root.path().join(format!("{}.json", draft.id));
        fs::write(&path, b"broken").unwrap();
        assert_eq!(list(root.path()).unwrap().1, 1);
        assert_eq!(fs::read(&path).unwrap(), b"broken");
        draft.version = 99;
        assert!(draft.validate().is_err());
        draft.version = 1;
        draft.text = "x".repeat(TEXT_LIMIT + 1);
        assert!(draft.validate().is_err());
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink("/dev/null", &path).unwrap();
        assert!(read(&path).is_err());
        assert!(Journal::new(root.path(), "../../escape").is_err());
    }
    #[test]
    fn export_never_replaces_an_existing_file_and_invalid_update_keeps_draft() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("copy.txt");
        export_copy(&path, "original").unwrap();
        assert!(export_copy(&path, "replacement").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let mut draft = draft();
        let journal = Journal::new(root.path(), &draft.id).unwrap();
        journal.advance(1);
        journal.write(1, &draft).unwrap();
        draft.text = "x".repeat(TEXT_LIMIT + 1);
        journal.advance(2);
        assert!(journal.write(2, &draft).is_err());
        assert_eq!(read(&journal.path).unwrap().text, "Cambios áé\n");
        draft.text = "valid".into();
        draft.source = Source::Local {
            path: "/tmp/\0invalid".into(),
            etag: None,
        };
        assert!(draft.validate().is_err());
    }
    #[test]
    fn document_identity_allows_version_updates_but_not_destination_changes() {
        let local = Source::Local {
            path: "/tmp/original.txt".into(),
            etag: Some("old".into()),
        };
        let mut newer = local.clone();
        if let Source::Local { etag, .. } = &mut newer {
            *etag = Some("new".into());
        }
        assert!(local.same_document(&newer));
        let remote = Source::Remote {
            profile_id: "profile".into(),
            host: "fixture.invalid".into(),
            destination: "0".repeat(64),
            path: "/tmp/original.txt".into(),
            fingerprint: Some("1".repeat(64)),
        };
        assert!(!local.same_document(&remote));
        let mut updated = remote.clone();
        if let Source::Remote { fingerprint, .. } = &mut updated {
            *fingerprint = Some("2".repeat(64));
        }
        assert!(remote.same_document(&updated));
        if let Source::Remote { destination, .. } = &mut updated {
            *destination = "3".repeat(64);
        }
        assert!(!remote.same_document(&updated));
    }
}
