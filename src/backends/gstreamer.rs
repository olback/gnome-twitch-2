// Parts are stolen from:
// https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/gtksink.rs and
// https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/gtkvideooverlay.rs

use {
    crate::{p, error::GtResult},
    super::{GtPlayerBackend, GtPlayerState},
    std::{rc::Rc, cell::RefCell},
    gtk::{Widget, prelude::*},
    gio::{Settings, SettingsExt, SettingsBindFlags},
    gst::{
        MessageView as GstMessageView,
        State as GstState,
        Element as GstElement,
        ElementFactory as GstElementFactory,
        prelude::*
    },
};

pub struct BackendGStreamer {
    playbin: GstElement,
    state: Rc<RefCell<GtPlayerState>>,
    widget: Widget
}

impl BackendGStreamer {

    pub fn new(settings: &Settings) -> GtResult<Self> {

        let (video_sink, widget) = {
            let sink = gst::ElementFactory::make("gtksink", None).unwrap();
            let widget = sink.get_property("widget").unwrap();
            (sink, widget.get::<gtk::Widget>().unwrap().unwrap())
        };

        let playbin = p!(GstElementFactory::make("playbin", None));
        p!(playbin.set_property("video-sink", &video_sink));

        settings.bind(
            "volume",
            &playbin,
            "volume",
            SettingsBindFlags::DEFAULT
        );

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

impl GtPlayerBackend for BackendGStreamer {

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
