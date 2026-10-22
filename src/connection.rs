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
    if p.username.starts_with('-')
        || p.username
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || "@/\\".contains(c))
    {
        return Err("Usuario inválido".into());
    }
    if p.identity.contains('\0') || p.domain.chars().any(char::is_control) {
        return Err("Identidad o dominio inválido".into());
    }
    if !p.jump_host.is_empty()
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
    let ipv6_host = if p.host.contains(':') {
        format!("[{}]", p.host)
    } else {
        p.host.clone()
    };
    let (candidates, args): (&[&str], Vec<String>) = match p.protocol {
        Protocol::Ssh => {
            let mut a = vec![
                "-p".into(),
                p.port.to_string(),
                "-o".into(),
                "ServerAliveInterval=30".into(),
                "-o".into(),
                "ServerAliveCountMax=3".into(),
            ];
            if !p.username.is_empty() {
                a.extend(["-l".into(), p.username.clone()]);
            }
            if !p.identity.is_empty() {
                a.extend(["-i".into(), p.identity.clone()]);
            }
            if !p.jump_host.is_empty() {
                a.extend(["-J".into(), p.jump_host.clone()]);
            }
            a.extend(["--".into(), p.host.clone()]);
            (&["ssh"], a)
        }
        Protocol::Rdp => {
            let mut a = vec![
                format!("/v:{ipv6_host}:{}", p.port),
                "/dynamic-resolution".into(),
                "+clipboard".into(),
            ];
            if !p.username.is_empty() {
                a.push(format!("/u:{}", p.username));
            }
            if !p.domain.is_empty() {
                a.push(format!("/d:{}", p.domain));
            }
            (&["xfreerdp3", "xfreerdp"], a)
        }
        Protocol::Vnc => (
            &["vncviewer", "xtigervncviewer"],
            vec![format!("{ipv6_host}::{}", p.port)],
        ),
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
    if p.protocol != Protocol::Ssh {
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
    fn rejects_option_hosts() {
        let mut p = profile();
        p.host = "-oProxyCommand=id".into();
        assert!(validate(&p).is_err());
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
    fn desktop_clients_keep_security_defaults_and_explicit_ports() {
        let mut p = profile();
        p.protocol = Protocol::Rdp;
        p.host = "::1".into();
        p.port = 3391;
        p.username = "testuser".into();
        p.domain = "LAB".into();
        let command = command_with_lookup(&p, |candidates| {
            assert_eq!(candidates, ["xfreerdp3", "xfreerdp"]);
            Some("/test/xfreerdp3".into())
        })
        .unwrap();
        assert!(command.args.contains(&"/v:[::1]:3391".into()));
        assert!(command.args.contains(&"/u:testuser".into()));
        assert!(
            !command
                .args
                .iter()
                .any(|a| a.contains("cert:ignore") || a.starts_with("/p:"))
        );
        p.protocol = Protocol::Vnc;
        p.port = 5907;
        let vnc = command_with_lookup(&p, |candidates| {
            assert_eq!(candidates, ["vncviewer", "xtigervncviewer"]);
            Some("/test/vncviewer".into())
        })
        .unwrap();
        assert_eq!(vnc.args, vec!["[::1]::5907"]);
    }

    #[test]
    fn missing_desktop_client_reports_candidates() {
        let mut p = profile();
        p.protocol = Protocol::Rdp;
        let error = command_with_lookup(&p, |_| None).unwrap_err();
        assert!(error.contains("xfreerdp3 / xfreerdp"));
    }
}
