use {
    crate::{get_obj, resources::APP_ID},
    std::rc::Rc,
    gtk::{Builder, Stack, prelude::*},
    gio::{Settings, SettingsExt}
};

pub struct ViewsSection {
    views: Stack
}

impl ViewsSection {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        let inner = Rc::new(Self {
            views: get_obj!(builder, "views-stack")
        });

        let settings = Settings::new(APP_ID);
        let view = settings
            .get_string("default-view")
            .map(|v| v.to_string())
            .unwrap_or("channels".into());
        inner.views.set_visible_child_name(&view);

        inner

    }

}
