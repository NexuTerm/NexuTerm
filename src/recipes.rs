//! Read-only Linux diagnostics compiled into reviewable supervised shell commands.
use crate::ssh::quote;

#[derive(Clone, Debug)]
pub enum Recipe {
    Disk {
        path: String,
        max_used_percent: u8,
    },
    Service {
        unit: String,
    },
    Journal {
        unit: String,
        lines: u32,
        minutes: u32,
    },
}
fn service(unit: &str) -> Result<(), String> {
    if unit.len() > 255
        || !unit.ends_with(".service")
        || unit.len() <= 8
        || !unit
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !unit
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.@:-".contains(&c))
    {
        return Err("Indica una unidad .service de hasta 255 caracteres (letras ASCII, números, _, ., @, : y -).".into());
    }
    Ok(())
}
impl Recipe {
    pub fn command(&self) -> Result<String, String> {
        let script = match self {
            Self::Disk {
                path,
                max_used_percent,
            } => {
                if !std::path::Path::new(path).is_absolute()
                    || path.len() > 4096
                    || path.chars().any(char::is_control)
                    || !(1..=100).contains(max_used_percent)
                {
                    return Err("Indica una ruta absoluta UTF-8 sin controles (hasta 4096 bytes) y un umbral entre 1 y 100 %.".into());
                }
                format!(
                    r#"# NexuTerm: espacio en disco (atención si uso >= {max_used_percent} %)
export LC_ALL=C
command -v df >/dev/null 2>&1 || {{ printf '%s\n' 'No se encuentra df' >&2; exit 127; }}
df -Pk -- {path} || exit 1
usage=$(df --output=pcent -- {path}) || exit 1
set -f
set -- $usage
[ "$#" -eq 2 ] || {{ printf '%s\n' 'No se pudo verificar el porcentaje de uso' >&2; exit 1; }}
used=${{2%\%}}
case "$used" in [0-9]|[0-9][0-9]|[0-9][0-9][0-9]) ;; *) printf '%s\n' 'Porcentaje de uso inválido' >&2; exit 1;; esac
if [ "$used" -ge {max_used_percent} ]; then
    printf 'VERIFICACION: ATENCION; uso %s%%, umbral {max_used_percent}%%\n' "$used"
    exit 2
fi
printf 'VERIFICACION: CORRECTA; uso %s%%, umbral {max_used_percent}%%\n' "$used"
"#,
                    path = quote(path)
                )
            }
            Self::Service { unit } => {
                service(unit)?;
                format!(
                    r#"# NexuTerm: estado de servicio (sin cambios ni reinicios)
export LC_ALL=C
command -v systemctl >/dev/null 2>&1 || {{ printf '%s\n' 'No se encuentra systemctl' >&2; exit 127; }}
properties=$(systemctl show --no-pager --property=LoadState --property=ActiveState --property=SubState --property=Result -- {unit}) || exit 1
printf '%s\n' "$properties"
case "
$properties
" in *'
LoadState=loaded
'*) ;; *) printf '%s\n' 'VERIFICACION: NO COMPLETADA; unidad no cargada' >&2; exit 1;; esac
systemctl is-active --quiet -- {unit}
code=$?
if [ "$code" -eq 0 ]; then
    printf '%s\n' 'VERIFICACION: CORRECTA; servicio activo'
    exit 0
fi
if [ "$code" -eq 3 ]; then
    printf '%s\n' 'VERIFICACION: ATENCION; servicio no activo'
    exit 3
fi
printf '%s\n' 'VERIFICACION: NO COMPLETADA; no se confirmó el estado' >&2
exit 1
"#,
                    unit = quote(unit)
                )
            }
            Self::Journal {
                unit,
                lines,
                minutes,
            } => {
                service(unit)?;
                if !(1..=5000).contains(lines) || !(1..=10080).contains(minutes) {
                    return Err(
                        "Limita el journal a 1–5000 líneas y 1–10080 minutos (siete días).".into(),
                    );
                }
                format!(
                    r#"# NexuTerm: registros de {unit}; últimas {lines} líneas en {minutes} minutos
export LC_ALL=C
command -v journalctl >/dev/null 2>&1 || {{ printf '%s\n' 'No se encuentra journalctl' >&2; exit 127; }}
journalctl --no-pager --output=short-iso --lines={lines} --since='-{minutes} minutes' --unit={unit}
code=$?
[ "$code" -eq 0 ] || {{ printf '%s\n' 'VERIFICACION: NO COMPLETADA; falló journalctl' >&2; exit 1; }}
printf '%s\n' 'VERIFICACION: CONSULTA COMPLETADA; solo registros accesibles para el usuario, puede no haber entradas'
"#,
                    unit = quote(unit)
                )
            }
        };
        if script.len() > 16384 {
            return Err("El comando generado supera 16 KiB; reduce la longitud de la ruta.".into());
        }
        Ok(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn run(recipe: Recipe, programs: &[(&str, &str)]) -> std::process::Output {
        let directory = tempfile::tempdir().unwrap();
        for (name, script) in programs {
            let path = directory.path().join(name);
            std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        std::process::Command::new("/bin/sh")
            .args(["-c", &recipe.command().unwrap()])
            .env("PATH", directory.path())
            .output()
            .unwrap()
    }
    #[test]
    fn rejects_invalid_paths_units_and_limits() {
        for path in ["relative", "/tmp\nother", "/tmp\0"] {
            assert!(
                Recipe::Disk {
                    path: path.into(),
                    max_used_percent: 80
                }
                .command()
                .is_err()
            );
        }
        assert!(
            Recipe::Disk {
                path: "/".into(),
                max_used_percent: 0
            }
            .command()
            .is_err()
        );
        for unit in [
            "-x.service",
            "a;touch x.service",
            "foo.service\n",
            "../a.service",
            "a.socket",
            ".service",
            "*.service",
        ] {
            assert!(Recipe::Service { unit: unit.into() }.command().is_err());
        }
        assert!(
            Recipe::Service {
                unit: "worker@instance-1.service".into()
            }
            .command()
            .is_ok()
        );
        for (lines, minutes) in [(0, 1), (5001, 1), (1, 0), (1, 10081)] {
            assert!(
                Recipe::Journal {
                    unit: "a.service".into(),
                    lines,
                    minutes
                }
                .command()
                .is_err()
            );
        }
    }
    #[test]
    fn disk_preserves_literal_path_and_checks_threshold_and_bad_output() {
        let recipe = || Recipe::Disk {
            path: "/space ' $(touch injected)".into(),
            max_used_percent: 80,
        };
        let df = format!(
            "[ \"$3\" = {} ] || exit 9\ncase \"$1\" in -Pk) printf 'evidence\\n';; *) printf 'Use%%\\n 79%%\\n';; esac",
            quote("/space ' $(touch injected)")
        );
        let result = run(recipe(), &[("df", &df)]);
        assert!(result.status.success(), "{:?}", result);
        assert!(String::from_utf8_lossy(&result.stdout).contains("CORRECTA"));
        assert_eq!(
            run(recipe(), &[("df", &df.replace("79%", "80%"))])
                .status
                .code(),
            Some(2)
        );
        assert_eq!(
            run(recipe(), &[("df", &df.replace("79%", "oops"))])
                .status
                .code(),
            Some(1)
        );
        assert_eq!(run(recipe(), &[("df", "exit 4")]).status.code(), Some(1));
        assert_eq!(run(recipe(), &[]).status.code(), Some(127));
    }
    #[test]
    fn service_distinguishes_active_inactive_missing_and_transport_errors() {
        let recipe = || Recipe::Service {
            unit: "demo.service".into(),
        };
        let systemctl = "case \"$1\" in show) printf 'LoadState=loaded\\nActiveState=active\\n';; is-active) exit 0;; *) exit 9;; esac";
        assert!(run(recipe(), &[("systemctl", systemctl)]).status.success());
        assert_eq!(
            run(
                recipe(),
                &[("systemctl", &systemctl.replace("exit 0", "exit 3"))]
            )
            .status
            .code(),
            Some(3)
        );
        assert_eq!(
            run(
                recipe(),
                &[(
                    "systemctl",
                    &systemctl.replace("LoadState=loaded", "LoadState=not-found")
                )]
            )
            .status
            .code(),
            Some(1)
        );
        assert_eq!(
            run(
                recipe(),
                &[("systemctl", &systemctl.replace("exit 0", "exit 4"))]
            )
            .status
            .code(),
            Some(1)
        );
        assert_eq!(run(recipe(), &[]).status.code(), Some(127));
    }
    #[test]
    fn journal_is_bounded_and_does_not_claim_completeness_or_service_health() {
        let recipe = || Recipe::Journal {
            unit: "demo.service".into(),
            lines: 123,
            minutes: 45,
        };
        let journal = "[ \"$3\" = '--lines=123' ] && [ \"$4\" = '--since=-45 minutes' ] && [ \"$5\" = '--unit=demo.service' ] || exit 9\nprintf 'sample log\\n'";
        let result = run(recipe(), &[("journalctl", journal)]);
        assert!(result.status.success(), "{:?}", result);
        assert!(String::from_utf8_lossy(&result.stdout).contains("solo registros accesibles"));
        assert_eq!(
            run(recipe(), &[("journalctl", "exit 1")]).status.code(),
            Some(1)
        );
        assert_eq!(run(recipe(), &[]).status.code(), Some(127));
    }
}
