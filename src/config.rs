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
    pub theme: String,
    pub font: String,
    pub color_scheme: String,
    pub scrollback: u32,
    pub sidebar_visible: bool,
    pub monitor_interval: u64,
    pub shortcuts: std::collections::BTreeMap<String, String>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            font: "Monospace 11".into(),
            color_scheme: "dark".into(),
            scrollback: 10000,
            sidebar_visible: true,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
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
    use super::*;
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
