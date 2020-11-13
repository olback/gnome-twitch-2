// const LOG_DOMAIN: &'static str = "GnomeTwitch2";

#[macro_export]
macro_rules! log_domain {
    () => {
        "GnomeTwitch2"
    }
}

#[macro_export]
macro_rules! prepend_line_file_info_if_debug {
    ($($arg:tt)*) => {
        if crate::is_debug!() {
            format!("{}#{}: {}", file!(), line!(), format!($($arg)*))
        } else {
            format!($($arg)*)
        }
    }
}

#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        glib::g_log!($crate::log_domain!(), glib::LogLevel::Message, &crate::prepend_line_file_info_if_debug!($($arg)*))
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        glib::g_log!($crate::log_domain!(), glib::LogLevel::Debug, &crate::prepend_line_file_info_if_debug!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        glib::g_log!($crate::log_domain!(), glib::LogLevel::Info, &crate::prepend_line_file_info_if_debug!($($arg)*))
    };
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        glib::g_log!($crate::log_domain!(), glib::LogLevel::Warning, &crate::prepend_line_file_info_if_debug!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        glib::g_log!($crate::log_domain!(), glib::LogLevel::Error, &crate::prepend_line_file_info_if_debug!($($arg)*))
    };
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)*) => {
        glib::g_log!($crate::log_domain!(), glib::LogLevel::Critical, &crate::prepend_line_file_info_if_debug!($($arg)*))
    };
}

