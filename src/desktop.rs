use crate::config::{Profile, Protocol};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;
pub struct Frame {
    pub width: u16,
    pub height: u16,
    pub rgba: Vec<u8>,
}
#[derive(Debug)]
pub enum Input {
    Key { keysym: u32, down: bool },
    Pointer { x: u16, y: u16, buttons: u8 },
    Resize { width: u16, height: u16 },
    Clipboard(String),
    Close,
}
pub struct Desktop {
    pub frame: Arc<Mutex<Option<Frame>>>,
    pub status: Arc<Mutex<String>>,
    pub clipboard: Arc<Mutex<Option<String>>>,
    pub input: mpsc::Sender<Input>,
    pub stop: Arc<AtomicBool>,
    pub connected: Arc<AtomicBool>,
}
impl Desktop {
    pub fn connect(profile: Profile, password: zeroize::Zeroizing<String>) -> Arc<Self> {
        let (input, rx) = mpsc::channel(128);
        let desktop = Arc::new(Self {
            frame: Default::default(),
            status: Arc::new(Mutex::new("Conectando…".into())),
            clipboard: Default::default(),
            input,
            stop: Default::default(),
            connected: Default::default(),
        });
        let d = desktop.clone();
        match profile.protocol {
            Protocol::Rdp => {
                crate::runtime::runtime().spawn(async move {
                    let result = crate::rdp::run(profile, password, rx, d.clone()).await;
                    d.connected.store(false, Ordering::Relaxed);
                    *d.status.lock().unwrap() = match result {
                        Ok(()) => "Desconectado".into(),
                        Err(e) => format!("RDP: {e:#}"),
                    };
                });
            }
            Protocol::Vnc => {
                std::thread::spawn(move || {
                    let result = crate::vnc::run(profile, password, rx, d.clone());
                    d.connected.store(false, Ordering::Relaxed);
                    *d.status.lock().unwrap() = match result {
                        Ok(()) => "Desconectado".into(),
                        Err(e) => format!("VNC: {e}"),
                    };
                });
            }
            _ => {}
        }
        desktop
    }
    pub fn send(&self, event: Input) {
        if self.input.try_send(event).is_err() {
            self.stop.store(true, Ordering::Relaxed);
            *self.status.lock().unwrap() =
                "Entrada saturada: desconectando para no perder liberaciones de teclas".into();
        }
    }
    pub fn close(&self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.input.try_send(Input::Close);
    }
}
