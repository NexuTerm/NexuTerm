//! Bounded static OpenSSH import. No ssh -G, Include traversal or Match execution.
use crate::{config::Profile, connection};
use std::{collections::HashMap, io::Read, path::Path};
const LIMIT: usize = 1024 * 1024;
pub struct Preview {
    pub profiles: Vec<Profile>,
    pub warnings: Vec<String>,
}
struct Block {
    patterns: Vec<String>,
    entries: Vec<(String, String)>,
}
fn words(text: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escape = false;
    for c in text.chars() {
        if escape {
            current.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                current.push(c);
            }
            continue;
        }
        match c {
            '\'' | '"' => quote = Some(c),
            '#' => break,
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(c),
        }
    }
    if quote.is_some() || escape {
        return Err("Comillas o escape sin cerrar".into());
    }
    if !current.is_empty() {
        result.push(current);
    }
    Ok(result)
}
fn glob(pattern: &str, value: &str) -> bool {
    let (p, v) = (pattern.as_bytes(), value.as_bytes());
    let (mut i, mut j, mut star, mut back) = (0, 0, None, 0);
    while j < v.len() {
        if i < p.len() && (p[i] == b'?' || p[i].eq_ignore_ascii_case(&v[j])) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == b'*' {
            star = Some(i);
            i += 1;
            back = j;
        } else if let Some(s) = star {
            back += 1;
            j = back;
            i = s + 1;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == b'*' {
        i += 1;
    }
    i == p.len()
}
fn applies(patterns: &[String], alias: &str) -> bool {
    !patterns
        .iter()
        .any(|p| p.strip_prefix('!').is_some_and(|p| glob(p, alias)))
        && patterns
            .iter()
            .any(|p| !p.starts_with('!') && glob(p, alias))
}
pub fn read(path: &Path, home: &Path) -> Result<Preview, String> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Selecciona un archivo regular".into());
    }
    let mut text = String::new();
    file.take((LIMIT + 1) as u64)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    parse(&text, home)
}
pub fn parse(text: &str, home: &Path) -> Result<Preview, String> {
    if text.len() > LIMIT
        || text
            .chars()
            .any(|c| c.is_control() && !"\n\r\t".contains(c))
    {
        return Err("Config SSH fuera de límites: máximo 1 MiB, texto UTF-8 sin controles".into());
    }
    let mut blocks = vec![Block {
        patterns: vec!["*".into()],
        entries: Vec::new(),
    }];
    let (mut aliases, mut warnings) = (Vec::<String>::new(), Vec::new());
    let mut conditional = false;
    for (index, line) in text.lines().enumerate() {
        if index >= 8192 || line.len() > 16384 {
            return Err("Config SSH: máximo 8192 líneas de 16 KiB".into());
        }
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let end = line
            .find(|c: char| c.is_whitespace() || c == '=')
            .unwrap_or(line.len());
        let key = line[..end].to_ascii_lowercase();
        let rest = line[end..]
            .trim_start()
            .strip_prefix('=')
            .unwrap_or(line[end..].trim_start())
            .trim_start();
        let args = words(rest).map_err(|e| format!("Línea {}: {e}", index + 1))?;
        if key == "host" {
            conditional = false;
            if args.is_empty() {
                return Err(format!("Host vacío en línea {}", index + 1));
            }
            for alias in &args {
                if !alias.contains(['*', '?', '!'])
                    && !aliases.iter().any(|a| a.eq_ignore_ascii_case(alias))
                {
                    aliases.push(alias.clone());
                }
            }
            if aliases.len() > 256 {
                return Err("Máximo 256 alias explícitos por importación".into());
            }
            blocks.push(Block {
                patterns: args,
                entries: Vec::new(),
            });
            continue;
        }
        if key == "match" {
            conditional = true;
        }
        if conditional
            || !matches!(key.as_str(), "hostname" | "user" | "port" | "identityfile")
            || args.len() != 1
        {
            if warnings.len() < 128 {
                warnings.push(format!(
                    "Línea {}: {} omitido; no se evalúa ni ejecuta.",
                    index + 1,
                    key
                ));
            }
            continue;
        }
        blocks
            .last_mut()
            .unwrap()
            .entries
            .push((key, args[0].clone()));
    }
    let mut profiles = Vec::new();
    for alias in aliases {
        let mut values = HashMap::new();
        for b in &blocks {
            if applies(&b.patterns, &alias) {
                for (k, v) in &b.entries {
                    values.entry(k.as_str()).or_insert(v.as_str());
                }
            }
        }
        let result = (|| {
            let host = values
                .get("hostname")
                .copied()
                .unwrap_or(&alias)
                .replace("%h", &alias);
            let username = values.get("user").copied().unwrap_or("").to_owned();
            if host.contains(['%', '$']) || username.contains(['%', '$']) {
                return Err("Host/usuario con expansiones no compatibles".into());
            }
            let identity = values.get("identityfile").copied().unwrap_or("");
            let identity = if let Some(rest) = identity.strip_prefix("~/") {
                home.join(rest).to_string_lossy().into_owned()
            } else {
                identity.into()
            };
            if !identity.is_empty()
                && (!Path::new(&identity).is_absolute()
                    || identity.contains(['%', '$'])
                    || identity.chars().any(char::is_control))
            {
                return Err("IdentityFile requiere ruta absoluta o ~/ sin expansiones".into());
            }
            let port = values
                .get("port")
                .copied()
                .unwrap_or("22")
                .parse::<u16>()
                .map_err(|_| "Puerto inválido")?;
            let p = Profile {
                id: uuid::Uuid::new_v4().to_string(),
                name: alias.clone(),
                host,
                port,
                username,
                identity,
                ssh_ignore_config: true,
                group: "Importados OpenSSH".into(),
                ..Default::default()
            };
            connection::validate(&p)?;
            Ok::<_, String>(p)
        })();
        match result {
            Ok(p) => profiles.push(p),
            Err(e) => {
                if warnings.len() < 128 {
                    warnings.push(format!("Alias {alias}: omitido ({e})."));
                }
            }
        }
    }
    if warnings.len() == 128 {
        warnings.push("Lista de avisos acotada: revisa el archivo original; puede haber más directivas omitidas.".into());
    }
    Ok(Preview { profiles, warnings })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn precedence_patterns_quotes_and_no_execution() {
        let p=parse("Host first second\n HostName example.invalid\n User \"demo user\"\nHost bad\n ProxyCommand touch /tmp/never\nHost * !bad\n User demo\n Port=2222\nHost second\n Port 23\nInclude other\nMatch exec touch /tmp/never\n HostName ignored\n",Path::new("/home/demo")).unwrap();
        // The invalid first User wins, so these aliases cannot silently fall back to demo.
        assert_eq!(p.profiles.len(), 1);
        assert_eq!(p.profiles[0].name, "bad");
        assert_eq!(p.profiles[0].port, 22);
        assert!(p.profiles[0].ssh_ignore_config);
        assert!(p.warnings.iter().any(|s| s.contains("proxycommand")));
        assert!(p.warnings.iter().any(|s| s.contains("match")));
    }
    #[test]
    fn supported_fields_first_value_and_identity() {
        let p=parse("Host one two\nHostName server.invalid\nUser demo\nIdentityFile \"~/key with spaces\"\nHost *\nPort 2222\nUser ignored\nHost two\nPort 99",Path::new("/home/demo")).unwrap();
        assert_eq!(p.profiles.len(), 2);
        for p in p.profiles {
            assert_eq!(p.port, 2222);
            assert_eq!(p.username, "demo");
            assert_eq!(p.identity, "/home/demo/key with spaces");
        }
    }
    #[test]
    fn rejects_limits_and_malformed_without_partial_import() {
        assert!(parse("Host \"oops", Path::new("/")).is_err());
        assert!(parse(&"x".repeat(LIMIT + 1), Path::new("/")).is_err());
        let p = parse(
            "Host only*\nUser demo\nHost bad\nPort 0\nHost good\nPort 22",
            Path::new("/"),
        )
        .unwrap();
        assert_eq!(p.profiles.len(), 1);
        assert_eq!(p.profiles[0].name, "good");
    }
}
