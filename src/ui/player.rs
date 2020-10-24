use {
    crate::{
        resource, get_obj, message, warning, error::GtResult, resources::APP_ID,
        backends::{GtPlayerBackend, BACKENDS, GtPlayerState, GtPlayerEvent},
        ui::show_info_bar
    },
    std::{rc::Rc, cell::RefCell},
    gtk::{
        Application, Builder, Box as GtkBox, Button, ToggleButton, IconSize, MessageType, Spinner,
        Revealer, MenuButton, VolumeButton, ApplicationWindow, Image, Label, EventBox, prelude::*
    },
    gio::{Settings, SettingsExt, SettingsBindFlags},
    glib::clone
    // gst::prelude::*
};

// let player_menu = gio::Menu::new();
// let quality_options = gio::Menu::new();
// let quality_options_source = gio::MenuItem::new(Some("Source (1080p60)"), None);
// quality_options_source.set_attribute_value("type", Some(&"radioitem".into()));
// quality_options.append_item(&quality_options_source);
// let quality_options_high = gio::MenuItem::new(Some("High (720p60)"), None);
// quality_options.append_item(&quality_options_high);
// let quality_options_medium = gio::MenuItem::new(Some("Medium (480p)"), None);
// quality_options.append_item(&quality_options_medium);
// let quality_options_low = gio::MenuItem::new(Some("Low (360p)"), None);
// quality_options.append_item(&quality_options_low);
// player_menu.append_submenu(Some("Quality"), &quality_options);
// get_obj!(gtk::MenuButton, builder, "player-menu-button").set_menu_model(Some(&player_menu));

pub struct PlayerSection {
    player: Box<dyn GtPlayerBackend>,
    settings_button: MenuButton
}

impl PlayerSection {

    pub fn configure(app: &Application, builder: &Builder, settings: &Settings) -> Rc<Self> {

        let is_fullscreen = Rc::new(RefCell::new(false));
        let fullscreen_image = get_obj!(Image, builder, "fullscreen-image");
        let main_window = get_obj!(ApplicationWindow, builder, "main-window");
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
            println!("New state: {:#?}", evt);
            match evt {
                GtPlayerEvent::StateChange(state) => match state {
                    GtPlayerState::Eos => {
                        play_pause_button.set_image(Some(&play_image));
                        show_info_bar("The stream has ended", "", MessageType::Info);
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
                    warning!("{}", warning);
                },
                GtPlayerEvent::Error(error) => {
                    show_info_bar("Player error", &error, MessageType::Error);
                    warning!("{}", error);
                }
            }
        });

        let inner = Rc::new(Self {
            player: get_backend(settings)(settings, Box::new(cb)).unwrap(),
            settings_button: get_obj!(builder, "player-menu-button")
        });

        let container: GtkBox = get_obj!(builder, "player-container");
        let video_widget = inner.player.get_widget();
        container.pack_start(video_widget, true, true, 0);
        container.show_all();
        video_widget.realize();

        play_pause_button.connect_clicked(clone!(@strong inner => move |_| {
            match inner.player.get_state() {
                GtPlayerState::Playing => {
                    if let Err(e) = inner.player.pause() {
                        show_info_bar("Could not pause", &format!("{}", e), MessageType::Error)
                    }
                },
                GtPlayerState::Paused => {
                    if let Err(e) = inner.player.play() {
                        show_info_bar("Could not play", &format!("{}", e), MessageType::Error)
                    }
                }
                _ => {
                    // TODO: Remove this
                    inner.player.set_uri(Some("https://video-weaver.cph01.hls.ttvnw.net/v1/playlist/CvQEHm17eVmE0IoW8S99plmLydkQASzzjKzRrJHfQ58fWWuWOFBVR2UXT-ps32qkj0XhFDkTxSCKL2jqlMn1o_nmJGv6N8SZmDjVMmt7ACVOjzxp_iha_FGuKsb8syK8qJYOeIudKJjNv7wACMRmZKllsMBly0ACoerVnYD7MM6HN8aregujJ7D6q3oCEL7kJRcpg1ZXi7zfVnswXMZ1E4yBHdVaPWHPuJxT2-RnrSSm5XljQ2U1ceujxu_4mseyvEiNKNTMWeAkY-VA-7PL-lfBTL1rTT-uZjO1Hm24BGIMN1Vtm2QrxlVxsE_PCghsv84DrnTHvXAKe-e05NZxiLxqB8CeDqIaxyawj57-DMpzYfxbF6uey2isySpb0s4lzmpocIrIytt8joixxkMh4tGY0DCKNbbXhetanVyDJa9Q9_FzRcaGr3UhBOh53xwiMB1Ubr6UDKGQvzOL4MgdP2k0KN-lz4YuptBxBE8xUSFePqzB2k4tiG3DXFo1R4lDlC45seny7rlz0-IrI63veu7g8nGZ81B-jkIscnzjNt_w-h2n-cKgJdljh70rIDeB-p6YeVRcV73_prCOVquxOO-oSaf1Weyx42aclHVUrBb1EmyX9AALXBJhcW2uMx2BLJm5HyorimrpS5kwyucpxJSQUrx6QP3gu_mho2hAtlup6Vx5s4n_YcQN7dffv-5xaeBpdGHNi__qH35j3f54cqFnzHvcAjBtJlsPbbSyAzI8ds1PQSUkWXeXLQj0xQy66SV-Vc1fSmw0fuwpb-Unf3QjNobhqGENZi7piYjmjoCOKf3IMPo5TgpbI23IEEZpCOxzw3IcSRIQHOeMKu7TYoiwdM1PRSHLzRoMaOqwNLPttjtgbQQr.m3u8".into()));
                    inner.player.play();
                }
            }
        }));

        // let play_image = get_obj!(Image, builder, "play-image");
        // let pause_image = get_obj!(Image, builder, "pause-image");
        // inner.player.set_state_cb(|evt| {
        //     println!("New state: {:#?}", evt);
        //     match evt {
        //         GtPlayerEvent::StateChange(state) => match state {
        //             GtPlayerState::Eos => {
        //                 // play_pause_button.set_image(Some(&play_image));
        //                 show_info_bar("The stream has ended", "", MessageType::Info);
        //             },
        //             GtPlayerState::Buffering => {

        //             },
        //             GtPlayerState::Playing => {

        //             },
        //             GtPlayerState::Paused => {

        //             },
        //             GtPlayerState::Stopped => {

        //             },
        //             _ => { }
        //         },
        //         GtPlayerEvent::Warning(warning) => {
        //             // show_info_bar("Player warning", &warning, MessageType::Warning)
        //             warning!("{}", warning);
        //         },
        //         GtPlayerEvent::Error(error) => {
        //             show_info_bar("Player error", &error, MessageType::Error);
        //             warning!("{}", error);
        //         }
        //     }
        // });

        inner

    }

    pub fn reload(&self) {

        warning!("TODO");

    }

    pub fn stop(&self) {

        drop(self.player.stop())

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
