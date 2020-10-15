#[macro_export]
macro_rules! p {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return Err($crate::error::GtError {
                line: line!(),
                file: file!(),
                error: Some(Box::new(e))
            })
        }
    };
}

#[macro_export]
macro_rules! new_err {
    ($cause:expr) => {
        $crate::error::GtError::new($cause, line!(), file!())
    };
}

#[macro_export]
macro_rules! get_obj {
    ($builder:expr, $id:expr) => {
        // Catch and panic manually to get useful file and line info
        {
            use gtk::prelude::BuilderExtManual;
            match $builder.get_object($id) {
                Some(o) => o,
                None => panic!("could not get {}", $id)
            }
        }
    };
    ($type:ty, $builder:expr, $id:expr) => {
        // Catch and panic manually to get useful file and line info
        {
            use gtk::prelude::BuilderExtManual;
            match $builder.get_object::<$type>($id) {
                Some(o) => o,
                None => panic!("could not get {}", $id)
            }
        }
    };
}

#[macro_export]
macro_rules! is_debug {
    () => {
        if cfg!(debug_assertions) {
            std::env::var("GT2_DEBUG").map(|v| v.to_lowercase()) != Ok("false".into())
        } else {
            std::env::var("GT2_DEBUG").map(|v| v.to_lowercase()) == Ok("true".into())
        }
    };
}

#[macro_export]
macro_rules! resource {
    ($res:expr) => {
        concat!("/net/olback/GnomeTwitch2/", $res)
    };
}
