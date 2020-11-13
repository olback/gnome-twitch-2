use {
    crate::{p, debug, warning, new_err, error::GtResult, ui::show_info_bar},
    super::{GtPlayerBackend, GtPlayerState, GtPlayerEvent, GtPlayerEventCb},
    std::{rc::Rc, cell::RefCell, os::raw::c_void},
    gtk::{DrawingArea, prelude::*},
    gdk::prelude::*,
    gio::{Settings, SettingsExt, SettingsBindFlags},
    gstv::prelude::*,
    gst::{
        State as GstState,
        Element as GstElement,
        ElementFactory as GstElementFactory
    }
};

pub struct BackendGStreamerVAAPI {
    playbin: Rc<GstElement>,
    state: Rc<RefCell<GtPlayerState>>,
    cb: Rc<GtPlayerEventCb>,
    widget: Rc<DrawingArea>,
    vo: Rc<gstv::VideoOverlay>
}

impl BackendGStreamerVAAPI {

    pub fn new(settings: &Settings, cb: GtPlayerEventCb) -> GtResult<Self> {

        let video_sink = p!(GstElementFactory::make("vaapisink", None));
        let playbin = p!(GstElementFactory::make("playbin", None));
        p!(playbin.set_property("video-sink", &video_sink));

        let widget = DrawingArea::new();
        widget.set_hexpand(true);
        widget.set_hexpand_set(true);
        widget.set_vexpand(true);
        widget.set_vexpand_set(true);

        settings.bind(
            "volume",
            &playbin,
            "volume",
            SettingsBindFlags::DEFAULT
       );

        let video_overlay = Rc::new(video_sink
            .dynamic_cast::<gstv::VideoOverlay>()
            .expect("VideoOverlay dynamic_cast failed"));

        let inner = Self {
            playbin: Rc::new(playbin),
            state: Rc::new(RefCell::new(GtPlayerState::Stopped)),
            cb: Rc::new(cb),
            widget: Rc::new(widget),
            vo: video_overlay
        };

        let bus = p!(inner.playbin.get_bus().ok_or("Could not get playbin bus"));
        p!(bus.add_watch_local(glib::clone!(
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

    fn set_window_handle(&self) -> GtResult<()> {

        let toplevel = self.widget.get_toplevel().expect("Could not get toplevel");
        let (x, y) = self.widget.translate_coordinates(&toplevel, 0, 0)
            .expect("Could not get coordinates of widget");
        let (width, height) = {
            let alloc = self.widget.get_allocation();
            (alloc.width, alloc.height)
        };

        debug!("(x: {}, y: {})", x, y);
        debug!("(w: {}, h: {})", width, height);

        let gdk_window = self.widget
            .get_toplevel()
            .expect("Could not get toplevel")
            .get_window()
            .expect("Window is None");

        if !gdk_window.ensure_native() {
            warning!("Can't create native window for widget");
            show_info_bar(
                "Internal error",
                "Can't create native window for widget",
                None::<&gtk::Widget>,
                gtk::MessageType::Error
            );
        }

        let display_type_name = gdk_window.get_display().get_type().name();

        // Check if we're using X11 or ...
        if display_type_name == "GdkX11Display" {

            extern "C" {
                pub fn gdk_x11_window_get_xid(window: *mut glib::object::GObject) -> *mut c_void;
            }

            #[allow(clippy::cast_ptr_alignment)]
            unsafe {
                let xid = gdk_x11_window_get_xid(gdk_window.as_ptr() as *mut _);
                let xid_u = xid as usize;
                debug!(
                    "xid_le: 0x{:0x} xid_be: 0x{:0x}",
                    xid_u.to_le(),
                    xid_u.to_be()
                );
                self.vo.set_window_handle(xid_u);
            }

            self.vo.set_render_rectangle(x, y, width, height)
                .expect("Could not set render rectangle");

            Ok(())

        } else {

            Err(new_err!(format!("Display type {} not supported", display_type_name)))

        }

    }

}

impl GtPlayerBackend for BackendGStreamerVAAPI {

    fn play(&self) -> GtResult<()> {
        self.set_window_handle()?;
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
        &self.widget.upcast_ref::<gtk::Widget>()
    }

}
