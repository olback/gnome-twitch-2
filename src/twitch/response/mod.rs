use serde::Deserialize;

mod game;
mod stream;
mod user;
mod user_follow;

pub use {
    game::Game,
    stream::Stream,
    user::User,
    user_follow::UserFollow
};

#[derive(Clone, Debug, Deserialize)]
pub struct TwitchPagination {
    pub cursor: Option<String>
}

#[derive(Clone, Debug, Deserialize)]
pub struct TwitchResponse<T> {
    pub data: Vec<T>,
    pub pagination: Option<TwitchPagination>
}
