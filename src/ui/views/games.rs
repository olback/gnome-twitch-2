use {
    std::rc::Rc,
    gtk::{Builder, FlowBox, ScrolledWindow, prelude::*}
};

pub struct GamesView {

}

impl GamesView {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        Rc::new(Self { })

    }

}
