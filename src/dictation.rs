//! Local-only voice input. Audio is temporary and never sent to a service.
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{io::AsyncReadExt, process::Command};
/// Resolve the optional project-local engine without overwriting explicit settings.
pub fn resolve_engine(program: &str, model: &str) -> (String, String) {
    if program == "whisper-cli" && model.is_empty()
        && let Ok(executable) = std::env::current_exe()
            && let Some(folder) = executable.parent() {
                for ancestor in folder.ancestors().take(4) {
                    if let Some(engine) = local_engine(&ancestor.join(".local/whisper")) {
                        return engine;
                    }
                }
            }
    (program.to_owned(), model.to_owned())
}
fn local_engine(root: &std::path::Path) -> Option<(String, String)> {
    let program = root.join("bin/whisper-cli");
    let model = root.join("models/ggml-tiny.bin");
    if program.is_file() && model.is_file() {
        Some((
            program.to_string_lossy().into_owned(),
            model.to_string_lossy().into_owned(),
        ))
    } else {
        None
    }
}
struct Recording(PathBuf);
impl Drop for Recording {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
pub async fn transcribe(
    program: String,
    model: String,
    language: String,
    stop: Arc<AtomicBool>,
) -> Result<String, String> {
    transcribe_with("pw-record", program, model, language, stop).await
}
async fn transcribe_with(
    recorder_program: &str,
    program: String,
    model: String,
    language: String,
    stop: Arc<AtomicBool>,
) -> Result<String, String> {
    use std::os::unix::fs::DirBuilderExt;
    let directory = std::env::temp_dir().join(format!("nexuterm-voice-{}", uuid::Uuid::new_v4()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&directory)
        .map_err(|_| "No se pudo crear el audio temporal")?;
    let recording = Recording(directory);
    let audio = recording.0.join("voice.wav");
    let mut recorder = Command::new(recorder_program)
        .args(["--rate", "16000", "--channels", "1", "--format", "s16"])
        .arg(&audio)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "No se encuentra pw-record para grabar el micrófono")?;
    let start = tokio::time::Instant::now();
    loop {
        if stop.load(Ordering::Relaxed) || start.elapsed() >= Duration::from_secs(60) {
            break;
        }
        if recorder
            .try_wait()
            .map_err(|_| "No se pudo consultar el micrófono")?
            .is_some()
        {
            return Err(
                "No se pudo grabar el micrófono. Comprueba PipeWire y sus permisos.".into(),
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if let Some(pid) = recorder.id() {
        unsafe {
            libc::kill(pid as i32, libc::SIGINT);
        }
    }
    tokio::time::timeout(Duration::from_secs(3), recorder.wait())
        .await
        .map_err(|_| "El micrófono no respondió al detenerlo")?
        .map_err(|_| "No se pudo cerrar el micrófono")?;
    let size = std::fs::metadata(&audio)
        .map_err(|_| "No se grabó audio")?
        .len();
    if !(44..=3_000_000).contains(&size) {
        return Err("El audio está vacío o supera el límite de un minuto".into());
    }
    let mut child = Command::new(program)
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(&audio)
        .args(["-nt", "-np", "-l", &language])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "No se pudo iniciar whisper-cli")?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or("No se pudo leer la transcripción")?
        .take(32769);
    let mut bytes = Vec::new();
    let operation = async {
        stdout
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| "Error leyendo el dictado")?;
        if bytes.len() > 32768 {
            return Err("Transcripción demasiado grande".into());
        }
        let status = child
            .wait()
            .await
            .map_err(|_| "Error al terminar el dictado")?;
        Ok::<_, String>(status.success())
    };
    let success = tokio::time::timeout(Duration::from_secs(120), operation)
        .await
        .map_err(|_| "El dictado agotó su tiempo")??;
    if !success || bytes.len() > 32768 {
        return Err("La transcripción falló o supera el límite permitido".into());
    }
    let text = String::from_utf8(bytes).map_err(|_| "Transcripción con codificación inválida")?;
    if text.trim().is_empty() {
        return Err("No se reconoció voz".into());
    }
    Ok(text.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_engine_requires_both_files_and_preserves_explicit_settings() {
        let root = tempfile::tempdir().unwrap();
        assert!(local_engine(root.path()).is_none());
        std::fs::create_dir(root.path().join("bin")).unwrap();
        std::fs::create_dir(root.path().join("models")).unwrap();
        std::fs::write(root.path().join("bin/whisper-cli"), "").unwrap();
        assert!(local_engine(root.path()).is_none());
        std::fs::write(root.path().join("models/ggml-tiny.bin"), "").unwrap();
        assert!(local_engine(root.path()).is_some());
        assert_eq!(
            resolve_engine("custom-whisper", "/explicit/model"),
            ("custom-whisper".into(), "/explicit/model".into())
        );
    }
    #[test]
    fn local_dictation_stops_recording_transcribes_and_removes_audio() {
        use std::os::unix::fs::PermissionsExt;
        let dir =
            std::env::temp_dir().join(format!("nexuterm-voice-fixture-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let recorder = dir.join("recorder");
        let whisper = dir.join("whisper");
        let observed = dir.join("audio-path");
        std::fs::write(&recorder,format!("#!/usr/bin/python3\nimport pathlib,signal,sys,time\np=pathlib.Path(sys.argv[-1]);p.write_bytes(b'RIFF'+b'0'*100)\npathlib.Path({:?}).write_text(str(p))\nsignal.signal(signal.SIGINT,lambda *_:sys.exit(0))\nwhile True:time.sleep(.02)\n",observed.to_string_lossy())).unwrap();
        std::fs::write(&whisper,"#!/usr/bin/python3\nimport sys\nassert '-nt' in sys.argv and '-np' in sys.argv and 'auto' in sys.argv\nprint('Mensaje dictado de prueba')\n").unwrap();
        for path in [&recorder, &whisper] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let result = crate::runtime::runtime().block_on(async {
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(250)).await;
                flag.store(true, Ordering::Relaxed);
            });
            transcribe_with(
                recorder.to_str().unwrap(),
                whisper.to_string_lossy().into(),
                "fixture-model".into(),
                "auto".into(),
                stop,
            )
            .await
        });
        assert_eq!(result.unwrap(), "Mensaje dictado de prueba");
        let audio = std::fs::read_to_string(observed).unwrap();
        assert!(!std::path::Path::new(&audio).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
