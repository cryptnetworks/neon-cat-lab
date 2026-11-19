use eframe::egui::{Pos2, Rect, pos2, vec2};
use winit::{monitor::MonitorHandle, window::Window};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Display {
    desktop: Rect,
    units_per_point: f32,
}

impl Display {
    #[cfg(target_os = "macos")]
    fn from_monitor(monitor: &MonitorHandle) -> Self {
        use core_graphics::display::CGDisplay;
        use winit::platform::macos::MonitorHandleExtMacOS;

        // Quartz reports both displays and cursor positions in desktop points.
        let bounds = CGDisplay::new(monitor.native_id()).bounds();
        Self {
            desktop: Rect::from_min_size(
                pos2(bounds.origin.x as f32, bounds.origin.y as f32),
                vec2(bounds.size.width as f32, bounds.size.height as f32),
            ),
            units_per_point: 1.0,
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn from_monitor(monitor: &MonitorHandle) -> Self {
        // Win32 and X11 pointer coordinates are in desktop pixels.
        let origin = monitor.position();
        let size = monitor.size();
        Self {
            desktop: Rect::from_min_size(
                pos2(origin.x as f32, origin.y as f32),
                vec2(size.width as f32, size.height as f32),
            ),
            units_per_point: monitor.scale_factor() as f32,
        }
    }

    pub(crate) fn local_bounds(self) -> Rect {
        Rect::from_min_size(Pos2::ZERO, self.desktop.size() / self.units_per_point)
    }

    pub(crate) fn to_local(self, point: Pos2) -> Pos2 {
        Pos2::ZERO + (point - self.desktop.min) / self.units_per_point
    }

    pub(crate) fn to_desktop(self, point: Pos2) -> Pos2 {
        self.desktop.min + point.to_vec2() * self.units_per_point
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn place_overlay(self, window: &Window) {
        use winit::dpi::{LogicalPosition, LogicalSize};

        let size = self.desktop.size();
        let _ = window.request_inner_size(LogicalSize::new(size.x, size.y));
        window.set_outer_position(LogicalPosition::new(self.desktop.min.x, self.desktop.min.y));
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn place_overlay(self, window: &Window) {
        use winit::dpi::{PhysicalPosition, PhysicalSize};

        window.set_outer_position(PhysicalPosition::new(
            self.desktop.min.x as i32,
            self.desktop.min.y as i32,
        ));
        let size = self.desktop.size();
        let _ = window.request_inner_size(PhysicalSize::new(size.x as u32, size.y as u32));
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn enable_separate_spaces(
    window: &Window,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use objc2_app_kit::{NSView, NSWindowCollectionBehavior};
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle = window.window_handle()?;
    let RawWindowHandle::AppKit(app_kit) = handle.as_raw() else {
        return Err("AppKit window handle unavailable.".into());
    };
    // Winit's handle points to an NSView owned by this live window on the main thread.
    let view = unsafe { app_kit.ns_view.cast::<NSView>().as_ref() };
    let window = view.window().ok_or("AppKit window unavailable.")?;
    let mut behavior = window.collectionBehavior();
    behavior.remove(NSWindowCollectionBehavior::MoveToActiveSpace);
    behavior.insert(NSWindowCollectionBehavior::CanJoinAllSpaces);
    window.setCollectionBehavior(behavior);
    Ok(())
}

pub(crate) fn connected_displays(window: &Window) -> Vec<Display> {
    let primary = window.primary_monitor();
    primary
        .iter()
        .cloned()
        .chain(
            window
                .available_monitors()
                .filter(|monitor| Some(monitor) != primary.as_ref()),
        )
        .map(|monitor| Display::from_monitor(&monitor))
        .filter(|display| display.desktop.is_positive())
        .collect()
}

pub(crate) fn select_display(
    displays: &[Display],
    current: Option<Display>,
    pointer: Option<Pos2>,
) -> Option<Display> {
    pointer
        .and_then(|point| {
            displays.iter().copied().find(|display| {
                display.desktop.contains(point)
                    && point.x < display.desktop.max.x
                    && point.y < display.desktop.max.y
            })
        })
        .or_else(|| current.filter(|display| displays.contains(display)))
        .or_else(|| displays.first().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY: Display = Display {
        desktop: Rect::from_min_max(pos2(0.0, 0.0), pos2(1920.0, 1080.0)),
        units_per_point: 1.0,
    };
    const LEFT: Display = Display {
        desktop: Rect::from_min_max(pos2(-2560.0, -300.0), pos2(0.0, 1140.0)),
        units_per_point: 2.0,
    };

    #[test]
    fn selects_the_monitor_at_a_negative_origin_and_converts_mixed_dpi() {
        let pointer = pos2(-1280.0, 420.0);
        let selected = select_display(&[PRIMARY, LEFT], Some(PRIMARY), Some(pointer));
        assert_eq!(selected, Some(LEFT));
        assert_eq!(LEFT.local_bounds().size(), vec2(1280.0, 720.0));
        assert_eq!(LEFT.to_local(pointer), pos2(640.0, 360.0));
        assert_eq!(LEFT.to_desktop(LEFT.to_local(pointer)), pointer);
    }

    #[test]
    fn assigns_a_shared_edge_once_and_falls_back_when_a_monitor_disconnects() {
        let shared_edge = pos2(0.0, 500.0);
        assert_eq!(
            select_display(&[LEFT, PRIMARY], Some(LEFT), Some(shared_edge)),
            Some(PRIMARY),
        );
        let gap = pos2(600.0, -200.0);
        assert_eq!(
            select_display(&[PRIMARY, LEFT], Some(LEFT), Some(gap)),
            Some(LEFT)
        );
        assert_eq!(
            select_display(&[PRIMARY], Some(LEFT), Some(gap)),
            Some(PRIMARY)
        );
    }
}
