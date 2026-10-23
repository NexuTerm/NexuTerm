mod askpass;
mod desktop_view;
mod file_browser;
mod files;
mod ftp;
mod ftp_legacy;
mod ftp_transfer;
mod remote_panel;
mod sftp_session;
mod terminal_colors;
mod ui;
use gtk::{glib, prelude::*};

fn main() -> glib::ExitCode {
    gtk::gio::resources_register_include!("nexuterm.gresource")
        .expect("No se pudo registrar el icono de NexuTerm");
    if std::env::var_os("NEXUTERM_ASKPASS").is_some() {
        return askpass::run();
    }
    let app = gtk::Application::builder()
        .application_id("io.nexuterm.NexuTerm")
        .build();
    app.connect_activate(ui::build);
    app.run()
}
