use {
    super::{TwResult, Twitch},
    reqwest
};

const TWITCH_ACCESS_TOKEN_CLIENT_ID: &'static str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub sig: String,
    pub mobile_restricted: bool,
    pub expires_at: String
}

pub struct TwitchExtras;

impl TwitchExtras {

    pub async fn access_token(username: &str, oauth_token: Option<String>) -> TwResult<AccessToken> {

        let client = Twitch::new(
            oauth_token,
            Some(TWITCH_ACCESS_TOKEN_CLIENT_ID.into())
        ).base_request()?;

        let response = client
            .get(&Twitch::url(&format!("/api/channels/{}/access_token.json", username), &[])?)
            .send().await
            .map(super::handle_response_errors)?.await?;
        Ok(response.json().await?)

    }

    pub async fn usher(username: &str, sig: String, token: String) -> TwResult<String> {

        let query: Vec<(&'static str, String)> = vec![
            ("player", "twitchweb".into()),
            ("type", "any".into()),
            ("fast_bread", "true".into()),
            ("playlist_include_framerate", "true".into()),
            ("allow_source", "true".into()),
            ("allow_audio_only", "true".into()),
            ("allow_spectre", "false".into()),
            ("sig", sig),
            ("token", token.replace("\\", ""))

        ];

        let url = usher_url(username, &query)?;

        Ok(reqwest::get(&url).await.map(super::handle_response_errors)?.await?.text().await?)

    }

}

fn usher_url(username: &str, query: &[(&str, String)]) -> TwResult<String> {

    let mut url = url::Url::parse(&format!("https://usher.ttvnw.net/api/channel/hls/{}.m3u8", username.to_lowercase()))?;
    let mut query_pairs = url.query_pairs_mut();
    for (k, v) in query {
        query_pairs.append_pair(k, &v);
    }

    drop(query_pairs);

    Ok(url.to_string())

}
