//! Private local working copies for files edited through an authenticated SSH session.
use crate::ssh::SshConnection;
use bytes::BytesMut;
use openssh_sftp_client::Sftp;
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};

pub const EDIT_LIMIT: usize = 8 * 1024 * 1024;

pub struct RemoteDocument {
    connection: Arc<SshConnection>,
    remote: PathBuf,
    local: PathBuf,
    _directory: tempfile::TempDir,
    fingerprint: Mutex<[u8; 32]>,
}
impl RemoteDocument {
    pub fn local_path(&self) -> &Path {
        &self.local
    }
    pub fn remote_path(&self) -> &Path {
        &self.remote
    }
    pub fn host(&self) -> &str {
        &self.connection.profile.host
    }
    pub async fn open(
        connection: Arc<SshConnection>,
        remote: PathBuf,
    ) -> Result<Arc<Self>, String> {
        if !remote.is_absolute() {
            return Err("La ruta remota debe ser absoluta".into());
        }
        let name = remote.file_name().ok_or("Archivo remoto sin nombre")?;
        let directory = tempfile::Builder::new()
            .prefix("nexuterm-edit-")
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir()
            .map_err(|e| e.to_string())?;
        let local = directory.path().join(name);
        let channel = connection.sftp().await?;
        let result =
            tokio::time::timeout(Duration::from_secs(45), read(&channel.sftp, &remote)).await;
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        let contents = result.map_err(|_| "Descarga de edición: tiempo agotado")??;
        validate_text(&contents)?;
        let mut output = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&local)
            .await
            .map_err(|e| e.to_string())?;
        output
            .write_all(&contents)
            .await
            .map_err(|e| e.to_string())?;
        output.flush().await.map_err(|e| e.to_string())?;
        Ok(Arc::new(Self {
            connection,
            remote,
            local,
            _directory: directory,
            fingerprint: Mutex::new(Sha256::digest(&contents).into()),
        }))
    }
    /// The copy remains available for retry if SCP or conflict checking fails.
    pub async fn save(&self, contents: String) -> Result<(), String> {
        validate_text(contents.as_bytes())?;
        if self.connection.closed.load(Ordering::Relaxed) {
            return Err("La sesión SSH está desconectada".into());
        }
        let mut expected = self.fingerprint.lock().await;
        let channel = self.connection.sftp().await?;
        let operation = async {
            let current = read(&channel.sftp, &self.remote).await?;
            if <[u8; 32]>::from(Sha256::digest(&current)) != *expected {
                return Err("Conflicto: el archivo remoto ha cambiado. No se ha sobrescrito; compara con tu copia temporal.".into());
            }
            // Read the exact local version being sent, not another concurrently edited copy.
            let input = tokio::fs::File::open(&self.local)
                .await
                .map_err(|e| e.to_string())?;
            let mut local = Vec::new();
            input
                .take(EDIT_LIMIT as u64 + 1)
                .read_to_end(&mut local)
                .await
                .map_err(|e| e.to_string())?;
            if local != contents.as_bytes() {
                return Err("La copia temporal cambió antes de subirla".into());
            }
            self.scp().await?;
            let uploaded = read(&channel.sftp, &self.remote).await?;
            if uploaded != contents.as_bytes() {
                return Err("La verificación remota no coincide con el texto guardado".into());
            }
            *expected = Sha256::digest(&uploaded).into();
            Ok(())
        };
        let result = tokio::time::timeout(Duration::from_secs(90), operation)
            .await
            .map_err(|_| {
                "Guardado remoto: tiempo agotado; se conserva la copia temporal".to_string()
            })
            .and_then(|r| r);
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        result
    }
    async fn scp(&self) -> Result<(), String> {
        if self.connection.closed.load(Ordering::Relaxed) {
            return Err("La sesión SSH está desconectada".into());
        }
        let profile = &self.connection.profile;
        let host = if profile.host.contains(':') {
            format!("[{}]", profile.host)
        } else {
            profile.host.clone()
        };
        let destination = format!(
            "{}{}{}:{}",
            profile.username,
            if profile.username.is_empty() { "" } else { "@" },
            host,
            self.remote.to_str().ok_or("SCP requiere una ruta UTF-8")?
        );
        let status = tokio::process::Command::new("scp")
            .args([
                "-s",
                "-B",
                "-o",
                "ControlMaster=no",
                "-o",
                "BatchMode=yes",
                "-o",
                "ProxyCommand=false",
                "-o",
            ])
            .arg(format!(
                "ControlPath={}",
                self.connection.socket().display()
            ))
            .arg("-P")
            .arg(profile.port.to_string())
            .arg("--")
            .arg(&self.local)
            .arg(destination)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .status()
            .await
            .map_err(|e| format!("No se pudo ejecutar scp: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("SCP no pudo guardar el archivo. Comprueba la conexión y los permisos; se conserva la copia temporal. El destino podría estar incompleto.".into())
        }
    }
}
fn validate_text(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > EDIT_LIMIT {
        return Err("Archivo mayor de 8 MiB".into());
    }
    if bytes.contains(&0) || std::str::from_utf8(bytes).is_err() {
        return Err("Solo se editan archivos de texto UTF-8; no binarios".into());
    }
    Ok(())
}
async fn read(sftp: &Sftp, path: &Path) -> Result<Vec<u8>, String> {
    let before = sftp
        .fs()
        .symlink_metadata(path)
        .await
        .map_err(|e| e.to_string())?;
    if !before.file_type().is_some_and(|t| t.is_file()) {
        return Err("Solo se editan archivos regulares; no enlaces simbólicos".into());
    }
    if before.len().is_some_and(|n| n > EDIT_LIMIT as u64) {
        return Err("Archivo mayor de 8 MiB".into());
    }
    let mut input = sftp.open(path).await.map_err(|e| e.to_string())?;
    let result = async {
        let mut bytes = Vec::new();
        loop {
            input
                .seek(std::io::SeekFrom::Start(bytes.len() as u64))
                .await
                .map_err(|e| e.to_string())?;
            let block = input
                .read(65536, BytesMut::with_capacity(65536))
                .await
                .map_err(|e| e.to_string())?;
            let Some(block) = block else {
                break;
            };
            if block.is_empty() {
                break;
            }
            if bytes.len() + block.len() > EDIT_LIMIT {
                return Err("Archivo mayor de 8 MiB".into());
            }
            bytes.extend_from_slice(&block);
        }
        let after = sftp
            .fs()
            .symlink_metadata(path)
            .await
            .map_err(|e| e.to_string())?;
        if before.len() != after.len() || before.modified() != after.modified() {
            return Err("El archivo remoto cambió durante la lectura".into());
        }
        Ok(bytes)
    }
    .await;
    let closed = input.close().await.map_err(|e| e.to_string());
    match result {
        Ok(bytes) => closed.map(|_| bytes),
        Err(error) => Err(error),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_limits_and_encoding() {
        assert!(validate_text("función\n".as_bytes()).is_ok());
        assert!(validate_text(&[0]).is_err());
        assert!(validate_text(&[0xff]).is_err());
        assert!(validate_text(&vec![b'x'; EDIT_LIMIT + 1]).is_err());
    }
}
