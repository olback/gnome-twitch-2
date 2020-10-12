use {
    crate::{resource, get_obj, message, warning, error},
    gtk::{Builder, Box as GtkBox, prelude::*},
    gst::prelude::*
};

pub fn configure(builder: &Builder) {

    let container: GtkBox = get_obj!(builder, "player-container");

    // Init GST
    gst::init().unwrap();

    let pipeline = gst::Pipeline::new(None);
    let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
    pipeline.set_state(gst::State::Playing).unwrap();

    let (sink, video_widget) = if let Ok(gtkglsink) = gst::ElementFactory::make("gtkglsink", None) {
        // GPU Acceleration :)
        message!("Using GPU Acceleration");
        let glsinkbin = gst::ElementFactory::make("glsinkbin", None).unwrap();
        glsinkbin.set_property("sink", &gtkglsink.to_value()).unwrap();

        let widget = gtkglsink.get_property("widget").unwrap();
        (glsinkbin, widget.get::<gtk::Widget>().unwrap().unwrap())
    } else {
        // CPU Accelerated :(
            warning!("Using CPU acceleration");
            let sink = gst::ElementFactory::make("gtksink", None).unwrap();
            let widget = sink.get_property("widget").unwrap();
            (sink, widget.get::<gtk::Widget>().unwrap().unwrap())
    };

    pipeline.add_many(&[&src, &sink]).unwrap();
    src.link(&sink).unwrap();

    container.pack_start(&video_widget, true, true, 0);

}
