use crate::error::GtResult;

#[cfg(feature = "backend-gstreamer")]
pub mod gstreamer;

#[cfg(feature = "backend-gstreamer-opengl")]
pub mod gstreamer_opengl;

#[cfg(not(any(
    feature = "backend-gstreamer",
    feature = "backend-gstreamer-opengl"
)))]
compile_error!("At least one backend must be enabled");

pub const BACKENDS: &'static [(&'static str, &'static str, fn(volume_button: &gio::Settings) -> GtResult<Box<dyn GtPlayerBackend>>)] = &[
    #[cfg(feature = "backend-gstreamer")]
    ("GStreamer", "gstreamer", gstreamer::BackendGStreamer::boxed),
    #[cfg(feature = "backend-gstreamer-opengl")]
    ("GStreamer OpenGL (Broken)", "gstreamer-opengl", gstreamer_opengl::BackendGStreamerOpenGL::boxed),
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
    Loading
}

pub trait GtPlayerBackend {
    fn play(&self) -> GtResult<()>;
    fn pause(&self) -> GtResult<()>;
    fn stop(&self) -> GtResult<()>;
    fn set_uri(&self, uri: Option<String>) -> GtResult<()>;
    fn get_state(&self) -> GtPlayerState;
    fn get_widget(&self) -> &gtk::Widget;
}
