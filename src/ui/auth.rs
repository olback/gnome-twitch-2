use {
    crate::{resource, get_obj, warning, rt, resources::KEYRING_NAME, twitch::Twitch},
    std::rc::Rc,
    gtk::{ApplicationWindow, Button, Window, Builder, Stack, Viewport, prelude::*},
    webkit2gtk::{WebView, WebViewExt, LoadEvent},
    keyring::Keyring
};

const AUTH_URL: &'static str = "https://id.twitch.tv/oauth2/authorize";
const CLIENT_ID: &'static str = "0v7le4jgexwgwggoker7yxrs84dr3x";
const REDIRECT_URL: &'static str = "gt2://auth";
const RESPONSE_TYPE: &'static str = "token";
const SCOPES: &[&'static str] = &[
    "user:read:email"
];

pub struct AuthWindow {
    window: Window,
    web_view: WebView,
    stack: Stack,
    try_again_button: Button
}

impl AuthWindow {

    pub fn configure(main_window: &ApplicationWindow) -> Rc<Self> {

        let b = Builder::from_resource(resource!("ui/twitch-auth"));

        let window: Window = get_obj!(b, "twitch-auth-window");
        let web_view = WebView::new();
        let stack: Stack = get_obj!(b, "twitch-auth-stack");
        let try_again_button: Button = get_obj!(b, "twitch-auth-try-again");

        let inner = Rc::new(Self {
            window,
            web_view,
            stack,
            try_again_button
        });

        inner.window.set_transient_for(Some(main_window));
        inner.window.set_attached_to(Some(main_window));
        inner.stack.set_visible_child_name("spinner");

        inner.window.connect_delete_event(|win, _| {
            win.hide();
            gtk::Inhibit(true)
        });

        let wkvp: Viewport = get_obj!(b, "wkvp");
        wkvp.add(&inner.web_view);

        let weak_stack = inner.stack.downgrade();
        inner.web_view.connect_load_changed(move |vw, le| {

            let stack = weak_stack.upgrade().unwrap();

            match le {
                LoadEvent::Finished => {
                    stack.set_visible_child_name("webview");
                    match vw.get_uri() {
                        Some(uri_g) => {
                            let uri = uri_g.to_string();
                            if uri.starts_with(REDIRECT_URL) {
                                let start = uri.find("=").unwrap_or(0);
                                let end = uri.find("&").unwrap_or(0);
                                let token = (&uri)[(start+1)..end].to_string();
                                let token_clone = token.clone();
                                rt::run_cb_local(async move {
                                    Twitch::builder()
                                        .set_token(Some(token_clone))
                                        .set_client_id(Some(CLIENT_ID.into()))
                                        .build()
                                        .unwrap()
                                        .get_users(None, None)
                                        .await
                                }, move |ret| {
                                    match ret {
                                        Ok(user) => {
                                            println!("Welcome, {}", user.data[0].display_name);
                                            let kr = Keyring::new(KEYRING_NAME, &user.data[0].display_name);
                                            match kr.set_password(&token) {
                                                Ok(_) => stack.set_visible_child_name("success"),
                                                Err(e) => {
                                                    warning!("{}", e);
                                                    stack.set_visible_child_name("failure");
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            warning!("{:#?}", e);
                                            stack.set_visible_child_name("failure");
                                        }
                                    }
                                });
                            }
                        },
                        None => {}
                    }
                },
                _ => {
                    if let Some(g_uri) = vw.get_uri() {
                        let uri = g_uri.to_string();
                        if uri.starts_with(REDIRECT_URL) {
                            stack.set_visible_child_name("spinner");
                        }
                    }
                }
                // _ => stack.set_visible_child_name("spinner")
            }

        });

        let inner_clone = Rc::clone(&inner);
        inner.try_again_button.connect_clicked(move |_| {
            inner_clone.show();
        });

        inner

    }

    pub fn show(&self) {

        self.stack.set_visible_child_name("spinner");
        self.web_view.load_uri(&Self::url());
        self.window.show_all();

    }

    pub fn url() -> String {

        format!(
            "{au}?client_id={cid}&redirect_uri={ru}&response_type={rt}&scope={scope}",
            au = AUTH_URL,
            cid = CLIENT_ID,
            ru = REDIRECT_URL,
            rt = RESPONSE_TYPE,
            scope = SCOPES.join(" ")
        )

    }

}
