use {
    crate::{p, warning, error::{GtError, GtResult}},
    super::{GtPlayerBackend, GtPlayerState},
    std::{rc::Rc, cell::RefCell},
    gtk::Widget,
    gst::{
        MessageView as GstMessageView,
        SeekFlags as GstSeekFlags,
        ClockTime as GstClockTime,
        State as GstState,
        Element as GstElement,
        ElementFactory as GstElementFactory,
        Bin as GstBin,
        Bus as GstBus,
        Pad as GstPad,
        GhostPad as GstGhostPad,
        prelude::*
    },
};

// https://github.com/GStreamer/gstreamer/blob/master/gst/gstformat.h#L56
const GST_FORMAT_TIME: u32 = 3;

pub struct BackendGstreamerOpenGL {
    playbin: Rc<GstElement>,
    upload: GstElement,
    video_sink: GstElement,
    video_bin: GstBin,
    bus: GstBus,
    pad: GstPad,
    ghost_pad: GstGhostPad,
    widget: Widget,
    uri: Option<String>,
    state: Rc<RefCell<GtPlayerState>>,
    volume: f64,
    buffer_fill: Rc<RefCell<f64>>,
    seekable: Rc<RefCell<bool>>,
    duration: Rc<RefCell<u64>>,
    position: i64,
    position_tick_id: Rc<RefCell<Option<glib::SourceId>>>
}

impl BackendGstreamerOpenGL {

    pub fn new() -> GtResult<Self> {

        let playbin = Rc::new(p!(GstElementFactory::make("playbin", None)));
        let video_sink = p!(GstElementFactory::make("gtkglsink", None));
        let video_bin = GstBin::new(Some("video_bin"));
        let upload = p!(GstElementFactory::make("glupload", None));

        let bus = p!(playbin.get_bus().ok_or("Failed to get bus for playbin"));

        let seekable = Rc::new(RefCell::new(false));
        let duration = Rc::new(RefCell::new(0u64));
        let state = Rc::new(RefCell::new(GtPlayerState::Stopped));
        let tick = Rc::new(RefCell::new(None));

        // TODO:
        let playbin_clone = Rc::clone(&playbin);
        let seekable_clone = Rc::clone(&seekable);
        let duration_clone = Rc::clone(&duration);
        let state_clone = Rc::clone(&state);
        let tick_clone = Rc::clone(&tick);
        p!(bus.add_watch_local(move |b, msg| {

            let _ = playbin_clone.set_state(GstState::Playing);

            match msg.view() {
                GstMessageView::Buffering(buffering) => {
                    println!("Buffering");
                    let perc = buffering.get_percent();
                    let _ = playbin_clone.set_state(if perc < 100 { GstState::Paused } else { GstState::Playing });
                    state_clone.replace(GtPlayerState::Buffering);
                },
                GstMessageView::StateChanged(state_changed) => {
                    let old_state = state_changed.get_old();
                    let new_state = state_changed.get_current();

                    if let Some(src) = msg.get_src() {
                        if src == *playbin_clone && old_state != new_state{
                            println!("StateChanged, {:#?}", new_state);
                            Self::reconfigure_position_tick(&tick_clone, if new_state <= GstState::Paused { 0 } else { 200 });
                            match new_state {
                                GstState::Paused => {
                                    state_clone.replace(GtPlayerState::Paused);
                                },
                                GstState::Ready | GstState::Null => {
                                    state_clone.replace(GtPlayerState::Stopped);
                                },
                                GstState::Playing => {
                                    state_clone.replace(GtPlayerState::Playing);
                                },
                                _ => {}
                            }
                        }
                    }

                },
                GstMessageView::DurationChanged(_) => {
                    println!("DurationChanged");
                    // TODO: g_object_notify_by_pspec?
                    match playbin_clone.query_duration::<GstClockTime>() {
                        Some(time) => {
                            println!("{:.0}", time);
                            seekable_clone.replace(true);
                            duration_clone.replace(time.seconds().unwrap_or(0));
                        },
                        None => {
                            seekable_clone.replace(false);
                        }
                    }
                },
                GstMessageView::Info(info) => {
                    warning!("Info received from GStreamer {}", info.get_error());
                },
                GstMessageView::Warning(warning) => {
                    warning!("Warning received from GStreamer {}", warning.get_error());
                },
                GstMessageView::Error(error) => {
                    warning!("Error received from GStreamer {}", error.get_error());
                },
                _ => {}
            };

            glib::Continue(true)
        }));

        p!(video_bin.add_many(&[&upload, &video_sink]));
        p!(upload.link(&video_sink));

        let pad = p!(upload.get_static_pad("sink").ok_or("Failed to get static pad 'sink'"));
        let ghost_pad = gst::GhostPad::new(Some("sink"), pad.get_direction());
        p!(ghost_pad.set_active(true));
        p!(video_bin.add_pad(&ghost_pad));

        p!(playbin.set_property("video-sink", &video_bin));
        let widget = p!(p!(video_sink.get_property("widget")).get::<Widget>()).unwrap();


        let inner = Self {
            playbin,
            upload,
            video_sink,
            video_bin,
            bus,
            pad,
            ghost_pad,
            widget,
            uri: None,
            state,
            volume: 0.3,
            buffer_fill: Rc::new(RefCell::new(0f64)),
            seekable,
            duration,
            position: 0,
            position_tick_id: tick
        };

        Ok(inner)

    }

    pub fn query(&mut self) {

        let position = self.playbin
            .query_position::<gst::ClockTime>()
            .unwrap_or_else(|| 0.into());

        println!("{:.0}", position);

    }

    fn reconfigure_position_tick(tick_id: &Rc<RefCell<Option<glib::SourceId>>>, timeout: usize) {

        if let Some(tid) = tick_id.borrow_mut().take() {
            glib::source_remove(tid);
        }

        // if timeout > 0 {
        //     tick_id.replace(glib::timeout_add_local(200, func: F))
        // }

    }

}

impl GtPlayerBackend for BackendGstreamerOpenGL {

    fn play(&mut self) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Playing));
        self.state.replace(GtPlayerState::Loading);
        Ok(())
    }

    fn pause(&mut self) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Paused));
        Ok(())
    }

    fn stop(&mut self) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Null));
        Ok(())
    }

    fn set_uri(&mut self, uri: Option<String>) -> GtResult<()> {
        self.uri = uri;
        p!(self.playbin.set_property("uri", &self.uri));
        Ok(())
    }

    fn set_position(&mut self, position: u64) -> GtResult<()> {
        p!(self.playbin.set_state(GstState::Paused));
        p!(self.playbin.seek_simple(
            GstSeekFlags::FLUSH | GstSeekFlags::KEY_UNIT | GstSeekFlags::from_bits(3).unwrap(), // TODO: GST_FORMAT_TIME??
            GstClockTime::from_mseconds(position) * gst::SECOND
        ));
        Ok(())
    }

    fn get_state(&self) -> GtResult<GtPlayerState> {
        Ok(self.state.borrow().clone())
    }

    fn get_widget(&self) -> GtResult<gtk::Widget> {
        Ok(self.widget.clone())
    }

}
