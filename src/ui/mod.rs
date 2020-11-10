use {
    crate::{
        USER, p, rt, get_obj, resource, warning, new_err,
        resources::APP_ID, twitch::{TwitchExtras, response::Stream}, error::GtResult
    },
    std::{rc::Rc, cell::RefCell},
    gtk::{
        Application, ApplicationInhibitFlags, ApplicationWindow, Label, InfoBar, Revealer,
        StackTransitionType, SettingsExt as GtkSettingsExt, ToggleButton, Button, Stack, prelude::*
    },
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
mod cards;
mod chat;

thread_local! {
    static INFO_BAR: RefCell<Option<(Label, Label, InfoBar)>> = RefCell::new(None)
}

pub struct Ui {
    pub main_window: ApplicationWindow,
    pub search_section: Rc<search::SearchSection>,
    pub auth_window: Rc<auth::AuthWindow>,
    pub about_dialog: Rc<about::AboutDialog>,
    pub settings_window: Rc<settings::SettingsWindow>,
    pub profile_window: Rc<profile::ProfileWindow>,
    pub views_section: Rc<views::ViewsSection>,
    pub chat_section: Rc<chat::ChatSection>,
    pub player_section: Rc<player::PlayerSection>,
    main_stack: Stack,
    views_spinner_overlay: Revealer
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

        // Start stream channel
        let (tx, rx) = glib::MainContext::channel::<Stream>(glib::Priority::default());

        let inner = Rc::new(Self {
            search_section: search::SearchSection::configure(&builder),
            auth_window: auth::AuthWindow::configure(app, &main_window),
            about_dialog: about::AboutDialog::configure(&main_window),
            settings_window: settings::SettingsWindow::configure(&main_window, &settings),
            profile_window: profile::ProfileWindow::configure(&main_window, &app_action_group),
            views_section: views::ViewsSection::configure(&builder, &settings, tx),
            chat_section: chat::ChatSection::configure(&builder),
            // Make sure to configure SettingsWindow before PlayerSection!
            player_section: player::PlayerSection::configure(&builder, &settings),
            main_window,
            main_stack: get_obj!(builder, "main-stack"),
            views_spinner_overlay: get_obj!(builder, "views-spinner-overlay")
        });

        rx.attach(None, clone!(
            @strong inner
        => move |stream| {

            inner.views_spinner_overlay.set_visible(true);

            let logged_in_user = (*USER.lock().unwrap()).clone().unwrap();
            let a_user_name = stream.user_name.clone();

            rt::run_cb_local(async move {

                let access_token = p!(TwitchExtras::access_token(&a_user_name, Some(logged_in_user.oauth_token)).await);

                // Add newline
                // See https://github.com/rutgersc/m3u8-rs/issues/12
                let usher = p!(TwitchExtras::usher(&a_user_name, access_token.sig, access_token.token).await) + "\n";

                let parsed = m3u8_rs::parse_playlist_res(usher.as_bytes());
                match parsed {
                    Ok(m3u8_rs::playlist::Playlist::MasterPlaylist(mp)) => {
                        let mut qualities = Vec::with_capacity(mp.variants.len());
                        for mut variant in mp.variants {
                            qualities.push((variant.alternatives.remove(0).name, variant.uri));
                        }
                        Ok(qualities)
                    },
                    _ => Err(new_err!("Failed to parse playlist"))
                }

            }, clone!(@strong inner => move |res: GtResult<Vec<(String, String)>>| {

                inner.views_spinner_overlay.set_visible(false);
                match res {
                    Ok(qualities) => {
                        //inner.show_player();
                        inner.chat_section.connect(stream.user_name.clone());
                        inner.player_section.play(qualities[0].1.clone());
                        inner.player_section.set_title(&stream.title);
                        inner.player_section.set_streamer(&stream.user_name);
                        inner.player_section.set_broadcaster_id(stream.user_id.clone());
                        inner.player_section.set_viewer_count(stream.viewer_count);
                        inner.player_section.start_update_loop(stream.user_id.clone());
                        inner.player_section.set_qualities(qualities.clone());
                        //inner.player_section.play(qualities[0].1.clone());
                        inner.show_player();
                    },
                    Err(e) => show_info_bar("Error", &e.to_string(), None::<&gtk::Widget>, gtk::MessageType::Error)
                }

            }));


            glib::Continue(true)

        }));

        INFO_BAR.with(|ib| {
            let infobar = get_obj!(InfoBar, builder, "main-info");
            infobar.connect_response(|ib, res| {
                if res == gtk::ResponseType::Close {
                    ib.set_revealed(false);
                    ib.set_visible(false);
                }
            });
            ib.borrow_mut().replace((
                get_obj!(Label, builder, "main-info-title"),
                get_obj!(Label, builder, "main-info-body"),
                infobar
            ));
        });

        let headerbar_stack = get_obj!(Stack, builder, "headerbar-stack");
        unsafe { inner.main_stack.connect_notify_unsafe(Some("visible-child-name"), clone!(@strong inner => move |stack, _| {
            let name = stack.get_visible_child_name().map(|n| n.to_string()).unwrap_or(String::new());
            match name.as_str() {
                "player" => headerbar_stack.set_visible_child_full("return-to-views", StackTransitionType::SlideRight),
                _ => {
                    inner.chat_section.disconnect();
                    inner.player_section.stop();
                    headerbar_stack.set_visible_child_full("main-menu", StackTransitionType::SlideLeft);
                }
            }
        })) };

        get_obj!(Button, builder, "player-return-to-views").connect_clicked(clone!(@strong inner => move |_| {
            inner.show_views();
        }));

        inner.main_window.connect_delete_event(move |win, _| {
            let (width, height) = win.get_size();
            settings.set_int("window-width", width).unwrap();
            settings.set_int("window-height", height).unwrap();
            gtk::Inhibit(false)
        });

        get_obj!(ToggleButton, builder, "app-search-button").connect_toggled(clone!(@strong inner => move |btn| {
            match btn.get_active() {
                true => inner.search_section.show(),
                false => inner.search_section.hide()
            }
        }));

        let reload_action = SimpleAction::new("reload", None);
        app_action_group.add_action(&reload_action);
        reload_action.connect_activate(clone!(@strong inner => move |_, _| {
            let name = inner.main_stack
                .get_visible_child_name()
                .map(|n| n.to_string())
                .unwrap_or(String::new());
            match name.as_str() {
                "views" => inner.views_section.reload(),
                "player" => inner.player_section.reload(),
                _ => { }
            }
        }));

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

pub(super) fn show_info_bar<W: glib::IsA<gtk::Widget>>(title: &str, body: &str, widget: Option<&W>, kind: gtk::MessageType) {

    INFO_BAR.with(|ib| {
        if let Some((ib_title, ib_body, infobar)) = &*ib.borrow() {
            if let Some(ae) = infobar.get_action_area() {
                for child in ae.get_children() {
                    ae.remove(&child);
                }
            }
            ib_title.set_text(title);
            ib_body.set_text(body);
            infobar.set_message_type(kind);
            if let Some(action_widget) = widget {
                action_widget.show();
                infobar.add_action_widget(action_widget, gtk::ResponseType::None)
            }
            infobar.set_visible(true);
            infobar.set_revealed(true);
            infobar.show_all();
        } else {
            warning!("Infobar not initialized")
        }
    });

}
