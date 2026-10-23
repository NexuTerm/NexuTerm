//! Compatibility with Windows CE LIST responses that GVfs silently ignores.
//! curl handles FTP transport; credentials travel only over its private stdin pipe.
use futures_util::future::{Either, select};
use gtk::{gio, glib, prelude::*};
use nexuterm::{
    config::Profile,
    transfer::{Entry, Event, PAGE_SIZE, Progress, Request, WorkerChannels},
};
use std::{
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
    let mut path = path.to_str().ok_or("Ruta FTP no UTF-8")?.to_owned();
    if listing && !path.ends_with('/') {
        path.push('/');
    }
    url.set_path(&path);
    Ok(url.into())
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
) -> Result<Vec<u8>, String> {
    let mut config = Zeroizing::new(format!(
        "url = {}\nuser = {}\n",
        quoted(&url(&profile, &path, matches!(action, Action::List))?)?,
        quoted(&format!("{}:{}", profile.username, *password))?
    ));
    if matches!(action, Action::List) {
        config.push_str("request = \"LIST\"\n");
    }
    let curl = nexuterm::connection::find_program(&["curl"])
        .ok_or("Compatibilidad FTP Windows CE: instala curl o el paquete .deb actualizado")?;
    let mut command = tokio::process::Command::new(curl);
    command
        .args([
            "--disable",
            "--config",
            "-",
            "--silent",
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
    match &action {
        Action::Download(local) => {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(local)
                .map_err(|e| e.to_string())?;
            command.stdout(Stdio::from(file));
        }
        Action::Upload(local) => {
            command.arg("--upload-file").arg(local);
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
    let mut stderr = child
        .stderr
        .take()
        .ok_or("FTP sin canal de errores")?
        .take(16384);
    let error_reader = async {
        let mut data = Vec::new();
        let _ = stderr.read_to_end(&mut data).await;
        data
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
        let p = Profile {
            host: "::1".into(),
            port: 21,
            ..Profile::default()
        };
        assert_eq!(
            url(&p, Path::new("/Program Files"), true).unwrap(),
            "ftp://[::1]/Program%20Files/"
        );
    }
}
