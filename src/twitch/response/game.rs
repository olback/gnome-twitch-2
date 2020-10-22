#[derive(Clone, Debug, serde::Deserialize)]
pub struct Game {
    pub box_art_url: String,
    pub id: String,
    pub name: String
}

// {
//     "id": "26936",
//     "name": "Music",
//     "box_art_url": "https://static-cdn.jtvnw.net/ttv-boxart/Music-{width}x{height}.jpg"
// }
