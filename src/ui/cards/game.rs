use {
    std::rc::Rc,
    crate::{ASSETS, rt, resource, resources::{GAME_COVER_SIZE, bytes_to_pixbuf}},
    gtk::{FlowBoxChild, Box as GtkBox, EventBox, Image, Label, prelude::*},
    glib::clone
};

pub struct GameCard {
    flow_box_child: FlowBoxChild
}

impl GameCard {

    pub fn new(img_url: String, game: &str) -> Self {

        let fbc = FlowBoxChild::new();
        fbc.set_halign(gtk::Align::Center);
        fbc.set_valign(gtk::Align::Start);

        let evbox = EventBox::new();
        let vbox = GtkBox::new(gtk::Orientation::Vertical, 6);
        evbox.add(&vbox);

        let image = Rc::new(Image::new());
        image.set_size_request(GAME_COVER_SIZE.0, GAME_COVER_SIZE.1);
        image.set_from_resource(Some(resource!("images/boxart-404")));

        let top_label = Label::new(Some(game));
        top_label.set_line_wrap(true);

        rt::run_cb_local(async move {
            ASSETS.load(&img_url).await
        }, clone!(@strong image => move |res| {
            if let Ok(bytes) = res {
                if let Ok(pixbuf) = bytes_to_pixbuf(&bytes, Some((GAME_COVER_SIZE.0, GAME_COVER_SIZE.1))) {
                    image.set_from_pixbuf(Some(&pixbuf));
                }
            }
        }));

        vbox.add(&*image);
        vbox.add(&top_label);

        fbc.add(&evbox);
        fbc.show_all();

        evbox.connect_button_press_event(|_, evbutton| {
            if evbutton.get_button() == 1 {
                println!("Game clicked");
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