use {
    crate::{resource, get_obj, warning, rt, resources::CLIENT_ID, twitch::Twitch, user::User, USER},
    std::rc::Rc,
    gtk::{Application, ApplicationWindow, Button, Window, Builder, Stack, Viewport, prelude::*},
    gio::prelude::*,
    webkit2gtk::{WebView, WebViewExt, LoadEvent, WebContextExt, CookieManagerExt},
    glib::clone
};

const AUTH_URL: &'static str = "https://id.twitch.tv/oauth2/authorize";
const REDIRECT_URL: &'static str = "gt2://auth";
const RESPONSE_TYPE: &'static str = "token";
const SCOPES: &[&'static str] = &[
    "user:read:email", // Is this needed?
    "chat:read", // Read chat messages
    "chat:edit", // Send chat messages
    "clips:edit" // Create clips
];

pub struct AuthWindow {
    window: Window,
    web_view: WebView,
    stack: Stack,
    continue_button: Button,
    try_again_button: Button
}

impl AuthWindow {

    pub fn configure(app: &Application, main_window: &ApplicationWindow) -> Rc<Self> {

        let b = Builder::from_resource(resource!("ui/twitch-auth"));

        let window: Window = get_obj!(b, "twitch-auth-window");
        let web_view = WebView::new();
        let stack: Stack = get_obj!(b, "twitch-auth-stack");
        let continue_button: Button = get_obj!(b, "twitch-auth-continue");
        let try_again_button: Button = get_obj!(b, "twitch-auth-try-again");

        let inner = Rc::new(Self {
            window,
            web_view,
            stack,
            continue_button,
            try_again_button
        });

        inner.window.set_transient_for(Some(main_window));
        inner.window.set_attached_to(Some(main_window));
        inner.stack.set_visible_child_name("spinner");

        inner.window.connect_delete_event(clone!(@strong app => move |_, _| {
            app.quit();
            gtk::Inhibit(false)
        }));

        let wkvp: Viewport = get_obj!(b, "wkvp");
        wkvp.add(&inner.web_view);

        inner.web_view.connect_load_changed(clone!(@strong inner => move |vw, le| {

            match le {
                LoadEvent::Finished => {
                    inner.stack.set_visible_child_name("webview");
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
                                }, clone!(@strong inner => move |ret| {
                                    match ret {
                                        Ok(mut user) => {
                                            let tw = user.data.remove(0);
                                            let u = User::new(tw.login, tw.id, token.clone());
                                            match u.login() {
                                                Ok(_) => {
                                                    USER.lock().unwrap().replace(u);
                                                    inner.stack.set_visible_child_name("success");
                                                    inner.window.set_deletable(false);
                                                },
                                                Err(e) => {
                                                    warning!("{}", e);
                                                    inner.stack.set_visible_child_name("failure");
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            warning!("{}", e);
                                            inner.stack.set_visible_child_name("failure");
                                        }
                                    }
                                }));
                            }
                        },
                        None => {}
                    }
                },
                _ => {
                    if let Some(g_uri) = vw.get_uri() {
                        let uri = g_uri.to_string();
                        if uri.starts_with(REDIRECT_URL) {
                            inner.stack.set_visible_child_name("spinner");
                        }
                    }
                }
                // _ => stack.set_visible_child_name("spinner")
            }

        }));

        inner.continue_button.connect_clicked(clone!(@strong inner => move |_| {
            inner.hide()
        }));

        inner.try_again_button.connect_clicked(clone!(@strong inner => move |_| {
            inner.show()
        }));

        inner

    }

    pub fn show(&self) {

        if let Some(context) = self.web_view.get_context() {
            context.clear_cache();
            if let Some(cm) = context.get_cookie_manager() {
                cm.delete_all_cookies()
            }
        }
        self.web_view.load_uri(&Self::url());
        self.stack.set_visible_child_name("spinner");
        self.window.set_deletable(true);
        self.window.show_all();

    }

    pub fn hide(&self) {

        self.window.hide();

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
