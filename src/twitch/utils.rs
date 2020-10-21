pub struct TwitchUtils;

impl TwitchUtils {

    pub fn thumbnail_sizer(url: &str, width: i32, height: i32) -> String {
        url.replace("{width}", &width.to_string()).replace("{height}", &height.to_string())
    }

}
