use {
    std::rc::Rc,
    gtk::{Builder, FlowBox, ScrolledWindow, prelude::*}
};

pub struct FollowingView {

}

impl FollowingView {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        Rc::new(Self { })

    }

}
