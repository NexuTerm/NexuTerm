mod files;
mod remote_panel;
mod ui;
use gtk::{glib, prelude::*};

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("io.nexuterm.NexuTerm")
        .build();
    app.connect_activate(ui::build);
    app.run()
}
