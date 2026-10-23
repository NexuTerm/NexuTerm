//! An OpenSSH master authenticated through our graphical SSH_ASKPASS helper.
use crate::ui::State;
use gtk::{gio, glib, prelude::*};
use nexuterm::{connection::CommandSpec, ssh::SshConnection};
use std::{
    process::Stdio,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    time::Duration,
};
use tokio::io::AsyncReadExt;

pub struct Master {
    pub pid: AtomicI32,
    connection: Arc<SshConnection>,
    status: Mutex<Option<String>>,
    finished: AtomicBool,
}
impl Master {
    pub fn close(&self) {
        self.connection.closed.store(true, Ordering::Relaxed);
        let pid = self.pid.swap(0, Ordering::Relaxed);
        if pid > 0 {
            unsafe {
                libc::kill(-pid, libc::SIGHUP);
            }
        }
    }
    pub fn start(
        state: &Rc<State>,
        connection: Arc<SshConnection>,
        spec: CommandSpec,
        label: &gtk::Label,
    ) -> Arc<Self> {
        let master = Arc::new(Self {
            pid: AtomicI32::new(0),
            connection,
            status: Mutex::new(None),
            finished: AtomicBool::new(false),
        });
        state
            .sftp_masters
            .borrow_mut()
            .retain(|master| master.strong_count() > 0);
        state
            .sftp_masters
            .borrow_mut()
            .push(Arc::downgrade(&master));
        let lifetime = gio::Cancellable::new();
        state.operations.borrow_mut().push(lifetime.downgrade());
        let weak = Arc::downgrade(&master);
        lifetime.connect_cancelled(move |_| {
            if let Some(master) = weak.upgrade() {
                master.close();
            }
        });
        let worker = master.clone();
        nexuterm::runtime::runtime().spawn(async move {
            let result = worker.run(spec).await;
            worker.pid.store(0, Ordering::Relaxed);
            if let Err(error) = result {
                #[cfg(debug_assertions)]
                if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some() {
                    eprintln!("NEXUTERM_SFTP_ERROR={error}");
                }
                *worker.status.lock().unwrap() = Some(error);
            }
            worker.connection.closed.store(true, Ordering::Relaxed);
            worker.finished.store(true, Ordering::Relaxed);
        });
        let view = label.downgrade();
        let weak = Arc::downgrade(&master);
        glib::timeout_add_local(Duration::from_millis(100), move || {
            let _keep_alive = &lifetime;
            let (Some(view), Some(master)) = (view.upgrade(), weak.upgrade()) else {
                return glib::ControlFlow::Break;
            };
            if let Some(status) = master.status.lock().unwrap().take() {
                view.set_text(&status);
                view.set_visible(true);
            }
            if master.finished.load(Ordering::Relaxed) {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
        master
    }
    async fn run(&self, spec: CommandSpec) -> Result<(), String> {
        if self.connection.closed.load(Ordering::Relaxed) {
            return Ok(());
        }
        let helper = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut command = tokio::process::Command::new(spec.program);
        command
            .args(["-o", "NumberOfPasswordPrompts=1"])
            .args(spec.args)
            .env("SSH_ASKPASS", helper)
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("NEXUTERM_ASKPASS", "1")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        command.process_group(0);
        let mut child = command.spawn().map_err(|error| format!("SFTP: {error}"))?;
        self.pid
            .store(child.id().unwrap_or(0) as i32, Ordering::Relaxed);
        let mut stderr = child.stderr.take().ok_or("SFTP sin canal de errores")?;
        let mut drain = tokio::spawn(async move {
            let mut output = Vec::new();
            let mut buffer = [0; 1024];
            while let Ok(length) = stderr.read(&mut buffer).await {
                if length == 0 {
                    break;
                }
                let keep = length.min(8192usize.saturating_sub(output.len()));
                output.extend_from_slice(&buffer[..keep]);
            }
            String::from_utf8_lossy(&output).into_owned()
        });
        let status = tokio::select! {
            result=child.wait()=>result,
            _=async {while !self.connection.closed.load(Ordering::Relaxed){tokio::time::sleep(Duration::from_millis(50)).await;}}=>{self.close();let _=child.start_kill();child.wait().await}
        };
        self.pid.store(0, Ordering::Relaxed);
        let output = match tokio::time::timeout(Duration::from_secs(1), &mut drain).await {
            Ok(Ok(output)) => output,
            _ => {
                drain.abort();
                let _ = drain.await;
                String::new()
            }
        };
        let status = status.map_err(|error| error.to_string())?;
        if status.success() || self.connection.closed.load(Ordering::Relaxed) {
            Ok(())
        } else {
            Err(format!("No se pudo autenticar SFTP. {}", output.trim()))
        }
    }
}
