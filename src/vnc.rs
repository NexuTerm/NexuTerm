use crate::{
    config::Profile,
    desktop::{Desktop, Frame, Input},
};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    sync::{Arc, atomic::Ordering},
};
use tokio::sync::mpsc;
unsafe extern "C" {
    fn nx_vnc_open(
        host: *const c_char,
        port: i32,
        username: *const c_char,
        password: *const c_char,
    ) -> *mut c_void;
    fn nx_vnc_poll(p: *mut c_void) -> i32;
    fn nx_vnc_frame(p: *mut c_void, w: *mut i32, h: *mut i32) -> *const u8;
    fn nx_vnc_key(p: *mut c_void, key: u32, down: i32);
    fn nx_vnc_pointer(p: *mut c_void, x: i32, y: i32, buttons: i32);
    fn nx_vnc_clipboard(p: *mut c_void, text: *const c_char, len: i32);
    fn nx_vnc_take_clipboard(p: *mut c_void) -> *const c_char;
    fn nx_vnc_clear_clipboard(p: *mut c_void);
    fn nx_vnc_close(p: *mut c_void);
}
struct Client(*mut c_void);
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { nx_vnc_close(self.0) }
    }
}
pub fn run(
    p: Profile,
    password: zeroize::Zeroizing<String>,
    mut input: mpsc::Receiver<Input>,
    desktop: Arc<Desktop>,
) -> Result<(), String> {
    let host = CString::new(p.host).map_err(|e| e.to_string())?;
    let user = CString::new(p.username).map_err(|e| e.to_string())?;
    if password.as_bytes().contains(&0) {
        return Err("Contraseña VNC no válida".into());
    }
    let mut pass = zeroize::Zeroizing::new(password.as_bytes().to_vec());
    pass.push(0);
    let pointer = unsafe {
        nx_vnc_open(
            host.as_ptr(),
            p.port.into(),
            user.as_ptr(),
            pass.as_ptr().cast(),
        )
    };
    drop(pass);
    drop(password);
    if pointer.is_null() {
        return Err("No se pudo autenticar o abrir el servidor VNC".into());
    }
    let client = Client(pointer);
    desktop.connected.store(true, Ordering::Relaxed);
    *desktop.status.lock().unwrap() = "VNC conectado".into();
    while !desktop.stop.load(Ordering::Relaxed) {
        while let Ok(event) = input.try_recv() {
            unsafe {
                match event {
                    Input::Close => return Ok(()),
                    Input::Key { keysym, down } => nx_vnc_key(client.0, keysym, down.into()),
                    Input::Pointer { x, y, buttons } => {
                        nx_vnc_pointer(client.0, x.into(), y.into(), buttons.into())
                    }
                    Input::Clipboard(text) => nx_vnc_clipboard(
                        client.0,
                        text.as_ptr().cast(),
                        text.len().min(1048576) as i32,
                    ),
                    Input::Resize { .. } => {}
                }
            }
        }
        if unsafe { nx_vnc_poll(client.0) } < 0 {
            return Err("El servidor cerró la conexión".into());
        }
        let (mut w, mut h) = (0, 0);
        let ptr = unsafe { nx_vnc_frame(client.0, &mut w, &mut h) };
        if !ptr.is_null() {
            if w < 1 || h < 1 || w > 65535 || h > 65535 || w as usize * h as usize > 16777216 {
                return Err("Framebuffer fuera de límite".into());
            }
            let mut rgba =
                unsafe { std::slice::from_raw_parts(ptr, w as usize * h as usize * 4) }.to_vec();
            for pixel in rgba.as_chunks_mut::<4>().0 {
                pixel[3] = 255;
            }
            *desktop.frame.lock().unwrap() = Some(Frame {
                width: w as u16,
                height: h as u16,
                rgba,
            });
        }
        let text = unsafe { nx_vnc_take_clipboard(client.0) };
        if !text.is_null() {
            let s = unsafe { CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned();
            *desktop.clipboard.lock().unwrap() = Some(s);
            unsafe { nx_vnc_clear_clipboard(client.0) };
        }
    }
    Ok(())
}
