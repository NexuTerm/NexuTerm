mod files;
mod remote_panel;
mod ui;
use gtk::{glib, prelude::*};

fn main() -> glib::ExitCode {
    gtk::gio::resources_register_include!("nexuterm.gresource")
        .expect("No se pudo registrar el icono de NexuTerm");
    let app = gtk::Application::builder()
        .application_id("io.nexuterm.NexuTerm")
        .build();
    app.connect_activate(ui::build);
    app.run()
}
