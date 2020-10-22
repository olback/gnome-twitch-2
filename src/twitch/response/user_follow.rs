#[derive(Clone, Debug, serde::Deserialize)]
pub struct UserFollow {
    pub from_id: String,
    pub from_name: String,
    pub to_id: String,
    pub to_name: String,
    pub followed_at: String
}
