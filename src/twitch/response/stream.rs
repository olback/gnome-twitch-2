#[derive(Clone, Debug, serde::Deserialize)]
pub struct Stream {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub game_id: String,
    pub r#type: String,
    pub title: String,
    pub viewer_count: u32,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String,
    pub tag_ids: Vec<String>
}

// {
//     "id": "39514561805",
//     "user_id": "27446517",
//     "user_name": "Monstercat",
//     "game_id": "26936",
//     "type": "live",
//     "title": "320 - Monstercat: Call of the Wild (Community Picks with Dylan Todd) - 1pPDT / 4pEDT / 10pCEST🎙",
//     "viewer_count": 363,
//     "started_at": "2020-10-14T06:13:38Z",
//     "language": "en",
//     "thumbnail_url": "https://static-cdn.jtvnw.net/previews-ttv/live_user_monstercat-{width}x{height}.jpg",
//     "tag_ids": [
//         "6ea6bca4-4712-4ab9-a906-e3336a9d8039"
//     ]
// }
