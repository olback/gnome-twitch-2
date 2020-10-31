use {
    crate::{get_obj, resource, backends::{BACKENDS, backend_id_valid}},
    std::rc::Rc,
    gtk::{ApplicationWindow, Builder, ComboBoxText, Button, InfoBar, Label, Window, prelude::*},
    gio::{Settings, SettingsExt, SettingsBindFlags},
    glib::clone
};

pub struct SettingsWindow {
    window: Window,
    info_bar: InfoBar,
    // info_bar_label: Label
}

impl SettingsWindow {

    pub fn configure(main_window: &ApplicationWindow, settings: &Settings) -> Rc<Self> {

        let builder = Self::builder();

        let backend_player_cbt = get_obj!(ComboBoxText, builder, "settings-player-backend");
        for (name, id, _) in BACKENDS {
            backend_player_cbt.append(Some(*id), *name);
        }
        let backend_value = settings.get_string("backend-player").map(|v| v.to_string()).unwrap_or(String::new());
        if backend_id_valid(&backend_value) {
            backend_player_cbt.set_active_id(Some(&backend_value));
        } else {
            settings.set_string("backend-player", BACKENDS[0].1).expect("Failed to set backend-player");
            backend_player_cbt.set_active_id(Some(BACKENDS[0].1));
        }

        settings.bind("theme", &get_obj!(ComboBoxText, builder, "settings-theme"), "active-id", SettingsBindFlags::DEFAULT);
        settings.bind("default-view", &get_obj!(ComboBoxText, builder, "settings-default-view"), "active-id", SettingsBindFlags::DEFAULT);
        settings.bind("backend-player", &backend_player_cbt, "active-id", SettingsBindFlags::DEFAULT);

        let info_bar = get_obj!(InfoBar, builder, "settings-info-bar");
        let info_bar_label = get_obj!(Label, builder, "settings-info-bar-label");
        info_bar.connect_response(|bar, _| {
            bar.set_visible(false);
            bar.set_revealed(false);
        });

        get_obj!(Button, builder, "settings-reset-button").connect_clicked(clone!(
            @weak settings,
            @weak info_bar,
            @weak info_bar_label
         => move |_| {
            settings.reset("theme");
            settings.reset("default-view");
            settings.reset("default-quality");
            settings.set_string("backend-player", BACKENDS[0].1).unwrap();
            info_bar_label.set_text("Settings reset");
            info_bar.set_visible(true);
            info_bar.set_revealed(true);
        }));

        let settings_window: Window = get_obj!(builder, "settings-window");
        settings_window.set_attached_to(Some(main_window));
        settings_window.set_transient_for(Some(main_window));

        let settings_window: Window = get_obj!(builder, "settings-window");
        settings_window.hide_on_delete();
        settings_window.connect_delete_event(|win, _| {
            win.hide();
            gtk::Inhibit(true)
        });

        Rc::new(Self {
            window: settings_window,
            info_bar,
            // info_bar_label
        })

    }

    pub fn show(&self) {

        self.info_bar.set_visible(false);
        self.info_bar.set_revealed(false);
        self.window.show();

    }

    pub fn hide(&self) {

        self.window.hide();

    }

    fn builder() -> Builder {

        Builder::from_resource(resource!("ui/settings"))

    }

}