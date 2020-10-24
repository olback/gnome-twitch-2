use {
    crate::{get_obj, warning},
    std::{rc::Rc, cell::RefCell},
    gtk::{Builder, Entry, TextView, TextBuffer, TextTagTable, prelude::*}
};

pub struct ChatSection {
    input: Entry,
    view: TextView,
    buffer: TextBuffer,
    tag_table: TextTagTable
}

impl ChatSection {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        let inner = Rc::new(Self {
            input: get_obj!(builder, "chat-input"),
            view: get_obj!(builder, "chat-view"),
            buffer: get_obj!(builder, "chat-buffer"),
            tag_table: get_obj!(builder, "chat-tag-table")
        });

        inner

    }

    pub fn append_message(&self, user: &str, text: &str) {

    }

}
