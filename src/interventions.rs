//! Private intervention notes and immutable references to approved executions. No replay engine.
use crate::execution::{Destination, Spec};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
const MAX: usize = 2 * 1024 * 1024;
pub const REPORT_MAX: usize = 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    Diagnosis,
    Proposal,
    Approval,
    Change,
    Verification,
    Note,
}
impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Diagnosis => "Diagnóstico",
            Self::Proposal => "Propuesta",
            Self::Approval => "Aprobación",
            Self::Change => "Cambio",
            Self::Verification => "Verificación",
            Self::Note => "Nota",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub phase: Phase,
    pub text: String,
    pub excluded: bool,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            phase: Phase::Diagnosis,
            text: String::new(),
            excluded: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub execution_id: String,
    pub phase: Phase,
    pub spec: Spec,
    pub destination: Destination,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub at: u64,
    pub phase: Phase,
    pub text: String,
    pub excluded: bool,
    pub evidence: Option<Evidence>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    pub version: u32,
    pub id: String,
    pub title: String,
    pub objective: String,
    pub targets: Vec<String>,
    pub created_at: u64,
    pub draft: Draft,
    pub entries: Vec<Entry>,
}
#[derive(Clone)]
pub struct Store {
    pub directory: PathBuf,
}
#[derive(Clone)]
pub struct Link {
    pub intervention_id: String,
    pub phase: Phase,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn text(value: &str, max: usize) -> bool {
    value.len() <= max && !value.contains('\0')
}
impl Intervention {
    pub fn validate(&self) -> Result<(), String> {
        uuid::Uuid::parse_str(&self.id).map_err(|_| "Identificador inválido")?;
        if self.version != 1
            || self.title.trim().is_empty()
            || !text(&self.title, 120)
            || self.title.chars().any(char::is_control)
            || !text(&self.objective, 16384)
            || self.objective.trim().is_empty()
            || self.targets.len() > 32
            || self.targets.iter().any(|v| !text(v, 4096))
            || !text(&self.draft.text, 4096)
            || self.entries.len() > 512
        {
            return Err("Intervención fuera de límites o versión no compatible".into());
        }
        let mut ids = std::collections::HashSet::new();
        for entry in &self.entries {
            uuid::Uuid::parse_str(&entry.id).map_err(|_| "Paso inválido")?;
            if !ids.insert(&entry.id) || !text(&entry.text, 4096) {
                return Err("Paso repetido o texto excesivo".into());
            }
            if let Some(e) = &entry.evidence {
                uuid::Uuid::parse_str(&e.execution_id).map_err(|_| "Evidencia inválida")?;
                e.spec.validate()?;
                if !e.spec.keep_history
                    || !text(&e.destination.id, 4096)
                    || !text(&e.destination.label, 4096)
                {
                    return Err("Evidencia sin historial o destino inválido".into());
                }
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX {
            return Err("Intervención superior a 2 MiB".into());
        }
        Ok(())
    }
}
struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}
impl Store {
    pub fn default_path() -> Result<Self, String> {
        Ok(Self {
            directory: crate::recovery::directory()?
                .parent()
                .ok_or("Directorio de datos inválido")?
                .join("interventions"),
        })
    }
    fn lock(&self) -> Result<Lock, String> {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.directory)
            .map_err(|e| e.to_string())?;
        if !fs::symlink_metadata(&self.directory)
            .map_err(|e| e.to_string())?
            .is_dir()
        {
            return Err("El directorio de intervenciones no es normal".into());
        }
        fs::set_permissions(&self.directory, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.directory.join(".lock"))
            .map_err(|e| e.to_string())?;
        if !file.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Bloqueo inválido".into());
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(Lock(file))
    }
    fn path(&self, id: &str) -> Result<PathBuf, String> {
        uuid::Uuid::parse_str(id).map_err(|_| "Identificador inválido")?;
        Ok(self.directory.join(format!("{id}.json")))
    }
    pub fn read(&self, id: &str) -> Result<Intervention, String> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(self.path(id)?)
            .map_err(|e| e.to_string())?;
        if !file.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Registro no regular".into());
        }
        let mut bytes = Vec::new();
        file.take((MAX + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX {
            return Err("Registro demasiado grande".into());
        }
        let value: Intervention = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        value.validate()?;
        if value.id != id {
            return Err("Identidad del registro no coincide".into());
        }
        Ok(value)
    }
    fn write(&self, value: &Intervention) -> Result<(), String> {
        value.validate()?;
        // Account for incomplete atomic writes as well as published records; never evict.
        let mut bytes = 0u64;
        for (index, entry) in fs::read_dir(&self.directory)
            .map_err(|e| e.to_string())?
            .enumerate()
        {
            if index >= 1024 {
                return Err(
                    "Demasiadas entradas en intervenciones; se conservan todos los registros"
                        .into(),
                );
            }
            let entry = entry.map_err(|e| e.to_string())?;
            let meta = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
            if !meta.is_file() {
                return Err("Entrada no regular en el directorio de intervenciones".into());
            }
            bytes = bytes.saturating_add(meta.len());
        }
        let pending = serde_json::to_vec(value).map_err(|e| e.to_string())?.len() as u64;
        if bytes.saturating_add(pending) > 65 * MAX as u64 {
            return Err(
                "Cuota de intervenciones llena (130 MiB incluidos temporales); no se borran datos"
                    .into(),
            );
        }

        let mut temp =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|e| e.to_string())?;
        temp.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        serde_json::to_writer(temp.as_file_mut(), value).map_err(|e| e.to_string())?;
        temp.flush().map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        temp.persist(self.path(&value.id)?)
            .map_err(|e| e.to_string())?;
        File::open(&self.directory)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
    pub fn create(
        &self,
        title: String,
        objective: String,
        targets: Vec<String>,
    ) -> Result<Intervention, String> {
        let _lock = self.lock()?;
        if fs::read_dir(&self.directory)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|s| s == "json"))
            .count()
            >= 64
        {
            return Err(
                "Máximo 64 intervenciones; exporta y elimina alguna antes de crear otra".into(),
            );
        }
        let value = Intervention {
            version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            title,
            objective,
            targets,
            created_at: now(),
            draft: Draft::default(),
            entries: Vec::new(),
        };
        self.write(&value)?;
        Ok(value)
    }
    fn edit(
        &self,
        id: &str,
        change: impl FnOnce(&mut Intervention) -> Result<(), String>,
    ) -> Result<Intervention, String> {
        let _lock = self.lock()?;
        let mut value = self.read(id)?;
        change(&mut value)?;
        self.write(&value)?;
        Ok(value)
    }
    pub fn list(&self) -> Result<Vec<(String, String)>, String> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.to_string()),
        };
        let mut result = Vec::new();
        for e in entries {
            let path = e.map_err(|e| e.to_string())?.path();
            if path.extension().is_none_or(|p| p != "json") {
                continue;
            }
            if result.len() >= 64 {
                return Err("Demasiados registros; no se elimina ninguno".into());
            }
            if let Some(id) = path.file_stem().and_then(|p| p.to_str()) {
                if uuid::Uuid::parse_str(id).is_ok() {
                    result.push((
                        id.into(),
                        match self.read(id) {
                            Ok(v) => v.title,
                            Err(_) => format!("No legible · {id}"),
                        },
                    ));
                }
            }
        }
        result.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(result)
    }
    pub fn draft(&self, id: &str, draft: Draft) -> Result<Intervention, String> {
        self.edit(id, |v| {
            v.draft = draft;
            Ok(())
        })
    }
    pub fn draft_if(
        &self,
        id: &str,
        draft: Draft,
        expected: Draft,
    ) -> Result<Intervention, String> {
        self.edit(id,|v|{if v.draft!=expected{return Err("El borrador cambió en otra ventana; recarga y conserva tu texto antes de sustituirlo".into());}v.draft=draft;Ok(())})
    }
    pub fn note_if(&self, id: &str, draft: Draft, expected: Draft) -> Result<Intervention, String> {
        if draft.text.trim().is_empty() {
            return Err("Escribe el paso antes de registrarlo".into());
        }
        self.edit(id, |v| {
            if v.draft != expected {
                return Err("El borrador cambió en otra ventana; el paso no se registró".into());
            }
            v.entries.push(Entry {
                id: uuid::Uuid::new_v4().to_string(),
                at: now(),
                phase: draft.phase,
                text: draft.text,
                excluded: draft.excluded,
                evidence: None,
            });
            v.draft = Draft::default();
            Ok(())
        })
    }

    pub fn note(&self, id: &str, draft: Draft) -> Result<Intervention, String> {
        if draft.text.trim().is_empty() {
            return Err("Escribe el paso antes de registrarlo".into());
        }
        self.edit(id, |v| {
            v.entries.push(Entry {
                id: uuid::Uuid::new_v4().to_string(),
                at: now(),
                phase: draft.phase,
                text: draft.text,
                excluded: draft.excluded,
                evidence: None,
            });
            Ok(())
        })
    }
    pub fn exclude(&self, id: &str, entry: &str, excluded: bool) -> Result<(), String> {
        self.edit(id, |v| {
            v.entries
                .iter_mut()
                .find(|e| e.id == entry)
                .ok_or("El paso ya no existe")?
                .excluded = excluded;
            Ok(())
        })
        .map(|_| ())
    }
    pub fn prepare(
        &self,
        link: &Link,
        spec: Spec,
        destination: Destination,
        execution_id: String,
    ) -> Result<(), String> {
        spec.validate()?;
        if !spec.keep_history {
            return Err("Vincular una intervención requiere conservar el historial de la ejecución; desvincula para excluirlo".into());
        }
        self.edit(&link.intervention_id,|v|{
  if v.entries.iter().any(|e|e.evidence.as_ref().is_some_and(|e|e.execution_id==execution_id)){return Err("La acción ya estaba registrada; no se repite".into());}
  if !v.targets.contains(&destination.label){v.targets.push(destination.label.clone());}
  let at=now();v.entries.push(Entry{id:uuid::Uuid::new_v4().to_string(),at,phase:Phase::Proposal,text:format!("Propuesta de {} para {} · acción {execution_id}",link.phase.label(),destination.label),excluded:false,evidence:None});
  v.entries.push(Entry{id:uuid::Uuid::new_v4().to_string(),at,phase:Phase::Approval,text:"Aprobación explícita de la operación y destino revisados. El inicio solo se confirma mediante la evidencia enlazada.".into(),excluded:false,evidence:Some(Evidence{execution_id,phase:link.phase,spec,destination})});Ok(())}).map(|_|())
    }
    pub fn attach(
        &self,
        id: &str,
        phase: Phase,
        execution_dir: &Path,
        execution_id: &str,
    ) -> Result<Intervention, String> {
        let r = crate::execution_history::read(execution_dir, execution_id)?;
        r.validate()?;
        if !r.spec.keep_history {
            return Err("Ejecución excluida del historial".into());
        }
        self.edit(id,|v|{v.entries.push(Entry{id:uuid::Uuid::new_v4().to_string(),at:now(),phase,text:"Evidencia existente adjuntada expresamente; no concede autorización para repetirla.".into(),excluded:false,evidence:Some(Evidence{execution_id:r.id,phase,spec:r.spec,destination:r.destination})});Ok(())})
    }
    pub fn remove(&self, id: &str) -> Result<(), String> {
        let _lock = self.lock()?;
        let path = self.path(id)?;
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !meta.is_file() {
            return Err("No se elimina un registro no regular".into());
        }
        fs::remove_file(path).map_err(|e| e.to_string())?;
        File::open(&self.directory)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Copy)]
pub struct ExportOptions {
    pub commands: bool,
    pub output: bool,
    pub json: bool,
}
pub fn report(
    value: &Intervention,
    execution_dir: &Path,
    options: ExportOptions,
) -> Result<String, String> {
    value.validate()?;
    let mut entries = Vec::new();
    let mut size = 0;
    for entry in value.entries.iter().filter(|e| !e.excluded) {
        let mut e = serde_json::json!({"id":entry.id,"at":entry.at,"phase":entry.phase.label(),"text":entry.text});
        if let Some(reference) = &entry.evidence {
            let mut data = serde_json::json!({"execution_id":reference.execution_id,"phase":reference.phase.label(),"destination":reference.destination.label});
            if options.commands {
                data["command"] = serde_json::json!(reference.spec.command);
                data["directory"] = serde_json::json!(reference.spec.directory);
            }
            match crate::execution_history::read(execution_dir, &reference.execution_id) {
                Ok(r) if r.spec == reference.spec && r.destination == reference.destination => {
                    data["status"] = serde_json::json!(format!("{:?}", r.status));
                    data["exit_code"] = serde_json::json!(r.exit_code);
                    data["duration_ms"] = serde_json::json!(r.duration_ms);
                    data["started_at"] = serde_json::json!(r.started_at);
                    data["capture_truncated"] = serde_json::json!(r.truncated);
                    if options.output {
                        data["stdout"] = serde_json::json!(String::from_utf8_lossy(
                            &r.stdout[..r.stdout.len().min(65536)]
                        ));
                        data["stderr"] = serde_json::json!(String::from_utf8_lossy(
                            &r.stderr[..r.stderr.len().min(65536)]
                        ));
                        data["report_output_truncated"] =
                            serde_json::json!(r.stdout.len() > 65536 || r.stderr.len() > 65536);
                        data["error"] = serde_json::json!(r.error);
                    }
                }
                _ => {
                    data["status"] = serde_json::json!(
                        "Inicio/resultado no confirmado: evidencia ausente, eliminada o incompatible. No se repite la operación."
                    );
                }
            }
            e["evidence"] = data;
        }
        size += serde_json::to_vec(&e).map_err(|e| e.to_string())?.len();
        if size > REPORT_MAX / 2 {
            return Err(
                "Informe demasiado grande: excluye pasos o exporta capturas individuales".into(),
            );
        }
        entries.push(e);
    }
    let report = serde_json::json!({"version":1,"intervention_id":value.id,"title":value.title,"objective":value.objective,"targets":value.targets,"created_at":value.created_at,"entries":entries});
    let text = if options.json {
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    } else {
        let mut text = format!(
            "# {}\n\nIntervención: {}\n\nObjetivo: {}\n\nDestinos: {}\n\n",
            value.title,
            value.id,
            value.objective,
            value.targets.join(", ")
        );
        for entry in report["entries"].as_array().unwrap() {
            text.push_str(&format!(
                "## {} · {}\n\n{}\n\n",
                entry["phase"].as_str().unwrap(),
                entry["at"],
                entry["text"].as_str().unwrap()
            ));
            if !entry["evidence"].is_null() {
                let evidence = serde_json::to_string_pretty(&entry["evidence"]).unwrap();
                for line in evidence.lines() {
                    text.push_str(&format!("> {line}\n"));
                }
                text.push('\n');
            }
        }
        text
    };
    if text.len() > REPORT_MAX {
        return Err("Informe superior a 8 MiB".into());
    }
    Ok(text)
}
pub fn export(path: &Path, text: &str) -> Result<(), String> {
    if text.is_empty() || text.len() > REPORT_MAX {
        return Err("Informe vacío o demasiado grande".into());
    }
    let parent = path.parent().ok_or("Destino inválido")?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.as_file()
        .set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes())
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist_noclobber(path).map_err(|e| e.to_string())?;
    File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_atomic_drafts_exclusion_and_no_recreation() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            directory: dir.path().join("interventions"),
        };
        let value = store
            .create("Prueba".into(), "Diagnosticar".into(), vec!["local".into()])
            .unwrap();
        store
            .draft(
                &value.id,
                Draft {
                    phase: Phase::Diagnosis,
                    text: "borrador privado".into(),
                    excluded: true,
                },
            )
            .unwrap();
        assert_eq!(
            store.read(&value.id).unwrap().draft.text,
            "borrador privado"
        );
        let saved = store.read(&value.id).unwrap().draft;
        let value = store.note_if(&value.id, saved.clone(), saved).unwrap();
        assert_eq!(value.entries.len(), 1);
        assert!(store.read(&value.id).unwrap().draft.text.is_empty());
        let out = report(
            &value,
            dir.path(),
            ExportOptions {
                commands: false,
                output: false,
                json: false,
            },
        )
        .unwrap();
        assert!(!out.contains("borrador privado"));
        assert_eq!(
            fs::metadata(&store.directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(store.path(&value.id).unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        store.remove(&value.id).unwrap();
        assert!(store.draft(&value.id, Draft::default()).is_err());
        assert!(store.list().unwrap().is_empty());
    }
    #[test]
    fn approval_is_durable_without_replay_and_reports_missing_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            directory: dir.path().join("i"),
        };
        let v = store
            .create("Prueba".into(), "Objetivo".into(), vec![])
            .unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let link = Link {
            intervention_id: v.id.clone(),
            phase: Phase::Verification,
        };
        let spec = Spec {
            command: "touch NEVER-RUN-BY-RESTORE".into(),
            directory: "/tmp".into(),
            timeout_secs: 1,
            keep_history: true,
        };
        let destination = crate::execution::Target::Local.destination();
        store
            .prepare(&link, spec.clone(), destination.clone(), id.clone())
            .unwrap();
        assert!(store.prepare(&link, spec, destination, id).is_err());
        let v = store.read(&v.id).unwrap();
        assert_eq!(v.entries.len(), 2);
        let public = report(
            &v,
            dir.path(),
            ExportOptions {
                commands: false,
                output: false,
                json: true,
            },
        )
        .unwrap();
        assert!(!public.contains("NEVER-RUN"));
        assert!(public.contains("no confirmado"));
        let private = report(
            &v,
            dir.path(),
            ExportOptions {
                commands: true,
                output: false,
                json: false,
            },
        )
        .unwrap();
        assert!(private.contains("NEVER-RUN"));
        let target = dir.path().join("report.md");
        export(&target, &public).unwrap();
        assert!(export(&target, "overwrite").is_err());
    }
    #[test]
    fn concurrent_notes_are_not_lost_and_corrupt_files_survive() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store {
            directory: dir.path().join("i"),
        };
        let v = store
            .create("Prueba".into(), "Objetivo".into(), vec![])
            .unwrap();
        let mut tasks = vec![];
        for n in 0..4 {
            let s = store.clone();
            let id = v.id.clone();
            tasks.push(std::thread::spawn(move || {
                for i in 0..10 {
                    s.note(
                        &id,
                        Draft {
                            phase: Phase::Note,
                            text: format!("{n}:{i}"),
                            excluded: false,
                        },
                    )
                    .unwrap();
                }
            }));
        }
        for t in tasks {
            t.join().unwrap();
        }
        assert_eq!(store.read(&v.id).unwrap().entries.len(), 40);
        fs::write(store.path(&v.id).unwrap(), "bad").unwrap();
        assert!(store.read(&v.id).is_err());
        assert_eq!(store.list().unwrap().len(), 1);
        assert_eq!(
            fs::read_to_string(store.path(&v.id).unwrap()).unwrap(),
            "bad"
        );
    }
}

#[test]
fn intervention_draft_conflict_does_not_overwrite_another_window() {
    let root = tempfile::tempdir().unwrap();
    let store = Store {
        directory: root.path().join("i"),
    };
    let v = store.create("A".into(), "Objetivo".into(), vec![]).unwrap();
    let first = Draft {
        phase: Phase::Note,
        text: "Primera ventana".into(),
        excluded: false,
    };
    store
        .draft_if(&v.id, first.clone(), Draft::default())
        .unwrap();
    assert!(
        store
            .draft_if(
                &v.id,
                Draft {
                    phase: Phase::Note,
                    text: "Segunda ventana".into(),
                    excluded: false
                },
                Draft::default()
            )
            .is_err()
    );
    assert_eq!(store.read(&v.id).unwrap().draft, first);
}

#[test]
fn intervention_quota_counts_orphans_and_never_follows_symlinks() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let store = Store {
        directory: root.path().join("i"),
    };
    let value = store.create("A".into(), "Objetivo".into(), vec![]).unwrap();
    let orphan = store.directory.join("incomplete.tmp");
    File::create(&orphan)
        .unwrap()
        .set_len(131 * 1024 * 1024)
        .unwrap();
    assert!(
        store
            .draft(
                &value.id,
                Draft {
                    phase: Phase::Note,
                    text: "No debe sustituir".into(),
                    excluded: false
                }
            )
            .is_err()
    );
    assert!(store.read(&value.id).unwrap().draft.text.is_empty());
    assert!(orphan.exists());
    fs::remove_file(orphan).unwrap();
    let outside = root.path().join("outside");
    fs::write(&outside, "outside unchanged").unwrap();
    let path = store.path(&value.id).unwrap();
    fs::remove_file(&path).unwrap();
    symlink(&outside, &path).unwrap();
    assert!(store.read(&value.id).is_err());
    assert!(store.draft(&value.id, Draft::default()).is_err());
    assert!(store.remove(&value.id).is_err());
    assert_eq!(fs::read_to_string(outside).unwrap(), "outside unchanged");
}
#[test]
fn intervention_report_rejects_evidence_from_a_different_operation() {
    let root = tempfile::tempdir().unwrap();
    let store = Store {
        directory: root.path().join("i"),
    };
    let value = store.create("A".into(), "Objetivo".into(), vec![]).unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let spec = Spec {
        command: "printf expected".into(),
        directory: "/tmp".into(),
        timeout_secs: 1,
        keep_history: true,
    };
    let destination = crate::execution::Target::Local.destination();
    store
        .prepare(
            &Link {
                intervention_id: value.id.clone(),
                phase: Phase::Diagnosis,
            },
            spec.clone(),
            destination.clone(),
            id.clone(),
        )
        .unwrap();
    let mut wrong = spec;
    wrong.command = "printf unrelated".into();
    let record = crate::execution::Record {
        version: 1,
        id: id.clone(),
        spec: wrong,
        destination,
        started_at: 1,
        duration_ms: 1,
        status: crate::execution::Status::Completed,
        exit_code: Some(0),
        stdout: b"UNRELATED-OUTPUT".to_vec(),
        stderr: vec![],
        truncated: false,
        error: String::new(),
        recording_error: String::new(),
    };
    fs::write(
        root.path().join(format!("{id}.json")),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    let text = report(
        &store.read(&value.id).unwrap(),
        root.path(),
        ExportOptions {
            commands: true,
            output: true,
            json: true,
        },
    )
    .unwrap();
    assert!(text.contains("incompatible"));
    assert!(!text.contains("UNRELATED-OUTPUT"));
}
