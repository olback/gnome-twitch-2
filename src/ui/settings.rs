use {
    crate::{get_obj, resource},
    std::rc::Rc,
    gtk::{ApplicationWindow, Builder, ComboBoxText, Button, Window, prelude::*}
};

pub struct SettingsWindow {
    window: Window
}

impl SettingsWindow {

    pub fn configure(main_window: &ApplicationWindow) -> Rc<Self> {

        let builder = Self::builder();

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
            window: settings_window
        })

    }

    pub fn show(&self) {

        self.window.show()

    }

    fn builder() -> Builder {

        Builder::from_resource(resource!("ui/settings"))

    }

}