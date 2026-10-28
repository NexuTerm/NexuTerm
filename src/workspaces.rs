//! Private, versioned workspace snapshots. Restoring a snapshot never executes it.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
};

const LIMIT: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Target {
    Local {
        directory: Option<PathBuf>,
    },
    Connection {
        profile_id: String,
        #[serde(default)]
        directory: Option<PathBuf>,
    },
    Unavailable {
        description: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Layout {
    Document {
        id: String,
        source: crate::recovery::Source,
        had_draft: bool,
    },
    Terminal {
        target: Target,
    },
    Split {
        vertical: bool,
        ratio: f64,
        first: Box<Layout>,
        second: Box<Layout>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tab {
    pub title: String,
    pub layout: Layout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub notes: String,
    pub tabs: Vec<Tab>,
    pub selected: usize,
    pub directory: Option<PathBuf>,
    pub sidebar_visible: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    pub version: u32,
    pub workspaces: Vec<Workspace>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: 1,
            workspaces: Vec::new(),
        }
    }
}

impl Layout {
    fn validate(&self, depth: usize, leaves: &mut usize) -> Result<(), String> {
        if depth > 32 {
            return Err("Demasiadas divisiones anidadas".into());
        }
        match self {
            Self::Document { id, source, .. } => {
                if depth != 0 {
                    return Err("Los documentos solo se restauran en pestañas completas".into());
                }
                uuid::Uuid::parse_str(id).map_err(|_| "Identificador de documento inválido")?;
                source.validate()?;
                *leaves += 1;
                if *leaves > 128 {
                    return Err("Máximo 128 paneles por espacio".into());
                }
            }
            Self::Terminal { target } => {
                *leaves += 1;
                if *leaves > 128 {
                    return Err("Máximo 128 paneles por espacio".into());
                }
                if let Target::Local {
                    directory: Some(path),
                } = target
                    && !path.is_absolute()
                {
                    return Err("La carpeta local debe ser absoluta".into());
                }
            }
            Self::Split {
                ratio,
                first,
                second,
                ..
            } => {
                if !ratio.is_finite() || !(0.05..=0.95).contains(ratio) {
                    return Err("Proporción de división inválida".into());
                }
                first.validate(depth + 1, leaves)?;
                second.validate(depth + 1, leaves)?;
            }
        }
        Ok(())
    }
}

impl Store {
    pub fn path() -> Result<PathBuf, String> {
        let base = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
            .ok_or("No se encuentra el directorio de datos")?;
        Ok(base.join("nexuterm/workspaces/workspaces.json"))
    }

    fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Versión de espacios desconocida; no se ha modificado el archivo".into());
        }
        if self.workspaces.len() > 100 {
            return Err("Máximo 100 espacios de trabajo".into());
        }
        let mut ids = std::collections::HashSet::new();
        for workspace in &self.workspaces {
            if workspace.id.is_empty() || !ids.insert(&workspace.id) {
                return Err("Identificador de espacio inválido o duplicado".into());
            }
            if workspace.name.trim().is_empty()
                || workspace.name.len() > 120
                || workspace.name.chars().any(char::is_control)
            {
                return Err(
                    "El nombre debe tener entre 1 y 120 bytes, sin caracteres de control".into(),
                );
            }
            if workspace.tabs.len() > 128
                || (!workspace.tabs.is_empty() && workspace.selected >= workspace.tabs.len())
            {
                return Err("Selección o número de pestañas inválido".into());
            }
            if workspace
                .directory
                .as_ref()
                .is_some_and(|p| !p.is_absolute())
            {
                return Err("Carpeta del explorador inválida".into());
            }
            let mut leaves = 0;
            let mut documents = std::collections::HashSet::new();
            for tab in &workspace.tabs {
                tab.layout.validate(0, &mut leaves)?;
                if let Layout::Document { id, .. } = &tab.layout
                    && !documents.insert(id)
                {
                    return Err(
                        "Un documento no puede ocupar dos pestañas del mismo espacio".into(),
                    );
                }
            }
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(error.to_string()),
        };
        let mut bytes = Vec::new();
        file.take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > LIMIT {
            return Err("Archivo de espacios demasiado grande".into());
        }
        let store: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        store.validate()?;
        Ok(store)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > LIMIT {
            return Err("Archivo de espacios demasiado grande".into());
        }
        // Do not overwrite an unknown or damaged existing format.
        Self::load(path)?;
        let parent = path.parent().ok_or("Ruta de espacios inválida")?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)
            .map_err(|e| e.to_string())?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        file.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist(path).map_err(|e| e.to_string())?;
        fs::File::open(parent)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn workspace() -> Workspace {
        Workspace {
            id: "fixture".into(),
            name: "Cliente de prueba".into(),
            notes: "Nota".into(),
            tabs: vec![Tab {
                title: "Terminal".into(),
                layout: Layout::Terminal {
                    target: Target::Local {
                        directory: Some("/tmp".into()),
                    },
                },
            }],
            selected: 0,
            directory: Some("/tmp".into()),
            sidebar_visible: true,
        }
    }
    #[test]
    fn private_roundtrip_and_replace() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("private/workspaces.json");
        let mut store = Store::load(&path).unwrap();
        store.workspaces.push(workspace());
        store.save(&path).unwrap();
        assert_eq!(Store::load(&path).unwrap().workspaces, store.workspaces);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        store.workspaces[0].notes = "Actualizada".into();
        store.save(&path).unwrap();
        assert_eq!(
            Store::load(&path).unwrap().workspaces[0].notes,
            "Actualizada"
        );
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }
    #[test]
    fn unknown_or_corrupt_file_is_preserved() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("workspaces.json");
        for bytes in [br#"{"version":99,"workspaces":[]}"#.as_slice(), b"invalid"] {
            fs::write(&path, bytes).unwrap();
            assert!(Store::load(&path).is_err());
            assert!(Store::default().save(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    #[test]
    fn rejects_invalid_layout_selection_and_duplicates() {
        let mut store = Store {
            version: 1,
            workspaces: vec![workspace()],
        };
        store.workspaces[0].selected = 1;
        assert!(store.validate().is_err());
        store.workspaces[0].selected = 0;
        store.workspaces.push(workspace());
        assert!(store.validate().is_err());
        store.workspaces.pop();
        let leaf = store.workspaces[0].tabs[0].layout.clone();
        store.workspaces[0].tabs[0].layout = Layout::Split {
            vertical: true,
            ratio: 0.0,
            first: Box::new(leaf.clone()),
            second: Box::new(leaf),
        };
        assert!(store.validate().is_err());
    }
    #[test]
    fn document_references_roundtrip_and_reject_unsafe_ids_and_nesting() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("workspaces.json");
        let mut workspace = workspace();
        let document = Layout::Document {
            id: uuid::Uuid::new_v4().to_string(),
            source: crate::recovery::Source::Local {
                path: "/tmp/document.txt".into(),
                etag: Some("baseline".into()),
            },
            had_draft: true,
        };
        workspace.tabs.push(Tab {
            title: "Documento".into(),
            layout: document.clone(),
        });
        let mut store = Store {
            version: 1,
            workspaces: vec![workspace],
        };
        store.save(&path).unwrap();
        assert_eq!(Store::load(&path).unwrap().workspaces, store.workspaces);
        let original = fs::read(&path).unwrap();
        let duplicate = store.workspaces[0].tabs[1].clone();
        store.workspaces[0].tabs.push(duplicate);
        assert!(store.save(&path).is_err());
        store.workspaces[0].tabs.pop();
        if let Layout::Document { id, .. } = &mut store.workspaces[0].tabs[1].layout {
            *id = "../../escape".into();
        }
        assert!(store.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        store.workspaces[0].tabs[1].layout = Layout::Split {
            vertical: false,
            ratio: 0.5,
            first: Box::new(document.clone()),
            second: Box::new(document),
        };
        assert!(store.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}
