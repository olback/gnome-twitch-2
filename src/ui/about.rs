use {
    crate::{resource, get_obj, resources},
    gtk::{AboutDialog, ApplicationWindow, Builder, prelude::*}
};

pub fn configure(main_window: &ApplicationWindow) -> AboutDialog {
    let about_builder = Builder::from_resource(resource!("ui/about"));
    let about_dialog = get_obj!(AboutDialog, about_builder, "about-dialog");
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
    about_dialog
}
