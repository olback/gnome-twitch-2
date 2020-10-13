use {
    gtk::{Application, prelude::*},
    gio::prelude::*,
    ui::Ui,
    error::GtError
};

mod resources;
mod ui;
mod macros;
mod log;
mod error;
mod backends;
mod helix;

fn main() {

    debug!("Starting...");

    // Init GST
    gst::init().unwrap();

    resources::register_resources();

    let app = Application::new(Some("net.olback.GnomeTwitch2"), Default::default()).expect("Failed to create application");
    glib::set_program_name(Some("GnomeTwitch2"));

    app.connect_activate(|app| {
        resources::register_css();
        Ui::build(app);
    });

    app.run(&std::env::args().collect::<Vec<_>>());

    debug!("Good bye\n");

}
