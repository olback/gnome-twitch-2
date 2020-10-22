use {
    crate::{
        ASSETS, USER, get_obj, resource, rt, warning,
        resources::{CLIENT_ID, bytes_to_pixbuf}, twitch::Twitch
    },
    std::rc::Rc,
    gtk::{ApplicationWindow, Builder, Image, Label, Window, prelude::*},
    gio::SimpleActionGroup,
    glib::clone
};

pub struct ProfileWindow {
    window: Window,
    image: Rc<Image>,
    username_label: Label,
    userid_label: Label
}

impl ProfileWindow {

    pub fn configure(main_window: &ApplicationWindow, action_group: &SimpleActionGroup) -> Rc<Self> {

        let builder = Self::builder();

        let inner = Rc::new(Self {
            window: get_obj!(builder, "profile-window"),
            image: Rc::new(get_obj!(builder, "profile-image")),
            username_label: get_obj!(builder, "profile-username"),
            userid_label: get_obj!(builder, "profile-user-id")
        });

        inner.window.insert_action_group("app", Some(action_group));
        inner.window.set_attached_to(Some(main_window));
        inner.window.set_transient_for(Some(main_window));

        inner.window.connect_delete_event(|win, _| {
            win.hide();
            gtk::Inhibit(true)
        });

        inner

    }

    pub fn show(&self) {

        match *USER.lock().unwrap() {
            Some(ref user) => {
                self.username_label.set_text(&user.username);
                self.userid_label.set_text(&user.user_id);
                let token = user.oauth_token.clone();
                rt::run_cb_local(async {
                    let tw = Twitch::new(Some(token), Some(CLIENT_ID.into()));
                    let user = match tw.get_users(None, None).await {
                        Ok(mut users) => users.data.remove(0),
                        Err(e) => {
                            warning!("{}", e);
                            return None
                        }
                    };
                    let data = match ASSETS.load(&user.profile_image_url).await {
                        Ok(data) => data,
                        Err(e) => {
                            warning!("{}", e);
                            return None
                        }
                    };
                    Some(data)
                }, clone!(@strong self.image as image => move |res| {
                    match res {
                        Some(data) => match bytes_to_pixbuf(&data, Some((200, 200))) {
                            Ok(pixbuf) => image.set_from_pixbuf(Some(&pixbuf)),
                            Err(e) => warning!("{}", e)
                        },
                        _ => {}
                    }
                }));
                self.window.show_all();
            },
            None => {
                // TODO: Show infobar with error
            }
        }

    }

    pub fn hide(&self) {

        self.window.hide();

    }

    fn builder() -> Builder {

        Builder::from_resource(resource!("ui/profile"))

    }

}