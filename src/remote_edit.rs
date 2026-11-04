//! Private local working copies for files edited through an authenticated SSH session.
use crate::ssh::SshConnection;
use bytes::BytesMut;
use openssh_sftp_client::{
    Sftp,
    metadata::{MetaData, MetaDataBuilder, Permissions},
};
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::{
    path::{Path, PathBuf},
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
    baseline: Mutex<Option<String>>,
    original_metadata: Mutex<Option<MetaData>>,
    review_required: std::sync::atomic::AtomicBool,
}
impl RemoteDocument {
    pub fn local_path(&self) -> &Path {
        &self.local
    }
    pub fn remote_path(&self) -> &Path {
        &self.remote
    }
    /// Explicit preview through this already-authenticated destination, never a new login.
    pub async fn read_image(&self, reference: &str) -> Result<Vec<u8>, String> {
        if reference.len() > 4096 || reference.chars().any(char::is_control)
            || url::Url::parse(reference).is_ok()
        {
            return Err("La imagen SSH requiere una ruta de archivo, no una URL".into());
        }
        let parent = self.remote.parent().ok_or("Documento sin carpeta remota")?;
        let path = url::Url::from_directory_path(parent)
            .map_err(|_| "Carpeta SSH inválida")?
            .join(reference).map_err(|e| e.to_string())?
            .to_file_path().map_err(|_| "Ruta de imagen SSH inválida")?;
        let channel = self.connection.sftp().await?;
        let result = tokio::time::timeout(Duration::from_secs(20),
            read_version_limit(&channel.sftp, &path, crate::markdown_image::MAX_BYTES)).await;
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        let (bytes, _) = result.map_err(|_| "Tiempo agotado al cargar la imagen SSH")??;
        Ok(bytes)
    }
    pub fn host(&self) -> &str {
        &self.connection.profile.host
    }
    pub fn recovery_source(&self) -> crate::recovery::Source {
        let profile = &self.connection.profile;
        crate::recovery::Source::Remote {
            profile_id: profile.id.clone(),
            host: profile.host.clone(),
            destination: recovery_destination(profile),
            path: self.remote.clone(),
            fingerprint: self
                .fingerprint
                .try_lock()
                .ok()
                .map(|hash| fingerprint_hex(&hash)),
        }
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
        let result = tokio::time::timeout(
            Duration::from_secs(45),
            read_version(&channel.sftp, &remote),
        )
        .await;
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        let (contents, metadata) = result.map_err(|_| "Descarga de edición: tiempo agotado")??;
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
            baseline: Mutex::new(Some(
                String::from_utf8(contents).map_err(|e| e.to_string())?,
            )),
            original_metadata: Mutex::new(Some(metadata)),
            review_required: std::sync::atomic::AtomicBool::new(false),
        }))
    }
    /// Rebind a recovery draft only to the same destination and original contents.
    pub async fn recover(
        connection: Arc<SshConnection>,
        source: &crate::recovery::Source,
    ) -> Result<Arc<Self>, String> {
        let crate::recovery::Source::Remote {
            profile_id,
            destination,
            path,
            fingerprint: Some(expected),
            ..
        } = source
        else {
            return Err("El borrador no contiene la versión original necesaria para reanudar el guardado. Compara o guarda una copia.".into());
        };
        if connection.profile.id != *profile_id
            || recovery_destination(&connection.profile) != *destination
        {
            return Err("El perfil SSH no corresponde al destino original del borrador".into());
        }
        let document = Self::open(connection, path.clone()).await?;
        if fingerprint_hex(&*document.fingerprint.lock().await) != *expected {
            return Err("Conflicto: el original remoto cambió desde la edición del borrador. No se ha vinculado ni sobrescrito.".into());
        }
        Ok(document)
    }

    pub fn require_review(&self){self.review_required.store(true,Ordering::Relaxed);}
    pub fn requires_review(&self) -> bool {
        self.review_required.load(Ordering::Relaxed)
    }
    /// Attach a recovered draft for comparison, never implicitly accept a changed origin.
    pub async fn recovery_preview(
        connection: Arc<SshConnection>,
        source: &crate::recovery::Source,
        base: Option<String>,
    ) -> Result<Arc<Self>, String> {
        let crate::recovery::Source::Remote {
            profile_id,
            destination,
            path,
            fingerprint: Some(expected),
            ..
        } = source
        else {
            return Err("Falta la identidad original del borrador".into());
        };
        if connection.profile.id != *profile_id
            || recovery_destination(&connection.profile) != *destination
        {
            return Err("El perfil SSH no corresponde al destino original del borrador".into());
        }
        let hash = decode_fingerprint(expected)?;
        if let Some(base) = &base {
            validate_text(base.as_bytes())?;
            if <[u8; 32]>::from(Sha256::digest(base.as_bytes())) != hash {
                return Err("La base del borrador no corresponde al original".into());
            }
        }
        let document = Self::open(connection, path.clone()).await?;
        let mut current = document.fingerprint.lock().await;
        if *current != hash {
            *current = hash;
            *document.baseline.lock().await = base;
            *document.original_metadata.lock().await = None;
            document.review_required.store(true, Ordering::Relaxed);
        }
        drop(current);
        Ok(document)
    }
    pub async fn comparison(&self) -> Result<Comparison, String> {
        let guard = self.fingerprint.lock().await;
        let expected = *guard;
        let base = self.baseline.lock().await.clone();
        let expected_metadata = *self.original_metadata.lock().await;
        drop(guard);
        let channel = self.connection.sftp().await?;
        let result = tokio::time::timeout(
            Duration::from_secs(45),
            read_version(&channel.sftp, &self.remote),
        )
        .await;
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        let (bytes, metadata) = result.map_err(|_| "Tiempo agotado al comparar el original")??;
        validate_text(&bytes)?;
        let remote = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        Ok(Comparison {
            base,
            remote,
            metadata,
            expected,
            base_metadata: expected_metadata,
        })
    }
    /// Only advance the baseline if the version the user reviewed is still current.
    pub async fn accept_comparison(&self, review: &Comparison) -> Result<(), String> {
        let mut expected = self.fingerprint.lock().await;
        if *expected != review.expected
            || !same_optional_metadata(*self.original_metadata.lock().await, review.base_metadata)
        {
            return Err("La base del editor cambió; vuelve a comparar".into());
        }
        let channel = self.connection.sftp().await?;
        let result = tokio::time::timeout(
            Duration::from_secs(45),
            read_version(&channel.sftp, &self.remote),
        )
        .await;
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        let (bytes, metadata) =
            result.map_err(|_| "Tiempo agotado al confirmar la comparación")??;
        if bytes != review.remote.as_bytes() || !same_metadata(metadata, review.metadata) {
            return Err("Conflicto: el original volvió a cambiar; abre una comparación nueva. No se ha guardado nada.".into());
        }
        *expected = Sha256::digest(&bytes).into();
        *self.baseline.lock().await = Some(review.remote.clone());
        *self.original_metadata.lock().await = Some(metadata);
        self.review_required.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Publish a verified sibling temporary through OpenSSH's atomic rename extension.
    pub async fn save(&self, contents: String) -> Result<(), String> {
        validate_text(contents.as_bytes())?;
        if self.requires_review() {
            return Err("Conflicto: revisa y aplica la comparación antes de guardar".into());
        }
        if self.connection.closed.load(Ordering::Relaxed) {
            return Err("La sesión SSH está desconectada".into());
        }
        let mut expected = self.fingerprint.lock().await;
        let channel = self.connection.sftp().await?;
        let parent = self.remote.parent().ok_or("Archivo remoto sin carpeta")?;
        let directory = parent.join(format!(".nexuterm-edit-{}", uuid::Uuid::new_v4()));
        let temporary = directory.join("content");
        let mut created = false;
        let operation = async {
            let sftp = &channel.sftp;
            if !sftp.support_posix_rename() {
                return Err("El servidor no ofrece reemplazo atómico SFTP; no se ha sobrescrito el original. Guarda una copia.".into());
            }
            let original = sftp
                .fs()
                .symlink_metadata(&self.remote)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(base) = *self.original_metadata.lock().await
                && !same_metadata(base, original)
            {
                return Err("Conflicto: el contenido o los metadatos del original cambiaron. Compara las versiones antes de guardar.".into());
            }
            let permissions = original
                .permissions()
                .ok_or("El servidor no informa de los permisos originales")?;
            let uid = original
                .uid()
                .ok_or("El servidor no informa del propietario original")?;
            let gid = original
                .gid()
                .ok_or("El servidor no informa del grupo original")?;
            if <[u8; 32]>::from(Sha256::digest(read(sftp, &self.remote).await?)) != *expected {
                return Err(
                    "Conflicto: el archivo remoto ha cambiado. No se ha sobrescrito.".into(),
                );
            }
            // Use the exact editor copy, preserving its existing size and version checks.
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
            drop(local);
            sftp.fs()
                .create_dir(&directory)
                .await
                .map_err(|e| e.to_string())?;
            created = true;
            // No data is written until the containing directory is private.
            let mut private = Permissions::new();
            private
                .set_read_by_owner(true)
                .set_write_by_owner(true)
                .set_execute_by_owner(true);
            sftp.fs()
                .set_permissions(&directory, private)
                .await
                .map_err(|e| e.to_string())?;
            if sftp
                .fs()
                .symlink_metadata(&directory)
                .await
                .map_err(|e| e.to_string())?
                .permissions()
                != Some(private)
            {
                return Err("No se pudo proteger el temporal remoto".into());
            }
            let mut output = sftp
                .options()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .await
                .map_err(|e| e.to_string())?;
            let uploaded = async {
                let mut file_private = Permissions::new();
                file_private
                    .set_read_by_owner(true)
                    .set_write_by_owner(true);
                output
                    .set_permissions(file_private)
                    .await
                    .map_err(|e| e.to_string())?;
                for chunk in contents.as_bytes().chunks(64 * 1024) {
                    output.write_all(chunk).await.map_err(|e| e.to_string())?;
                }
                if sftp.support_fsync() {
                    output.sync_all().await.map_err(|e| e.to_string())?;
                }
                Ok::<(), String>(())
            }
            .await;
            let closed = output.close().await.map_err(|e| e.to_string());
            uploaded.and(closed)?;
            if read(sftp, &temporary).await? != contents.as_bytes() {
                return Err("El temporal remoto no coincide con el texto guardado".into());
            }
            sftp.fs()
                .set_metadata(&temporary, MetaDataBuilder::new().id((uid, gid)).create())
                .await
                .map_err(|e| format!("No se pudo conservar propietario/grupo: {e}"))?;
            sftp.fs()
                .set_permissions(&temporary, permissions)
                .await
                .map_err(|e| format!("No se pudieron conservar los permisos: {e}"))?;
            let replacement = sftp
                .fs()
                .symlink_metadata(&temporary)
                .await
                .map_err(|e| e.to_string())?;
            if replacement.uid() != Some(uid)
                || replacement.gid() != Some(gid)
                || replacement.permissions() != Some(permissions)
            {
                return Err("El temporal no conserva los metadatos originales; no se ha sustituido el archivo".into());
            }
            let current = read(sftp, &self.remote).await?;
            let metadata = sftp
                .fs()
                .symlink_metadata(&self.remote)
                .await
                .map_err(|e| e.to_string())?;
            if <[u8; 32]>::from(Sha256::digest(&current)) != *expected
                || metadata.permissions() != Some(permissions)
                || metadata.uid() != Some(uid)
                || metadata.gid() != Some(gid)
            {
                return Err(
                    "Conflicto antes del reemplazo: el original cambió. No se ha sobrescrito."
                        .into(),
                );
            }
            drop(current);
            sftp.fs()
                .rename(&temporary, &self.remote)
                .await
                .map_err(|e| e.to_string())?;
            let uploaded = read(sftp, &self.remote).await?;
            if uploaded != contents.as_bytes() {
                return Err("No se pudo verificar el archivo tras el reemplazo. Comprueba el servidor; se conserva el borrador.".into());
            }
            let final_metadata = sftp
                .fs()
                .symlink_metadata(&self.remote)
                .await
                .map_err(|e| e.to_string())?;
            if final_metadata.permissions() != Some(permissions)
                || final_metadata.uid() != Some(uid)
                || final_metadata.gid() != Some(gid)
            {
                return Err("El archivo publicado cambió de metadatos durante la verificación; se conserva el borrador".into());
            }
            *expected = Sha256::digest(&uploaded).into();
            *self.baseline.lock().await = Some(contents.clone());
            *self.original_metadata.lock().await = Some(final_metadata);
            Ok(())
        };
        let result = tokio::time::timeout(Duration::from_secs(90), operation).await
            .map_err(|_|"Guardado remoto: tiempo agotado. No se ha confirmado el resultado; comprueba el original. Se conserva el borrador.".to_string()).and_then(|r|r);
        let cleanup = if created {
            tokio::time::timeout(Duration::from_secs(3), async {
                let _ = channel.sftp.fs().remove_file(&temporary).await;
                channel.sftp.fs().remove_dir(&directory).await
            })
            .await
            .is_ok_and(|r| r.is_ok())
        } else {
            true
        };
        let _ = tokio::time::timeout(Duration::from_secs(3), channel.close()).await;
        if !cleanup {
            return Err(format!(
                "{} Puede quedar un temporal privado en {}. Se conserva la copia local.",
                result.err().unwrap_or_else(|| {
                    "Archivo reemplazado y verificado, pero no se pudo retirar el temporal.".into()
                }),
                directory.display()
            ));
        }
        result
    }
}
#[derive(Clone)]
pub struct Comparison {
    pub base: Option<String>,
    pub remote: String,
    pub metadata: MetaData,
    expected: [u8; 32],
    pub base_metadata: Option<MetaData>,
}
fn same_metadata(a: MetaData, b: MetaData) -> bool {
    a.len() == b.len()
        && a.modified() == b.modified()
        && a.uid() == b.uid()
        && a.gid() == b.gid()
        && a.permissions() == b.permissions()
        && a.file_type() == b.file_type()
}
fn same_optional_metadata(a: Option<MetaData>, b: Option<MetaData>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => same_metadata(a, b),
        (None, None) => true,
        _ => false,
    }
}
fn decode_fingerprint(value: &str) -> Result<[u8; 32], String> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Huella original inválida".into());
    }
    let mut out = [0; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16)
            .map_err(|_| "Huella original inválida")?;
    }
    Ok(out)
}

fn fingerprint_hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn recovery_destination(profile: &crate::config::Profile) -> String {
    format!(
        "{:x}",
        Sha256::digest(
            serde_json::to_vec(&(&profile.host, profile.port, &profile.username,))
                .expect("serializable destination")
        )
    )
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
    Ok(read_version(sftp, path).await?.0)
}
async fn read_version(sftp: &Sftp, path: &Path) -> Result<(Vec<u8>, MetaData), String> {
    read_version_limit(sftp, path, EDIT_LIMIT).await
}
async fn read_version_limit(sftp: &Sftp, path: &Path, limit: usize) -> Result<(Vec<u8>, MetaData), String> {
    let limit_error = if limit == EDIT_LIMIT { "Archivo mayor de 8 MiB" } else { "Imagen superior a 4 MiB" };
    let before = sftp
        .fs()
        .symlink_metadata(path)
        .await
        .map_err(|e| e.to_string())?;
    if !before.file_type().is_some_and(|t| t.is_file()) {
        return Err("Solo se editan archivos regulares; no enlaces simbólicos".into());
    }
    if before.len().is_some_and(|n| n > limit as u64) {
        return Err(limit_error.into());
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
            if bytes.len() + block.len() > limit {
                return Err(limit_error.into());
            }
            bytes.extend_from_slice(&block);
        }
        let after = sftp
            .fs()
            .symlink_metadata(path)
            .await
            .map_err(|e| e.to_string())?;
        if !same_metadata(before, after) {
            return Err("El archivo remoto cambió durante la lectura".into());
        }
        Ok((bytes, after))
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
