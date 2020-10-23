// Parts are stolen from:
// https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/gtksink.rs and
// https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/gtkvideooverlay.rs

use {
    crate::{p, warning, new_err, error::GtResult},
    super::{GtPlayerBackend, GtPlayerState},
    std::{rc::Rc, cell::RefCell, os::raw::c_void},
    gtk::{Widget, prelude::*},
    gdk::prelude::*,
    gio::{Settings, SettingsExt, SettingsBindFlags},
    gstv::prelude::*,
    gst::{
        MessageView as GstMessageView,
        State as GstState,
        Element as GstElement,
        ElementFactory as GstElementFactory,
        prelude::*
    },
};

pub struct BackendGStreamerOpenGL {
    playbin: GstElement,
    state: Rc<RefCell<GtPlayerState>>,
    widget: Widget
}

impl BackendGStreamerOpenGL {

    pub fn new(settings: &Settings) -> GtResult<Self> {

        let gtkglsink = p!(GstElementFactory::make("gtkglsink", None));
        let widget = p!(p!(gtkglsink.get_property("widget")).get::<Widget>()).expect("Widget not created");
        let video_sink = p!(GstElementFactory::make("glsinkbin", None));
        p!(video_sink.set_property("sink", &gtkglsink));

        let playbin = p!(GstElementFactory::make("playbin", None));
        p!(playbin.set_property("video-sink", &video_sink));

        let video_overlay = video_sink
                .dynamic_cast::<gstv::VideoOverlay>()
                .unwrap()
                .downgrade();

            widget.connect_realize(move |video_window| {
                let video_overlay = match video_overlay.upgrade() {
                    Some(video_overlay) => video_overlay,
                    None => return,
                };

                let gdk_window = video_window.get_window().unwrap();

                if !gdk_window.ensure_native() {
                    warning!("Can't create native window for widget");
                }

                let res = set_window_handle(&video_overlay, &gdk_window);
                println!("{:#?}", res);
            });

        Ok(Self {
            playbin,
            state: Rc::new(RefCell::new(GtPlayerState::Stopped)),
            widget
        })

    }

    pub fn boxed(settings: &Settings) -> GtResult<Box<dyn GtPlayerBackend>> {
        let inner = Self::new(settings)?;
        Ok(Box::new(inner))
    }

}

impl GtPlayerBackend for BackendGStreamerOpenGL {

    fn play(&self) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Playing));
        self.state.replace(GtPlayerState::Loading);
        Ok(())
    }

    fn pause(&self) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Paused));
        Ok(())
    }

    fn stop(&self) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Null));
        Ok(())
    }

    fn set_uri(&self, uri: Option<String>) -> GtResult<()> {
        p!(self.playbin.set_property("uri", &uri));
        Ok(())
    }

    fn get_state(&self) -> GtPlayerState {
        self.state.borrow().clone()
    }

    fn get_widget(&self) -> &gtk::Widget {
        &self.widget
    }

}

fn set_window_handle(video_overlay: &gstv::VideoOverlay, gdk_window: &gdk::Window) -> GtResult<()> {

    let display_type_name = gdk_window.get_display().get_type().name();

    // Check if we're using X11 or ...
    if display_type_name == "GdkX11Display" {

        extern "C" {
            pub fn gdk_x11_window_get_xid(window: *mut glib::object::GObject) -> *mut c_void;
        }

        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            let xid = gdk_x11_window_get_xid(gdk_window.as_ptr() as *mut _);
            video_overlay.set_window_handle(xid as usize);
        }

        Ok(())

    } else {

        Err(new_err!(format!("Display type {} not supported", display_type_name)))

    }

}
