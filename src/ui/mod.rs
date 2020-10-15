use {
    crate::{get_obj, resource},
    std::rc::Rc,
    gtk::{Application, ApplicationWindow, prelude::*},
    gio::{SimpleAction, SimpleActionGroup, prelude::*},
    glib::clone
};

mod about;
mod settings;
mod auth;
mod player;
mod profile;

pub struct Ui {
    pub auth_window: Rc<auth::AuthWindow>,
    pub about_dialog: Rc<about::AboutDialog>,
    pub settings_window: Rc<settings::SettingsWindow>,
    // pub profile_window: Rc<profile::ProfileWindow>,
    // pub chat_section: Rc<chat::ChatSection>
    // pub player_section: Rc<player::PlayerSection>
}

impl Ui {
    pub fn build(app: &Application) -> Ui {

        let builder = Self::builder();

        let main_window: ApplicationWindow = get_obj!(builder, "main-window");
        main_window.set_application(Some(app));
        main_window.show_all();

        let app_action_group = SimpleActionGroup::new();
        main_window.insert_action_group("app", Some(&app_action_group));

        let inner = Self {
            auth_window: auth::AuthWindow::configure(&main_window),
            about_dialog: about::AboutDialog::configure(&main_window),
            settings_window: settings::SettingsWindow::configure(&main_window)
        };

        // let open_profile_action = SimpleAction::new("profile", None);
        // app_action_group.add_action(&open_profile_action);
        // open_profile_action.connect_activate(clone!(@strong inner.profile_window as pw => move |_, _| {

        // }));

        let open_settings_action = SimpleAction::new("settings", None);
        app_action_group.add_action(&open_settings_action);
        open_settings_action.connect_activate(clone!(@strong inner.settings_window as sw => move |_, _| {
            sw.show()
        }));

        let open_about_action = SimpleAction::new("about", None);
        app_action_group.add_action(&open_about_action);
        open_about_action.connect_activate(clone!(@strong inner.about_dialog as ad => move |_, _| {
            ad.show()
        }));

        let quit_action = SimpleAction::new("quit", None);
        app_action_group.add_action(&quit_action);
        quit_action.connect_activate(clone!(@weak app => move |_, _| {
            app.quit()
        }));


        inner

    }

    fn builder() -> gtk::Builder {

        let b = gtk::Builder::new();
        b.add_from_resource(resource!("ui/main")).unwrap();

        b

    }

}
