use crate::ui::{State, append_tab, icon_button};
use glib::translate::IntoGlib;
use gtk::{gdk, glib, prelude::*};
use nexuterm::{
    config::{Profile, Protocol},
    desktop::{Desktop, Input},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
    sync::{Arc, atomic::Ordering},
};

// The image uses ContentFit::Contain. Ignore the letterbox margins.
fn position(x: f64, y: f64, width: i32, height: i32, frame: (u16, u16)) -> Option<(u16, u16)> {
    if width <= 0 || height <= 0 || frame.0 == 0 || frame.1 == 0 {
        return None;
    }
    let scale = (width as f64 / frame.0 as f64).min(height as f64 / frame.1 as f64);
    let x = (x - (width as f64 - frame.0 as f64 * scale) / 2.0) / scale;
    let y = (y - (height as f64 - frame.1 as f64 * scale) / 2.0) / scale;
    (x >= 0.0 && y >= 0.0 && x < frame.0 as f64 && y < frame.1 as f64)
        .then_some((x.max(0.0) as u16, y.max(0.0) as u16))
}

pub fn show(state: &Rc<State>, profile: &Profile) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.add_css_class("nexuterm-section");
    let rdp = profile.protocol == Protocol::Rdp;
    let username = gtk::Entry::builder()
        .text(&profile.username)
        .placeholder_text("Usuario RDP")
        .build();
    let authority = gtk::Entry::builder()
        .text(&profile.certificate_authority)
        .placeholder_text("Certificado CA de confianza (ruta PEM, opcional)")
        .hexpand(true)
        .build();
    authority.set_tooltip_text(Some("Para certificados privados, indica la CA de confianza. Se comprueba también el nombre del servidor."));
    if rdp {
        let options = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        options.append(&username);
        options.append(&authority);
        root.append(&options);
    }
    let tools = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    tools.add_css_class("toolbar");
    let password = gtk::PasswordEntry::builder()
        .placeholder_text(if rdp {
            "Contraseña RDP"
        } else {
            "Contraseña VNC"
        })
        .show_peek_icon(true)
        .build();
    password.set_tooltip_text(Some("Se utiliza solo para esta conexión; no se guarda"));
    let connect = gtk::Button::with_label("Conectar");
    let disconnect = icon_button("network-offline-symbolic", "Desconectar escritorio remoto");
    tools.append(&password);
    tools.append(&connect);
    tools.append(&disconnect);
    let status = gtk::Label::new(Some(
        "Introduce la contraseña y pulsa Conectar; déjala vacía si el servidor no la requiere.",
    ));
    status.set_wrap(true);
    status.set_hexpand(true);
    status.set_xalign(0.0);
    tools.append(&status);
    root.append(&tools);
    let picture = gtk::Picture::builder()
        .can_shrink(true)
        .content_fit(gtk::ContentFit::Contain)
        .hexpand(true)
        .vexpand(true)
        .focusable(true)
        .build();
    root.append(&picture);
    let current: Rc<RefCell<Option<Arc<Desktop>>>> = Rc::new(RefCell::new(None));
    let dimensions = Rc::new(Cell::new((0, 0)));
    let buttons = Rc::new(Cell::new(0u8));
    let last = Rc::new(Cell::new((0u16, 0u16)));
    let pressed = Rc::new(RefCell::new(HashSet::<u32>::new()));
    let first_frame = Rc::new(Cell::new(true));

    let c = current.clone();
    let s = Rc::downgrade(state);
    let p = profile.clone();
    let secret = password.downgrade();
    let first = first_frame.clone();
    let size = dimensions.clone();
    let user = username.downgrade();
    let ca = authority.downgrade();
    let message = status.downgrade();
    connect.connect_clicked(move |_| {
        let (Some(s), Some(secret)) = (s.upgrade(), secret.upgrade()) else {
            return;
        };
        if c.borrow()
            .as_ref()
            .is_some_and(|d| !d.finished.load(Ordering::Relaxed))
        {
            return;
        }
        let mut profile = p.clone();
        if rdp {
            let (Some(user), Some(ca)) = (user.upgrade(), ca.upgrade()) else {
                return;
            };
            profile.username = user.text().trim().into();
            profile.certificate_authority = ca.text().trim().into();
            if profile.username.is_empty() {
                if let Some(label) = message.upgrade() {
                    label.set_text("Indique el usuario RDP antes de conectar.");
                }
                return;
            }
            if let Err(error) = nexuterm::connection::validate(&profile) {
                if let Some(label) = message.upgrade() {
                    label.set_text(&error);
                }
                return;
            }
        }
        if let Some(old) = c.borrow_mut().take() {
            old.close();
        }
        let pass = zeroize::Zeroizing::new(secret.text().to_string());
        secret.set_text("");
        first.set(true);
        size.set((0, 0));
        let desktop = Desktop::connect(profile, pass);
        s.desktops.borrow_mut().retain(|d| d.strong_count() > 0);
        s.desktops.borrow_mut().push(Arc::downgrade(&desktop));
        *c.borrow_mut() = Some(desktop);
    });
    let start = connect.downgrade();
    password.connect_activate(move |_| {
        if let Some(start) = start.upgrade() {
            start.emit_clicked();
        }
    });
    let c = current.clone();
    disconnect.connect_clicked(move |_| {
        if let Some(d) = c.borrow().as_ref() {
            d.close();
        }
    });

    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let c = current.clone();
    let held = pressed.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        if let Some(d) = c
            .borrow()
            .as_ref()
            .filter(|d| d.connected.load(Ordering::Relaxed))
        {
            let keysym = key.into_glib();
            held.borrow_mut().insert(keysym);
            d.send(Input::Key { keysym, down: true });
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    let c = current.clone();
    let held = pressed.clone();
    keys.connect_key_released(move |_, key, _, _| {
        let keysym = key.into_glib();
        if held.borrow_mut().remove(&keysym)
            && let Some(d) = c.borrow().as_ref()
        {
            d.send(Input::Key {
                keysym,
                down: false,
            });
        }
    });
    #[cfg(debug_assertions)]
    let test_keys = keys.clone();
    picture.add_controller(keys);
    let focus = gtk::EventControllerFocus::new();
    let c = current.clone();
    let held = pressed.clone();
    let mask = buttons.clone();
    let point = last.clone();
    focus.connect_leave(move |_| {
        if let Some(d) = c.borrow().as_ref() {
            for keysym in held.borrow_mut().drain() {
                d.send(Input::Key {
                    keysym,
                    down: false,
                });
            }
            if mask.replace(0) != 0 {
                let (x, y) = point.get();
                d.send(Input::Pointer { x, y, buttons: 0 });
            }
        }
    });
    picture.add_controller(focus);
    let click = gtk::GestureClick::new();
    click.set_button(0);
    let c = current.clone();
    let view = picture.downgrade();
    let size = dimensions.clone();
    let mask = buttons.clone();
    let point = last.clone();
    click.connect_pressed(move |gesture, _, x, y| {
        let Some(view) = view.upgrade() else {
            return;
        };
        let Some((x, y)) = position(x, y, view.width(), view.height(), size.get()) else {
            return;
        };
        view.grab_focus();
        let bit = match gesture.current_button() {
            1 => 1,
            2 => 2,
            3 => 4,
            _ => return,
        };
        mask.set(mask.get() | bit);
        point.set((x, y));
        if let Some(d) = c.borrow().as_ref() {
            d.send(Input::Pointer {
                x,
                y,
                buttons: mask.get(),
            });
        }
    });
    let c = current.clone();
    let mask = buttons.clone();
    let point = last.clone();
    click.connect_released(move |gesture, _, _, _| {
        let bit = match gesture.current_button() {
            1 => 1,
            2 => 2,
            3 => 4,
            _ => return,
        };
        mask.set(mask.get() & !bit);
        let (x, y) = point.get();
        if let Some(d) = c.borrow().as_ref() {
            d.send(Input::Pointer {
                x,
                y,
                buttons: mask.get(),
            });
        }
    });
    let c = current.clone();
    let mask = buttons.clone();
    let point = last.clone();
    click.connect_cancel(move |_, _| {
        if mask.replace(0) != 0
            && let Some(d) = c.borrow().as_ref()
        {
            let (x, y) = point.get();
            d.send(Input::Pointer { x, y, buttons: 0 });
        }
    });
    picture.add_controller(click);
    let motion = gtk::EventControllerMotion::new();
    let c = current.clone();
    let view = picture.downgrade();
    let size = dimensions.clone();
    let mask = buttons.clone();
    let point = last.clone();
    motion.connect_motion(move |_, x, y| {
        let Some(view) = view.upgrade() else {
            return;
        };
        if let Some((x, y)) = position(x, y, view.width(), view.height(), size.get()) {
            point.set((x, y));
            if let Some(d) = c.borrow().as_ref() {
                d.send(Input::Pointer {
                    x,
                    y,
                    buttons: mask.get(),
                });
            }
        }
    });
    #[cfg(debug_assertions)]
    let test_motion = motion.clone();
    picture.add_controller(motion);
    let scroll = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::BOTH_AXES | gtk::EventControllerScrollFlags::DISCRETE,
    );
    let c = current.clone();
    let mask = buttons.clone();
    let point = last.clone();
    scroll.connect_scroll(move |_, dx, dy| {
        if let Some(d) = c.borrow().as_ref() {
            let wheel = if dy < 0.0 {
                8
            } else if dy > 0.0 {
                16
            } else if dx < 0.0 {
                32
            } else if dx > 0.0 {
                64
            } else {
                return glib::Propagation::Proceed;
            };
            let (x, y) = point.get();
            d.send(Input::Pointer {
                x,
                y,
                buttons: mask.get() | wheel,
            });
            d.send(Input::Pointer {
                x,
                y,
                buttons: mask.get(),
            });
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    #[cfg(debug_assertions)]
    let test_scroll = scroll.clone();
    picture.add_controller(scroll);

    state.workspace_documents.borrow_mut().retain(|(w,_)|w.upgrade().is_some());
    state.workspace_documents.borrow_mut().push((root.upcast_ref::<gtk::Widget>().downgrade(),
        nexuterm::workspaces::Layout::Desktop {profile_id:profile.id.clone(),protocol:profile.protocol}));
    let s=Rc::downgrade(state);
    let r=root.downgrade();
    let c = current.clone();
    append_tab(&state.notebook, &root, &profile.name, move || {
        if let (Some(state),Some(root))=(s.upgrade(),r.upgrade()) {
            state.workspace_documents.borrow_mut().retain(|(w,_)|w.upgrade().as_ref()!=Some(root.upcast_ref()));
        }
        if let Some(d) = c.borrow_mut().take() {
            d.close();
        }
    });
    let view = picture.downgrade();
    let label = status.downgrade();
    let start = connect.downgrade();
    let secret = password.downgrade();
    let stop = disconnect.downgrade();
    #[cfg(debug_assertions)]
    let test_tab = (state.notebook.downgrade(), root.downgrade());
    glib::timeout_add_local(std::time::Duration::from_millis(33), move || {
        let (Some(view), Some(label)) = (view.upgrade(), label.upgrade()) else {
            if let Some(d) = current.borrow_mut().take() {
                d.close();
            }
            return glib::ControlFlow::Break;
        };
        if let Some(d) = current.borrow().as_ref() {
            let challenge = d
                .certificate
                .try_lock()
                .ok()
                .and_then(|mut pending| pending.take());
            if let Some(challenge) = challenge
                && let Some(window) = view.root().and_downcast::<gtk::Window>()
            {
                let dialog = gtk::AlertDialog::builder()
                    .modal(true)
                    .message("No se puede verificar el certificado RDP")
                    .detail(&challenge.details)
                    .buttons(["Cancelar", "Confiar solo esta sesión"])
                    .cancel_button(0)
                    .default_button(0)
                    .build();
                #[cfg(debug_assertions)]
                if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                    && std::env::var_os("NEXUTERM_SMOKE_RDP_CERTIFICATE").is_some()
                {
                    glib::timeout_add_local_once(
                        std::time::Duration::from_secs(1),
                        smoke_cancel_certificate,
                    );
                }
                let desktop = Arc::downgrade(d);
                glib::spawn_future_local(async move {
                    let accepted = dialog.choose_future(Some(&window)).await == Ok(1);
                    let active = desktop.upgrade().is_some_and(|d| {
                        !d.stop.load(Ordering::Relaxed) && !d.finished.load(Ordering::Relaxed)
                    });
                    let _ = challenge.response.send(accepted && active);
                    #[cfg(debug_assertions)]
                    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                        && std::env::var_os("NEXUTERM_SMOKE_RDP_CERTIFICATE").is_some()
                    {
                        assert!(!accepted);
                        eprintln!("NEXUTERM_SMOKE_RDP_CERTIFICATE_CANCELLED");
                    }
                });
            }
            let finished = d.finished.load(Ordering::Relaxed);
            if let Some(start) = start.upgrade() {
                start.set_sensitive(finished);
            }
            if let Some(secret) = secret.upgrade() {
                secret.set_sensitive(finished);
            }
            if let Some(stop) = stop.upgrade() {
                stop.set_sensitive(!finished);
            }
            if let Ok(status) = d.status.try_lock() {
                label.set_text(&status);
            }
            let frame = d.frame.try_lock().ok().and_then(|mut f| f.take());
            if let Some(frame) = frame {
                let first = first_frame.replace(false);
                dimensions.set((frame.width, frame.height));
                let stride = usize::from(frame.width) * 4;
                let bytes = glib::Bytes::from_owned(frame.rgba);
                let texture = gdk::MemoryTexture::new(
                    frame.width.into(),
                    frame.height.into(),
                    gdk::MemoryFormat::R8g8b8a8,
                    &bytes,
                    stride,
                );
                view.set_paintable(Some(&texture));
                if first && view.is_mapped() {
                    view.grab_focus();
                }
                #[cfg(debug_assertions)]
                if first
                    && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                    && std::env::var_os("NEXUTERM_SMOKE_RDP").is_some()
                {
                    assert!(texture.width() > 0 && texture.height() > 0);
                    let keys = test_keys.clone();
                    keys.emit_by_name::<bool>(
                        "key-pressed",
                        &[&gdk::Key::a, &38u32, &gdk::ModifierType::empty()],
                    );
                    keys.emit_by_name::<()>(
                        "key-released",
                        &[&gdk::Key::a, &38u32, &gdk::ModifierType::empty()],
                    );
                    d.send(Input::Pointer {
                        x: 100,
                        y: 100,
                        buttons: 1,
                    });
                    d.send(Input::Pointer {
                        x: 100,
                        y: 100,
                        buttons: 0,
                    });
                    let tab = test_tab.clone();
                    let desktop = d.clone();
                    glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                        if let (Some(notebook), Some(root)) = (tab.0.upgrade(), tab.1.upgrade()) {
                            if let Some(path) = std::env::var_os("NEXUTERM_RDP_SCREENSHOT")
                                && let Some(window) = root.root().and_downcast::<gtk::Window>()
                            {
                                let paintable = gtk::WidgetPaintable::new(Some(&window));
                                let snapshot = gtk::Snapshot::new();
                                paintable.snapshot(
                                    &snapshot,
                                    window.width() as f64,
                                    window.height() as f64,
                                );
                                if let (Some(node), Some(renderer)) =
                                    (snapshot.to_node(), window.renderer())
                                {
                                    renderer
                                        .render_texture(&node, None)
                                        .save_to_png(path)
                                        .unwrap();
                                }
                            }
                            notebook.close_page(&notebook.page(&root));
                            assert!(desktop.stop.load(Ordering::Relaxed));
                            glib::timeout_add_local_once(
                                std::time::Duration::from_millis(500),
                                move || {
                                    assert!(desktop.finished.load(Ordering::Relaxed));
                                    eprintln!(
                                        "NEXUTERM_SMOKE_RDP_OK frame=1 input=key,mouse closed=1"
                                    );
                                },
                            );
                        }
                    });
                    eprintln!(
                        "NEXUTERM_SMOKE_RDP_FRAME {}x{}",
                        texture.width(),
                        texture.height()
                    );
                }
                #[cfg(debug_assertions)]
                if first
                    && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
                    && std::env::var_os("NEXUTERM_SMOKE_VNC").is_some()
                {
                    assert_eq!((texture.width(), texture.height()), (64, 48));
                    assert_eq!(&bytes.as_ref()[..4], &[32, 96, 160, 255]);
                    let view = view.clone();
                    let test_keys = test_keys.clone();
                    let test_motion = test_motion.clone();
                    let test_scroll = test_scroll.clone();
                    let tab = test_tab.clone();
                    let desktop = d.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(300),
                        move || {
                            test_keys.emit_by_name::<bool>(
                                "key-pressed",
                                &[&gdk::Key::a, &38u32, &gdk::ModifierType::empty()],
                            );
                            test_keys.emit_by_name::<()>(
                                "key-released",
                                &[&gdk::Key::a, &38u32, &gdk::ModifierType::empty()],
                            );
                            test_motion.emit_by_name::<()>(
                                "motion",
                                &[&(view.width() as f64 / 2.0), &(view.height() as f64 / 2.0)],
                            );
                            test_scroll.emit_by_name::<bool>("scroll", &[&0.0f64, &-1.0f64]);
                            if let Some(path) = std::env::var_os("NEXUTERM_VNC_SCREENSHOT")
                                && let Some(window) = view.root().and_downcast::<gtk::Window>()
                            {
                                let paintable = gtk::WidgetPaintable::new(Some(&window));
                                let snapshot = gtk::Snapshot::new();
                                paintable.snapshot(
                                    &snapshot,
                                    window.width() as f64,
                                    window.height() as f64,
                                );
                                if let Some(node) = snapshot.to_node()
                                    && let Some(renderer) = window.renderer()
                                {
                                    renderer
                                        .render_texture(&node, None)
                                        .save_to_png(path)
                                        .unwrap();
                                }
                            }
                            glib::timeout_add_local_once(
                                std::time::Duration::from_millis(250),
                                move || {
                                    if let (Some(notebook), Some(root)) =
                                        (tab.0.upgrade(), tab.1.upgrade())
                                    {
                                        notebook.close_page(&notebook.page(&root));
                                        assert!(desktop.stop.load(Ordering::Relaxed));
                                        glib::timeout_add_local_once(
                                            std::time::Duration::from_millis(250),
                                            move || {
                                                assert!(desktop.finished.load(Ordering::Relaxed));
                                                eprintln!(
                                                    "NEXUTERM_SMOKE_VNC_OK frame=64x48 input=key,pointer,scroll closed=1"
                                                );
                                            },
                                        );
                                    }
                                },
                            );
                        },
                    );
                }
            }
        }
        glib::ControlFlow::Continue
    });
    #[cfg(debug_assertions)]
    if rdp
        && std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_RDP").is_some()
        && profile.host == "127.0.0.1"
        && profile.port == 13389
    {
        let lab = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab");
        password.set_text(&std::fs::read_to_string(lab.join("password")).unwrap());
        connect.emit_clicked();
    }
    #[cfg(debug_assertions)]
    if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
        && std::env::var_os("NEXUTERM_SMOKE_VNC").is_some()
        && profile.host == "127.0.0.1"
    {
        if std::env::var_os("NEXUTERM_SMOKE_VNC_PASSWORD").is_some() {
            password.set_text("test-only");
        }
        connect.emit_clicked();
    }
    root
}

#[cfg(debug_assertions)]
fn smoke_cancel_certificate() {
    fn cancel(widget: &gtk::Widget) -> Option<gtk::Button> {
        if let Some(button) = widget.downcast_ref::<gtk::Button>()
            && button
                .label()
                .is_some_and(|label| label.replace('_', "") == "Cancelar")
        {
            return Some(button.clone());
        }
        let mut child = widget.first_child();
        while let Some(widget) = child {
            if let Some(button) = cancel(&widget) {
                return Some(button);
            }
            child = widget.next_sibling();
        }
        None
    }
    let (window, button) = gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Window>().ok())
        .filter(|window| window.is_visible())
        .find_map(|window| cancel(window.upcast_ref()).map(|button| (window, button)))
        .expect("Certificate confirmation must be visible");
    assert!(
        button.has_focus(),
        "Reject must be the default focused choice"
    );
    let paintable = gtk::WidgetPaintable::new(Some(&window));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
    if let (Some(node), Some(renderer)) = (snapshot.to_node(), window.renderer()) {
        renderer
            .render_texture(&node, None)
            .save_to_png(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("artifacts/rdp-certificate-dialog.png"),
            )
            .unwrap();
    }
    button.emit_clicked();
}

#[cfg(test)]
mod tests {
    use super::position;
    #[test]
    fn maps_scaled_frame_and_excludes_margins() {
        assert_eq!(
            position(200.0, 150.0, 400, 300, (800, 600)),
            Some((400, 300))
        );
        assert_eq!(position(5.0, 100.0, 500, 300, (800, 600)), None);
        assert_eq!(position(50.0, 0.0, 500, 300, (800, 600)), Some((0, 0)));
        assert_eq!(position(450.0, 300.0, 500, 300, (800, 600)), None);
        assert_eq!(position(0.0, 0.0, 0, 0, (0, 0)), None);
    }
}
