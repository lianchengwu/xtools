use std::env;

/// Read then drop `XDG_ACTIVATION_TOKEN` so a grandchild cannot reuse it.
pub fn take_activation_token() -> Option<String> {
    let token = env::var("XDG_ACTIVATION_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    unsafe {
        env::remove_var("XDG_ACTIVATION_TOKEN");
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
