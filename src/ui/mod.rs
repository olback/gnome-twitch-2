use {
    crate::{USER, get_obj, resource, resources::APP_ID},
    std::rc::Rc,
    gtk::{Application, ApplicationInhibitFlags, ApplicationWindow, SettingsExt as GtkSettingsExt, prelude::*},
    gio::{SimpleAction, SimpleActionGroup, Settings, SettingsExt, prelude::*},
    glib::clone
};

mod about;
mod settings;
mod auth;
mod player;
mod profile;
mod search;
mod views;
pub mod cards;

pub struct Ui {
    pub main_window: ApplicationWindow,
    pub search_section: Rc<search::SearchSection>,
    pub auth_window: Rc<auth::AuthWindow>,
    pub about_dialog: Rc<about::AboutDialog>,
    pub settings_window: Rc<settings::SettingsWindow>,
    pub profile_window: Rc<profile::ProfileWindow>,
    pub views_section: Rc<views::ViewsSection>,
    // pub chat_section: Rc<chat::ChatSection>
    pub player_section: Rc<player::PlayerSection>,
    main_stack: gtk::Stack
}

impl Ui {
    pub fn build(app: &Application) -> Rc<Ui> {

        let builder = Self::builder();
        let settings = Settings::new(APP_ID);

        let main_window: ApplicationWindow = get_obj!(builder, "main-window");
        main_window.set_default_size(settings.get_int("window-width"), settings.get_int("window-height"));
        main_window.set_application(Some(app));
        main_window.show_all();
        app.inhibit(
            Some(&main_window),
            ApplicationInhibitFlags::IDLE,
            Some("This is a video player, locking the desktop could lead to a bad experience.")
        );

        let app_action_group = SimpleActionGroup::new();
        main_window.insert_action_group("app", Some(&app_action_group));

        // Theme change
        let gtk_settings = gtk::Settings::get_default().unwrap();
        set_theme(&settings, &gtk_settings);
        settings.connect_changed(move |settings, key| {
            if key == "theme" {
                set_theme(&settings, &gtk_settings);
            }
        });

        let inner = Rc::new(Self {
            search_section: search::SearchSection::configure(&builder),
            auth_window: auth::AuthWindow::configure(app, &main_window),
            about_dialog: about::AboutDialog::configure(&main_window),
            settings_window: settings::SettingsWindow::configure(&main_window, &settings),
            profile_window: profile::ProfileWindow::configure(&main_window, &app_action_group),
            views_section: views::ViewsSection::configure(&builder, &settings),
            // chat_section
            // Make sure to configure SettingsWindow before PlayerSection!
            player_section: player::PlayerSection::configure(app, &builder, &settings),
            main_window,
            main_stack: get_obj!(builder, "main-stack")
        });

        inner.main_window.connect_delete_event(move |win, _| {
            let (width, height) = win.get_size();
            settings.set_int("window-width", width).unwrap();
            settings.set_int("window-height", height).unwrap();
            gtk::Inhibit(false)
        });

        let logout_action = SimpleAction::new("logout", None);
        app_action_group.add_action(&logout_action);
        logout_action.connect_activate(clone!(@strong inner => move |_, _| {
            match *USER.lock().unwrap() {
                Some(ref user) => user.logout(),
                None => unreachable!()
            }.expect("Failed to log out");
            *USER.lock().unwrap() = None;
            inner.show_main_spinner();
            inner.settings_window.hide();
            inner.profile_window.hide();
            inner.auth_window.show();
            glib::timeout_add_local(200, clone!(@strong inner => move || {
                let logged_in = USER.lock().unwrap().is_some();
                match logged_in {
                    true => {
                        inner.views_section.notify();
                        inner.show_views();
                        glib::Continue(false)
                    },
                    false => glib::Continue(true)
                }
            }));
        }));

        let open_profile_action = SimpleAction::new("profile", None);
        app_action_group.add_action(&open_profile_action);
        open_profile_action.connect_activate(clone!(@strong inner.profile_window as pw => move |_, _| {
            pw.show()
        }));

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

    pub fn show_main_spinner(&self) {
        self.main_stack.set_visible_child_name("spinner")
    }

    pub fn show_views(&self) {
        self.main_stack.set_visible_child_name("views")
    }

    pub fn show_player(&self) {
        self.main_stack.set_visible_child_name("player")
    }

    fn builder() -> gtk::Builder {

        let b = gtk::Builder::new();
        b.add_from_resource(resource!("ui/main")).unwrap();

        b

    }

}

fn set_theme(settings: &Settings, gtk_settings: &gtk::Settings) {
    match settings.get_string("theme").map(|v| v.to_string()) {
        Some(theme) => match theme.as_str() {
            "default" => gtk_settings.reset_property("gtk-application-prefer-dark-theme"),
            "light" => gtk_settings.set_property("gtk-application-prefer-dark-theme", &false).unwrap(),
            "dark" => gtk_settings.set_property("gtk-application-prefer-dark-theme", &true).unwrap(),
            _ => { } // Do nothing
        },
        _ => { } // Do nothing
    }
}
