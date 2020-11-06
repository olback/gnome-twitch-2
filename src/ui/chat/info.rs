use {
    crate::get_obj,
    gtk::{prelude::*, Builder, Label},
    twitchchat::messages::FollowersOnly,
};

pub struct ChatInfo {
    emotes_only: Label,
    followers_only: Label,
    slow_mode: Label,
    subsribers_only: Label,
}

impl ChatInfo {
    pub fn configure(builder: &Builder) -> Self {
        Self {
            emotes_only: get_obj!(builder, "chat-info-emotes"),
            followers_only: get_obj!(builder, "chat-info-followers"),
            slow_mode: get_obj!(builder, "chat-info-slow"),
            subsribers_only: get_obj!(builder, "chat-info-slow")
        }
    }

    pub fn set_emotes_only(&self, value: bool) {
        match value {
            false => self.emotes_only.set_text("No"),
            true => self.emotes_only.set_text("Yes")
        }
    }

    pub fn set_followers_only(&self, value: FollowersOnly) {
        match value {
            FollowersOnly::Disabled => self.followers_only.set_text("No"),
            FollowersOnly::All => self.followers_only.set_text("Yes"),
            FollowersOnly::Limit(minutes) => match minutes {
                10 => self.followers_only.set_text("10 minutes"),
                60 => self.followers_only.set_text("1 hour"),
                _ => {}
            }
        }
    }

    pub fn set_slow_mode(&self, value: u64) {
        match value { // All possible value as of writing this
            0 => self.slow_mode.set_text("No"),
            3 => self.slow_mode.set_text("3 seconds"),
            5 => self.slow_mode.set_text("5 seconds"),
            10 => self.slow_mode.set_text("10 seconds"),
            20 => self.slow_mode.set_text("20 seconds"),
            30 => self.slow_mode.set_text("30 seconds"),
            60 => self.slow_mode.set_text("1 minute"),
            120 => self.slow_mode.set_text("2 minutes"),
            value => self.slow_mode.set_text(&format!("{} secoonds", value)),
        }
    }

    pub fn set_subscribers_only(&self, value: bool) {
        match value {
            true => self.subsribers_only.set_text("Yes"),
            false => self.subsribers_only.set_text("No")
        }
    }
}

