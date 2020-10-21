use {
    crate::{p, resource, error::GtResult},
    gdk_pixbuf::PixbufLoaderExt,
    gtk::{self, CssProviderExt},
    gio,
    glib
};

const RESOURCE_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/gnome-twitch.gresource"));
pub const APP_ID: &'static str = "net.olback.GnomeTwitch2";
pub const VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/version.txt"));
pub const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
pub const HOME_PAGE: &'static str = env!("CARGO_PKG_HOMEPAGE");
pub const LICENSE: &'static str = include_str!("../../LICENSE");
pub const KEYRING_NAME: &'static str = "Gnome Twitch 2";
pub const GAME_COVER_SIZE: (i32, i32) = (300, 400);
pub const STREAM_COVER_SIZE: (i32, i32) = (320, 180);

pub const REQUEST_SIZE: u8 = 50;
pub const CLIENT_ID: &'static str = "0v7le4jgexwgwggoker7yxrs84dr3x";

mod cache;
pub mod loader;

pub fn register_resources() {

    let glib_resource_bytes = glib::Bytes::from_static(RESOURCE_DATA);
    let resources = gio::Resource::from_data(&glib_resource_bytes).expect("Failed to register resources");
    gio::resources_register(&resources);

}

pub fn register_css() {

    let provider = gtk::CssProvider::new();
    provider.load_from_resource(resource!("css/custom.css"));
    // #[cfg(not(target_os = "windows"))]
    // provider.load_from_resource(resource!("css/app.css"));
    // #[cfg(target_os = "windows")]
    // provider.load_from_resource(resource!("css/windows.css"));
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

}

// TODO: Close loader in case of failure
pub fn bytes_to_pixbuf(data: &[u8], size: Option<(i32, i32)>) -> GtResult<gdk_pixbuf::Pixbuf> {

    let pixbufloader = gdk_pixbuf::PixbufLoader::new();
    if let Some(size) = size {
        pixbufloader.set_size(size.0, size.1);
    }
    p!(pixbufloader.write(&data));
    let pixbuf = p!(pixbufloader.get_pixbuf().ok_or("Could not get pixbuf"));
    drop(pixbufloader.close());

    Ok(pixbuf)

}
