use {
    std::sync::Mutex,
    gtk::Application,
    gio::prelude::*,
    ui::Ui,
    resources::loader::ResourceLoader,
    user::User
};

mod resources;
mod ui;
mod macros;
mod log;
mod error;
mod backends;
mod twitch;
mod rt;
mod user;

lazy_static::lazy_static! {
    pub static ref ASSETS: ResourceLoader = ResourceLoader::new("assets.db").unwrap();
    pub static ref USER: Mutex<Option<User>> = Mutex::new(None);
}

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
        match User::load() {
            Ok(user) => { USER.lock().unwrap().replace(user); },
            Err(_) => u.auth_window.show()
        }
    });

    app.run(&std::env::args().collect::<Vec<_>>());

}
