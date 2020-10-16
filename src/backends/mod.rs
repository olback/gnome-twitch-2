use {
    crate::error::GtResult
};

mod gstreamer_opengl;
pub use gstreamer_opengl::BackendGstreamerOpenGL;

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
    fn set_position(&self, position: u64) -> GtResult<()>;
    fn get_state(&self) -> GtResult<GtPlayerState>;
    fn get_widget(&self) -> GtResult<gtk::Widget>;
}
