mod builder;
mod error;
pub mod response;
mod utils;

pub use {
    utils::TwitchUtils,
    error::{TwError, TwResult},
};

use {
    builder::TwitchBuilder,
    reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}},
    url::Url,
    response::*
};

macro_rules! olen {
    ($v:expr) => {
        match $v {
            Some(ref _ignore) => 1usize,
            None => 0usize
        }
    };
}

macro_rules! ovlen {
    ($v:expr) => {
        $v.as_ref().map(Vec::len).unwrap_or(0usize)
    };
}

macro_rules! val_query {
    ($name:expr, $query_option:expr, $qvr:expr) => {
        if let Some(qv) = $query_option {
            $qvr.push(($name, qv.to_string()))
        }
    };
}

macro_rules! multi_val_query {
    ($name:expr, $query_option:expr, $qvr:expr) => {
        if let Some(qov) = $query_option {
            for qv in qov {
                $qvr.push(($name, qv));
            }
        }
    };
}

const TWITCH_API_URL: &'static str = "https://api.twitch.tv";
const USER_AGENT: &'static str = concat!("Gnome-Twitch-2 ", include_str!(concat!(env!("OUT_DIR"), "/version.txt")));

#[derive(Debug, Clone)]
pub struct Twitch {
    pub(super) token: Option<String>,
    pub(super) client_id: Option<String>
}

impl Twitch {

    pub fn new(token: Option<String>, client_id: Option<String>) -> Self {
        Self {
            token,
            client_id
        }
    }

    pub fn builder() -> TwitchBuilder {
        TwitchBuilder {
            token: None,
            client_id: None
        }
    }

    // https://dev.twitch.tv/docs/api/reference#start-commercial
    // pub async fn start_commercial(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-extension-analytics
    // pub async fn get_extension_analytics(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-game-analytics
    // pub async fn get_cheermotes(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-bits-leaderboard
    // pub async fn get_bits_leaderboard(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-extension-transactions
    // pub async fn get_extension_transactions(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#create-clip
    // pub async fn create_clip(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-clips
    // pub async fn get_clips(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#create-entitlement-grants-upload-url
    // pub async fn create_entitlement_grants_upload_url(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-code-status
    // pub async fn get_code_status(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-drops-entitlements
    // pub async fn get_drops_entitlements(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#redeem-code
    // pub async fn redeem_code(&self) -> TwResult<TwitchResponse<T>> {}

    /// Get top games
    /// Pagination is available via `before` and `after`.
    /// The amount of games is determined by `first`. (Up to 100 games)
    /// https://dev.twitch.tv/docs/api/reference#get-top-games
    pub async fn get_top_games(
        &self,
        after: Option<String>,
        before: Option<String>,
        first: Option<u8>
    ) -> TwResult<TwitchResponse<Game>> {

        let len =
            olen!(after) +
            olen!(before) +
            olen!(first);

        let mut query = Vec::with_capacity(len);
        val_query!("after", after, query);
        val_query!("before", before, query);
        val_query!("first", first, query);

        let client = self.base_request()?;
        let response = client.get(&Self::url("/helix/games/top", &query)?).send().await?;
        Ok(response.json().await?)

    }

    /// Get games details.
    /// Either `ids` or `names` has to be set. Maximum 100 values may be set.
    /// https://dev.twitch.tv/docs/api/reference#get-games
    pub async fn get_games(
        &self,
        ids: Option<Vec<String>>,
        names: Option<Vec<String>>
    ) -> TwResult<TwitchResponse<Game>> {

        let len =
            ovlen!(ids) +
            ovlen!(names);

        let mut query = Vec::<(&str, String)>::with_capacity(len);
        multi_val_query!("id", ids, query);
        multi_val_query!("name", names, query);

        let client = self.base_request()?;
        let response = client.get(&Self::url("/helix/games", &query)?).send().await?;
        Ok(response.json().await?)

    }

    // https://dev.twitch.tv/docs/api/reference#get-hype-train-events
    // pub async fn get_hype_train_events(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#check-automod-status
    // pub async fn check_automod_status(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-banned-users
    // pub async fn get_banned_users(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-banned-events
    // pub async fn get_banned_events(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-moderators
    // pub async fn get_moderators(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-moderator-events
    // pub async fn get_moderator_events(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#search-categories
    // pub async fn search_categories(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#search-channels
    // pub async fn search_channels(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-stream-key
    // pub async fn get_stream_key(&self) -> TwResult<TwitchResponse<T>> {}

    /// Get streams.
    /// You may specify up to 100 `game_ids`, `languages`, `user_ids` and `user_logins`.
    /// https://dev.twitch.tv/docs/api/reference#get-streams
    pub async fn get_streams(
        &self,
        after: Option<String>,
        before: Option<String>,
        first: Option<u8>,
        game_ids: Option<Vec<String>>,
        languages: Option<Vec<String>>,
        user_ids: Option<Vec<String>>,
        user_logins: Option<Vec<String>>
    ) -> TwResult<TwitchResponse<Stream>> {

        let query_len =
            olen!(after) +
            olen!(before) +
            olen!(first) +
            ovlen!(game_ids) +
            ovlen!(languages) +
            ovlen!(user_ids) +
            ovlen!(user_logins);

        let mut query = Vec::<(&str, String)>::with_capacity(query_len);
        val_query!("after", after, query);
        val_query!("before", before, query);
        val_query!("first", first, query);
        multi_val_query!("game_id", game_ids, query);
        multi_val_query!("language", languages, query);
        multi_val_query!("user_id", user_ids, query);
        multi_val_query!("user_login", user_logins, query);

        let client = self.base_request()?;
        let response = client.get(&Self::url("/helix/streams", &query)?).send().await?;
        Ok(response.json().await?)

    }

    // https://dev.twitch.tv/docs/api/reference#create-stream-marker
    // pub async fn create_stream_marker(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-stream-markers
    // pub async fn get_stream_markers(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-channel-information
    // pub async fn get_channel_information(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#modify-channel-information
    // pub async fn modify_channel_information(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-broadcaster-subscriptions
    // pub async fn get_broadcaster_subscriptions(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-all-stream-tags
    // pub async fn get_all_stream_tags(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-stream-tags
    // pub async fn get_stream_tags(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#replace-stream-tags
    // pub async fn replace_stream_tags(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#create-user-follows
    // pub async fn create_user_follows(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#delete-user-follows
    // pub async fn delete_user_follows(&self) -> TwResult<TwitchResponse<T>> {}

    /// https://dev.twitch.tv/docs/api/reference#get-users
    pub async fn get_users(
        &self,
        ids: Option<Vec<String>>,
        logins: Option<Vec<String>>
    ) -> TwResult<TwitchResponse<User>> {

        let len =
            ovlen!(ids) +
            ovlen!(logins);

        let mut query = Vec::with_capacity(len);
        multi_val_query!("id", ids, query);
        multi_val_query!("login", logins, query);

        let client = self.base_request()?;
        let response = client.get(&Self::url("/helix/users", &query)?).send().await?;
        Ok(response.json().await?)

    }

    /// Get a users followers/following.
    /// Either `from_id` or `to_id` must be set.
    /// https://dev.twitch.tv/docs/api/reference#get-users-follows
    pub async fn get_users_follows(
        &self,
        after: Option<String>,
        first: Option<u8>,
        from_id: Option<String>,
        to_id: Option<String>
    ) -> TwResult<TwitchResponse<UserFollow>> {

        let len =
            olen!(after) +
            olen!(first) +
            olen!(from_id) +
            olen!(to_id);

        let mut query = Vec::with_capacity(len);
        val_query!("after", after, query);
        val_query!("first", first, query);
        val_query!("from_id", from_id, query);
        val_query!("to_id", to_id, query);

        let client = self.base_request()?;
        let response = client.get(&Self::url("/helix/users/follows", &query)?).send().await?;

        Ok(response.json().await?)

    }

    // https://dev.twitch.tv/docs/api/reference#update-user
    // pub async fn update_user(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-user-extensions
    // pub async fn get_user_extensions(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-user-active-extensions
    // pub async fn get_user_active_extensions(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#update-user-extensions
    // pub async fn update_user_extensions(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-videos
    // pub async fn get_videos(&self) -> TwResult<TwitchResponse<T>> {}

    // https://dev.twitch.tv/docs/api/reference#get-webhook-subscriptions
    // pub async fn get_webhook_subscriptions(&self) -> TwResult<TwitchResponse<T>> {}

    fn base_request(&self) -> TwResult<Client> {
        let mut dh = HeaderMap::new();
        if let Some(token) = &self.token {
            dh.insert(HeaderName::from_static("authorization"), HeaderValue::from_str(&format!("Bearer {}", token))?);
        }
        if let Some(client_id) = &self.client_id {
            dh.insert(HeaderName::from_static("client-id"), HeaderValue::from_str(&client_id)?);
        }
        Ok(Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(dh)
            .build()?)
    }

    fn url(path: &str, query: &[(&str, String)]) -> TwResult<String> {

        let mut url = Url::parse(TWITCH_API_URL)?.join(path)?;
        let mut query_pairs = url.query_pairs_mut();
        for q in query {
            query_pairs.append_pair(q.0, &q.1);
        }

        drop(query_pairs);

        Ok(url.to_string())

    }

}

