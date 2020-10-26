// Parts are stolen from:
// https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/gtksink.rs and
// https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/master/examples/src/bin/gtkvideooverlay.rs

use {
    crate::{p, error::GtResult},
    super::{GtPlayerBackend, GtPlayerState, GtPlayerEvent, GtPlayerEventCb},
    std::{rc::Rc, cell::RefCell},
    gtk::{Widget, prelude::*},
    gio::{Settings, SettingsExt, SettingsBindFlags},
    gst::{
        State as GstState,
        Element as GstElement,
        ElementFactory as GstElementFactory,
        prelude::*
    },
    glib::clone
};

pub struct BackendGStreamer {
    playbin: Rc<GstElement>,
    state: Rc<RefCell<GtPlayerState>>,
    cb: Rc<GtPlayerEventCb>,
    widget: Widget
}

impl BackendGStreamer {

    pub fn new(settings: &Settings, cb: GtPlayerEventCb) -> GtResult<Self> {

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

        let inner = Self {
            playbin: Rc::new(playbin),
            state: Rc::new(RefCell::new(GtPlayerState::Stopped)),
            cb: Rc::new(cb),
            widget
        };

        let bus = p!(inner.playbin.get_bus().ok_or("Could not get playbin bus"));
        p!(bus.add_watch_local(clone!(
            @strong inner.playbin as playbin,
            @strong inner.state as state,
            @strong inner.cb as cb
        => move |_, msg| {

            super::bus_event_handler(msg, &*playbin, &*state, &*cb);

            glib::Continue(true)

        })));

        Ok(inner)

    }

    pub fn boxed(settings: &Settings, cb: GtPlayerEventCb) -> GtResult<Box<dyn GtPlayerBackend>> {
        let inner = Self::new(settings, cb)?;
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
        // Emit event here since playbin does not emit anything when set to null
        self.state.replace(GtPlayerState::Stopped);
        (self.cb)(GtPlayerEvent::StateChange(self.state.borrow().clone()));
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
