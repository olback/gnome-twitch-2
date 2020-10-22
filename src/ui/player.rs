use {
    crate::{resource, get_obj, message, warning, error, resources::APP_ID, backends::{GtPlayerBackend, BackendGstreamerOpenGL}},
    std::{rc::Rc, cell::RefCell},
    gtk::{Application, Builder, Box as GtkBox, Button, ToggleButton, IconSize,
        MenuButton, VolumeButton, ApplicationWindow, Image, Label, prelude::*
    },
    gio::{Settings, SettingsExt, SettingsBindFlags},
    glib::clone
    // gst::prelude::*
};

// pub fn configure(app: &Application, builder: &Builder) {

//     let container: GtkBox = get_obj!(builder, "player-container");
//     let timestamp_label: Label = get_obj!(builder, "player-timestamp-label");

//     let pipeline = gst::Pipeline::new(None);
//     let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
//     // let src = gst::ElementFactory::make("filesrc", None).unwrap();
//     // src.set_property("location", &"/home/olback/Videos/bbb4k60.mp4").unwrap();
//     // let src = gst::ElementFactory::make("filesrc", None).unwrap();
//     // src.set_property("uri", &"https://olback.net/download/pingu.mp4").unwrap();
//     // let src = gst::ElementFactory::make("playbin", None).unwrap();
//     // src.set_property("uri", &"file:///home/olback/Videos/bbb4k60.mp4").unwrap();

//     let (sink, video_widget) = if let Ok(gtkglsink) = gst::ElementFactory::make("gtkglsink", None) {
//         // GPU Acceleration :)
//         message!("Using GPU Acceleration");
//         let glsinkbin = gst::ElementFactory::make("glsinkbin", None).unwrap();
//         glsinkbin.set_property("sink", &gtkglsink.to_value()).unwrap();

//         let widget = gtkglsink.get_property("widget").unwrap();
//         (glsinkbin, widget.get::<gtk::Widget>().unwrap().unwrap())
//     } else {
//         // CPU Accelerated :(
//         warning!("Using CPU acceleration");
//         let sink = gst::ElementFactory::make("gtksink", None).unwrap();
//         let widget = sink.get_property("widget").unwrap();
//         (sink, widget.get::<gtk::Widget>().unwrap().unwrap())
//     };

//     pipeline.add_many(&[&src, &sink]).unwrap();
//     src.link(&sink).unwrap();

//     container.pack_start(&video_widget, true, true, 0);

//     let pipeline_weak = pipeline.downgrade();

//     let timeout_id = glib::timeout_add_local(500, move || {

//         let pipeline = match pipeline_weak.upgrade() {
//             Some(p) => p,
//             None => return glib::Continue(true)
//         };

//         let position = pipeline
//             .query_position::<gst::ClockTime>()
//             .unwrap_or_else(|| 0.into());

//         timestamp_label.set_text(&format!("{:.0}", position));

//         glib::Continue(true)

//     });

//     // let bus = pipeline.get_bus().unwrap();

//     pipeline.set_state(gst::State::Playing).unwrap();

//     // let app_weak = app.downgrade();
//     // bus.add_watch_local(move |_, msg| {

//     //     glib::Continue(true)

//     // }).unwrap();

//     // let timeout_id = RefCell::new(Some(timeout_id));
//     // app.connect_shutdown(move |_| {
//     //     pipeline.set_state(gst::State::Null).unwrap();
//     //     bus.remove_watch().unwrap();
//     //     if let Some(tid) = timeout_id.borrow_mut().take() {
//     //         glib::source_remove(tid);
//     //     }
//     // });

// }

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

    pub fn configure(app: &Application, builder: &Builder) -> Rc<Self> {

        let inner = Rc::new(Self {
            player: Box::new(BackendGstreamerOpenGL::new().unwrap()),
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

        let settings = Settings::new(APP_ID);
        // settings.bind(&inner.volume_button, "volume", &);
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
        let video_widget = inner.player.get_widget().unwrap();
        video_widget.show_all();
        container.pack_start(&video_widget, true, true, 0);
        container.show_all();

        // inner.player.stop().unwrap();
        // inner.player.set_uri(Some("file:///home/olback/Videos/test.mp4".into())).unwrap();
        // inner.player.play().unwrap();
        // message!("should play :(");

        inner.play_pause_button.connect_clicked(clone!(@strong inner => move |btn| {
            inner.player.play().unwrap();
        }));

        inner.player.stop().unwrap();
        inner.player.set_uri(Some("https://video-weaver.cph01.hls.ttvnw.net/v1/playlist/CvMExjUEeRbp_Wspkjh3OeINkkftyQhMVwze9p2tV7TjLBU0KZaUjioAcdMwxwmhKTu33AGBrA-4RV2ajNARTbXnX8DLtg9_tXnPPD-uE8CbIXBuzIHwo19QhKZq30qaxnfj5k6gv5O_GA0iEukiBv-e2QSmTtc_8oOqcz8EBNc8HMMfHF-Yx4g34abWQHFQItzF901d5TtiEo_ZbpgEHpdDaMalqrofmbjZp4EEDwgalPrXn68IvyG4AtV7CXqz9UmT6OMcUq51MaM0834a9qhojCIhmxIaHwEw7wj8JiQXKN67dKc_22TxGu7P3pPi_6FtMS5meqUpk1ARtIF60Fib2s_1IFp6K6RZCZtvK2k8QIG7XmRCNfqXHleZvJ1GRhYTbwja1rtMxHGRkwagp9nud6f2ACPnPygQ16p0g5WRNP4nfzI5Nk-5r-9S8sOQZyBe_LlWxyyraxzNSj92MQC8Or786ALKBvdba5v3-CfegcQcDYNSh_RCwXQvBwd83kj7epKvQsRse4hZ2agjfGSW1J37tw1CQkp5cztFDdJhERQx9BCZMLgSG5AMDMtVNOiessc5uC03zzAjbT8OZLdsnHIZZ2b2Xz-jrbrnnsKhNZ5Oed2MFdgYPjxllAHLliOxo46lY91Wj_ng460S8BS3bZqXbwDTQmRWtTxEMKrQyTYGmkoLHUgCF33X1pVRYYLj68XfrkvVhseslL7VO8wrbLAsy7Dc0HwKbQh0qbjJK8I4cal_IxdF45j0flNScw3ORIest2p2gnx3K9DMW7XqNmVGw8ytPRJbxFPvHU0B0jzREDGGtNvf-KijFgNdAF1z4d_jEhAxgu_6hZKlmKQWYC-nlQhaGgwCHpgFF9ZYHBU2Efs.m3u8".into())).unwrap();
        // inner.player.play().unwrap();

        // let inner_clone = Rc::clone(&inner);
        // glib::timeout_add_local(200, move || {
        //     inner_clone.player.set_uri(Some("https://video-edge-346540.cph01.abs.hls.ttvnw.net/v1/segment/CocFZ7ahlmpWaWxmExZ3ljXyP8LbQakj9qs2c1TluRrAjpgNQ9hI481-9TUlJwfbTnbW__CjONDhA6AmSOcpmF-BP93-8e8zaRwsPXOB9sOnLeAWsJovGKLUdES_9Kcozh4KaUJHkiWjafUKJ5bGCsTu1qtFftD8a_0bFcdf6Jr0Hb_n5OrT9ijiONt_ldhGwJhQKf1qZaPK_HtR56z9-4JvoubWZ5KbjVIDY4FphqOSpIEBqSW26fKUvI10zamFZ0GKOeqoFcmBgyn2bZ-EDzXOKeU37xfAism52bbpNTzLzTeY4eamL1gZo5RJZ4DI_3-e-VR5VEn0iD4fKI5as_7NOcsIZNl0gJ0f1GKhFKVQ750H1SYJ9XnQg4-UuYYnJiOWzxneuerK0igKKRVccABGrFvg_z3SupObj0gMCvI2tRLNpiDHj5GKblkHa3gegd9gFpm9Tqnb1IjUy8QLjYmx6TidUm7vfl9wTQdqPwAeFK_uPzwPMH_T5PCqJg2AqLvogstL5X9-H8hRLaQBwGNwbcFuauXyYWD8p6cH4MxYd53-QfNetWWyzcaNQh6Af-Who2doAdSYtRe81CsvJrriDl8d8HyQIoqE09xz0W0uOiP5d7PbHaGXKFYoQkBs05zF3PxDwR4TpzsQ9bFKs_Ik5zTApGBt4NekHUmqBhDyWybWT-dsOQO3vQspIaeuogjTQEFXahNk1aXeQB-Q-3dnih332EkoD5rKGiq-Os4LP2nLcYv27GELAJMK-hnX0EPHFvAU4XI2ofL3XNUhubZGeR-ZMBcwUN5E3bsDrbfmzRv30iGtTjdKNpnWgi8q9I_dvkgVippMB_IQK7RCif5GrHSYXWmIpZASEH2TjmgvY4peNFiF1eKiQesaDNuy-ao7vC813h5Sqw.ts".into()));
        //     inner_clone.player.set_position(0);
        //     inner_clone.player.play().unwrap();
        //     glib::Continue(true)
        // });

        inner

    }

}

// pub fn configure(app: &Application, builder: &Builder) {

//     let container: GtkBox = get_obj!(builder, "player-container");
//     let timestamp_label: Label = get_obj!(builder, "player-timestamp-label");

//     // glib::timeout_add_local(2000, move || {

//         let mut player = BackendGstreamerOpenGL::new().unwrap();
//         let video_widget = player.get_widget().unwrap();
//         video_widget.show_all();

//         container.pack_start(&video_widget, true, true, 0);

//         // player.set_uri(Some("https://olback.net/download/pingu.mp4".into())).unwrap();
//         // player.set_uri(Some("file:///home/olback/Videos/bbb4k60.mp4".into())).unwrap();
//         // player.play().unwrap();

//         player.stop().unwrap();
//         player.set_uri(Some("file:///home/olback/Videos/test.mp4".into())).unwrap();
//         player.play().unwrap();
//         message!("should play :(");
//         // player.set_position(100).unwrap();
//         // player.pause().unwrap();
//         // glib::Continue(false)
//     // });

//     glib::timeout_add_local(200, move || {
//         player.query();
//         glib::Continue(true)
//     });

// }
