use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

const MAX_CONFIG: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    #[default]
    Ssh,
    Rdp,
    Vnc,
    Telnet,
    Ftp,
    Sftp,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity: String,
    pub ssh_password: bool,
    pub jump_host: String,
    pub domain: String,
    pub group: String,
    pub favorite: bool,
    pub secret_ref: Option<String>,
    pub follow_folder: bool,
    pub monitor: bool,
    pub certificate_authority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub dictation_program: String,
    pub dictation_model: String,
    pub dictation_language: String,
    pub theme: String,
    pub font: String,
    pub color_scheme: String,
    pub scrollback: u32,
    pub sidebar_visible: bool,
    pub files_use_system_app: bool,
    pub browser_show_hidden: bool,
    pub browser_show_type: bool,
    pub browser_show_size: bool,
    pub browser_show_permissions: bool,
    pub editor_line_numbers: bool,
    pub monitor_interval: u64,
    pub shortcuts: std::collections::BTreeMap<String, String>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            dictation_program: "whisper-cli".into(),
            dictation_model: String::new(),
            dictation_language: "es".into(),
            theme: "system".into(),
            font: "Monospace 11".into(),
            color_scheme: "dark".into(),
            scrollback: 10000,
            sidebar_visible: true,
            files_use_system_app: false,
            browser_show_hidden: false,
            browser_show_type: true,
            browser_show_size: true,
            browser_show_permissions: true,
            editor_line_numbers: true,
            monitor_interval: 5,
            shortcuts: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tunnel {
    pub id: String,
    pub profile_id: String,
    pub name: String,
    pub kind: String,
    pub bind_host: String,
    pub bind_port: u16,
    pub destination_host: String,
    pub destination_port: u16,
}

/// AI credentials are stored separately; this schema contains only a reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AiAgent {
    pub id: String,
    pub name: String,
    pub kind: crate::ai_config::AiKind,
    pub program: String,
    pub args: Vec<String>,
    pub directory: String,
    pub endpoint: String,
    pub model: String,
    pub models: Vec<String>,
    pub credential_env:String,
    #[serde(skip)]
    pub wait_credentials:bool,
    pub terminal_access: bool,
    #[serde(skip)]
    pub terminal_socket: Option<String>,
    #[serde(skip)]
    pub runtime_root:Option<PathBuf>,
    pub system_prompt: String,
    pub max_output_tokens: u32,
    pub timeout_seconds: u32,
    pub secret_ref: Option<String>,
    #[serde(rename="extensions", alias="opencode")]
    pub opencode: crate::ai_config::OpenCodeConfig,
}
impl Default for AiAgent {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            kind: Default::default(),
            program: String::new(),
            args: Vec::new(),
            directory: String::new(),
            endpoint: String::new(),
            model: String::new(),
            models: Vec::new(),
            credential_env:String::new(),
            wait_credentials:false,
            terminal_access: true,
            terminal_socket: None,
            runtime_root:None,
            system_prompt: String::new(),
            max_output_tokens: 2048,
            timeout_seconds: 120,
            secret_ref: None,
            opencode: Default::default(),
        }
    }
}
impl AiAgent {
    pub fn is_codex(&self) -> bool {
        self.kind == crate::ai_config::AiKind::Codex
            || (self.kind == crate::ai_config::AiKind::Local
                && Path::new(&self.program).file_name()
                    .is_some_and(|name| name == "codex" || name == "codex.exe"))
    }

    pub fn credential_key(&self) -> (String, String) {
        (
            self.id.clone(),
            if self.kind.is_api(){self.endpoint.trim_end_matches('/').to_owned()}else{format!("native:{}",self.credential_env)},
        )
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["","OPENAI_API_KEY","ANTHROPIC_API_KEY","GEMINI_API_KEY","DEEPSEEK_API_KEY","OPENCODE_API_KEY","COPILOT_GITHUB_TOKEN","GH_TOKEN"].contains(&self.credential_env.as_str()){return Err("Variable de credencial no admitida".into());}
        if self.id.is_empty() || self.name.trim().is_empty() {
            return Err("Indica un nombre para el agente".into());
        }
        if self.models.len()>128 || self.models.iter().any(|m|m.trim().is_empty()||m.len()>256||m.chars().any(char::is_control)){return Err("Lista de modelos inválida (máximo 128)".into());}
        if self.kind.is_api() {
            crate::ai_config::validate_url(&self.endpoint)?;
            if self.model.len() > 256
                || self.model.chars().any(|c| c.is_control())
            {
                return Err("Indica un modelo válido".into());
            }
            if !(1..=32768).contains(&self.max_output_tokens)
                || !(5..=600).contains(&self.timeout_seconds)
                || self.system_prompt.len() > 32768
            {
                return Err("Límites de API fuera de rango".into());
            }
        } else {
            if self.program.trim().is_empty()
                || self.program.starts_with('-')
                || self.program.contains('\0')
                || self.directory.contains('\0')
                || self.args.iter().any(|a| a.contains('\0'))
                || self.model.contains('\0')
            {
                return Err("Ejecutable, argumentos o carpeta inválidos".into());
            }
            if !self.directory.is_empty() && !Path::new(&self.directory).is_absolute() {
                return Err("La carpeta de trabajo debe ser una ruta absoluta".into());
            }
        }
        self.opencode.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub ai_agents: Vec<AiAgent>,
    pub version: u32,
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub preferences: Preferences,
    #[serde(default)]
    pub tunnels: Vec<Tunnel>,
    #[serde(default)]
    pub history: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 2,
            ai_agents: Vec::new(),
            profiles: Vec::new(),
            preferences: Preferences::default(),
            tunnels: Vec::new(),
            history: Vec::new(),
        }
    }
}

pub fn config_path() -> Result<PathBuf, String> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .ok_or("No se encuentra HOME ni XDG_CONFIG_HOME absoluto")?;
    Ok(base.join("nexuterm/sessions.json"))
}

pub fn load(path: &Path) -> Result<Config, String> {
    let legacy = path.parent().and_then(|parent| {
        (parent.file_name().is_some_and(|name| name == "nexuterm"))
            .then(|| {
                parent
                    .parent()
                    .map(|base| base.join("tuxterm/sessions.json"))
            })
            .flatten()
    });
    let importing_legacy = !path.exists() && legacy.as_ref().is_some_and(|p| p.is_file());
    let source = if importing_legacy {
        legacy.as_deref().unwrap()
    } else {
        path
    };
    let file = match fs::File::open(source) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(MAX_CONFIG + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_CONFIG {
        return Err("Configuración demasiado grande".into());
    }
    let mut config: Config = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if config.version != 1 && config.version != 2 {
        return Err("Versión de configuración no compatible".into());
    }
    for profile in &config.profiles {
        crate::connection::validate(profile)?;
    }
    let mut agent_ids = std::collections::HashSet::new();
    for agent in &config.ai_agents {
        agent.validate()?;
        if !agent_ids.insert(&agent.id) {
            return Err("Identificador de agente duplicado".into());
        }
    }
    let needs_upgrade = config.version == 1;
    let mut ids = std::collections::HashSet::new();
    for p in &mut config.profiles {
        if p.id.is_empty() {
            p.id = uuid::Uuid::new_v4().to_string();
        }
        if !ids.insert(p.id.clone()) {
            return Err("Identificador de sesión duplicado".into());
        }
    }
    if !(1..=3600).contains(&config.preferences.monitor_interval)
        || config.preferences.scrollback > 1000000
    {
        return Err("Preferencias fuera de rango".into());
    }
    if needs_upgrade || importing_legacy {
        fs::create_dir_all(path.parent().ok_or("Ruta de configuración inválida")?)
            .map_err(|e| e.to_string())?;
        let backup = path.with_extension(if importing_legacy {
            "json.tuxterm.backup"
        } else {
            "json.v1.backup"
        });
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&backup)
        {
            Ok(mut out) => out
                .write_all(&bytes)
                .and_then(|_| out.sync_all())
                .map_err(|e| e.to_string())?,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                if fs::read(&backup).map_err(|e| e.to_string())? != bytes {
                    return Err(
                        "La copia previa es distinta; se conservan ambos archivos sin migrar"
                            .into(),
                    );
                }
            }
            Err(e) => {
                return Err(format!(
                    "No se creó la copia de migración {}: {e}",
                    backup.display()
                ));
            }
        }
        config.version = 2;
        save(path, &config)?;
    }
    Ok(config)
}

pub fn save(path: &Path, config: &Config) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(config).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_CONFIG {
        return Err("Configuración demasiado grande".into());
    }
    let parent = path.parent().ok_or("Ruta de configuración inválida")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = parent.join(format!(".sessions-{}.tmp", std::process::id()));
    let result = (|| -> io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        fs::File::open(parent)?.sync_all()
    })();
    // Never remove a pre-existing temporary file that this process did not create.
    if let Err(e) = result {
        return Err(format!(
            "No se guardó la configuración: {e}. Temporal: {}",
            temporary.display()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn explorer_preferences_are_backward_compatible_and_persist() {
        let mut preferences: super::Preferences = serde_json::from_str("{}").unwrap();
        assert!(!preferences.files_use_system_app);
        assert!(preferences.editor_line_numbers);
        assert!(!preferences.browser_show_hidden);
        assert!(preferences.browser_show_type && preferences.browser_show_size && preferences.browser_show_permissions);
        preferences.browser_show_hidden = true;
        preferences.browser_show_type = false;
        preferences.browser_show_size = false;
        preferences.browser_show_permissions = false;
        preferences.files_use_system_app = true;
        preferences.editor_line_numbers = false;
        let json = serde_json::to_string(&preferences).unwrap();
        let restored: super::Preferences = serde_json::from_str(&json).unwrap();
        assert!(restored.files_use_system_app);
        assert!(!restored.editor_line_numbers);
        assert!(restored.browser_show_hidden);
        assert!(!restored.browser_show_type && !restored.browser_show_size && !restored.browser_show_permissions);
    }


    use super::*;
    #[test]
    fn ai_agents_are_optional_and_roundtrip() {
        let legacy: Config = serde_json::from_str(r#"{"version":2,"profiles":[]}"#).unwrap();
        assert!(legacy.ai_agents.is_empty());
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sessions.json");
        let mut config = legacy;
        config.ai_agents.push(AiAgent {
            id: "local-test".into(),
            name: "Test".into(),
            program: "/usr/bin/example".into(),
            args: vec!["a b".into(), "$(not-a-shell)".into()],
            directory: "/tmp".into(),
            ..AiAgent::default()
        });
        save(&path, &config).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.ai_agents[0].args, config.ai_agents[0].args);
        config.ai_agents.push(config.ai_agents[0].clone());
        save(&path, &config).unwrap();
        assert!(load(&path).is_err());
    }

    #[test]
    fn ai_agents_reject_invalid_fields_and_credentials() {
        let mut agent = AiAgent {
            id: "test".into(),
            name: "Test".into(),
            program: "tool".into(),
            ..AiAgent::default()
        };
        assert!(agent.validate().is_ok());
        agent.directory = "relative".into();
        assert!(agent.validate().is_err());
        agent.directory.clear();
        agent.args.push("bad\0argument".into());
        assert!(agent.validate().is_err());
        assert!(serde_json::from_str::<AiAgent>(r#"{"api_key":"not-a-real-key"}"#).is_err());
    }

    #[test]
    fn persists_basic_protocol_profiles() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("sessions.json");
        let mut config = Config::default();
        for (protocol, port) in [
            (Protocol::Telnet, 23),
            (Protocol::Ftp, 21),
            (Protocol::Sftp, 22),
        ] {
            config.profiles.push(Profile {
                id: uuid::Uuid::new_v4().to_string(),
                name: format!("{protocol:?}"),
                protocol,
                port,
                host: "example.invalid".into(),
                ..Profile::default()
            });
        }
        save(&path, &config).unwrap();
        let loaded = load(&path).unwrap();
        for (before, after) in config.profiles.iter().zip(loaded.profiles.iter()) {
            assert_eq!(before.protocol, after.protocol);
            assert_eq!(before.port, after.port);
        }
    }
    #[test]
    fn roundtrip_and_version_rejection() {
        let path = env::temp_dir().join(format!("nexuterm-test-{}.json", std::process::id()));
        assert!(!path.exists());
        save(&path, &Config::default()).unwrap();
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(load(&path).unwrap().version, 2);
        fs::write(&path, r#"{"version":99,"profiles":[]}"#).unwrap();
        assert!(load(&path).is_err());
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn secrets_are_not_part_of_schema() {
        let bad = r#"{"name":"test","password":"secret"}"#;
        assert!(serde_json::from_str::<Profile>(bad).is_err());
    }
    #[test]
    fn migrate_preserves_original_and_profile() {
        let dir = env::temp_dir().join(format!("nexuterm-migrate-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("sessions.json");
        let bytes=br#"{"version":1,"profiles":[{"name":"old","host":"example.invalid","port":22,"protocol":"Ssh"}]}"#;
        fs::write(&path, bytes).unwrap();
        let config = load(&path).unwrap();
        assert_eq!(config.version, 2);
        assert!(!config.profiles[0].id.is_empty());
        assert_eq!(config.profiles[0].name, "old");
        assert_eq!(
            fs::read(path.with_extension("json.v1.backup")).unwrap(),
            bytes
        );
        assert_eq!(load(&path).unwrap().profiles[0].id, config.profiles[0].id);
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn import_legacy_name_without_changing_source() {
        let dir = env::temp_dir().join(format!("nexuterm-legacy-{}", uuid::Uuid::new_v4()));
        let old = dir.join("tuxterm/sessions.json");
        let new = dir.join("nexuterm/sessions.json");
        fs::create_dir_all(old.parent().unwrap()).unwrap();
        let bytes =
            br#"{"version":1,"profiles":[{"name":"legacy","host":"example.invalid","port":22}]}"#;
        fs::write(&old, bytes).unwrap();
        let config = load(&new).unwrap();
        assert_eq!(config.profiles[0].name, "legacy");
        assert_eq!(fs::read(&old).unwrap(), bytes);
        assert!(new.exists());
        assert!(new.with_extension("json.tuxterm.backup").exists());
        assert_eq!(load(&new).unwrap().profiles[0].id, config.profiles[0].id);
        fs::remove_dir_all(dir).unwrap();
    }
}
