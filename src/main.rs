mod askpass;
mod context_menu;
mod desktop_view;
mod file_browser;
mod files;
mod editor_compare;
mod widget_state;
mod recovery_ui;
mod chat_history_ui;
mod syntax;
mod ftp;
mod ftp_legacy;
mod ftp_transfer;
mod remote_panel;
mod transfer_queue_ui;
mod sftp_session;
mod terminal_colors;
mod ui;
mod supervised_ui;
use gtk::{glib, prelude::*};

fn main() -> glib::ExitCode {
    let args:Vec<String>=std::env::args().collect();
    if args.get(1).is_some_and(|a|a=="--terminal-mcp"){return if args.len()==4&&nexuterm::terminal_mcp::stdio(&args[2],&args[3])==0{glib::ExitCode::SUCCESS}else{glib::ExitCode::FAILURE};}
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
