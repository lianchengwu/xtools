use std::os::unix::net::UnixListener;
use std::time::Duration;

use slint::ComponentHandle;
use xtools_ui::slint_chrome::{
    WindowDragState, copy_to_clipboard, setup_auto_exit_on_focus_loss_timer, setup_raise_timer,
    setup_skip_taskbar_timer,
};

use crate::convert;

slint::include_modules!();

pub struct TimeApp {
    ui: TimeWindow,
    _lock: UnixListener,
    _raise_timer: slint::Timer,
    _skip_timer: Option<slint::Timer>,
    _focus_loss_timer: Option<slint::Timer>,
}

impl TimeApp {
    pub fn new(lock: UnixListener) -> Result<Self, slint::PlatformError> {
        let ui = TimeWindow::new()?;
        let (s, ms, local) = convert::from_now();

        ui.set_seconds(s.to_string().into());
        ui.set_millis(ms.to_string().into());
        ui.set_local(local.into());
        ui.set_sec_ok(true);
        ui.set_ms_ok(true);

        let drag_state = WindowDragState::new();

        // Window drag callbacks
        {
            let drag = drag_state.clone();
            let ui_weak = ui.as_weak();
            ui.on_window_drag_started(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    drag.on_drag_started(ui.window());
                }
            });
        }
        {
            let drag = drag_state;
            let ui_weak = ui.as_weak();
            ui.on_window_dragged(move |dx, dy| {
                if let Some(ui) = ui_weak.upgrade() {
                    drag.on_dragged(ui.window(), dx, dy);
                }
            });
        }

        // Close callback
        {
            let ui_weak = ui.as_weak();
            ui.on_close_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let _ = ui.hide();
                }
                std::process::exit(0);
            });
        }

        // Seconds edited
        {
            let ui_weak = ui.as_weak();
            ui.on_seconds_edited(move |val| {
                if let Some(ui) = ui_weak.upgrade() {
                    let trimmed = val.trim();
                    if trimmed.is_empty() {
                        ui.set_error_seconds(
                            "这一栏是空的\n输入一个值，或点「现在」填入当前时间。".into(),
                        );
                        ui.set_sec_ok(false);
                        return;
                    }
                    match trimmed
                        .parse::<i64>()
                        .ok()
                        .and_then(|n| convert::from_seconds(n).ok())
                    {
                        Some((s, ms, local)) => {
                            ui.set_seconds(s.to_string().into());
                            ui.set_millis(ms.to_string().into());
                            ui.set_local(local.into());
                            ui.set_sec_ok(true);
                            ui.set_ms_ok(true);
                            ui.set_error_seconds("".into());
                            ui.set_error_millis("".into());
                            ui.set_error_local("".into());
                        }
                        None => {
                            ui.set_error_seconds("秒数无效。输入 Unix 秒，或点「现在」。".into());
                            ui.set_sec_ok(false);
                        }
                    }
                }
            });
        }

        // Millis edited
        {
            let ui_weak = ui.as_weak();
            ui.on_millis_edited(move |val| {
                if let Some(ui) = ui_weak.upgrade() {
                    let trimmed = val.trim();
                    if trimmed.is_empty() {
                        ui.set_error_millis(
                            "这一栏是空的\n输入一个值，或点「现在」填入当前时间。".into(),
                        );
                        ui.set_ms_ok(false);
                        return;
                    }
                    match trimmed
                        .parse::<i64>()
                        .ok()
                        .and_then(|n| convert::from_millis(n).ok())
                    {
                        Some((s, ms, local)) => {
                            ui.set_seconds(s.to_string().into());
                            ui.set_millis(ms.to_string().into());
                            ui.set_local(local.into());
                            ui.set_sec_ok(true);
                            ui.set_ms_ok(true);
                            ui.set_error_seconds("".into());
                            ui.set_error_millis("".into());
                            ui.set_error_local("".into());
                        }
                        None => {
                            ui.set_error_millis("毫秒无效。输入 Unix 毫秒，或点「现在」。".into());
                            ui.set_ms_ok(false);
                        }
                    }
                }
            });
        }

        // Local edited
        {
            let ui_weak = ui.as_weak();
            ui.on_local_edited(move |val| {
                if let Some(ui) = ui_weak.upgrade() {
                    let trimmed = val.trim();
                    if trimmed.is_empty() {
                        ui.set_error_local(
                            "这一栏是空的\n输入一个值，或点「现在」填入当前时间。".into(),
                        );
                        return;
                    }
                    match convert::from_local(trimmed) {
                        Ok((s, ms, local)) => {
                            ui.set_seconds(s.to_string().into());
                            ui.set_millis(ms.to_string().into());
                            ui.set_local(local.into());
                            ui.set_sec_ok(true);
                            ui.set_ms_ok(true);
                            ui.set_error_seconds("".into());
                            ui.set_error_millis("".into());
                            ui.set_error_local("".into());
                        }
                        Err(_) => {
                            ui.set_error_local(
                                "本地时间无效。按 年-月-日 时:分:秒 填写，或点「现在」。".into(),
                            );
                        }
                    }
                }
            });
        }

        // Now clicked
        {
            let ui_weak = ui.as_weak();
            ui.on_now_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let (s, ms, local) = convert::from_now();
                    ui.set_seconds(s.to_string().into());
                    ui.set_millis(ms.to_string().into());
                    ui.set_local(local.into());
                    ui.set_sec_ok(true);
                    ui.set_ms_ok(true);
                    ui.set_error_seconds("".into());
                    ui.set_error_millis("".into());
                    ui.set_error_local("".into());
                }
            });
        }

        // Copy seconds
        {
            let ui_weak = ui.as_weak();
            ui.on_copy_seconds(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_seconds().to_string();
                    let trimmed = text.trim();
                    if !trimmed.is_empty() && trimmed.parse::<i64>().is_ok() {
                        copy_to_clipboard(trimmed);
                        ui.set_sec_copied(true);
                        let ui_reset = ui_weak.clone();
                        slint::Timer::single_shot(Duration::from_secs(1), move || {
                            if let Some(ui) = ui_reset.upgrade() {
                                ui.set_sec_copied(false);
                            }
                        });
                    }
                }
            });
        }

        // Copy millis
        {
            let ui_weak = ui.as_weak();
            ui.on_copy_millis(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_millis().to_string();
                    let trimmed = text.trim();
                    if !trimmed.is_empty() && trimmed.parse::<i64>().is_ok() {
                        copy_to_clipboard(trimmed);
                        ui.set_ms_copied(true);
                        let ui_reset = ui_weak.clone();
                        slint::Timer::single_shot(Duration::from_secs(1), move || {
                            if let Some(ui) = ui_reset.upgrade() {
                                ui.set_ms_copied(false);
                            }
                        });
                    }
                }
            });
        }

        let raise_timer = setup_raise_timer(lock.try_clone().unwrap(), ui.as_weak());
        let skip_timer = setup_skip_taskbar_timer();
        let focus_timer = setup_auto_exit_on_focus_loss_timer();

        Ok(Self {
            ui,
            _lock: lock,
            _raise_timer: raise_timer,
            _skip_timer: Some(skip_timer),
            _focus_loss_timer: Some(focus_timer),
        })
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.ui.run()
    }
}
