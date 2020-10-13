use {
    crate::{get_obj, resource},
    gtk::{Application, ApplicationWindow, prelude::*}
};

mod about;
mod login;
mod settings;
mod twitch;
mod player;

pub struct Ui;

impl Ui {
    pub fn build(app: &Application) {
        let builder = Self::builder();

        // TODO: Move this
        let player_menu = gio::Menu::new();
        let quality_options = gio::Menu::new();
        let quality_options_source = gio::MenuItem::new(Some("Source (1080p60)"), None);
        quality_options_source.set_attribute_value("type", Some(&"radioitem".into()));
        quality_options.append_item(&quality_options_source);
        let quality_options_high = gio::MenuItem::new(Some("High (720p60)"), None);
        quality_options.append_item(&quality_options_high);
        let quality_options_medium = gio::MenuItem::new(Some("Medium (480p)"), None);
        quality_options.append_item(&quality_options_medium);
        let quality_options_low = gio::MenuItem::new(Some("Low (360p)"), None);
        quality_options.append_item(&quality_options_low);
        player_menu.append_submenu(Some("Quality"), &quality_options);
        get_obj!(gtk::MenuButton, builder, "player-menu-button").set_menu_model(Some(&player_menu));

        let main_window: ApplicationWindow = get_obj!(builder, "main-window");
        about::configure(&main_window);
        // twitch::configure(&main_window);
        player::configure(app, &builder);
        main_window.set_application(Some(app));
        main_window.show_all();
    }

    fn builder() -> gtk::Builder {

        let b = gtk::Builder::new();
        b.add_from_resource(resource!("ui/main")).unwrap();

        b

    }

}
