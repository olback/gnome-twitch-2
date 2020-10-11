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
macro_rules! resource {
    ($res:expr) => {
        concat!("/net/olback/GnomeTwitch2/", $res)
    };
}
