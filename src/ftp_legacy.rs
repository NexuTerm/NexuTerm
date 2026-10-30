//! Private FTP upload transport and Windows CE LIST compatibility.
//! curl handles FTP transport; credentials travel only over its private stdin pipe.
use futures_util::future::{Either, select};
use gtk::{gio, glib, prelude::*};
use nexuterm::{
    config::Profile,
    transfer::{Entry, Event, PAGE_SIZE, Progress, Request, WorkerChannels},
};
use std::{
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::OpenOptionsExt,
    },
    path::{Component, Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

fn quoted(value: &str) -> Result<String, String> {
    if value.chars().any(|c| c.is_control()) {
        return Err("FTP: valor con caracteres de control".into());
    }
    Ok(format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}
fn url(profile: &Profile, path: &Path, listing: bool) -> Result<String, String> {
    let host = if profile.host.contains(':') {
        format!("[{}]", profile.host)
    } else {
        profile.host.clone()
    };
    let mut url =
        url::Url::parse(&format!("ftp://{host}:{}/", profile.port)).map_err(|e| e.to_string())?;
    {
        let mut segments=url.path_segments_mut().map_err(|_|"Ruta FTP inválida")?;segments.clear();
        for component in path.components(){match component {
            Component::Normal(name)=>{segments.push(name.to_str().ok_or("Ruta FTP no UTF-8")?);},
            Component::ParentDir=>{segments.pop();},_=>{},
        }}
        if listing {segments.push("");}
    }
    // Curl needs an additional slash for a server-absolute FTP path.
    let absolute=format!("/{}",url.path());url.set_path(&absolute);
    Ok(url.into())
}
fn credential_config(
    profile: &Profile,
    password: &str,
    path: &Path,
    listing: bool,
) -> Result<Zeroizing<String>, String> {
    if password.chars().any(char::is_control) || profile.username.chars().any(char::is_control) {
        return Err("FTP: credencial con caracteres de control".into());
    }
    let mut address = url::Url::parse(&url(profile, path, listing)?).map_err(|e| e.to_string())?;
    address
        .set_username(&profile.username.replace('%',"%25"))
        .map_err(|_| "Usuario FTP inválido")?;
    let encoded_password=Zeroizing::new(password.replace('%',"%25"));
    address
        .set_password(Some(&encoded_password))
        .map_err(|_| "Contraseña FTP inválida")?;
    let address = Zeroizing::new(String::from(address));
    let encoded = Zeroizing::new(quoted(&address)?);
    Ok(Zeroizing::new(format!("url = {}\n", &*encoded)))
}
fn progress_percent(text: &str) -> Option<f64> {
    let end = text.rfind('%')?;
    let token = text[..end].split_whitespace().last()?;
    let value = token.parse::<f64>().ok()?;
    (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
}
/// Normal FTP uploads also avoid GVfs 1.60's NULL metadata lookup on create.
/// The caller has checked the destination directory without querying a missing file.
pub async fn upload_file(
    profile: &Profile,
    password: &str,
    local: PathBuf,
    remote: PathBuf,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<(), String> {
    run(
        profile,
        password,
        &remote,
        Action::Upload(local),
        channels,
        lifetime,
    )
    .await
    .map(|_| ())
}
enum Action {
    List,
    Download(PathBuf),
    Upload(PathBuf),
}

async fn transport(
    profile: Profile,
    password: Zeroizing<String>,
    path: PathBuf,
    action: Action,
    progress: tokio::sync::watch::Sender<Progress>,
) -> Result<Vec<u8>, String> {
    let mut config = credential_config(&profile, &password, &path, matches!(action, Action::List))?;
    if matches!(action, Action::List) {
        config.push_str("request = \"LIST\"\n");
    }
    let curl = nexuterm::connection::find_program(&["curl"])
        .ok_or("FTP: instala curl o el paquete .deb actualizado")?;
    let mut command = tokio::process::Command::new(curl);
    command
        .args([
            "--disable",
            "--config",
            "-",
            "--show-error",
            "--fail",
            "--globoff",
            "--proto",
            "=ftp",
            "--connect-timeout",
            "10",
            "--max-time",
            "120",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let uploading = matches!(&action, Action::Upload(_));
    if uploading {
        command.arg("--progress-bar");
    } else {
        command.arg("--silent");
    }
    let mut upload_handle = None;
    let mut upload_metadata = None;
    match &action {
        Action::Download(local) => {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(local)
                .map_err(|e| e.to_string())?;
            command.stdout(Stdio::from(file));
        }
        Action::Upload(local) => {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(local)
                .map_err(|e| e.to_string())?;
            let metadata = file.metadata().map_err(|e| e.to_string())?;
            if !metadata.is_file() {
                return Err("Solo se suben archivos regulares; no se siguen enlaces".into());
            }
            let fd = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3) };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            let handle = unsafe { std::fs::File::from_raw_fd(fd) };
            command
                .arg("--upload-file")
                .arg(format!("/proc/self/fd/{fd}"));
            // Only this read-only descriptor is inherited; the credential uses stdin.
            unsafe {
                command.pre_exec(move || {
                    if libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                        Err(std::io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                });
            }
            progress.send_replace(Progress {
                bytes: 0,
                total: Some(metadata.len()),
            });
            upload_metadata = Some(metadata);
            upload_handle = Some(handle);
        }
        Action::List => {}
    }
    let mut child = command.spawn().map_err(|e| format!("FTP: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("FTP sin canal privado")?
        .write_all(config.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    let mut stderr = child.stderr.take().ok_or("FTP sin canal de errores")?;
    let upload_total = upload_metadata.as_ref().map(|m| m.len());
    let error_reader = async {
        let mut errors = Vec::new();
        let mut line = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let n = match stderr.read(&mut buffer).await {
                Ok(n) => n,
                Err(_) => break,
            };
            if n == 0 {
                break;
            }
            for &byte in &buffer[..n] {
                if byte == b'\r' || byte == b'\n' {
                    let text = String::from_utf8_lossy(&line);
                    if uploading && text.trim_start().starts_with('#') && text.contains('%') {
                        if let Some(percent) = progress_percent(&text) {
                            if let Some(total) = upload_total {
                                let bytes = (total as f64 * percent / 100.0) as u64;
                                progress.send_replace(Progress {
                                    bytes: bytes.min(total),
                                    total: Some(total),
                                });
                            }
                        }
                    } else if !text.trim_start().starts_with('#') {
                        errors.extend_from_slice(&line);
                        errors.push(b'\n');
                        if errors.len() > 16384 {
                            errors.drain(..errors.len() - 16384);
                        }
                    }
                    line.clear();
                } else if line.len() < 1024 {
                    line.push(byte);
                }
            }
        }
        errors.extend_from_slice(&line);
        if errors.len() > 16384 {
            errors.drain(..errors.len() - 16384);
        }
        errors
    };
    let stdout = child.stdout.take();
    let output_reader = async {
        let mut data = Vec::new();
        if let Some(stdout) = stdout {
            stdout
                .take(8 * 1024 * 1024 + 1)
                .read_to_end(&mut data)
                .await
                .map_err(|e| e.to_string())?;
        }
        if data.len() > 8 * 1024 * 1024 {
            return Err("Listado FTP demasiado grande (máximo 8 MiB)".to_string());
        }
        Ok(data)
    };
    let (data, error) = tokio::join!(output_reader, error_reader);
    let data = data?;
    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("FTP: {}", String::from_utf8_lossy(&error).trim()));
    }
    if let (Some(handle), Some(before)) = (upload_handle, upload_metadata) {
        let after = handle.metadata().map_err(|e| e.to_string())?;
        if after.len() != before.len() || after.modified().ok() != before.modified().ok() {
            return Err("El archivo local cambió durante la subida; revisa el destino FTP".into());
        }
        progress.send_replace(Progress {
            bytes: before.len(),
            total: Some(before.len()),
        });
    }
    Ok(data)
}
async fn run(
    profile: &Profile,
    password: &str,
    path: &Path,
    action: Action,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<Vec<u8>, String> {
    let mut worker = nexuterm::runtime::runtime().spawn(transport(
        profile.clone(),
        Zeroizing::new(password.into()),
        path.into(),
        action,
        channels.progress.clone(),
    ));
    let interrupt = async {
        while !channels.cancel.load(std::sync::atomic::Ordering::Relaxed)
            && !channels.stop.load(std::sync::atomic::Ordering::Relaxed)
            && !lifetime.is_cancelled()
        {
            glib::timeout_future(Duration::from_millis(50)).await;
        }
    };
    match select(Box::pin(&mut worker), Box::pin(interrupt)).await {
        Either::Left((result, _)) => result.map_err(|e| e.to_string())?,
        Either::Right(_) => {
            worker.abort();
            let _ = worker.await;
            Err("Operación FTP cancelada; puede quedar un archivo parcial".into())
        }
    }
}

/// A strict DOS/Windows CE parser: reject unrecognized rows rather than report empty.
pub fn parse(data: &[u8]) -> Result<Vec<Entry>, String> {
    let text = std::str::from_utf8(data).map_err(|_| "Listado FTP Windows CE no UTF-8")?;
    let mut entries = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut remaining = line.trim_start();
        let mut token = || {
            let end = remaining
                .find(char::is_whitespace)
                .unwrap_or(remaining.len());
            let word = &remaining[..end];
            remaining = remaining[end..].trim_start();
            word
        };
        let date = token();
        let time = token();
        let kind = token();
        let valid_date = date.len() == 8
            && date.as_bytes()[2] == b'-'
            && date.as_bytes()[5] == b'-'
            && date
                .bytes()
                .enumerate()
                .all(|(i, c)| i == 2 || i == 5 || c.is_ascii_digit());
        let valid_time = time.len() >= 5
            && time.as_bytes()[2] == b':'
            && time.as_bytes()[..2].iter().all(|c| c.is_ascii_digit())
            && time.as_bytes()[3..5].iter().all(|c| c.is_ascii_digit());
        if !valid_date || !valid_time {
            return Err("Formato de listado FTP no reconocido".into());
        }
        let directory = kind.eq_ignore_ascii_case("<DIR>");
        let size = if directory {
            None
        } else {
            Some(kind.parse::<u64>().map_err(|_| "Tamaño FTP inválido")?)
        };
        if remaining == "." || remaining == ".." {
            continue;
        }
        let name = PathBuf::from(remaining);
        let mut components = name.components();
        if !matches!(components.next(), Some(Component::Normal(_)))
            || components.next().is_some()
            || remaining.contains(['\\', '\r', '\n', '\0'])
        {
            return Err("Nombre FTP inválido".into());
        }
        entries.push(Entry {
            name,
            directory,
            regular: !directory,
            size,
            permissions: None,
        });
    }
    entries.sort_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(entries)
}
pub struct Listing {
    pub directory: PathBuf,
    entries: Vec<Entry>,
    offset: usize,
}
impl Listing {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn page(&mut self) -> Event {
        let end = (self.offset + PAGE_SIZE).min(self.entries.len());
        let entries = self.entries[self.offset..end].to_vec();
        self.offset = end;
        Event::Page {
            directory: self.directory.clone(),
            entries,
            more: end < self.entries.len(),
        }
    }
}
pub async fn list(
    profile: &Profile,
    password: &str,
    path: &Path,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<Listing, String> {
    let data = run(profile, password, path, Action::List, channels, lifetime).await?;
    Ok(Listing {
        directory: path.into(),
        entries: parse(&data)?,
        offset: 0,
    })
}
pub async fn transfer(
    profile: &Profile,
    password: &str,
    request: Request,
    channels: &WorkerChannels,
    lifetime: &gio::Cancellable,
) -> Result<(), String> {
    let (path, action, total) = match request {
        Request::Upload { local, directory } => {
            let name = local.file_name().ok_or("Archivo local inválido")?;
            let listing = list(profile, password, &directory, channels, lifetime).await?;
            if listing
                .entries
                .iter()
                .any(|entry| entry.name == Path::new(name))
            {
                return Err("El archivo remoto ya existe; no se sobrescribe".into());
            }
            let path = directory.join(name);
            let total = std::fs::metadata(&local).map_err(|e| e.to_string())?.len();
            (path, Action::Upload(local), Some(total))
        }
        Request::Download { remote, local } => (remote, Action::Download(local), None),
        Request::Create { .. } => {
            return Err("Este servidor FTP heredado no admite crear desde este navegador".into());
        }
        _ => return Err("Operación FTP inválida".into()),
    };
    channels.progress.send_replace(Progress { bytes: 0, total });
    let downloaded = match &action {
        Action::Download(local) => Some(local.clone()),
        _ => None,
    };
    run(profile, password, &path, action, channels, lifetime).await?;
    let bytes = downloaded
        .and_then(|local| std::fs::metadata(local).ok())
        .map(|info| info.len())
        .or(total)
        .unwrap_or(0);
    channels.progress.send_replace(Progress {
        bytes,
        total: Some(bytes),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn bounded_upload_progress_parser() {
        assert_eq!(super::progress_percent("#### 33.5%"), Some(33.5));
        assert_eq!(super::progress_percent("### 100.0%"), Some(100.0));
        for value in ["NaN%", "101%", "-1%", "error", "# nope%"] {
            assert_eq!(super::progress_percent(value), None);
        }
    }

    use super::*;
    #[test]
    fn windows_ce_24_hour_listing_preserves_spaces_and_sizes() {
        let rows=parse(b"09-29-22  04:05       <DIR>          Windows\r\n09-29-22  11:05       <DIR>          Program Files\r\n06-30-27  05:20               325632 fixture.dll\r\n").unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[0].directory);
        assert_eq!(rows[0].name, Path::new("Program Files"));
        assert_eq!(rows[2].size, Some(325632));
        assert!(parse(b"garbled output").is_err());
        assert!(parse(b"09-29-22  04:05  10 ../escape").is_err());
        assert!(parse(b"09-29-22  04:05  10 nested\\escape").is_err());
        assert!(parse(b"").unwrap().is_empty());
    }
    #[test]
    fn config_escaping_and_urls_do_not_expose_passwords() {
        assert_eq!(quoted("a\"\\b").unwrap(), "\"a\\\"\\\\b\"");
        assert!(quoted("a\nb").is_err());
        let profile = Profile {
            host: "127.0.0.1".into(),
            port: 21,
            username: "demo:user".into(),
            ..Default::default()
        };
        let config =
            credential_config(&profile, "p:@ \"\\", Path::new("/home/demo/a b.txt"), false)
                .unwrap();
        assert!(config.contains("demo%3Auser:"));
        assert!(config.contains("//home/demo/a%20b.txt"));
        assert!(!config.contains("user ="));
        assert!(!config.contains("p:@"));
        let profile=Profile {username:"literal%3Aname".into(),..profile};
        let encoded=credential_config(&profile,"literal%40pass",Path::new("/%20 #?.txt"),false).unwrap();
        assert!(encoded.contains("literal%253Aname:literal%2540pass"));
        assert!(encoded.contains("//%2520%20%23%3F.txt"));
        assert!(credential_config(&profile, "bad\nsecret", Path::new("/x"), false).is_err());
        let p = Profile {
            host: "::1".into(),
            port: 21,
            ..Profile::default()
        };
        assert_eq!(
            url(&p, Path::new("/Program Files"), true).unwrap(),
            "ftp://[::1]//Program%20Files/"
        );
    }
}
