use {
    crate::{get_obj, warning},
    std::{rc::Rc, cell::RefCell},
    gtk::{Builder, Entry, TextView, TextBuffer, TextTag, TextIter, TextTagTable, prelude::*},
    gdk_pixbuf::Pixbuf
};

pub struct ChatSection {
    input: Entry,
    view: TextView,
    buffer: TextBuffer,
    tag_table: TextTagTable
}

type RGB = (u8, u8, u8);

impl ChatSection {

    pub fn configure(builder: &Builder) -> Rc<Self> {

        let inner = Rc::new(Self {
            input: get_obj!(builder, "chat-input"),
            view: get_obj!(builder, "chat-view"),
            buffer: get_obj!(builder, "chat-buffer"),
            tag_table: get_obj!(builder, "chat-tag-table")
        });

        let mod_icon = gdk_pixbuf::Pixbuf::from_file("/home/olback/Downloads/mod.png").unwrap();

        inner.append_message(
            &[&mod_icon],
            (0xff, 0x00, 0xff),
            "olback",
            "Hello this is a regular chat message that should wrap niceley. Byeee :wave:",
            &[]
        );

        inner.append_message(
            &[&mod_icon],
            (0xff, 0x00, 0xff),
            "olback",
            "Hello this is a regular chat message that should wrap niceley. Byeee :wave:",
            &[]
        );

        inner.append_message(
            &[&mod_icon],
            (0xff, 0x00, 0xff),
            "olback",
            "Hello this is a regular chat message that should wrap niceley. Byeee :wave:",
            &[]
        );

        // inner.append_message(
        //     (0xff, 0x00, 0xff),
        //     &[&mod_icon],
        //     "olback",
        //     "This is a REEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE message",
        //     &[]
        // );

        // inner.append_message(
        //     (0xff, 0x00, 0xff),
        //     &[&mod_icon],
        //     "olback",
        //     "https://tempfiles.ninja/d/hTOTdrWkpYxjdAaA/GSA6zbq4PEmGFR0HGexGlFKIMm3GYLaK",
        //     &[]
        // );

        inner

    }

    pub fn append_message(
        &self,
        badges: &[&Pixbuf],
        color: RGB,
        from: &str,
        text: &str,
        emotes: &[((usize, usize), &Pixbuf)]
    ) {

        // TODO: Emotes

        // Badges
        for badge in badges {
            self.buffer.insert_pixbuf(
                &mut self.buffer.get_end_iter(),
                *badge
            );
            self.buffer.insert_at_cursor(" ");
        }

        let rgb_str = format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2);
        let color_tag = match self.tag_table.lookup(&rgb_str) {
            Some(tag) => tag,
            None => {
                let tag = TextTag::new(Some(&rgb_str));
                tag.set_properties(&[
                    ("foreground", &rgb_str),
                    ("foreground-set", &true),
                    ("weight", &600),
                    ("weight-set", &true)
                ]).unwrap();
                self.tag_table.add(&tag);
                tag
            }
        };

        // Username
        self.buffer.insert_with_tag(
            &mut self.buffer.get_end_iter(),
            from, &color_tag
        );

        // Colon and space after username
        self.buffer.insert_at_cursor(": ");

        // Message
        self.buffer.insert(
            &mut self.buffer.get_end_iter(),
            text
        );

        // New line after each message
        self.buffer.insert_at_cursor("\n\n");

    }

}

// Whis isn't this in GTK-RS? :(
trait InsertWithTag {
    fn insert_with_tag<T: glib::IsA<TextTag>>(&self, iter: &mut TextIter, text: &str, tag: &T) {
        self.insert_with_tags(iter, text, &[tag])
    }
    fn insert_with_tags<T: glib::IsA<TextTag>>(&self, iter: &mut TextIter, text: &str, tags: &[&T]);
}

impl InsertWithTag for TextBuffer {
    fn insert_with_tags<T: glib::IsA<TextTag>>(&self, iter: &mut TextIter, text: &str, tags: &[&T]) {
        self.insert(iter, text);
        let end_iter = self.get_end_iter();
        iter.backward_chars(text.len() as i32);
        for tag in tags {
            self.apply_tag(*tag, &iter, &end_iter);
        }
    }
}
