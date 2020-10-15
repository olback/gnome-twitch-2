pub struct TwitchUtils;

impl TwitchUtils {

    pub fn thumbnail_sizer(url: &str, width: u16, height: u16) -> String {
        url.replace("{width}", &width.to_string()).replace("{height}", &height.to_string())
    }

}
