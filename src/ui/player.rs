use {
    crate::{resource, get_obj, message, warning, error, backends::{GtPlayerBackend, BackendGstreamerOpenGL}},
    std::cell::RefCell,
    gtk::{Application, Builder, Box as GtkBox, Label, prelude::*},
    gst::prelude::*
};

// pub fn configure(app: &Application, builder: &Builder) {

//     let container: GtkBox = get_obj!(builder, "player-container");
//     let timestamp_label: Label = get_obj!(builder, "timestamp-label");

//     let pipeline = gst::Pipeline::new(None);
//     let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
//     // let src = gst::ElementFactory::make("filesrc", None).unwrap();
//     // src.set_property("location", &"/home/olback/Videos/bbb4k60.mp4").unwrap();
//     // let src = gst::ElementFactory::make("filesrc", None).unwrap();
//     // src.set_property("uri", &"https://olback.net/download/pingu.mp4").unwrap();
//     // let src = gst::ElementFactory::make("playbin", None).unwrap();
//     // src.set_property("uri", &"file:///home/olback/Videos/bbb4k60.mp4").unwrap();

//     let (sink, video_widget) = if let Ok(gtkglsink) = gst::ElementFactory::make("gtkglsink", None) {
//         // GPU Acceleration :)
//         message!("Using GPU Acceleration");
//         let glsinkbin = gst::ElementFactory::make("glsinkbin", None).unwrap();
//         glsinkbin.set_property("sink", &gtkglsink.to_value()).unwrap();

//         let widget = gtkglsink.get_property("widget").unwrap();
//         (glsinkbin, widget.get::<gtk::Widget>().unwrap().unwrap())
//     } else {
//         // CPU Accelerated :(
//         warning!("Using CPU acceleration");
//         let sink = gst::ElementFactory::make("gtksink", None).unwrap();
//         let widget = sink.get_property("widget").unwrap();
//         (sink, widget.get::<gtk::Widget>().unwrap().unwrap())
//     };

//     pipeline.add_many(&[&src, &sink]).unwrap();
//     src.link(&sink).unwrap();

//     container.pack_start(&video_widget, true, true, 0);

//     let pipeline_weak = pipeline.downgrade();

//     let timeout_id = glib::timeout_add_local(500, move || {

//         let pipeline = match pipeline_weak.upgrade() {
//             Some(p) => p,
//             None => return glib::Continue(true)
//         };

//         let position = pipeline
//             .query_position::<gst::ClockTime>()
//             .unwrap_or_else(|| 0.into());

//         timestamp_label.set_text(&format!("{:.0}", position));

//         glib::Continue(true)

//     });

//     // let bus = pipeline.get_bus().unwrap();

//     pipeline.set_state(gst::State::Playing).unwrap();

//     // let app_weak = app.downgrade();
//     // bus.add_watch_local(move |_, msg| {

//     //     glib::Continue(true)

//     // }).unwrap();

//     // let timeout_id = RefCell::new(Some(timeout_id));
//     // app.connect_shutdown(move |_| {
//     //     pipeline.set_state(gst::State::Null).unwrap();
//     //     bus.remove_watch().unwrap();
//     //     if let Some(tid) = timeout_id.borrow_mut().take() {
//     //         glib::source_remove(tid);
//     //     }
//     // });

// }


pub fn configure(app: &Application, builder: &Builder) {

    let container: GtkBox = get_obj!(builder, "player-container");
    let timestamp_label: Label = get_obj!(builder, "timestamp-label");

    // glib::timeout_add_local(2000, move || {

        let mut player = BackendGstreamerOpenGL::new().unwrap();
        let video_widget = player.get_widget().unwrap();
        video_widget.show_all();

        container.pack_start(&video_widget, true, true, 0);

        // player.set_uri(Some("https://olback.net/download/pingu.mp4".into())).unwrap();
        // player.set_uri(Some("file:///home/olback/Videos/bbb4k60.mp4".into())).unwrap();
        // player.play().unwrap();

        player.stop().unwrap();
        player.set_uri(Some("file:///home/olback/Videos/test.mp4".into())).unwrap();
        player.play().unwrap();
        message!("should play :(");
        // player.set_position(100).unwrap();
        // player.pause().unwrap();
        // glib::Continue(false)
    // });

    glib::timeout_add_local(200, move || {
        player.query();
        glib::Continue(true)
    });

}
