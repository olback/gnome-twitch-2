use {
    crate::{
        get_obj, warning, error::GtResult, resources::CLIENT_ID,
        backends::{GtPlayerBackend, BACKENDS, GtPlayerState, GtPlayerEvent},
        rt, USER, ui::show_info_bar, twitch::{Twitch, TwResult}
    },
    std::{rc::Rc, cell::RefCell},
    gtk::{
        Builder, Box as GtkBox, Button, ToggleButton, IconSize, MessageType, Spinner,
        Revealer, MenuButton, VolumeButton, ApplicationWindow, Image, Label, EventBox, prelude::*
    },
    gio::{SimpleAction, Settings, SettingsExt, SettingsBindFlags, prelude::*},
    glib::{clone, SourceId, Sender}
};

const UPDATE_INTERVAL: u32 = 15;

pub struct PlayerSection {
    player: Box<dyn GtPlayerBackend>,
    title: Label,
    streamer: Label,
    viewers: Label,
    settings_button: MenuButton,
    quality_selection_action: SimpleAction,
    loop_handle: RefCell<Option<SourceId>>,
    tx: Sender<(String, u32)>,
    broadcaster_id: RefCell<Option<String>>
}

impl PlayerSection {

    pub fn configure(builder: &Builder, settings: &Settings) -> Rc<Self> {

        let is_fullscreen = Rc::new(RefCell::new(false));
        let fullscreen_image = get_obj!(Image, builder, "fullscreen-image");
        let main_window = get_obj!(ApplicationWindow, builder, "main-window");
        let player_action_group = gio::SimpleActionGroup::new();
        let quality_selection_action = gio::SimpleAction::new_stateful(
            "quality-selection",
            Some(&glib::VariantType::new("s").unwrap()),
            &String::new().into()
        );
        main_window.insert_action_group("player", Some(&player_action_group));
        player_action_group.add_action(&quality_selection_action);
        main_window.connect_window_state_event(clone!(@strong is_fullscreen => move |_, state| {
            let new_state = state.get_new_window_state();
            if new_state.contains(gdk::WindowState::FULLSCREEN) {
                is_fullscreen.replace(true);
                fullscreen_image.set_from_icon_name(Some("view-restore-symbolic"), IconSize::Button);
            } else {
                is_fullscreen.replace(false);
                fullscreen_image.set_from_icon_name(Some("view-fullscreen-symbolic"), IconSize::Button);
            }
            gtk::Inhibit(false)
        }));

        get_obj!(Button, builder, "player-fullscreen").connect_clicked(move |_| {
            if *is_fullscreen.borrow() {
                main_window.unfullscreen()
            } else {
                main_window.fullscreen()
            }
        });

        let chat_section = get_obj!(GtkBox, builder, "chat-section");
        get_obj!(ToggleButton, builder, "player-toggle-chat").connect_toggled(move |btn| {
            chat_section.set_visible(!btn.get_active());
        });

        let player_top_revealer = get_obj!(Revealer, builder, "player-top-revealer");
        let player_bottom_revealer = get_obj!(Revealer, builder, "player-bottom-revealer");
        let revealed = RefCell::new(true); // This is stupied
        get_obj!(EventBox, builder, "player-controls-trigger").connect_button_press_event(move |_, _| {
            let v = *revealed.borrow();
            if v {
                player_top_revealer.set_reveal_child(false);
                player_bottom_revealer.set_reveal_child(false);
            } else {
                player_top_revealer.set_reveal_child(true);
                player_bottom_revealer.set_reveal_child(true);
            }
            revealed.replace(!v);
            gtk::Inhibit(false)
        });

        settings.bind("volume", &get_obj!(VolumeButton, builder, "player-volume"), "value", SettingsBindFlags::DEFAULT);

        let play_pause_button = Rc::new(get_obj!(Button, builder, "player-play-pause"));
        let play_image = get_obj!(Image, builder, "play-image");
        let pause_image = get_obj!(Image, builder, "pause-image");
        let buffer_spinner = get_obj!(Spinner, builder, "player-buffering");
        let cb = clone!(
            @strong play_pause_button
        => move |evt| {
            match evt {
                GtPlayerEvent::StateChange(state) => match state {
                    GtPlayerState::Eos => {
                        play_pause_button.set_image(Some(&play_image));
                        show_info_bar(
                            "The stream has ended",
                            "The stramer ended the stream. Thanks for watching!",
                            None::<&gtk::Widget>,
                            MessageType::Info
                        );
                        buffer_spinner.set_visible(false);
                    },
                    GtPlayerState::Buffering => {
                        buffer_spinner.set_visible(true);
                    },
                    GtPlayerState::Playing => {
                        play_pause_button.set_image(Some(&pause_image));
                        buffer_spinner.set_visible(false);
                    },
                    GtPlayerState::Paused => {
                        play_pause_button.set_image(Some(&play_image));
                        buffer_spinner.set_visible(false);
                    },
                    GtPlayerState::Stopped => {
                        play_pause_button.set_image(Some(&play_image));
                        buffer_spinner.set_visible(false);
                    },
                    _ => { }
                },
                GtPlayerEvent::Warning(warning) => {
                    // show_info_bar("Player warning", &warning, MessageType::Warning)
                    show_info_bar("Player warning", &warning, None::<&gtk::Widget>, MessageType::Warning);
                    warning!("{}", warning);
                },
                GtPlayerEvent::Error(error) => {
                    show_info_bar("Player error", &error, None::<&gtk::Widget>, MessageType::Error);
                    warning!("{}", error);
                }
            }
        });

        let (tx, rx) = glib::MainContext::channel::<(String, u32)>(glib::PRIORITY_DEFAULT);

        let inner = Rc::new(Self {
            player: get_backend(settings)(settings, Box::new(cb)).unwrap(),
            title: get_obj!(builder, "player-title"),
            streamer: get_obj!(builder, "player-streamer"),
            viewers: get_obj!(builder, "player-viewers"),
            settings_button: get_obj!(builder, "player-menu-button"),
            quality_selection_action,
            loop_handle: RefCell::new(None),
            tx,
            broadcaster_id: RefCell::new(None)
        });

        rx.attach(None, clone!(@strong inner => move |msg| {

            inner.set_title(&msg.0);
            inner.set_viewer_count(msg.1);

            glib::Continue(true)

        }));

        get_obj!(Button, builder, "player-clip").connect_clicked(clone!(@strong inner => move |_| {

            if let Some(user) = (*USER.lock().unwrap()).clone() {
                if let Some(bid) = (&*inner.broadcaster_id.borrow()).clone() {
                    rt::run_cb_local(async move {
                        let tw = Twitch::new(Some(user.oauth_token), Some(CLIENT_ID.into()));
                        let res = tw.create_clip(bid, None).await?;
                        Ok(res.data)
                    }, |msg: TwResult<_>| {
                        match msg {
                            // This vec should always contain one item on success, but check anyways
                            // since we don't want to crash if Twitch messes up :)
                            Ok(mut clips) => match clips.pop() {
                                Some(clip) => {
                                    let button = Button::with_label("Edit");
                                    button.connect_clicked(move |_| {
                                        if let Err(e) = gtk::show_uri_on_window(None::<&gtk::Window>, &clip.edit_url, 0) {
                                            warning!("{}", e)
                                        }
                                    });
                                    show_info_bar(
                                        "Clip created",
                                        "Click the edit button to edit your new clip.",
                                        Some(&button),
                                        gtk::MessageType::Info
                                    )
                                },
                                None => show_info_bar(
                                    "Clip error",
                                    "Clip sucessfully created but no data returned.",
                                    None::<&gtk::Widget>,
                                    gtk::MessageType::Error
                                )
                            },
                            Err(e) => show_info_bar(
                                "Clip error",
                                &e.to_string(),
                                None::<&gtk::Widget>,
                                gtk::MessageType::Error
                            )
                        }
                    });


                } else { show_info_bar(
                    "Clip error",
                    "Broadcaster ID not set",
                    None::<&gtk::Widget>,
                    gtk::MessageType::Error
                ) }
            }

        }));

        inner.quality_selection_action.connect_activate(clone!(@strong inner => move |act, url| {
            if let Some(ref val) = url {
                act.set_state(val);
            }
            drop(inner.player.stop());
            drop(inner.player.set_uri(Some(url.map(|u| u.to_string().replace("'", "")).expect("Failed to get url from variant"))));
            drop(inner.player.play());
        }));

        let container: GtkBox = get_obj!(builder, "player-container");
        let video_widget = inner.player.get_widget();
        container.pack_start(video_widget, true, true, 0);
        video_widget.show_all();
        video_widget.realize();

        play_pause_button.connect_clicked(clone!(@strong inner => move |_| {
            match inner.player.get_state() {
                GtPlayerState::Playing => {
                    if let Err(e) = inner.player.pause() {
                        show_info_bar(
                            "Could not pause",
                            &e.to_string(),
                            None::<&gtk::Widget>,
                            MessageType::Error
                        )
                    }
                },
                GtPlayerState::Paused => {
                    if let Err(e) = inner.player.play() {
                        show_info_bar(
                            "Could not play",
                            &e.to_string(),
                            None::<&gtk::Widget>,
                            MessageType::Error
                        )
                    }
                }
                _ => { }
            }
        }));

        inner

    }

    pub fn reload(&self) {

        warning!("TODO");

    }

    pub fn set_title(&self, title: &str) {

        self.title.set_text(title)

    }

    pub fn set_streamer(&self, title: &str) {

        self.streamer.set_text(title)

    }

    pub fn set_broadcaster_id(&self, id: String) {

        self.broadcaster_id.replace(Some(id));

    }

    pub fn set_viewer_count(&self, views: u32) {

        self.viewers.set_text(&format!("{}", views))

    }

    pub fn start_update_loop(&self, bid: String) {

        let tx = self.tx.clone();

        let id = glib::timeout_add_seconds_local(UPDATE_INTERVAL, move || {

            if let Some(user) = (*USER.lock().unwrap()).clone() {

                let bid = bid.clone();

                rt::run_cb_local(async move {

                    let tw = Twitch::new(Some(user.oauth_token), Some(CLIENT_ID.into()));
                    let res = tw.get_streams(
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(vec![bid]),
                        None
                    ).await?;
                    Ok(res.data)

                }, clone!(@strong tx => move |msg: TwResult<_>| {

                    match msg {
                        Ok(mut streams) => {
                            // Pop removes the last element but it should not matter here
                            // since we expect max one stream.
                            if let Some(stream) = streams.pop() {
                                tx.send((stream.title, stream.viewer_count)).expect("Could not send update");
                            }
                        },
                        Err(e) => warning!("{}", e)
                    }

                }));

            }

            glib::Continue(true)

        });

        self.loop_handle.replace(Some(id));

    }

    pub fn set_qualities(&self, qualities: Vec<(String, String)>) {

        let player_menu = gio::Menu::new();
        for (name, url) in qualities {
            let menu_item = gio::MenuItem::new(
                Some(&name.trim().replace("source", "Source").replace("audio_only", "Audio Only")),
                None
            );
            menu_item.set_action_and_target_value(
                Some("player.quality-selection"),
                Some(&url.into())
            );
            player_menu.append_item(&menu_item);
        }

        self.settings_button.set_menu_model(Some(&player_menu));

    }

    pub fn play(&self, uri: String) {

        self.quality_selection_action.set_state(&uri.clone().into());
        drop(self.player.set_uri(Some(uri)));
        drop(self.player.play());

    }

    pub fn stop(&self) {

        drop(self.player.stop());
        self.loop_handle.borrow_mut().take().map(|id| {
            glib::source_remove(id)
        });

    }

}

fn get_backend(settings: &Settings) -> fn(&Settings, Box<dyn Fn(GtPlayerEvent)>) -> GtResult<Box<dyn GtPlayerBackend>> {

    let backend_id = settings.get_string("backend-player").unwrap().to_string();

    for (_, id, ptr) in BACKENDS {
        if backend_id == *id {
            return *ptr
        }
    }

    unreachable!()

}
