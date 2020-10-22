use {
    std::rc::Rc,
    crate::{ASSETS, rt, resource, resources::{STREAM_COVER_SIZE, bytes_to_pixbuf}},
    gtk::{FlowBoxChild, Box as GtkBox, Image, Label, prelude::*},
    glib::clone
};

pub struct LiveCard {
    flow_box_child: FlowBoxChild
}

impl LiveCard {

    pub fn new(img_url: String, title: &str, streamer: &str) -> Self {

        let fbc = FlowBoxChild::new();
        fbc.set_halign(gtk::Align::Center);
        fbc.set_valign(gtk::Align::Start);

        let vbox = GtkBox::new(gtk::Orientation::Vertical, 6);

        let image = Rc::new(Image::new());
        image.set_size_request(STREAM_COVER_SIZE.0, STREAM_COVER_SIZE.1);
        image.set_from_resource(Some(resource!("images/thumbnail-404")));
        image.set_tooltip_text(Some(title));

        let bottom_label = Label::new(Some(streamer));

        rt::run_cb_local(async move {
            ASSETS.load(&img_url).await
        }, clone!(@strong image => move |res| {
            if let Ok(bytes) = res {
                if let Ok(pixbuf) = bytes_to_pixbuf(&bytes, Some((STREAM_COVER_SIZE.0, STREAM_COVER_SIZE.1))) {
                    image.set_from_pixbuf(Some(&pixbuf));
                }
            }
        }));

        vbox.add(&*image);
        vbox.add(&bottom_label);

        fbc.add(&vbox);
        fbc.show_all();

        Self {
            flow_box_child: fbc
        }

    }

    pub fn get_widget(&self) -> &FlowBoxChild {
        &self.flow_box_child
    }

}
