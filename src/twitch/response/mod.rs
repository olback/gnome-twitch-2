use serde::Deserialize;

mod game;
mod stream;
mod user;
mod user_follow;
mod clip;

pub use {
    game::Game,
    stream::Stream,
    user::User,
    user_follow::UserFollow,
    clip::{ClipCreated/*, Clip*/}
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

#[derive(Debug, Deserialize)]
pub(super) struct TwitchResponseError {
    pub(super) error: String,
    pub(super) status: u16,
    pub(super) message: String
}
