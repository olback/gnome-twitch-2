use {
    crate::{resource, get_obj, resources},
    std::rc::Rc,
    gtk::{AboutDialog as GtkAboutDialog, ApplicationWindow, Builder, prelude::*}
};

pub struct AboutDialog {
    dialog: GtkAboutDialog
}

impl AboutDialog {

    pub fn configure(main_window: &ApplicationWindow) -> Rc<AboutDialog> {
        let about_builder = Builder::from_resource(resource!("ui/about"));
        let about_dialog = get_obj!(GtkAboutDialog, about_builder, "about-dialog");
        about_dialog.set_transient_for(Some(main_window));
        about_dialog.set_attached_to(Some(main_window));
        about_dialog.set_version(Some(resources::VERSION));
        about_dialog.set_comments(Some(resources::DESCRIPTION));
        about_dialog.set_website(Some(resources::HOME_PAGE));
        about_dialog.set_website_label(Some(&resources::HOME_PAGE.replace("https://", "")));
        about_dialog.set_license(Some(&resources::LICENSE));
        about_dialog.set_authors(&[
            resources::AUTHORS.split(':').collect::<Vec<&str>>(),
            about_dialog.get_authors().iter().map(|a| a.as_str()).collect::<Vec<&str>>()
        ].concat());

        Rc::new(Self {
            dialog: about_dialog
        })
    }

    pub fn show(&self) {

        match self.dialog.run() {
            _ => self.hide()
        }

    }

    pub fn hide(&self) {

        self.dialog.hide()

    }

}
