use crate::{
    config::Profile,
    connection::{self, CommandSpec},
};
use openssh_sftp_client::{Sftp, SftpOptions};
use std::os::unix::fs::PermissionsExt;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
};

pub struct SshConnection {
    pub profile: Profile,
    directory: tempfile::TempDir,
    pub closed: AtomicBool,
}
impl SshConnection {
    pub fn new(profile: Profile) -> Result<Arc<Self>, String> {
        if !matches!(
            profile.protocol,
            crate::config::Protocol::Ssh | crate::config::Protocol::Sftp
        ) {
            return Err("Esta conexión requiere SSH o SFTP".into());
        }
        connection::validate(&profile)?;
        let directory = tempfile::Builder::new()
            .prefix("nx-")
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir()
            .map_err(|e| e.to_string())?;
        Ok(Arc::new(Self {
            profile,
            directory,
            closed: AtomicBool::new(false),
        }))
    }
    pub fn socket(&self) -> PathBuf {
        self.directory.path().join("c")
    }
    pub fn terminal_command(&self) -> Result<CommandSpec, String> {
        let mut c = connection::command(&self.profile)?;
        let mut opts = vec![
            "-o".into(),
            "ControlMaster=auto".into(),
            "-S".into(),
            self.socket().to_string_lossy().into_owned(),
            "-o".into(),
            "ControlPersist=no".into(),
        ];
        opts.extend(c.args);
        c.args = opts;
        Ok(c)
    }
    fn channel_command(&self) -> Command {
        let mut cmd = Command::new("ssh");
        if self.profile.ssh_ignore_config { cmd.args(["-F", "/dev/null"]); }
        cmd.args(["-S"])
            .arg(self.socket())
            .args([
                "-o",
                "ControlMaster=no",
                "-o",
                "BatchMode=yes",
                "-o",
                "ProxyCommand=false",
                "-p",
            ])
            .arg(self.profile.port.to_string());
        if !self.profile.username.is_empty() {
            cmd.arg("-l").arg(&self.profile.username);
        }
        cmd.kill_on_drop(true);
        cmd
    }
    pub(crate) async fn forward_control(&self, operation: &str, arguments: &[String]) -> Result<String, String> {
        let mut cmd = self.channel_command();
        cmd.args(["-F", "/dev/null", "-o", "PermitLocalCommand=no", "-o", "ExitOnForwardFailure=yes", "-O", operation])
            .args(arguments).arg("--").arg(&self.profile.host)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped());
        let mut child=cmd.spawn().map_err(|e|e.to_string())?;
        let operation=async {
            let mut bytes=Vec::new();
            child.stderr.take().unwrap().take(16385).read_to_end(&mut bytes).await.map_err(|e|e.to_string())?;
            let status=child.wait().await.map_err(|e|e.to_string())?;
            let detail=String::from_utf8_lossy(&bytes[..bytes.len().min(16384)]).into_owned();
            if status.success(){Ok(detail)}else{Err(format!("SSH {:?}: {detail}",status.code()))}
        };
        tokio::time::timeout(Duration::from_secs(5),operation).await.map_err(|_|"SSH no confirmó la operación en cinco segundos".to_string())?
    }
    pub async fn ready(&self) -> Result<(), String> {
        for _ in 0..60 {
            if self.closed.load(Ordering::Relaxed) {
                return Err("Sesión desconectada".into());
            }
            let mut cmd = self.channel_command();
            let status = tokio::time::timeout(
                Duration::from_secs(2),
                cmd.args(["-O", "check", "--"])
                    .arg(&self.profile.host)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status(),
            )
            .await;
            if matches!(status,Ok(Ok(s)) if s.success()) {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Err("La conexión SSH aún no está autenticada. Completa la autenticación y vuelve a intentarlo.".into())
    }
    pub async fn sftp(&self) -> Result<SftpChannel, String> {
        self.ready().await?;
        let mut child = self
            .channel_command()
            .args(["-s", "--"])
            .arg(&self.profile.host)
            .arg("sftp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let input = child.stdin.take().ok_or("SFTP sin stdin")?;
        let output = child.stdout.take().ok_or("SFTP sin stdout")?;
        let sftp = tokio::time::timeout(
            Duration::from_secs(15),
            Sftp::new(input, output, SftpOptions::default()),
        )
        .await
        .map_err(|_| "SFTP: timeout")?
        .map_err(|e| e.to_string())?;
        Ok(SftpChannel { sftp, child })
    }
    pub(crate) fn execution_command(&self,script:&str,directory:&Path)->Command{
        let mut command=self.channel_command();
        command.arg("--").arg(&self.profile.host).arg(format!("cd {} && exec /bin/sh -c {}",quote(&directory.to_string_lossy()),quote(script)));
        command
    }
    pub async fn capture(&self, script: &str) -> Result<String, String> {
        self.ready().await?;
        let mut child = self
            .channel_command()
            .arg("--")
            .arg(&self.profile.host)
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut output = Vec::new();
        let mut stdout = child.stdout.take().ok_or("SSH sin stdout")?.take(65537);
        let read = async {
            stdout.read_to_end(&mut output).await?;
            child.wait().await
        };
        let status = tokio::time::timeout(Duration::from_secs(10), read)
            .await
            .map_err(|_| "Consulta SSH: timeout")?
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("La consulta remota falló".into());
        }
        if output.len() > 65536 {
            return Err("Respuesta remota superior a 64 KiB".into());
        }
        String::from_utf8(output).map_err(|_| "Respuesta no UTF-8".into())
    }
}
pub struct SftpChannel {
    pub sftp: Sftp,
    child: Child,
}
impl SftpChannel {
    pub async fn close(mut self) {
        let _ = self.sftp.close().await;
        let _ = self.child.wait().await;
    }
}
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
pub fn join_remote(parent: &Path, name: &Path) -> Result<PathBuf, String> {
    if name.components().count() != 1
        || name == Path::new(".")
        || name == Path::new("..")
        || name.is_absolute()
    {
        return Err("Nombre remoto inválido".into());
    }
    Ok(parent.join(name))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_quoting() {
        assert_eq!(quote("a'b"), "'a'\\''b'");
    }
    #[test]
    fn path_traversal() {
        assert!(join_remote(Path::new("/safe"), Path::new("../bad")).is_err());
        assert_eq!(
            join_remote(Path::new("/safe"), Path::new("á file")).unwrap(),
            PathBuf::from("/safe/á file")
        );
    }
}

#[test]
fn imported_multiplexed_channels_also_ignore_external_configuration() {
    let c=SshConnection::new(Profile{name:"imported".into(),host:"example.invalid".into(),port:22,ssh_ignore_config:true,..Default::default()}).unwrap();
    let command=c.channel_command();
    let args=command.as_std().get_args().map(|s|s.to_string_lossy().into_owned()).collect::<Vec<_>>();
    assert!(args.windows(2).any(|a|a==["-F","/dev/null"]));
}
