//! Graphical SSH_ASKPASS entry point. stdout is exclusively the SSH response pipe.
use gtk::{glib, prelude::*};
use std::{cell::RefCell, io::Write, rc::Rc};

pub fn run() -> glib::ExitCode {
    if gtk::init().is_err() {
        return glib::ExitCode::FAILURE;
    }
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Autenticación SFTP".into());
    let prompt: String = prompt.chars().take(4096).collect();
    let loop_ = glib::MainLoop::new(None, false);
    let answer: Rc<RefCell<Option<zeroize::Zeroizing<String>>>> = Rc::new(RefCell::new(None));
    let mut auth_window = None;
    if std::env::var("SSH_ASKPASS_PROMPT").as_deref() == Ok("confirm")
        || prompt.contains("Are you sure you want to continue connecting")
    {
        let dialog = gtk::AlertDialog::builder()
            .modal(true)
            .message("Verificar identidad del servidor SFTP")
            .detail(&prompt)
            .buttons(["Cancelar", "Confiar en este servidor"])
            .cancel_button(0)
            .default_button(0)
            .build();
        let result = answer.clone();
        let main = loop_.clone();
        glib::spawn_future_local(async move {
            if dialog.choose_future(gtk::Window::NONE).await == Ok(1) {
                *result.borrow_mut() = Some(zeroize::Zeroizing::new("yes".into()));
            }
            main.quit();
        });
    } else {
        let window = gtk::Window::builder()
            .title("Autenticación SFTP · NexuTerm")
            .default_width(440)
            .resizable(false)
            .build();
        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        for set in [
            gtk::Widget::set_margin_top,
            gtk::Widget::set_margin_bottom,
            gtk::Widget::set_margin_start,
            gtk::Widget::set_margin_end,
        ] {
            set(content.upcast_ref(), 18);
        }
        let label = gtk::Label::new(Some(&prompt));
        label.set_wrap(true);
        label.set_xalign(0.0);
        content.append(&label);
        let password = gtk::PasswordEntry::builder()
            .show_peek_icon(true)
            .placeholder_text("Contraseña o frase de paso")
            .build();
        content.append(&password);
        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let cancel = gtk::Button::with_label("Cancelar");
        let accept = gtk::Button::with_label("Conectar");
        accept.add_css_class("suggested-action");
        buttons.append(&cancel);
        buttons.append(&accept);
        content.append(&buttons);
        window.set_child(Some(&content));
        let main = loop_.clone();
        cancel.connect_clicked(move |_| main.quit());
        let main = loop_.clone();
        window.connect_close_request(move |_| {
            main.quit();
            glib::Propagation::Proceed
        });
        let main = loop_.clone();
        let result = answer.clone();
        let entry = password.downgrade();
        accept.connect_clicked(move |_| {
            if let Some(entry) = entry.upgrade() {
                *result.borrow_mut() = Some(zeroize::Zeroizing::new(entry.text().to_string()));
                entry.set_text("");
            }
            main.quit();
        });
        let button = accept.downgrade();
        password.connect_activate(move |_| {
            if let Some(button) = button.upgrade() {
                button.emit_clicked();
            }
        });
        window.present();
        auth_window = Some(window);
        password.grab_focus();
        #[cfg(debug_assertions)]
        if std::env::var_os("NEXUTERM_SMOKE_TEST").is_some()
            && std::env::var_os("NEXUTERM_SMOKE_SFTP_PASSWORD").is_some()
            && prompt.contains("tester@127.0.0.1")
        {
            glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
                password.set_text(
                    std::fs::read_to_string(
                        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab/password"),
                    )
                    .unwrap()
                    .trim_end_matches(['\r', '\n']),
                );
                accept.emit_clicked();
            });
        }
    }
    loop_.run();
    if let Some(window) = auth_window {
        window.close();
    }
    let Some(answer) = answer.borrow_mut().take() else {
        return glib::ExitCode::FAILURE;
    };
    let mut stdout = std::io::stdout().lock();
    if stdout
        .write_all(answer.as_bytes())
        .and_then(|_| stdout.write_all(b"\n"))
        .and_then(|_| stdout.flush())
        .is_err()
    {
        return glib::ExitCode::FAILURE;
    }
    glib::ExitCode::SUCCESS
}
