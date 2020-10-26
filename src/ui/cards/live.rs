use {
    std::rc::Rc,
    crate::{ASSETS, rt, resource, resources::{STREAM_COVER_SIZE, bytes_to_pixbuf}},
    gtk::{FlowBoxChild, Box as GtkBox, EventBox, Image, Label, prelude::*},
    glib::{clone, Sender}
};

pub struct LiveCard {
    flow_box_child: FlowBoxChild
}

impl LiveCard {

    pub fn new(img_url: String, title: String, streamer: String, tx: Sender<(String, String)>) -> Self {

        let fbc = FlowBoxChild::new();
        fbc.set_halign(gtk::Align::Center);
        fbc.set_valign(gtk::Align::Start);

        let evbox = EventBox::new();
        let vbox = GtkBox::new(gtk::Orientation::Vertical, 6);
        evbox.add(&vbox);

        let image = Rc::new(Image::new());
        image.set_size_request(STREAM_COVER_SIZE.0, STREAM_COVER_SIZE.1);
        image.set_from_resource(Some(resource!("images/thumbnail-404")));
        image.set_tooltip_text(Some(&title));

        let bottom_label = Label::new(Some(&streamer));

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

        fbc.add(&evbox);
        fbc.show_all();

        evbox.connect_button_press_event(move |_, evbutton| {
            if evbutton.get_button() == 1 {
                tx.send((title.clone(), streamer.clone())).expect("Failed to send stream info");
            }
            gtk::Inhibit(false)
        });

        Self {
            flow_box_child: fbc
        }

    }

    pub fn get_widget(&self) -> &FlowBoxChild {
        &self.flow_box_child
    }

}
