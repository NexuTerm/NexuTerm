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
        browser: Option<BrowserView>,
        #[serde(default)]
        directory: Option<PathBuf>,
    },
    Unavailable {
        description: String,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EditorView {
    pub cursor: u32,
    pub anchor: u32,
    pub vertical: f64,
    pub horizontal: f64,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrowserView {
    pub hidden: bool,
    pub selected: Option<PathBuf>,
    pub vertical: f64,
    pub horizontal: f64,
}
fn valid_scroll(vertical: f64, horizontal: f64) -> bool {
    [vertical, horizontal]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
}
impl EditorView {
    fn validate(&self) -> Result<(), String> {
        if self.cursor > 8 * 1024 * 1024
            || self.anchor > 8 * 1024 * 1024
            || !valid_scroll(self.vertical, self.horizontal)
        {
            return Err("Estado del editor inválido".into());
        }
        Ok(())
    }
}
impl BrowserView {
    fn validate(&self) -> Result<(), String> {
        if !valid_scroll(self.vertical, self.horizontal)
            || self.selected.as_ref().is_some_and(|p| {
                p.as_os_str().len() > 4096
                    || p.components().count() != 1
                    || !matches!(p.components().next(), Some(std::path::Component::Normal(_)))
                    || p.to_string_lossy().chars().any(char::is_control)
            })
        {
            return Err("Estado del explorador inválido".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TerminalView {
    pub zoom: f64,
    pub focused: bool,
}
impl Default for TerminalView {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Layout {
    Desktop {
        profile_id: String,
        protocol: crate::config::Protocol,
    },
    FileBrowser {
        profile_id: String,
        #[serde(default)]
        view: Option<BrowserView>,
        protocol: crate::config::Protocol,
        directory: Option<PathBuf>,
    },
    Document {
        id: String,
        source: crate::recovery::Source,
        had_draft: bool,
        #[serde(default)]
        view: Option<EditorView>,
    },
    Terminal {
        target: Target,
        #[serde(default)]
        view: TerminalView,
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

/// A bounded, offline copy of the visible conversation, never executable permissions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessage {
    pub title: String,
    pub text: String,
    pub tool: bool,
    pub response: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub agent_id: Option<String>,
    pub model: String,
    pub directory: Option<PathBuf>,
    pub codex_thread: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub draft: String,
    #[serde(default)]
    pub attachments: Vec<PathBuf>,
    #[serde(default)]
    pub reasoning: String,
    #[serde(default)]
    pub continuation: Option<crate::chat_state::Continuation>,
    #[serde(default)]
    pub archive: Option<String>,
    #[serde(default = "capture_history_default")]
    pub capture_history: bool,
    #[serde(default)]
    pub scroll: Option<f64>,
}
fn capture_history_default() -> bool {
    true
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Chats {
    pub tabs: Vec<Chat>,
    pub selected: usize,
    pub visible: bool,
}
impl Chats {
    pub fn validate(&self) -> Result<(), String> {
        if self.tabs.len() > 8 || (!self.tabs.is_empty() && self.selected >= self.tabs.len()) {
            return Err("Selección o número de chats inválido".into());
        }
        let mut ids = std::collections::HashSet::new();
        for chat in &self.tabs {
            if let Some(continuation) = &chat.continuation {
                continuation.validate()?;
            }
            if let Some(id) = &chat.archive {
                uuid::Uuid::parse_str(id).map_err(|_| "Referencia de historial inválida")?;
                if id != &chat.id {
                    return Err("El historial no corresponde al chat".into());
                }
            }
            if chat
                .scroll
                .is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
            {
                return Err("Posición del chat inválida".into());
            }
            if chat.attachments.len() > 20
                || chat
                    .attachments
                    .iter()
                    .any(|p| !p.is_absolute() || p.as_os_str().len() > 4096)
                || chat.reasoning.len() > 128
            {
                return Err("Adjuntos o razonamiento inválidos".into());
            }
            uuid::Uuid::parse_str(&chat.id).map_err(|_| "Identificador de chat inválido")?;
            if !ids.insert(&chat.id)
                || chat.messages.len() > 128
                || chat.draft.len() > 65536
                || chat.title.len() > 1024
                || chat.model.len() > 4096
                || chat.agent_id.as_ref().is_some_and(|v| v.len() > 4096)
                || chat.codex_thread.as_ref().is_some_and(|v| {
                    v.is_empty() || v.len() > 4096 || v.chars().any(char::is_control)
                })
                || chat.directory.as_ref().is_some_and(|p| !p.is_absolute())
                || chat
                    .messages
                    .iter()
                    .any(|m| m.title.len() > 4096 || m.text.len() > 262144)
            {
                return Err("Contenido de chat inválido o demasiado grande".into());
            }
        }
        Ok(())
    }
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
    #[serde(default)]
    pub browser: Option<BrowserView>,
    #[serde(default)]
    pub chats: Chats,
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
            Self::Desktop {
                profile_id,
                protocol,
            } => {
                if depth != 0
                    || profile_id.is_empty()
                    || profile_id.len() > 4096
                    || !matches!(
                        protocol,
                        crate::config::Protocol::Rdp | crate::config::Protocol::Vnc
                    )
                {
                    return Err("Referencia de escritorio remoto inválida".into());
                }
                *leaves += 1;
                if *leaves > 128 {
                    return Err("Máximo 128 paneles por espacio".into());
                }
            }
            Self::FileBrowser {
                profile_id,
                protocol,
                directory,
                view,
            } => {
                if let Some(view) = view {
                    view.validate()?;
                }
                if depth != 0
                    || profile_id.is_empty()
                    || profile_id.len() > 4096
                    || !matches!(
                        protocol,
                        crate::config::Protocol::Ftp | crate::config::Protocol::Sftp
                    )
                    || directory.as_ref().is_some_and(|p| {
                        !p.is_absolute()
                            || p.as_os_str().len() > 4096
                            || p.to_string_lossy().chars().any(char::is_control)
                    })
                {
                    return Err("Referencia de explorador remoto inválida".into());
                }
                *leaves += 1;
                if *leaves > 128 {
                    return Err("Máximo 128 paneles por espacio".into());
                }
            }
            Self::Document {
                id, source, view, ..
            } => {
                if let Some(view) = view {
                    view.validate()?;
                }
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
            Self::Terminal { target, view } => {
                if let Target::Connection {
                    browser: Some(browser),
                    ..
                } = target
                {
                    browser.validate()?;
                }
                if !view.zoom.is_finite() || !(0.5..=3.0).contains(&view.zoom) {
                    return Err("Zoom de terminal inválido".into());
                }
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
            workspace.chats.validate()?;
            if let Some(view) = &workspace.browser {
                view.validate()?;
            }
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
    #[test]
    fn remote_browser_references_roundtrip_and_reject_invalid_layouts() {
        for protocol in [crate::config::Protocol::Ftp, crate::config::Protocol::Sftp] {
            let layout = Layout::FileBrowser {
                view: None,
                profile_id: "saved-profile".into(),
                protocol,
                directory: Some(PathBuf::from("/data/project with spaces")),
            };
            let encoded = serde_json::to_vec(&layout).unwrap();
            let decoded: Layout = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(layout, decoded);
            decoded.validate(0, &mut 0).unwrap();
            assert!(decoded.validate(1, &mut 0).is_err());
        }
        for (protocol, path) in [
            (crate::config::Protocol::Ssh, "/valid"),
            (crate::config::Protocol::Ftp, "relative"),
            (crate::config::Protocol::Sftp, "/bad\npath"),
        ] {
            assert!(
                Layout::FileBrowser {
                    view: None,
                    profile_id: "profile".into(),
                    protocol,
                    directory: Some(PathBuf::from(path))
                }
                .validate(0, &mut 0)
                .is_err()
            );
        }
    }
    #[test]
    fn desktop_references_are_offline_and_validated() {
        for protocol in [crate::config::Protocol::Rdp, crate::config::Protocol::Vnc] {
            let layout = Layout::Desktop {
                profile_id: "desktop-profile".into(),
                protocol,
            };
            let text = serde_json::to_string(&layout).unwrap();
            assert_eq!(serde_json::from_str::<Layout>(&text).unwrap(), layout);
            layout.validate(0, &mut 0).unwrap();
            assert!(layout.validate(1, &mut 0).is_err());
            assert!(layout.validate(0, &mut 128).is_err());
            assert!(!text.contains("password"));
        }
        for (profile_id, protocol) in [
            ("", crate::config::Protocol::Rdp),
            ("profile", crate::config::Protocol::Ssh),
        ] {
            assert!(
                Layout::Desktop {
                    profile_id: profile_id.into(),
                    protocol
                }
                .validate(0, &mut 0)
                .is_err()
            );
        }
    }
    #[test]
    fn visual_defaults_and_limits() {
        let old: Layout = serde_json::from_str(
            r#"{"kind":"Terminal","target":{"kind":"Local","directory":null}}"#,
        )
        .unwrap();
        assert!(matches!(
            old,
            Layout::Terminal {
                view: TerminalView {
                    zoom: 1.0,
                    focused: false
                },
                ..
            }
        ));
        for zoom in [0.49, 3.01, f64::NAN] {
            assert!(
                Layout::Terminal {
                    target: Target::Local { directory: None },
                    view: TerminalView {
                        zoom,
                        focused: true
                    }
                }
                .validate(0, &mut 0)
                .is_err()
            );
        }
        let mut w = workspace();
        let chat:Chat=serde_json::from_value(serde_json::json!({"id":uuid::Uuid::new_v4().to_string(),"title":"fixture","agent_id":null,"model":"","directory":null,"codex_thread":null,"messages":[],"draft":""})).unwrap();
        assert_eq!(chat.scroll, None);
        w.chats.tabs.push(chat);
        for scroll in [Some(-0.1), Some(1.1), Some(f64::NAN)] {
            w.chats.tabs[0].scroll = scroll;
            assert!(w.chats.validate().is_err());
        }
        for scroll in [None, Some(0.0), Some(0.35), Some(1.0)] {
            w.chats.tabs[0].scroll = scroll;
            w.chats.validate().unwrap();
        }
    }
    #[test]
    fn editor_browser_views_defaults_limits_and_paths() {
        let view: BrowserView = serde_json::from_str("{}").unwrap();
        assert_eq!(view, BrowserView::default());
        for name in ["../other", "/absolute", "nested/file", ".", "bad\nname", ""] {
            assert!(
                BrowserView {
                    selected: Some(name.into()),
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
        for value in [-0.1, 1.1, f64::NAN] {
            assert!(
                BrowserView {
                    vertical: value,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
            assert!(
                EditorView {
                    horizontal: value,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            EditorView {
                cursor: 8 * 1024 * 1024 + 1,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        let mut w = workspace();
        w.browser = Some(BrowserView {
            hidden: true,
            selected: Some(".á.txt".into()),
            vertical: 0.4,
            horizontal: 0.2,
        });
        w.tabs[0].layout = Layout::Document {
            id: uuid::Uuid::new_v4().to_string(),
            source: crate::recovery::Source::Local {
                path: "/tmp/fixture".into(),
                etag: None,
            },
            had_draft: false,
            view: Some(EditorView {
                cursor: 80,
                anchor: 15,
                vertical: 0.45,
                horizontal: 0.0,
            }),
        };
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("workspace.json");
        let store = Store {
            version: 1,
            workspaces: vec![w],
        };
        store.save(&path).unwrap();
        assert_eq!(Store::load(&path).unwrap().workspaces, store.workspaces);
        let legacy:Layout=serde_json::from_value(serde_json::json!({"kind":"Document","id":uuid::Uuid::new_v4().to_string(),"source":{"kind":"Local","path":"/tmp/fixture","etag":null},"had_draft":false})).unwrap();
        assert!(matches!(legacy, Layout::Document { view: None, .. }));
    }
    fn workspace() -> Workspace {
        Workspace {
            id: "fixture".into(),
            name: "Cliente de prueba".into(),
            notes: "Nota".into(),
            tabs: vec![Tab {
                title: "Terminal".into(),
                layout: Layout::Terminal {
                    view: Default::default(),
                    target: Target::Local {
                        directory: Some("/tmp".into()),
                    },
                },
            }],
            selected: 0,
            directory: Some("/tmp".into()),
            sidebar_visible: true,
            browser: None,
            chats: Chats::default(),
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
            view: None,
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
    #[test]
    fn chats_roundtrip_limits_and_legacy() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("workspaces.json");
        let mut w = workspace();
        w.chats = Chats {
            tabs: vec![Chat {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Prueba".into(),
                agent_id: Some("fixture".into()),
                model: "modelo".into(),
                directory: Some("/tmp".into()),
                codex_thread: Some("thread".into()),
                messages: vec![ChatMessage {
                    title: "Tú".into(),
                    text: "Texto áé\n```rust\n```".into(),
                    tool: false,
                    response: false,
                }],
                draft: "Sin enviar".into(),
                attachments: vec![],
                reasoning: String::new(),
                continuation: None,
                archive: None,
                capture_history: true,
                scroll: None,
            }],
            selected: 0,
            visible: true,
        };
        let mut store = Store {
            version: 1,
            workspaces: vec![w],
        };
        store.save(&path).unwrap();
        assert_eq!(Store::load(&path).unwrap().workspaces, store.workspaces);
        let original = fs::read(&path).unwrap();
        let chat = store.workspaces[0].chats.tabs[0].clone();
        store.workspaces[0].chats.tabs.push(chat);
        assert!(store.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        store.workspaces[0].chats.tabs.pop();
        store.workspaces[0].chats.tabs[0].draft = "x".repeat(65537);
        assert!(store.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        store.workspaces[0].chats.tabs[0].draft.clear();
        store.workspaces[0].chats.selected = 1;
        assert!(store.save(&path).is_err());
        store.workspaces[0].chats.selected = 0;
        store.workspaces[0].chats.tabs[0].directory = Some("relative".into());
        assert!(store.save(&path).is_err());
        let mut legacy: serde_json::Value = serde_json::from_slice(&original).unwrap();
        legacy["workspaces"][0]
            .as_object_mut()
            .unwrap()
            .remove("chats");
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        assert_eq!(
            Store::load(&path).unwrap().workspaces[0].chats,
            Chats::default()
        );
    }
}
