use serde::Deserialize;

mod game;
mod stream;
mod user;

pub use {
    game::Game,
    stream::Stream,
    user::User
};

#[derive(Debug, Deserialize)]
pub struct TwitchPagination {
    pub cursor: String
}

#[derive(Debug, Deserialize)]
pub struct TwitchResponse<T> {
    pub data: Vec<T>,
    pub pagination: Option<TwitchPagination>
}
