#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod art;
mod display;
mod flight;
mod pointer;

mod settings {
    include!(concat!(env!("OUT_DIR"), "/lab_settings.rs"));
}

use std::error::Error;
use std::time::{Duration, Instant};

use eframe::egui::{self, ViewportBuilder, ViewportCommand};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};

use crate::{
    display::{Display, connected_displays, select_display},
    flight::Flight,
    pointer::Pointer,
};

const SESSION_LENGTH: Duration = Duration::from_secs(settings::SESSION_SECONDS);
const FRAME_INTERVAL: Duration = Duration::from_millis(16);
const DISPLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

fn main() -> Result<(), Box<dyn Error>> {
    let pointer = Pointer::new()?;
    let viewport = ViewportBuilder::default()
        .with_inner_size([128.0, 96.0])
        .with_decorations(false)
        .with_resizable(false)
        .with_has_shadow(false)
        .with_active(false)
        .with_taskbar(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_mouse_passthrough(true);
    #[cfg(target_os = "linux")]
    let viewport = viewport
        .with_window_type(egui::X11WindowType::Notification)
        .with_override_redirect(true);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    #[cfg(target_os = "macos")]
    let options = eframe::NativeOptions {
        event_loop_builder: Some(Box::new(|builder| {
            use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

            builder
                .with_activation_policy(ActivationPolicy::Accessory)
                .with_activate_ignoring_other_apps(false);
        })),
        ..options
    };
    eframe::run_native(
        "Neon Cat Lab",
        options,
        Box::new(|creation| {
            let window = creation
                .winit_window()
                .ok_or("Desktop window unavailable.")?;
            window.set_cursor_hittest(false)?;
            #[cfg(target_os = "macos")]
            display::enable_separate_spaces(window)?;
            let displays = connected_displays(window);
            let display = select_display(&displays, None, pointer.position())
                .ok_or("Desktop monitor unavailable.")?;
            display.place_overlay(window);
            Ok(Box::new(Overlay::new(pointer, displays, display)?))
        }),
    )?;
    Ok(())
}

struct Overlay {
    pointer: Pointer,
    flight: Flight,
    displays: Vec<Display>,
    current_display: Display,
    displays_checked: Instant,
    started: Instant,
    last_frame: Instant,
    quit: HotKey,
    _hotkeys: GlobalHotKeyManager,
}

impl Overlay {
    fn new(
        pointer: Pointer,
        displays: Vec<Display>,
        current_display: Display,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let hotkeys = GlobalHotKeyManager::new()?;
        let quit = settings::QUIT_SHORTCUT.parse()?;
        hotkeys.register(quit)?;
        let now = Instant::now();
        Ok(Self {
            pointer,
            flight: Flight::new(current_display.local_bounds(), settings::CAT_SPEED),
            displays,
            current_display,
            displays_checked: now,
            started: now,
            last_frame: now,
            quit,
            _hotkeys: hotkeys,
        })
    }
}

impl eframe::App for Overlay {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let now = Instant::now();
        if now.duration_since(self.started) >= SESSION_LENGTH
            || GlobalHotKeyEvent::receiver()
                .try_iter()
                .any(|event| event.id == self.quit.id() && event.state == HotKeyState::Pressed)
        {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        ui.ctx().request_repaint_after(FRAME_INTERVAL);
        let elapsed = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        let Some(window) = frame.winit_window() else {
            return;
        };
        if now.duration_since(self.displays_checked) >= DISPLAY_REFRESH_INTERVAL {
            self.displays = connected_displays(window);
            self.displays_checked = now;
        }

        let pointer = self.pointer.position();
        let Some(display) = select_display(&self.displays, Some(self.current_display), pointer)
        else {
            return;
        };
        let bounds = display.local_bounds();
        if display != self.current_display {
            let previous_position = self.current_display.to_desktop(self.flight.position);
            self.flight
                .enter_display(bounds, display.to_local(previous_position));
            self.current_display = display;
            display.place_overlay(window);
            return;
        }

        self.flight.advance(
            bounds,
            pointer.map(|point| display.to_local(point)),
            elapsed,
        );
        art::draw(ui.painter(), &self.flight);
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}
