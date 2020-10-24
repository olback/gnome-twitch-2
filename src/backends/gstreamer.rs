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
        MessageView as GstMessageView,
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

            let evt = match msg.view() {

                GstMessageView::Buffering(buffering) => {
                    state.replace(GtPlayerState::Buffering);
                    let perc = buffering.get_percent();
                    drop(playbin.set_state(if perc < 100 {
                        GstState::Paused
                    } else {
                        GstState::Playing
                    }));
                    Some(GtPlayerEvent::StateChange(GtPlayerState::Buffering))
                },

                GstMessageView::Eos(_) => {
                    state.replace(GtPlayerState::Eos);
                    drop(playbin.set_state(GstState::Null));
                    Some(GtPlayerEvent::StateChange(GtPlayerState::Eos))
                },

                GstMessageView::StateChanged(state_change) => {
                    let old_state = state_change.get_old();
                    let new_state = state_change.get_current();
                    if let Some(src) = msg.get_src() {
                        if src == *playbin && old_state != new_state {
                            match new_state {
                                GstState::Paused => {
                                    state.replace(GtPlayerState::Paused);
                                    Some(GtPlayerEvent::StateChange(GtPlayerState::Paused))
                                },
                                GstState::Ready | GstState::Null => {
                                    state.replace(GtPlayerState::Stopped);
                                    Some(GtPlayerEvent::StateChange(GtPlayerState::Stopped))
                                },
                                GstState::Playing => {
                                    state.replace(GtPlayerState::Playing);
                                    Some(GtPlayerEvent::StateChange(GtPlayerState::Playing))
                                },
                                _ => None
                            }
                        } else { None }
                    } else { None }
                },

                GstMessageView::Warning(warning) => {
                    Some(GtPlayerEvent::Warning(format!("{}", warning.get_error())))
                },

                GstMessageView::Error(error) => {
                    Some(GtPlayerEvent::Error(format!("{}", error.get_error())))
                }

                _ => None

            };

            if let Some(e) = evt {
                // if let Some(ptr) = *cb {
                //     ptr(e)
                // }
                cb(e)
            }

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
