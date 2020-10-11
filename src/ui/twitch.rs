use {
    crate::{resource, get_obj, resources},
    gtk::{ApplicationWindow, Window, Builder, Viewport, prelude::*},
    webkit2gtk::{WebView, WebViewExt}
};

const AUTH_URL: &'static str = "https://id.twitch.tv/oauth2/authorize";
const CLIENT_ID: &'static str = "0v7le4jgexwgwggoker7yxrs84dr3x";
const REDIRECT_URL: &'static str = "gt2://auth:";
const RESPONSE_TYPE: &'static str = "token";
const SCOPES: &[&'static str] = &[
    ""
];

pub fn configure(main_window: &ApplicationWindow) {

    let url = format!(
        "{au}?client_id={cid}&redirect_uri={ru}&response_type={rt}&scope={scope}",
        au = AUTH_URL,
        cid = CLIENT_ID,
        ru = REDIRECT_URL,
        rt = RESPONSE_TYPE,
        scope = SCOPES.join(" ")
    );

    let b = Builder::from_resource(resource!("ui/twitch-auth"));
    let w: Window = get_obj!(b, "twitch-auth-window");
    w.set_transient_for(Some(main_window));
    w.set_attached_to(Some(main_window));

    let wkvp: Viewport = get_obj!(b, "wkvp");
    let wv = WebView::new();
    wv.load_uri(&url);

    wv.connect_load_changed(|wv, le| {
        println!("{:#?}", wv.get_uri());
        println!("{:#?}", le);
    });



    wkvp.add(&wv);










    w.show_all();

}
