use super::{Twitch, TwResult};

pub struct TwitchBuilder {
    pub(super) token: Option<String>,
    pub(super) client_id: Option<String>
}

impl TwitchBuilder {

    pub fn set_token(mut self, token: Option<String>) -> Self{
        self.token = token;
        self
    }

    pub fn set_client_id(mut self, client_id: Option<String>) -> Self{
        self.client_id = client_id;
        self
    }

    pub fn build(self) -> TwResult<Twitch> {
        // TODO: Validate fields
        Ok(Twitch {
            token: self.token,
            client_id: self.client_id
        })
    }

}
