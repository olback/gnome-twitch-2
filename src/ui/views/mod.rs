use {
    crate::{get_obj, resources::APP_ID},
    super::cards::{GameCard, LiveCard},
    std::rc::Rc,
    gtk::{Builder, Stack, FlowBox, prelude::*},
    gio::{Settings, SettingsExt},
    glib::clone
};

mod channels;
mod following;
mod games;

pub struct ViewsSection {
    views: Stack,
    pub channels: Rc<channels::ChannelsView>,
    pub following: Rc<following::FollowingView>,
    pub games: Rc<games::GamesView>
}

impl ViewsSection {

    pub fn configure(builder: &Builder, settings: &Settings) -> Rc<Self> {

        let inner = Rc::new(Self {
            views: get_obj!(builder, "views-stack"),
            channels: channels::ChannelsView::configure(builder),
            following: following::FollowingView::configure(builder),
            games: games::GamesView::configure(builder),
        });

        let view = settings
            .get_string("default-view")
            .map(|v| v.to_string())
            .unwrap_or("channels".into());
        inner.views.set_visible_child_name(&view);

        // TODO:
        // Figure out why connect_notify require things
        // to be Send + Sync.
        // This *seems* to be working though.
        unsafe { inner.views.connect_notify_unsafe(Some("visible-child-name"), clone!(@strong inner => move |stack, ps| {
            let view_name = stack.get_visible_child_name().map(|v| v.to_string()).unwrap_or(String::new());
            match view_name.as_str() {
                "channels" => inner.channels.refresh(),
                "following" => inner.following.refresh(),
                "games" => inner.games.refresh(),
                _ => {}
            }
        })) };

        inner

    }

    pub fn notify(&self) {
        self.views.notify("visible-child-name");
    }

}
