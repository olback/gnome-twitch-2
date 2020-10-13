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
    fn play(&mut self) -> GtResult<()>;
    fn pause(&mut self) -> GtResult<()>;
    fn stop(&mut self) -> GtResult<()>;
    fn set_uri(&mut self, uri: Option<String>) -> GtResult<()>;
    fn set_position(&mut self, position: u64) -> GtResult<()>;
    fn get_state(&self) -> GtResult<GtPlayerState>;
    fn get_widget(&self) -> GtResult<gtk::Widget>;
}
