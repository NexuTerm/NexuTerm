//! Bounded image sources. Relative SSH images never resolve against a local cache directory.
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
pub const MAX_BYTES: usize = 4 * 1024 * 1024;
#[derive(Clone, Debug)]
pub enum Location {
    Local {
        path: PathBuf,
        base: Option<PathBuf>,
        explicit: bool,
    },
    Http(String),
}
pub fn location(base: Option<&Path>, reference: &str, explicit: bool) -> Result<Location, String> {
    if reference.len() > 4096 || reference.chars().any(char::is_control) {
        return Err("Referencia de imagen inválida".into());
    }
    if let Ok(url) = url::Url::parse(reference) {
        if !url.username().is_empty() || url.password().is_some() {
            return Err("No se admiten credenciales en la URL de una imagen".into());
        }
        return match url.scheme() {
            "http" | "https" if explicit => Ok(Location::Http(url.to_string())),
            "http" | "https" => {
                Err("Imagen remota: pulsa Cargar para autorizar la conexión".into())
            }
            "file" if explicit => Ok(Location::Local {
                path: url.to_file_path().map_err(|_| "Ruta file inválida")?,
                base: base.map(Path::to_owned),
                explicit,
            }),
            _ => Err("Solo imágenes locales o HTTP/HTTPS con carga expresa".into()),
        };
    }
    let Some(base) = base else {
        return Err("Imagen relativa SSH/desconectada: no se busca en el disco local. Usa una URL o una copia local explícita.".into());
    };
    let url = url::Url::from_directory_path(base)
        .map_err(|_| "Directorio de imagen inválido")?
        .join(reference)
        .map_err(|e| e.to_string())?;
    if url.scheme() != "file" {
        return Err("La imagen externa necesita una carga expresa".into());
    }
    Ok(Location::Local {
        path: url.to_file_path().map_err(|_| "Ruta inválida")?,
        base: Some(base.to_owned()),
        explicit,
    })
}
pub fn read_local(path: &Path, base: Option<&Path>, explicit: bool) -> Result<Vec<u8>, String> {
    let resolved = path
        .canonicalize()
        .map_err(|e| format!("No se encontró la imagen: {e}"))?;
    if !explicit {
        let root = base
            .ok_or("Sin directorio local autorizado")?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !resolved.starts_with(root) {
            return Err(
                "La imagen está fuera de la carpeta del documento; pulsa Cargar para autorizarla"
                    .into(),
            );
        }
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(resolved)
        .map_err(|e| e.to_string())?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > MAX_BYTES as u64 {
        return Err("Imagen no regular o superior a 4 MiB".into());
    }
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("Imagen superior a 4 MiB".into());
    }
    Ok(bytes)
}
pub async fn download(url: &str) -> Result<Vec<u8>, String> {
    use tokio::io::AsyncReadExt;
    let parsed = url::Url::parse(url).map_err(|_| "URL de imagen inválida")?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err("URL HTTP/HTTPS sin credenciales requerida".into());
    }
    let program = crate::connection::find_program(&["curl"]).ok_or("No se encontró curl")?;
    let mut child = tokio::process::Command::new(program)
        .args([
            "--disable",
            "--disallow-username-in-url",
            "--silent",
            "--show-error",
            "--fail",
            "--location",
            "--max-redirs",
            "3",
            "--proto",
            "=http,https",
            "--proto-redir",
            "=http,https",
            "--connect-timeout",
            "5",
            "--max-time",
            "15",
            "--max-filesize",
            "4194304",
            "--url",
            url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut out = child.stdout.take().unwrap().take((MAX_BYTES + 1) as u64);
    let mut err = child.stderr.take().unwrap().take(8192);
    let op = async {
        let output = async move {
            let mut bytes = Vec::new();
            out.read_to_end(&mut bytes).await?;
            drop(out);
            Ok::<_, std::io::Error>(bytes)
        };
        let errors = async move {
            let mut bytes = Vec::new();
            err.read_to_end(&mut bytes).await?;
            drop(err);
            Ok::<_, std::io::Error>(bytes)
        };
        let (bytes, errors) = tokio::try_join!(output, errors).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_BYTES {
            let _ = child.kill().await;
            return Err("Imagen superior a 4 MiB".into());
        }
        let status = child.wait().await.map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!(
                "No se descargó la imagen: {}",
                String::from_utf8_lossy(&errors)
            ));
        }
        Ok(bytes)
    };
    tokio::time::timeout(Duration::from_secs(20), op)
        .await
        .map_err(|_| "Tiempo de carga de imagen agotado".to_string())?
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_and_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("doc");
        std::fs::create_dir(&base).unwrap();
        std::fs::write(base.join("imagen con espacio.png"), b"test").unwrap();
        let Location::Local { path, .. } =
            location(Some(&base), "imagen%20con%20espacio.png", false).unwrap()
        else {
            panic!()
        };
        assert_eq!(read_local(&path, Some(&base), false).unwrap(), b"test");
        assert!(location(None, "foo.png", true).is_err());
        assert!(location(Some(&base), "https://example.invalid/x", false).is_err());
        assert!(location(Some(&base), "data:image/png,x", true).is_err());
        let outside = dir.path().join("outside");
        std::fs::write(&outside, b"secret").unwrap();
        std::os::unix::fs::symlink(&outside, base.join("link.png")).unwrap();
        assert!(read_local(&base.join("link.png"), Some(&base), false).is_err());
        assert_eq!(read_local(&outside, Some(&base), true).unwrap(), b"secret");
        let big = base.join("big.png");
        std::fs::File::create(&big)
            .unwrap()
            .set_len(MAX_BYTES as u64 + 1)
            .unwrap();
        assert!(read_local(&big, Some(&base), true).is_err());
    }
}
