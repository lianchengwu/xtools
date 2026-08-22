mod app;
mod convert;

use std::env;

use eframe::egui::{self, ViewportBuilder, X11WindowType};
use xtools_ui::{claim_instance, raise_instance, TIME_INSTANCE};

use crate::app::TimeApp;

fn take_activation_token() -> Option<String> {
    let token = env::var("XDG_ACTIVATION_TOKEN").ok().filter(|s| !s.is_empty());
    unsafe {
        env::remove_var("XDG_ACTIVATION_TOKEN");
    }
    token
}

fn main() -> eframe::Result {
    let token = take_activation_token();
    match claim_instance(TIME_INSTANCE) {
        Ok(None) => {
            let _ = raise_instance(TIME_INSTANCE, token.as_deref());
            return Ok(());
        }
        Ok(Some(lock)) => {
            let viewport = ViewportBuilder::default()
                .with_title("xtools · 时间戳")
                .with_inner_size([560.0, 480.0])
                .with_min_inner_size([480.0, 360.0])
                .with_decorations(true)
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
                Box::new(move |_cc| Ok(Box::new(TimeApp::new(lock)))),
            )
        }
        Err(err) => {
            eprintln!("xtools-time: instance lock failed: {err}");
            std::process::exit(1);
        }
    }
}
