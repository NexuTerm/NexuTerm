//! Local-only voice input. Audio is temporary and never sent to a service.
use std::{
    io::Read,
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
// 100 ms windows at 16 kHz, signed 16-bit mono. Energy is only an endpoint
// heuristic; Whisper still decides which words were spoken.
#[derive(Default)]
struct Endpoint {
    voiced: usize,
    quiet: usize,
}
impl Endpoint {
    fn push(&mut self, pcm: &[u8]) -> bool {
        let energy = pcm
            .chunks_exact(2)
            .map(|b| {
                let sample = i16::from_le_bytes([b[0], b[1]]) as f64;
                sample * sample
            })
            .sum::<f64>()
            / (pcm.len() / 2).max(1) as f64;
        if energy.sqrt() >= 200.0 {
            self.voiced += 1;
            self.quiet = 0;
        } else {
            self.quiet += 1;
        }
        self.voiced >= 3 && self.quiet >= 15
    }
}
fn wav(pcm: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(44 + pcm.len());
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&16000u32.to_le_bytes());
    bytes.extend_from_slice(&32000u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    bytes.extend_from_slice(pcm);
    bytes
}
fn speech_text(text: &str) -> Result<String, String> {
    // Non-speech captions are not dictation. Preserve ordinary punctuation
    // and parentheses in spoken text.
    let mut result = String::new();
    let mut caption = false;
    for c in text.chars() {
        match c {
            '[' => caption = true,
            ']' if caption => {
                caption = false;
                result.push(' ');
            }
            _ if !caption => result.push(c),
            _ => {}
        }
    }
    let result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    if result.is_empty() {
        Err("No se reconoció voz. Comprueba el micrófono y vuelve a intentarlo.".into())
    } else {
        Ok(result)
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
    let raw = recording.0.join("voice.raw");
    std::fs::File::create(&raw).map_err(|_| "No se pudo preparar la grabación")?;
    let mut samples = std::fs::File::open(&raw).map_err(|_| "No se pudo abrir la grabación")?;
    let mut endpoint = Endpoint::default();
    let mut pending = Vec::new();
    let mut recorder = Command::new(recorder_program)
        .args([
            "--rate",
            "16000",
            "--channels",
            "1",
            "--format",
            "s16",
            "--raw",
            "--media-role",
            "Communication",
        ])
        .arg(&raw)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "No se encuentra pw-record para grabar el micrófono")?;
    let start = tokio::time::Instant::now();
    loop {
        samples
            .by_ref()
            .take(64000)
            .read_to_end(&mut pending)
            .map_err(|_| "No se pudo leer el micrófono")?;
        let mut ended = false;
        let complete = pending.len() / 3200 * 3200;
        for frame in pending[..complete].chunks_exact(3200) {
            ended |= endpoint.push(frame);
        }
        pending.drain(..complete);
        if ended || stop.load(Ordering::Relaxed) || start.elapsed() >= Duration::from_secs(60) {
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
    let pcm = std::fs::read(&raw).map_err(|_| "No se grabó audio")?;
    if pcm.len() > 2_000_000 {
        return Err("El audio supera el límite de un minuto".into());
    }
    if endpoint.voiced < 3 {
        return Err(
            "No se detectó voz. Comprueba que el micrófono esté activo y no silenciado.".into(),
        );
    }
    std::fs::write(&audio, wav(&pcm[..pcm.len() / 2 * 2]))
        .map_err(|_| "No se pudo preparar el audio para transcribir")?;
    let mut child = Command::new(program)
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(&audio)
        .args(["-nt", "-np", "-sns", "-l", &language])
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
    speech_text(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_waits_for_speech_then_a_pause() {
        let mut endpoint = Endpoint::default();
        for _ in 0..30 {
            assert!(!endpoint.push(&[0; 3200]));
        }
        for _ in 0..3 {
            assert!(!endpoint.push(&[16; 3200]));
        }
        for _ in 0..14 {
            assert!(!endpoint.push(&[0; 3200]));
        }
        assert!(endpoint.push(&[0; 3200]));
    }
    #[test]
    fn captions_are_not_inserted_as_dictation() {
        assert!(speech_text("[Música]").is_err());
        assert!(speech_text("[BLANK_AUDIO]").is_err());
        assert_eq!(
            speech_text("[Music] Hola, mundo. [ruido]").unwrap(),
            "Hola, mundo."
        );
        assert_eq!(speech_text("Hola (otra vez).").unwrap(), "Hola (otra vez).");
    }
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
        recording_fixture(false, true);
    }
    #[test]
    fn local_dictation_finishes_after_silence() {
        recording_fixture(true, true);
    }
    #[test]
    fn silent_recording_does_not_reach_whisper() {
        recording_fixture(false, false);
    }
    fn recording_fixture(automatic: bool, speech: bool) {
        use std::os::unix::fs::PermissionsExt;
        let dir =
            std::env::temp_dir().join(format!("nexuterm-voice-fixture-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let recorder = dir.join("recorder");
        let whisper = dir.join("whisper");
        let observed = dir.join("audio-path");
        std::fs::write(&recorder,format!("#!/usr/bin/python3\nimport pathlib,signal,sys,time\np=pathlib.Path(sys.argv[-1]);p.write_bytes(bytes([0, {}])*6400 + bytes(48000))\npathlib.Path({:?}).write_text(str(p))\nsignal.signal(signal.SIGINT,lambda *_:sys.exit(0))\nwhile True:time.sleep(.02)\n",if speech {16} else {0}, observed.to_string_lossy())).unwrap();
        std::fs::write(&whisper,"#!/usr/bin/python3\nimport sys,wave\nwith wave.open(sys.argv[sys.argv.index('-f')+1]) as w: assert w.getframerate()==16000 and w.getnchannels()==1 and w.getsampwidth()==2\nassert '-sns' in sys.argv and '-nt' in sys.argv and '-np' in sys.argv and 'auto' in sys.argv\nprint('Mensaje dictado de prueba')\n").unwrap();
        for path in [&recorder, &whisper] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let result = crate::runtime::runtime().block_on(async {
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(250)).await;
                if !automatic {
                    flag.store(true, Ordering::Relaxed);
                }
            });
            tokio::time::timeout(
                Duration::from_secs(5),
                transcribe_with(
                    recorder.to_str().unwrap(),
                    whisper.to_string_lossy().into(),
                    "fixture-model".into(),
                    "auto".into(),
                    stop,
                ),
            )
            .await
            .unwrap()
        });
        if speech {
            assert_eq!(result.unwrap(), "Mensaje dictado de prueba");
        } else {
            assert!(result.unwrap_err().contains("No se detectó voz"));
        }
        let audio = std::fs::read_to_string(observed).unwrap();
        assert!(!std::path::Path::new(&audio).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
