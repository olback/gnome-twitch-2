use {
    crate::{get_obj},
    std::rc::Rc,
    gtk::{Builder, SearchBar, prelude::*}
};

pub struct SearchSection {
    search_bar: SearchBar
}

impl SearchSection {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        let inner = Rc::new(Self {
            search_bar: get_obj!(builder, "app-search-bar")
        });

        inner

    }

    pub fn show(&self) {
        self.search_bar.set_search_mode(true)
    }

    pub fn hide(&self) {
        self.search_bar.set_search_mode(false)
    }

}
