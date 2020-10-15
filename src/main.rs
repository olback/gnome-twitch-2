use {
    gtk::{Application, prelude::*},
    gio::prelude::*,
    ui::Ui,
    twitch::Twitch
};

mod resources;
mod ui;
mod macros;
mod log;
mod error;
mod backends;
mod twitch;
mod rt;

fn main() {

    debug!("Starting...");

    // Init GST
    gst::init().unwrap();

    resources::register_resources();

    // let m = std::sync::Mutex::new(c);

    let app = Application::new(Some("net.olback.GnomeTwitch2"), Default::default()).expect("Failed to create application");
    glib::set_program_name(Some("GnomeTwitch2"));

    app.connect_activate(|app| {
        resources::register_css();
        let u = Ui::build(app);
        // u.auth_window.show();
    });

    app.run(&std::env::args().collect::<Vec<_>>());

}
