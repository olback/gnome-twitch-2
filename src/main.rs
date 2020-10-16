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

    gst::init().unwrap(); // Init GST
    resources::register_resources(); // Load resources

    let app = Application::new(Some(resources::APP_ID), Default::default()).expect("Failed to create application");
    glib::set_program_name(Some("GnomeTwitch2"));

    app.connect_activate(|app| {
        resources::register_css();
        let u = Ui::build(app);
        // u.auth_window.show();
    });

    app.run(&std::env::args().collect::<Vec<_>>());

}
