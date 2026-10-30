//! Bounded, non-interactive jobs. No terminal input injection or automatic replay.
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use std::os::unix::process::CommandExt;
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io::AsyncReadExt, sync::Semaphore, time::Instant};
pub const OUTPUT_LIMIT: usize = 4 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub command: String,
    pub directory: PathBuf,
    pub timeout_secs: u64,
    pub keep_history: bool,
}
impl Spec {
    pub fn validate(&self) -> Result<(), String> {
        if self.command.trim().is_empty()
            || self.command.len() > 16384
            || self.command.contains('\0')
        {
            return Err("Introduce un comando de hasta 16 KiB sin bytes nulos".into());
        }
        if !self.directory.is_absolute()
            || self.directory.as_os_str().len() > 4096
            || self
                .directory
                .to_str()
                .is_none_or(|p| p.chars().any(char::is_control))
        {
            return Err("La carpeta debe ser una ruta absoluta UTF-8 de hasta 4096 bytes".into());
        }
        if !(1..=120).contains(&self.timeout_secs) {
            return Err("El tiempo límite debe estar entre 1 y 120 segundos".into());
        }
        Ok(())
    }
}
#[derive(Clone)]
pub enum Target {
    Local,
    Ssh(Arc<crate::ssh::SshConnection>),
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub id: String,
    pub label: String,
    pub remote: bool,
}
impl Target {
    pub fn destination(&self) -> Destination {
        match self {
            Self::Local => Destination {
                id: "local".into(),
                label: "Este equipo".into(),
                remote: false,
            },
            Self::Ssh(c) => Destination {
                id: c.profile.id.clone(),
                label: format!(
                    "{}@{}:{}",
                    c.profile.username, c.profile.host, c.profile.port
                ),
                remote: true,
            },
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Preparing,
    Running,
    Completed,
    Cancelled,
    TimedOut,
    Unknown,
    Failed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub version: u32,
    pub id: String,
    pub spec: Spec,
    pub destination: Destination,
    pub started_at: u64,
    pub duration_ms: u64,
    pub status: Status,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub truncated: bool,
    pub error: String,
    pub recording_error: String,
}
impl Record {
    pub fn finished(&self) -> bool {
        !matches!(self.status, Status::Preparing | Status::Running)
    }
    pub fn copy_limited(&self, n: usize) -> Self {
        Self {
            version: self.version,
            id: self.id.clone(),
            spec: self.spec.clone(),
            destination: self.destination.clone(),
            started_at: self.started_at,
            duration_ms: self.duration_ms,
            status: self.status.clone(),
            exit_code: self.exit_code,
            stdout: self.stdout[..self.stdout.len().min(n)].to_vec(),
            stderr: self.stderr[..self.stderr.len().min(n)].to_vec(),
            truncated: self.truncated,
            error: self.error.clone(),
            recording_error: self.recording_error.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.spec.validate()?;
        uuid::Uuid::parse_str(&self.id).map_err(|_| "Identificador de ejecución inválido")?;
        if self.version != 1
            || self.stdout.len() + self.stderr.len() > OUTPUT_LIMIT
            || self.error.len() > 8192
            || self.recording_error.len() > 8192
            || self.destination.id.len() > 4096
            || self.destination.label.len() > 4096
            || self.destination.id.contains('\0')
            || self.destination.label.contains('\0')
            || self.error.contains('\0')
            || self.recording_error.contains('\0')
        {
            return Err("Registro de ejecución inválido o demasiado grande".into());
        }
        Ok(())
    }
}
pub struct Handle {
    state: Arc<Mutex<Record>>,
    cancel: Arc<AtomicBool>,
    settled: AtomicBool,
}
impl Handle {
    pub fn snapshot(&self) -> Record {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn view(&self) -> Record {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .copy_limited(128 * 1024)
    }
    pub fn id(&self) -> String {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .id
            .clone()
    }
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
    pub fn finished(&self) -> bool {
        self.settled.load(Ordering::Acquire)
    }
}
fn slots() -> &'static Arc<Semaphore> {
    static SLOTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SLOTS.get_or_init(|| Arc::new(Semaphore::new(4)))
}
/// Call only after the GUI has approved this exact spec and captured destination.
pub fn start(spec: Spec, target: Target) -> Result<Arc<Handle>, String> {
    spec.validate()?;
    let slot = Arc::new(
        slots()
            .clone()
            .try_acquire_owned()
            .map_err(|_| "Hay cuatro ejecuciones activas; espera a que termine alguna")?,
    );
    let record = Record {
        version: 1,
        id: uuid::Uuid::new_v4().to_string(),
        spec,
        destination: target.destination(),
        started_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        duration_ms: 0,
        status: Status::Preparing,
        exit_code: None,
        stdout: Vec::new(),
        stderr: Vec::new(),
        truncated: false,
        error: String::new(),
        recording_error: String::new(),
    };
    let handle = Arc::new(Handle {
        state: Arc::new(Mutex::new(record)),
        cancel: Arc::new(AtomicBool::new(false)),
        settled: AtomicBool::new(false),
    });
    let job = handle.clone();
    crate::runtime::runtime().spawn(async move {
        if std::panic::AssertUnwindSafe(run(job.clone(), target, slot))
            .catch_unwind()
            .await
            .is_err()
        {
            let mut r = job.state.lock().unwrap_or_else(|e| e.into_inner());
            r.status = Status::Unknown;
            r.exit_code = None;
            r.error = "Fallo interno: no se confirma el resultado del proceso".into();
            if r.spec.keep_history {
                r.recording_error = "No se confirmó el registro final".into();
            }
        }
        job.settled.store(true, Ordering::Release);
    });
    Ok(handle)
}
struct Group {
    pid: i32,
}
impl Group {
    fn signal(&self, signal: i32) {
        if self.pid > 0 {
            unsafe {
                libc::kill(-self.pid, signal);
            }
        }
    }
}
impl Drop for Group {
    fn drop(&mut self) {
        self.signal(libc::SIGKILL);
    }
}
async fn run(handle: Arc<Handle>, target: Target, slot: Arc<tokio::sync::OwnedSemaphorePermit>) {
    let initial = handle.snapshot();
    let journal = if initial.spec.keep_history {
        let record = initial.clone();
        let slot = slot.clone();
        match tokio::task::spawn_blocking(move || {
            crate::execution_history::Journal::start(
                &crate::execution_history::directory()?,
                &record,
                slot,
            )
        })
        .await
        {
            Ok(Ok(j)) => Some(Arc::new(j)),
            other => {
                let e = match other {
                    Ok(Err(e)) => e,
                    Err(e) => e.to_string(),
                    _ => unreachable!(),
                };
                let mut s = handle.state.lock().unwrap();
                s.status = Status::Failed;
                s.recording_error = e.clone();
                s.error = format!("No se inició el comando: no se pudo preparar su historial. {e}");
                return;
            }
        }
    } else {
        None
    };
    let start = Instant::now();
    let deadline = start + Duration::from_secs(initial.spec.timeout_secs);
    let mut command = match &target {
        Target::Local => {
            let mut c = tokio::process::Command::new("/bin/sh");
            c.arg("-c")
                .arg(&initial.spec.command)
                .current_dir(&initial.spec.directory);
            c
        }
        Target::Ssh(connection) => {
            let ready = async { connection.ready().await };
            tokio::pin!(ready);
            let outcome = loop {
                tokio::select! {result=&mut ready=>break result,_=tokio::time::sleep(Duration::from_millis(25))=>{if handle.cancel.load(Ordering::Relaxed){break Err("Cancelado antes de abrir el canal SSH".into());}if Instant::now()>=deadline{break Err("Tiempo agotado antes de abrir el canal SSH".into());}}}
            };
            if let Err(e) = outcome {
                finish_before_start(
                    &handle,
                    &journal,
                    start,
                    if handle.cancel.load(Ordering::Relaxed) {
                        Status::Cancelled
                    } else if Instant::now() >= deadline {
                        Status::TimedOut
                    } else {
                        Status::Failed
                    },
                    e,
                )
                .await;
                return;
            }
            connection.execution_command(&initial.spec.command, &initial.spec.directory)
        }
    };
    if handle.cancel.load(Ordering::Relaxed) || Instant::now() >= deadline {
        finish_before_start(
            &handle,
            &journal,
            start,
            if handle.cancel.load(Ordering::Relaxed) {
                Status::Cancelled
            } else {
                Status::TimedOut
            },
            "El comando no llegó a iniciarse".into(),
        )
        .await;
        return;
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command.as_std_mut().process_group(0);
    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            finish_before_start(&handle, &journal, start, Status::Failed, e.to_string()).await;
            return;
        }
    };
    let mut group = Group {
        pid: child.id().unwrap_or(0) as i32,
    };
    {
        handle.state.lock().unwrap().status = Status::Running;
    }
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let mut out = [0u8; 16384];
    let mut err = [0u8; 16384];
    let (mut out_open, mut err_open) = (true, true);
    let mut exited = None;
    let mut reason = None::<String>;
    let mut timed_out = false;
    let mut read_failed = false;
    let mut tick = tokio::time::interval(Duration::from_millis(25));
    let mut checkpoint = Instant::now();
    let mut last_capture = None;
    let mut writer = None::<tokio::task::JoinHandle<Result<(), String>>>;
    loop {
        tokio::select! {
         result=stdout.read(&mut out),if out_open=>match result{Ok(0)=>out_open=false,Ok(n)=>append(&handle,true,&out[..n]),Err(e)=>{read_failed=true;reason=Some(format!("No se pudo leer stdout: {e}"));}},
         result=stderr.read(&mut err),if err_open=>match result{Ok(0)=>err_open=false,Ok(n)=>append(&handle,false,&err[..n]),Err(e)=>{read_failed=true;reason=Some(format!("No se pudo leer stderr: {e}"));}},
         result=child.wait(),if exited.is_none()&&!out_open&&!err_open=>{match result{Ok(status)=>{exited=Some(status);group.pid=0;},Err(e)=>{read_failed=true;reason=Some(format!("No se pudo confirmar el proceso: {e}"));}}},
         _=tick.tick()=>{
          handle.state.lock().unwrap().duration_ms=start.elapsed().as_millis() as u64;
          if matches!(&target,Target::Ssh(c) if c.closed.load(Ordering::Relaxed)){reason=Some("La sesión SSH se desconectó".into());}
          else if handle.cancel.load(Ordering::Relaxed){reason=Some("Cancelación solicitada".into());}
          else if Instant::now()>=deadline{reason=Some("Tiempo límite agotado".into());timed_out=true;}
          if writer.as_ref().is_some_and(|w|w.is_finished()){if let Some(w)=writer.take(){if let Err(e)=w.await.unwrap_or_else(|e|Err(e.to_string())){handle.state.lock().unwrap().recording_error=e;}}}
          if writer.is_none()&&checkpoint.elapsed()>=Duration::from_secs(1)&&handle.state.lock().unwrap().recording_error.is_empty(){
            if let Some(journal)=&journal{
                let progress={let r=handle.state.lock().unwrap();(r.stdout.len(),r.stderr.len(),r.truncated)};
                if last_capture!=Some(progress){let record=handle.snapshot();let j=journal.clone();writer=Some(tokio::task::spawn_blocking(move||j.write(&record)));last_capture=Some(progress);}
            }checkpoint=Instant::now();
          }
         }
        }
        if reason.is_some() || (exited.is_some() && !out_open && !err_open) {
            break;
        }
    }
    if reason.is_some() {
        group.signal(libc::SIGTERM);
        tokio::time::sleep(Duration::from_millis(150)).await;
        group.signal(libc::SIGKILL);
        if exited.is_none() {
            exited = tokio::time::timeout(Duration::from_secs(2), child.wait())
                .await
                .ok()
                .and_then(Result::ok);
        }
        group.pid = 0;
        // Drain only what is immediately available after termination; inherited remote FDs may remain open.
        let _=tokio::time::timeout(Duration::from_millis(200),async{while out_open||err_open{tokio::select!{r=stdout.read(&mut out),if out_open=>match r{Ok(0)|Err(_)=>out_open=false,Ok(n)=>append(&handle,true,&out[..n])},r=stderr.read(&mut err),if err_open=>match r{Ok(0)|Err(_)=>err_open=false,Ok(n)=>append(&handle,false,&err[..n])}}if !out_open&&!err_open{break;}}}).await;
    }
    // Do not signal a completed and reaped group later when its PID could be reused.
    group.pid = 0;
    {
        let mut record = handle.state.lock().unwrap();
        record.duration_ms = start.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let (status, code) = outcome(
            record.destination.remote,
            exited,
            reason.is_some(),
            timed_out,
            read_failed,
        );
        record.status = status;
        record.exit_code = code;
        record.error = reason.unwrap_or_default();
        if record.status == Status::Unknown {
            let message = if record.destination.remote {
                " No se confirma la terminación ni los efectos del proceso remoto; comprueba el servidor antes de repetirlo."
            } else {
                " No se pudo confirmar la terminación local; comprueba el proceso antes de repetirlo."
            };
            record.error.push_str(message);
        }
    }
    if let Some(writer) = writer {
        let result = writer.await.unwrap_or_else(|e| Err(e.to_string()));
        if let Err(e) = result {
            handle.state.lock().unwrap().recording_error = e;
        }
    }
    persist_final(&handle, &journal).await;
}
fn outcome(
    remote: bool,
    exited: Option<std::process::ExitStatus>,
    stopped: bool,
    timed_out: bool,
    read_failed: bool,
) -> (Status, Option<i32>) {
    let code = exited.and_then(|s| s.code());
    if exited.is_none() || (remote && (stopped || code == Some(255) || code.is_none())) {
        return (Status::Unknown, None);
    }
    let status = if timed_out {
        Status::TimedOut
    } else if read_failed {
        Status::Failed
    } else if stopped {
        Status::Cancelled
    } else if code.is_none() {
        Status::Failed
    } else {
        Status::Completed
    };
    (status, code)
}
fn append(handle: &Handle, stdout: bool, bytes: &[u8]) {
    let mut s = handle.state.lock().unwrap();
    let remaining = OUTPUT_LIMIT.saturating_sub(s.stdout.len() + s.stderr.len());
    let n = remaining.min(bytes.len());
    if stdout {
        s.stdout.extend_from_slice(&bytes[..n]);
    } else {
        s.stderr.extend_from_slice(&bytes[..n]);
    }
    if n < bytes.len() {
        s.truncated = true;
    }
}
async fn persist_final(handle: &Handle, journal: &Option<Arc<crate::execution_history::Journal>>) {
    if let Some(j) = journal {
        let mut record = handle.snapshot();
        record.recording_error.clear();
        let j = j.clone();
        match tokio::task::spawn_blocking(move || j.write(&record))
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        {
            Ok(()) => handle.state.lock().unwrap().recording_error.clear(),
            Err(e) => handle.state.lock().unwrap().recording_error = e,
        }
    }
}
async fn finish_before_start(
    handle: &Handle,
    journal: &Option<Arc<crate::execution_history::Journal>>,
    start: Instant,
    status: Status,
    error: String,
) {
    {
        let mut r = handle.state.lock().unwrap();
        r.status = status;
        r.error = error;
        r.duration_ms = start.elapsed().as_millis() as u64;
    }
    persist_final(handle, journal).await;
}
#[cfg(test)]
mod tests {
    use super::*;
    fn spec(command: &str) -> Spec {
        Spec {
            command: command.into(),
            directory: "/tmp".into(),
            timeout_secs: 2,
            keep_history: false,
        }
    }
    async fn done(handle: &Handle) -> Record {
        tokio::time::timeout(Duration::from_secs(5), async {
            while !handle.finished() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        handle.snapshot()
    }
    #[tokio::test]
    async fn captures_both_streams_exit_and_explicit_directory() {
        let h = start(
            spec("printf '%s' \"$PWD\"; printf 'é error' >&2; exit 7"),
            Target::Local,
        )
        .unwrap();
        let r = done(&h).await;
        assert_eq!(r.stdout, b"/tmp");
        assert_eq!(r.stderr, "é error".as_bytes());
        assert_eq!(r.exit_code, Some(7));
        assert_eq!(r.status, Status::Completed);
    }
    #[tokio::test]
    async fn caps_output_but_drains_both_pipes() {
        let h = start(
            spec("head -c 5000000 /dev/zero; printf 'fin' >&2"),
            Target::Local,
        )
        .unwrap();
        let r = done(&h).await;
        assert_eq!(r.stdout.len() + r.stderr.len(), OUTPUT_LIMIT);
        assert!(r.truncated);
        assert_eq!(r.exit_code, Some(0));
    }
    #[tokio::test]
    async fn cancellation_and_timeout_terminate_process_groups() {
        let h = start(spec("sleep 20 & wait"), Target::Local).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        h.cancel();
        assert_eq!(done(&h).await.status, Status::Cancelled);
        let mut s = spec("sleep 20 & wait");
        s.timeout_secs = 1;
        let h = start(s, Target::Local).unwrap();
        assert_eq!(done(&h).await.status, Status::TimedOut);
    }
    #[test]
    fn unconfirmed_exit_is_unknown_and_signal_exit_is_not_success() {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(
            outcome(false, None, true, true, false),
            (Status::Unknown, None)
        );
        assert_eq!(
            outcome(
                true,
                Some(std::process::ExitStatus::from_raw(255 << 8)),
                false,
                false,
                false
            ),
            (Status::Unknown, None)
        );
        assert_eq!(
            outcome(
                false,
                Some(std::process::ExitStatus::from_raw(libc::SIGTERM)),
                false,
                false,
                false
            ),
            (Status::Failed, None)
        );
        assert_eq!(
            outcome(
                false,
                Some(std::process::ExitStatus::from_raw(0)),
                false,
                false,
                false
            ),
            (Status::Completed, Some(0))
        );
    }
    #[test]
    fn invalid_requests_never_spawn() {
        let mut s = spec("true");
        s.directory = "relative".into();
        assert!(start(s, Target::Local).is_err());
        let mut s = spec("true");
        s.timeout_secs = 0;
        assert!(s.validate().is_err());
        assert!(spec("\0").validate().is_err());
    }
}
