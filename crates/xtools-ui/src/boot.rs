use std::env;
use std::sync::OnceLock;

static TARGET_DESKTOP: OnceLock<Option<String>> = OnceLock::new();

/// Snapshot the active virtual desktop before any tool window maps.
/// Prefers `XTOOLS_TARGET_DESKTOP` from the host click, else queries KWin.
pub fn capture_target_desktop() {
    TARGET_DESKTOP.get_or_init(|| {
        env::var("XTOOLS_TARGET_DESKTOP")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(crate::kwin::current_desktop)
    });
}

/// Desktop UUID captured by [`capture_target_desktop`].
pub fn target_desktop() -> Option<String> {
    TARGET_DESKTOP.get().cloned().flatten()
}

/// Read then drop `XDG_ACTIVATION_TOKEN` so a grandchild cannot reuse it.
pub fn take_activation_token() -> Option<String> {
    let token = env::var("XDG_ACTIVATION_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    unsafe {
        env::remove_var("XDG_ACTIVATION_TOKEN");
        env::remove_var("DESKTOP_STARTUP_ID");
    }
    token
}

/// Utility + skip-taskbar only apply on X11. This session is Wayland; XWayland is up.
pub fn prefer_x11_for_skip_taskbar() {
    if env::var_os("DISPLAY").is_some() {
        unsafe {
            env::remove_var("WAYLAND_DISPLAY");
        }
    }
}
