use crate::config::{Profile, Protocol};
use std::{env, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub fn validate(p: &Profile) -> Result<(), String> {
    if p.name.trim().is_empty() || p.name.len() > 120 || p.name.chars().any(char::is_control) {
        return Err("El nombre debe tener entre 1 y 120 bytes, sin caracteres de control".into());
    }
    if p.host.is_empty()
        || p.host.starts_with('-')
        || p.host
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "@/\\[]?#".contains(c))
    {
        return Err("Host inválido: usa DNS, IPv4 o IPv6 sin corchetes ni opciones".into());
    }
    if p.port == 0 {
        return Err("Puerto fuera del rango 1–65535".into());
    }
    if p.username.chars().any(char::is_control)
        || (p.protocol != Protocol::Ftp
            && (p.username.starts_with('-')
                || p.username
                    .chars()
                    .any(|c| c.is_whitespace() || "@/\\".contains(c))))
    {
        return Err("Usuario inválido".into());
    }
    if matches!(p.protocol, Protocol::Ssh | Protocol::Sftp) && p.identity.contains('\0') {
        return Err("Clave SSH inválida".into());
    }
    if p.protocol == Protocol::Rdp && p.domain.chars().any(char::is_control) {
        return Err("Dominio RDP inválido".into());
    }
    if matches!(p.protocol, Protocol::Ssh | Protocol::Sftp)
        && !p.jump_host.is_empty()
        && (p.jump_host.starts_with('-')
            || p.jump_host
                .chars()
                .any(|c| c.is_whitespace() || c.is_control()))
    {
        return Err("Salto SSH inválido".into());
    }
    Ok(())
}

pub fn find_program(names: &[&str]) -> Option<String> {
    let path = env::var_os("PATH")?;
    for name in names {
        for dir in env::split_paths(&path) {
            if !dir.is_absolute() {
                continue;
            }
            let p: PathBuf = dir.join(name);
            use std::os::unix::fs::PermissionsExt;
            if let Ok(m) = p.metadata()
                && m.is_file()
                && m.permissions().mode() & 0o111 != 0
            {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }
    None
}

pub fn command(p: &Profile) -> Result<CommandSpec, String> {
    command_with_lookup(p, find_program)
}

fn command_with_lookup(
    p: &Profile,
    lookup: impl FnOnce(&[&str]) -> Option<String>,
) -> Result<CommandSpec, String> {
    validate(p)?;
    let (candidates, args): (&[&str], Vec<String>) = match p.protocol {
        Protocol::Ssh | Protocol::Sftp => {
            let mut a = vec![
                "-p".into(),
                p.port.to_string(),
                "-o".into(),
                "ServerAliveInterval=30".into(),
                "-o".into(),
                "ServerAliveCountMax=3".into(),
            ];
            if p.ssh_ignore_config { a.extend(["-F".into(), "/dev/null".into()]); }
            if !p.username.is_empty() {
                a.extend(["-l".into(), p.username.clone()]);
            }
            if p.ssh_password {
                a.extend(["-o".into(),"PreferredAuthentications=password,keyboard-interactive".into(),"-o".into(),"PubkeyAuthentication=no".into()]);
            }
            if !p.identity.is_empty() && !p.ssh_password {
                a.extend(["-i".into(), p.identity.clone()]);
            }
            if !p.jump_host.is_empty() {
                a.extend(["-J".into(), p.jump_host.clone()]);
            }
            if p.protocol == Protocol::Sftp {
                a.push("-N".into());
            }
            a.extend(["--".into(), p.host.clone()]);
            (&["ssh"], a)
        }
        Protocol::Telnet => (&["telnet"], vec![p.host.clone(), p.port.to_string()]),
        Protocol::Ftp => return Err("FTP se abre en el explorador gráfico de NexuTerm".into()),
        Protocol::Rdp => return Err("RDP se abre en el visor integrado de NexuTerm".into()),
        Protocol::Vnc => return Err("VNC se abre en el visor integrado de NexuTerm".into()),
    };
    let program = lookup(candidates).ok_or_else(|| {
        format!(
            "Falta el programa {}. Consulta README.md para instalarlo.",
            candidates.join(" / ")
        )
    })?;
    Ok(CommandSpec { program, args })
}

pub fn sftp_uri(p: &Profile) -> Result<String, String> {
    validate(p)?;
    if !matches!(p.protocol, Protocol::Ssh | Protocol::Sftp) {
        return Err("SFTP requiere una sesión SSH".into());
    }
    let host = if p.host.contains(':') {
        format!("[{}]", p.host)
    } else {
        p.host.clone()
    };
    let mut u =
        url::Url::parse(&format!("sftp://{host}:{}/", p.port)).map_err(|e| e.to_string())?;
    u.set_username(&p.username)
        .map_err(|_| "Usuario no válido en URI SFTP")?;
    Ok(u.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> Profile {
        Profile {
            name: "test".into(),
            host: "example.invalid".into(),
            port: 22,
            ..Profile::default()
        }
    }
    #[test]
    fn ssh_password_mode_avoids_keys_and_never_stores_password_arguments(){
        let mut p=profile();p.ssh_password=true;p.identity="/tmp/ignored-key".into();
        let spec=command_with_lookup(&p, |_|Some("ssh".into())).unwrap();
        assert!(spec.args.iter().any(|s|s=="PreferredAuthentications=password,keyboard-interactive"));
        assert!(spec.args.iter().any(|s|s=="PubkeyAuthentication=no"));assert!(!spec.args.iter().any(|s|s=="-i"||s=="/tmp/ignored-key"));
    }
    #[test]
    fn rejects_option_hosts() {
        let mut p = profile();
        p.host = "-oProxyCommand=id".into();
        assert!(validate(&p).is_err());
    }
    #[test]
    fn ftp_usernames_allow_email_and_domain_without_control_characters() {
        let mut p = profile();
        p.protocol = Protocol::Ftp;
        p.username = "domain\\demo@example.invalid".into();
        assert!(validate(&p).is_ok());
        p.username.push('\n');
        assert!(validate(&p).is_err());
        p.username.pop();
        p.protocol = Protocol::Ssh;
        assert!(validate(&p).is_err());
    }
    #[test]
    fn basic_protocols_use_separate_arguments_and_no_shell() {
        let p = Profile {
            protocol: Protocol::Telnet,
            port: 23,
            host: "::1".into(),
            ..profile()
        };
        let command = command_with_lookup(&p, |candidates| {
            assert_eq!(candidates, ["telnet"]);
            Some("/test/telnet".into())
        })
        .unwrap();
        assert_eq!(command.args, ["::1", "23"]);
        let p = Profile {
            protocol: Protocol::Sftp,
            port: 2200,
            ..profile()
        };
        let command = command_with_lookup(&p, |_| Some("/test/ssh".into())).unwrap();
        assert!(command.args.contains(&"-N".into()));
        assert!(command.args.contains(&"2200".into()));
        assert!(sftp_uri(&p).unwrap().starts_with("sftp://"));
    }
    #[test]
    fn validates_only_fields_used_by_protocol() {
        let mut p = profile();
        p.identity = "invalid\0key".into();
        p.jump_host = "-invalid".into();
        p.domain = "invalid\ndomain".into();
        assert!(validate(&p).is_err());
        p.protocol = Protocol::Rdp;
        assert!(validate(&p).is_err());
        p.domain.clear();
        assert!(validate(&p).is_ok());
        p.protocol = Protocol::Vnc;
        p.domain = "invalid\ndomain".into();
        assert!(validate(&p).is_ok());
    }
    #[test]
    fn ipv6_sftp_and_escaped_user() {
        let mut p = profile();
        p.host = "::1".into();
        p.username = "test+user".into();
        assert_eq!(sftp_uri(&p).unwrap(), "sftp://test+user@[::1]:22/");
    }
    #[test]
    fn secrets_never_in_ssh_arguments() {
        let s = command(&profile()).unwrap();
        assert!(!s.args.iter().any(|a| a.contains("password")));
        assert_eq!(s.args.last().unwrap(), "example.invalid");
    }
    #[test]
    fn no_shell_interpolation() {
        let mut p = profile();
        p.identity = "/tmp/key $(touch dangerous)".into();
        let s = command(&p).unwrap();
        assert!(s.args.contains(&p.identity));
        assert_ne!(s.program, "sh");
    }

    #[test]
    fn ftp_never_launches_a_console_client() {
        let p = Profile {
            protocol: Protocol::Ftp,
            ..profile()
        };
        assert!(
            command_with_lookup(&p, |_| panic!("FTP must use the graphical browser"))
                .unwrap_err()
                .contains("explorador gráfico")
        );
    }
    #[test]
    fn vnc_never_launches_an_external_viewer() {
        let p = Profile {
            protocol: Protocol::Vnc,
            ..profile()
        };
        let error = command_with_lookup(&p, |_| panic!("VNC must not look up external clients"))
            .unwrap_err();
        assert!(error.contains("visor integrado"));
    }

    #[test]
    fn rdp_never_launches_an_external_viewer() {
        let p = Profile {
            protocol: Protocol::Rdp,
            ..profile()
        };
        let error = command_with_lookup(&p, |_| panic!("RDP must not look up external clients"))
            .unwrap_err();
        assert!(error.contains("visor integrado"));
    }
}

#[test]
fn imported_profile_uses_only_reviewed_fields() {
    let profile=Profile{name:"imported".into(),host:"example.invalid".into(),port:22,ssh_ignore_config:true,..Default::default()};
    let command=command_with_lookup(&profile, |_|Some("ssh".into())).unwrap();
    assert!(command.args.windows(2).any(|a|a==["-F","/dev/null"]));
    assert!(!Profile::default().ssh_ignore_config);
}
