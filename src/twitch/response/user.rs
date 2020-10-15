#[derive(Debug, serde::Deserialize)]
pub struct User {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub r#type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u64,
    pub email: Option<String>
}

// {
//     "id": "27446517",
//     "login": "monstercat",
//     "display_name": "Monstercat",
//     "type": "",
//     "broadcaster_type": "partner",
//     "description": "Monstercat prides itself in supporting rising electronic artists from around the globe. We are proving that independent labels have the ability to reshape the music industry landscape.",
//     "profile_image_url": "https://static-cdn.jtvnw.net/jtv_user_pictures/monstercat-profile_image-3e109d75f8413319-300x300.jpeg",
//     "offline_image_url": "https://static-cdn.jtvnw.net/jtv_user_pictures/7d8f79b14cff3966-channel_offline_image-1920x1080.jpeg",
//     "view_count": 32040844,
//     "email": "name@provider.tld"
// }
