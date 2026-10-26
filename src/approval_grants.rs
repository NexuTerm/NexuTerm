//! Exact-operation approvals, scoped to one agent. Only hashes are persisted.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, io::Read, os::unix::fs::DirBuilderExt, path::PathBuf};

pub struct Grants {
    path: PathBuf,
    session: BTreeSet<String>,
}
impl Grants {
    pub fn new(root: PathBuf) -> Self {
        Self {
            path: root.join("permission-grants.json"),
            session: BTreeSet::new(),
        }
    }
    pub fn clear_session(&mut self) {
        self.session.clear();
    }
    fn saved(&self) -> BTreeSet<String> {
        let mut bytes = Vec::new();
        let Ok(file) = std::fs::File::open(&self.path) else {
            return BTreeSet::new();
        };
        if file.take(32769).read_to_end(&mut bytes).is_err() || bytes.len() > 32768 {
            return BTreeSet::new();
        }
        serde_json::from_slice::<BTreeSet<String>>(&bytes)
            .ok()
            .filter(|keys| {
                keys.len() <= 256
                    && keys
                        .iter()
                        .all(|key| key.len() == 64 && key.bytes().all(|b| b.is_ascii_hexdigit()))
            })
            .unwrap_or_default()
    }
    pub fn allows(&self, key: &str) -> bool {
        self.session.contains(key) || self.saved().contains(key)
    }
    pub fn remember(&mut self, key: String, always: bool) -> Result<(), String> {
        if always {
            let mut keys = self.saved();
            keys.insert(key);
            if keys.len() > 256 {
                return Err("Límite de permisos guardados alcanzado".into());
            }
            let parent = self.path.parent().ok_or("Perfil no disponible")?;
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)
                .map_err(|_| "No se pudo preparar el perfil")?;
            crate::agent_profile::private_write(
                &self.path,
                &serde_json::to_vec(&keys).map_err(|_| "Permiso inválido")?,
            )
        } else {
            if self.session.len() >= 256 {
                return Err("Límite de permisos de sesión alcanzado".into());
            }
            self.session.insert(key);
            Ok(())
        }
    }
}
pub fn fingerprint(value: &Value) -> String {
    format!("{:x}", Sha256::digest(value.to_string().as_bytes()))
}
pub fn operation(method: &str, params: &Value, item: Option<&Value>) -> Option<String> {
    let cwd = params["cwd"]
        .as_str()
        .or_else(|| item.and_then(|item| item["cwd"].as_str()))?;
    let mut request = params.as_object()?.clone();
    for field in ["threadId", "turnId", "itemId", "reason"] {
        request.remove(field);
    }
    request.insert("cwd".into(), json!(cwd));
    if !request.contains_key("command") {
        if let Some(command) = item.and_then(|item| item.get("command")) {
            request.insert("command".into(), command.clone());
        }
    }
    if let Some(operation) = request.remove("approvalOperation") {
        request.insert("command".into(), operation);
    }
    Some(fingerprint(
        &json!({"method":method,"request":request,"changes":item.and_then(|item|item.get("changes"))}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_scope_and_private_persistence() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("one");
        let key = operation(
            "command",
            &json!({"cwd":"/work","command":"printf test","itemId":"one"}),
            None,
        )
        .unwrap();
        assert_eq!(
            Some(key.clone()),
            operation(
                "command",
                &json!({"cwd":"/work","command":"printf test","itemId":"two"}),
                None
            )
        );
        assert_ne!(
            Some(key.clone()),
            operation(
                "command",
                &json!({"cwd":"/other","command":"printf test"}),
                None
            )
        );
        assert_ne!(
            Some(key.clone()),
            operation(
                "command",
                &json!({"cwd":"/work","command":"printf other"}),
                None
            )
        );
        assert_ne!(
            Some(key.clone()),
            operation(
                "command",
                &json!({"cwd":"/work","command":"printf test","additionalPermissions":{"network":true}}),
                None
            )
        );
        assert!(operation("command", &json!({"command":"printf test"}), None).is_none());
        let mut grants = Grants::new(first.clone());
        grants.remember(key.clone(), false).unwrap();
        assert!(grants.allows(&key));
        assert!(!Grants::new(first.clone()).allows(&key));
        grants.remember(key.clone(), true).unwrap();
        assert!(Grants::new(first.clone()).allows(&key));
        assert!(!Grants::new(directory.path().join("two")).allows(&key));
        assert!(
            !std::fs::read_to_string(first.join("permission-grants.json"))
                .unwrap()
                .contains("printf")
        );
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(first.join("permission-grants.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
