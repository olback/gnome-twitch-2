use {
    std::sync::Mutex,
    gtk::Application,
    gio::prelude::*,
    glib::clone,
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
    // TODO: This should probably be a tokio::sync::Mutex instead of std::sync::Mutex
    pub static ref USER: Mutex<Option<User>> = Mutex::new(None);
}

fn main() {

    debug!("Starting...");

    gst::init().unwrap(); // Init GST
    gtk::init().unwrap(); // Init GTK
    resources::register_resources(); // Load resources

    let app = Application::new(Some(resources::APP_ID), Default::default()).expect("Failed to create application");
    glib::set_program_name(Some("Gnome Twitch 2"));

    app.connect_activate(|app| {
        resources::register_css();
        let u = Ui::build(app);
        match User::load() {
            Ok(user) => { USER.lock().unwrap().replace(user); },
            Err(_) => u.auth_window.show()
        }
        glib::timeout_add_local(200, clone!(@strong u => move || {
            let logged_in = USER.lock().unwrap().is_some();
            match logged_in {
                true => {
                    u.views_section.notify();
                    u.show_views();
                    glib::Continue(false)
                },
                false => glib::Continue(true)
            }
        }));
    });

    app.run(&std::env::args().collect::<Vec<_>>());

}
