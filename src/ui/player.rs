use {
    crate::{resource, get_obj, message, warning, error, resources::APP_ID, backends::{GtPlayerBackend, BACKENDS}},
    std::{rc::Rc, cell::RefCell},
    gtk::{Application, Builder, Box as GtkBox, Button, ToggleButton, IconSize,
        MenuButton, VolumeButton, ApplicationWindow, Image, Label, prelude::*
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
    main_window: ApplicationWindow,
    chat_section: GtkBox,
    play_pause_button: Button,
    volume_button: VolumeButton,
    timestamp_label: Label,
    settings_button: MenuButton,
    hide_chat_button: ToggleButton,
    fullscreen_button: Button,
    fullscreen_image: Image,
    is_fullscreen: RefCell<bool>
}

impl PlayerSection {

    pub fn configure(app: &Application, builder: &Builder, settings: &Settings) -> Rc<Self> {

        let inner = Rc::new(Self {
            player: (BACKENDS[0].2)(settings).unwrap(),
            main_window: get_obj!(builder, "main-window"),
            chat_section: get_obj!(builder, "chat-section"),
            play_pause_button: get_obj!(builder, "player-play-pause"),
            volume_button: get_obj!(builder, "player-volume"),
            timestamp_label: get_obj!(builder, "player-timestamp-label"),
            settings_button: get_obj!(builder, "player-menu-button"),
            hide_chat_button: get_obj!(builder, "player-toggle-chat"),
            fullscreen_button: get_obj!(builder, "player-fullscreen"),
            fullscreen_image: get_obj!(builder, "fullscreen-image"),
            is_fullscreen: RefCell::new(false)
        });

        inner.hide_chat_button.connect_toggled(clone!(@strong inner => move |btn| {
            inner.chat_section.set_visible(!btn.get_active())
        }));

        settings.bind("volume", &inner.volume_button, "value", SettingsBindFlags::DEFAULT);

        inner.main_window.connect_window_state_event(clone!(@strong inner => move |_, state| {
            let new_state = state.get_new_window_state();
            if new_state.contains(gdk::WindowState::FULLSCREEN) {
                inner.is_fullscreen.replace(true);
                inner.fullscreen_image.set_from_icon_name(Some("view-restore-symbolic"), IconSize::Button);
            } else {
                inner.is_fullscreen.replace(false);
                inner.fullscreen_image.set_from_icon_name(Some("view-fullscreen-symbolic"), IconSize::Button);
            }
            gtk::Inhibit(false)
        }));

        inner.fullscreen_button.connect_clicked(clone!(@strong inner => move |_| {
            if *inner.is_fullscreen.borrow() {
                inner.main_window.unfullscreen()
            } else {
                inner.main_window.fullscreen()
            }
        }));

        let container: GtkBox = get_obj!(builder, "player-container");
        let video_widget = inner.player.get_widget();
        container.pack_start(video_widget, true, true, 0);
        container.show_all();
        video_widget.realize();

        inner.play_pause_button.connect_clicked(clone!(@strong inner => move |btn| {
            inner.player.stop().unwrap();
            inner.player.set_uri(Some("https://video-weaver.cph01.hls.ttvnw.net/v1/playlist/CvQEyDItikn5CyJ6BCVmndFW6sFtWvRv725A86WzNOrnvQv9Zc483imtrLlS-qZMh6lgPrISkR5AoRDggvYwUA_BDn3n58C3dMmX9B4juvbahh9SHve6CW6d98ebUZd_RyEiHTIE6HN-bcrqIVYDmL2kFagZlpppR3dg7hg4OdePkQUBDCLb9hiBGApT8rpiG0t0S2CL_K9nmB3kEVI2o7XJaMSSdtAjJG9jv57szB2pzryOjDHbykGYNr1CtvTrDlT87Hot85XG4BbO2CgXIrwmWof3pZbjJ9YF1zFFtL4VABf5BJkGQ813EZnOWtFqGe-2XqCetjGCEFj67IRGFAZMCeqhH3se5KEMnwhJMxRIn_LxWfnD3XmKctxbZIi5qdu5n1wzFc6alMStjZs5LrzVXbaFlpK2Xcb5H6F5OXP8dzWBdnvfnfiff9yduMVF7Lw2Lwj_nG6DhZDN1LIBm5Wz3pTfaZNDkVP3yQE3e3q_4NqZkp45YaoEq2T2D4aJIBM2EBck5epTY01yePMf_bD5TpGjusiZG5EUQ1zEMYe7JTCyUE2FPnpQI6eGSJh_vyPnmOPGihF_NmaJ2ppBkDMycLLkdv1Hkv9uTzy9xlsbSQon4_y62pabdb_FeN51lBH69FRFlQjXVkDfC3eR2HzidYmHtCYYCoSbrpuBDEk2ZOeqLwrSm8ZYw6dLR-lrU_NK5Uj882kguz-nuGgIt_2hK2K5y-sGrkIjdQrVhJrgs9Oa4hYEj8_meN0bUUkzgeyOYo6cjQs1eNLSGveJFdMRXl_JR1i92KyVBV5xiB2HzeAGTKz_NbU0xpZeOL2eVNuWsy4ZNRIQWb2nzEtMW5W-5TQpno31OxoMioICb4XUT1Fg3weG.m3u8".into())).unwrap();
            inner.player.play().unwrap();
        }));

        inner.player.stop().unwrap();

        inner

    }

}
