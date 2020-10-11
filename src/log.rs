// const LOG_DOMAIN: &'static str = "GnomeTwitch2";

#[macro_export]
macro_rules! log_domain {
    () => {
        "GnomeTwitch2"
    }
}

#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        glib::g_log!(log_domain!(), glib::LogLevel::Message, $($arg)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        glib::g_log!(log_domain!(), glib::LogLevel::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        glib::g_log!(log_domain!(), glib::LogLevel::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        glib::g_log!(log_domain!(), glib::LogLevel::Warning, $($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        glib::g_log!(log_domain!(), glib::LogLevel::Error, $($arg)*)
    };
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {
        glib::g_log!(log_domain!(), glib::LogLevel::Critical, $($arg)*)
    };
}

