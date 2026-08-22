mod app;
mod convert;
mod skip_taskbar;

use std::env;

use eframe::egui::{ViewportBuilder, X11WindowType};
use xtools_ui::chrome::install_fonts;
use xtools_ui::{TIME_INSTANCE, claim_instance, raise_instance};

use crate::app::TimeApp;

fn take_activation_token() -> Option<String> {
    let token = env::var("XDG_ACTIVATION_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    unsafe {
        env::remove_var("XDG_ACTIVATION_TOKEN");
    }
    token
}

/// Utility + skip-taskbar only apply on X11. This session is Wayland; XWayland is up.
fn prefer_x11_for_skip_taskbar() {
    if env::var_os("DISPLAY").is_some() {
        unsafe {
            env::remove_var("WAYLAND_DISPLAY");
        }
    }
}

fn main() -> eframe::Result {
    prefer_x11_for_skip_taskbar();
    let token = take_activation_token();
    match claim_instance(TIME_INSTANCE) {
        Ok(None) => {
            let _ = raise_instance(TIME_INSTANCE, token.as_deref());
            return Ok(());
        }
        Ok(Some(lock)) => {
            let viewport = ViewportBuilder::default()
                .with_title("xtools · 时间戳")
                .with_inner_size([440.0, 400.0])
                .with_min_inner_size([400.0, 340.0])
                .with_decorations(false)
                .with_transparent(true)
                .with_app_id("dev.xtools.timestamp")
                .with_window_type(X11WindowType::Utility);
            let options = eframe::NativeOptions {
                viewport,
                persist_window: false,
                centered: true,
                ..Default::default()
            };
            eframe::run_native(
                "dev.xtools.timestamp",
                options,
                Box::new(move |cc| {
                    install_fonts(&cc.egui_ctx);
                    Ok(Box::new(TimeApp::new(lock)))
                }),
            )
        }
        Err(err) => {
            eprintln!("xtools-time: instance lock failed: {err}");
            std::process::exit(1);
        }
    }
}
