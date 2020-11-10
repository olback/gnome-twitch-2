use {
    crate::error::GtResult,
    std::cell::RefCell,
    gst::{
        Message as GstMessage,
        MessageView as GstMessageView,
        State as GstState,
        prelude::*
    }
};

#[cfg(feature = "backend-gstreamer")]
pub mod gstreamer;

#[cfg(feature = "backend-gstreamer-opengl")]
pub mod gstreamer_opengl;

#[cfg(feature = "backend-gstreamer-vaapi")]
pub mod gstreamer_vaapi;

#[cfg(not(any(
    feature = "backend-gstreamer",
    feature = "backend-gstreamer-opengl",
    feature = "backend-gstreamer-vaapi"
)))]
compile_error!("At least one backend must be enabled");

pub type GtPlayerEventCb = Box<dyn Fn(GtPlayerEvent)>;

// The order here is important, the first one enabled is the default.
// Default to GPU accelerated rendering.
pub const BACKENDS: &'static [(&'static str, &'static str, fn(settings: &gio::Settings, cb: GtPlayerEventCb) -> GtResult<Box<dyn GtPlayerBackend>>)] = &[
    #[cfg(feature = "backend-gstreamer-vaapi")]
    ("GStreamer VAAPI", "gstreamer-vaapi", gstreamer_vaapi::BackendGStreamerVAAPI::boxed),
    #[cfg(feature = "backend-gstreamer-opengl")]
    ("GStreamer OpenGL", "gstreamer-opengl", gstreamer_opengl::BackendGStreamerOpenGL::boxed),
    #[cfg(feature = "backend-gstreamer")]
    ("GStreamer", "gstreamer", gstreamer::BackendGStreamer::boxed),
];

pub fn backend_id_valid(backend_id: &str) -> bool {

    for (_, id, _) in BACKENDS {
        if backend_id == *id {
            return true
        }
    }

    false

}

#[derive(Debug, Clone)]
pub enum GtPlayerState {
    Playing,
    Paused,
    Stopped,
    Buffering,
    Loading,
    Eos
}

#[derive(Debug, Clone)]
pub enum GtPlayerEvent {
    StateChange(GtPlayerState),
    Warning(String),
    Error(String)
}

pub trait GtPlayerBackend {
    fn play(&self) -> GtResult<()>;
    fn pause(&self) -> GtResult<()>;
    fn stop(&self) -> GtResult<()>;
    fn set_uri(&self, uri: Option<String>) -> GtResult<()>;
    fn get_state(&self) -> GtPlayerState;
    fn get_widget(&self) -> &gtk::Widget;
}

pub(super) fn bus_event_handler(msg: &GstMessage, playbin: &gst::Element, state: &RefCell<GtPlayerState>, cb: &GtPlayerEventCb) {

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
        cb(e)
    }

}
