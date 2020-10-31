use {
    crate::{p, new_err, error::GtResult, resources::{APP_ID, KEYRING_NAME}},
    keyring::Keyring,
    gio::{Settings, SettingsExt}
};

#[derive(Clone, Debug)]
pub struct User {
    pub username: String,
    pub user_id: String,
    pub oauth_token: String
}

impl User {

    pub fn new(username: String, user_id: String, oauth_token: String) -> Self {

        Self {
            username,
            user_id,
            oauth_token
        }

    }

    pub fn load() -> GtResult<Self> {

        let settings = Settings::new(APP_ID);

        let username = settings
            .get_string("user-name")
            .map(|u| u.to_string())
            .unwrap_or(String::new());
        if username.trim().is_empty() {
            return Err(new_err!("Username not set"));
        }

        let user_id = settings
            .get_string("user-id")
            .map(|u| u.to_string())
            .unwrap_or(String::new());
        if user_id.trim().is_empty() {
            return Err(new_err!("User ID not set"));
        }

        let kr = Keyring::new(KEYRING_NAME, &username);
        let maybe_oauth_token = kr.get_password();
        let oauth_token = match maybe_oauth_token {
            Ok(token) if !token.trim().is_empty() => token,
            _ => return Err(new_err!("Oauth token not set"))
        };

        Ok(Self {
            username,
            user_id,
            oauth_token
        })

    }

    pub fn login(&self) -> GtResult<()> {

        let settings = Settings::new(APP_ID);
        p!(settings.set_string("user-name", &self.username));
        p!(settings.set_string("user-id", &self.user_id));

        let kr = Keyring::new(KEYRING_NAME, &self.username);
        p!(kr.set_password(&self.oauth_token));

        Ok(())

    }

    pub fn logout(&self) -> GtResult<()> {

        let settings = Settings::new(APP_ID);
        settings.reset("user-name");
        settings.reset("user-id");

        let kr = Keyring::new(KEYRING_NAME, &self.username);
        p!(kr.delete_password());

        Ok(())

    }


}
