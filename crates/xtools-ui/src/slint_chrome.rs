//! Shared Slint helpers, theme, and clipboard.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;

pub const THEME_BG: &str = "#F5F5F7";
pub const THEME_FG: &str = "#1C1E26";
pub const THEME_HAIRLINE: &str = "#D8D8DF";
pub const THEME_MUTED: &str = "#6E6E78";
pub const THEME_DESTRUCTIVE: &str = "#DC2626";

/// Copy text to system clipboard.
pub fn copy_to_clipboard(text: &str) {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text);
    }
}

/// Helper for smooth window dragging on undecorated Slint windows.
#[derive(Clone, Default)]
pub struct WindowDragState {
    start_pos: Arc<Mutex<Option<slint::PhysicalPosition>>>,
}

impl WindowDragState {
    pub fn new() -> Self {
        Self {
            start_pos: Arc::new(Mutex::new(None)),
        }
    }

    pub fn on_drag_started(&self, window: &slint::Window) {
        let pos = window.position();
        *self.start_pos.lock() = Some(pos);
    }

    pub fn on_dragged(&self, window: &slint::Window, dx: f32, dy: f32) {
        if let Some(base_pos) = *self.start_pos.lock() {
            let new_x = base_pos.x + dx.round() as i32;
            let new_y = base_pos.y + dy.round() as i32;
            window.set_position(slint::PhysicalPosition::new(new_x, new_y));
        }
    }
}

/// Helper for resizing undecorated Slint windows.
#[derive(Clone, Default)]
pub struct WindowResizeState {
    start_size: Arc<Mutex<Option<slint::PhysicalSize>>>,
    saved_size: Arc<Mutex<Option<slint::PhysicalSize>>>,
}

impl WindowResizeState {
    pub fn new() -> Self {
        Self {
            start_size: Arc::new(Mutex::new(None)),
            saved_size: Arc::new(Mutex::new(None)),
        }
    }

    pub fn on_resize_started(&self, window: &slint::Window) {
        let size = window.size();
        *self.start_size.lock() = Some(size);
    }

    pub fn on_resized(&self, window: &slint::Window, dx: f32, dy: f32, min_w: u32, min_h: u32) {
        if let Some(base_size) = *self.start_size.lock() {
            let scale = window.scale_factor();
            let min_phys_w = (min_w as f32 * scale).round() as u32;
            let min_phys_h = (min_h as f32 * scale).round() as u32;
            let raw_w = base_size.width as f32 + dx * scale;
            let raw_h = base_size.height as f32 + dy * scale;
            let new_w = (raw_w.round() as u32).max(min_phys_w);
            let new_h = (raw_h.round() as u32).max(min_phys_h);
            window.set_size(slint::PhysicalSize::new(new_w, new_h));
        }
    }

    pub fn toggle_expand(
        &self,
        window: &slint::Window,
        normal_w: u32,
        normal_h: u32,
        expanded_w: u32,
        expanded_h: u32,
    ) -> bool {
        let current_size = window.size();
        let scale = window.scale_factor();
        let normal_phys_w = (normal_w as f32 * scale).round() as u32;
        let normal_phys_h = (normal_h as f32 * scale).round() as u32;
        let expanded_phys_w = (expanded_w as f32 * scale).round() as u32;
        let expanded_phys_h = (expanded_h as f32 * scale).round() as u32;

        let mut saved = self.saved_size.lock();
        let tolerance = (20.0 * scale).round() as u32;
        if current_size.width >= (expanded_phys_w.saturating_sub(tolerance))
            && current_size.height >= (expanded_phys_h.saturating_sub(tolerance))
        {
            let target = saved
                .take()
                .unwrap_or(slint::PhysicalSize::new(normal_phys_w, normal_phys_h));
            window.set_size(target);
            false
        } else {
            *saved = Some(current_size);
            window.set_size(slint::PhysicalSize::new(expanded_phys_w, expanded_phys_h));
            true
        }
    }
}

/// Start a timer that polls the instance lock and handles raise or quit commands.
pub fn setup_raise_timer(
    listener: crate::InstanceListener,
    window: slint::Weak<impl slint::ComponentHandle + 'static>,
) -> slint::Timer {
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(50),
        move || match crate::instance::accept_command(&listener) {
            Some(crate::instance::InstanceCommand::Quit) => {
                std::process::exit(0);
            }
            Some(crate::instance::InstanceCommand::Raise(_token)) => {
                if let Some(ui) = window.upgrade() {
                    let _ = ui.window().show();
                    #[cfg(feature = "x11-skip-taskbar")]
                    crate::skip_taskbar::raise_x11_window();
                }
            }
            None => {}
        },
    );
    timer
}

/// Start a timer that applies X11 skip taskbar for the first few ticks.
#[cfg(feature = "x11-skip-taskbar")]
pub fn setup_skip_taskbar_timer() -> slint::Timer {
    let timer = slint::Timer::default();
    let tries = Arc::new(Mutex::new(0u8));
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(80),
        move || {
            let mut count = tries.lock();
            if *count < 12 {
                crate::skip_taskbar::apply();
                *count += 1;
            }
        },
    );
    timer
}

/// Start a timer that automatically exits the process when window focus is lost.
#[cfg(feature = "x11-skip-taskbar")]
pub fn setup_auto_exit_on_focus_loss_timer() -> slint::Timer {
    let timer = slint::Timer::default();
    let was_active = Arc::new(Mutex::new(false));
    let initial_ticks = Arc::new(Mutex::new(0u8));
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(60),
        move || {
            let res = crate::skip_taskbar::is_process_window_active();
            let was = *was_active.lock();
            let mut ticks = initial_ticks.lock();
            if *ticks < 4 {
                *ticks += 1;
                if let Some(true) = res {
                    *was_active.lock() = true;
                }
                return;
            }

            match res {
                Some(true) => {
                    *was_active.lock() = true;
                }
                Some(false) if was => {
                    std::process::exit(0);
                }
                _ => {}
            }
        },
    );
    timer
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_drag_and_focus() {
        slint::slint! {
            export component TestWindow inherits Window {
                width: 100px;
                height: 100px;
                callback focus-lost();
                forward-focus: fs;
                fs := FocusScope {
                    changed has-focus => {
                        if (!self.has-focus) {
                            root.focus-lost();
                        }
                    }
                    Text { text: "Hello"; }
                }
            }
        }
        let Ok(Ok(win)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(TestWindow::new))
        else {
            // Headless CI without display server or missing xkbcommon libs: skip GUI window test
            return;
        };
        let drag = WindowDragState::new();
        drag.on_drag_started(win.window());
        drag.on_dragged(win.window(), 10.0, 20.0);

        let resize = WindowResizeState::new();
        resize.on_resize_started(win.window());
        resize.on_resized(win.window(), 50.0, 60.0, 50, 50);
        let exp = resize.toggle_expand(win.window(), 100, 100, 200, 200);
        assert!(exp);
        let restored = resize.toggle_expand(win.window(), 100, 100, 200, 200);
        assert!(!restored);
    }
}
