use eframe::egui::{Pos2, pos2};

#[cfg(target_os = "linux")]
pub(crate) struct Pointer {
    connection: x11rb::rust_connection::RustConnection,
    root: u32,
}

#[cfg(target_os = "linux")]
impl Pointer {
    pub(crate) fn new() -> Result<Self, String> {
        use x11rb::{connection::Connection, protocol::xproto::ConnectionExt};

        let (connection, screen) =
            x11rb::connect(None).map_err(|error| format!("X11 display unavailable: {error}"))?;
        let supports_input_shape = connection
            .query_extension(b"SHAPE")
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .is_some_and(|extension| extension.present);
        if !supports_input_shape {
            return Err("This X11 display does not support click-through windows.".to_owned());
        }
        let compositor = format!("_NET_WM_CM_S{screen}");
        let compositor_owner = connection
            .intern_atom(true, compositor.as_bytes())
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .filter(|atom| atom.atom != x11rb::NONE)
            .and_then(|atom| connection.get_selection_owner(atom.atom).ok())
            .and_then(|cookie| cookie.reply().ok());
        if !compositor_owner.is_some_and(|owner| owner.owner != x11rb::NONE) {
            return Err("A composited X11 desktop is required for transparency.".to_owned());
        }
        let root = connection.setup().roots[screen].root;
        Ok(Self { connection, root })
    }

    pub(crate) fn position(&self) -> Option<Pos2> {
        use x11rb::protocol::xproto::ConnectionExt;
        let position = self
            .connection
            .query_pointer(self.root)
            .ok()?
            .reply()
            .ok()?;
        position
            .same_screen
            .then(|| pos2(position.root_x.into(), position.root_y.into()))
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) struct Pointer;

#[cfg(any(target_os = "macos", target_os = "windows"))]
impl Pointer {
    pub(crate) fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

#[cfg(target_os = "macos")]
impl Pointer {
    pub(crate) fn position(&self) -> Option<Pos2> {
        use core_graphics::event::CGEvent;
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let point = CGEvent::new(source).ok()?.location();
        Some(pos2(point.x as f32, point.y as f32))
    }
}

#[cfg(target_os = "windows")]
impl Pointer {
    pub(crate) fn position(&self) -> Option<Pos2> {
        use windows_sys::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

        let mut position = POINT { x: 0, y: 0 };
        // GetCursorPos writes to a valid POINT for the duration of the call.
        let success = unsafe { GetCursorPos(&mut position) } != 0;
        success.then(|| pos2(position.x as f32, position.y as f32))
    }
}
