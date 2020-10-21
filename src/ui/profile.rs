use {
    crate::{USER, get_obj, resource, resources::APP_ID, user::User},
    std::rc::Rc,
    gtk::{ApplicationWindow, Builder, ComboBoxText, Button, InfoBar, Label, Window, prelude::*},
    glib::clone
};

pub struct ProfileWindow {

}

impl ProfileWindow {

    pub fn configure(main_window: &ApplicationWindow) -> Rc<Self> {

        let builder = Self::builder();

        todo!()

    }

    fn builder() -> Builder {

        Builder::from_resource(resource!("ui/settings"))

    }

}