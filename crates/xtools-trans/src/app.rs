use std::thread;
use std::time::Duration;

use slint::ComponentHandle;
use xtools_ui::slint_chrome::{
    WindowDragState, copy_to_clipboard, setup_auto_exit_on_focus_loss_timer, setup_raise_timer,
    setup_skip_taskbar_timer,
};

use crate::engine::{MyMemoryEngine, TranslateEngine};

slint::include_modules!();

const SOURCE: &[(&str, &str)] = &[
    ("auto", "自动"),
    ("zh-CN", "中文"),
    ("en", "英语"),
    ("ja", "日语"),
    ("ko", "韩语"),
    ("fr", "法语"),
    ("de", "德语"),
    ("es", "西班牙语"),
    ("ru", "俄语"),
];

const TARGET: &[(&str, &str)] = &[
    ("zh-CN", "中文"),
    ("en", "英语"),
    ("ja", "日语"),
    ("ko", "韩语"),
    ("fr", "法语"),
    ("de", "德语"),
    ("es", "西班牙语"),
    ("ru", "俄语"),
];

pub struct TransApp {
    ui: TransWindow,
    _lock: xtools_ui::InstanceListener,
    _raise_timer: slint::Timer,
    _skip_timer: Option<slint::Timer>,
    _focus_loss_timer: Option<slint::Timer>,
}

impl TransApp {
    pub fn new(lock: xtools_ui::InstanceListener) -> Result<Self, slint::PlatformError> {
        let ui = TransWindow::new()?;
        ui.set_status_text(format!("引擎：{}", MyMemoryEngine.name()).into());
        ui.set_can_translate(false);
        ui.set_can_copy(false);

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

        // Input edited
        {
            let ui_weak = ui.as_weak();
            ui.on_input_edited(move |val| {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = val.to_string();
                    let trimmed = text.trim();
                    ui.set_can_translate(!trimmed.is_empty());
                }
            });
        }

        // Swap languages
        {
            let ui_weak = ui.as_weak();
            ui.on_swap_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let (new_src, new_dst, new_input, new_output) = swap_state(
                        ui.get_src_index() as usize,
                        ui.get_dst_index() as usize,
                        ui.get_input_text().to_string(),
                        ui.get_output_text().to_string(),
                    );
                    ui.set_src_index(new_src as i32);
                    ui.set_dst_index(new_dst as i32);
                    ui.set_input_text(new_input.into());
                    ui.set_output_text(new_output.into());
                    ui.set_can_translate(!ui.get_input_text().trim().is_empty());
                    ui.set_can_copy(!ui.get_output_text().trim().is_empty());
                }
            });
        }

        // Translate clicked
        {
            let ui_weak = ui.as_weak();
            ui.on_translate_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    if ui.get_pending() {
                        return;
                    }
                    let text = ui.get_input_text().to_string();
                    if text.trim().is_empty() {
                        ui.set_error_text("先输入要翻译的文字。".into());
                        return;
                    }
                    let src_idx = (ui.get_src_index() as usize).min(SOURCE.len() - 1);
                    let dst_idx = (ui.get_dst_index() as usize).min(TARGET.len() - 1);
                    let src = SOURCE[src_idx].0.to_string();
                    let dst = TARGET[dst_idx].0.to_string();

                    ui.set_pending(true);
                    ui.set_error_text("".into());

                    let ui_handle = ui_weak.clone();
                    thread::spawn(move || {
                        let engine = MyMemoryEngine;
                        let result = engine
                            .translate(&text, &src, &dst)
                            .map_err(|err| err.to_string());

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle.upgrade() {
                                ui.set_pending(false);
                                match result {
                                    Ok(translated) => {
                                        ui.set_output_text(translated.into());
                                        ui.set_error_text("".into());
                                        ui.set_can_copy(!ui.get_output_text().trim().is_empty());
                                    }
                                    Err(err) => {
                                        ui.set_error_text(err.into());
                                    }
                                }
                            }
                        });
                    });
                }
            });
        }

        // Copy clicked
        {
            let ui_weak = ui.as_weak();
            ui.on_copy_clicked(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_output_text().to_string();
                    if !text.trim().is_empty() {
                        copy_to_clipboard(&text);
                        ui.set_copied(true);
                        let ui_reset = ui_weak.clone();
                        slint::Timer::single_shot(Duration::from_secs(1), move || {
                            if let Some(ui) = ui_reset.upgrade() {
                                ui.set_copied(false);
                            }
                        });
                    }
                }
            });
        }

        // Raise timer & skip taskbar & auto exit on focus loss
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

pub fn swap_state(
    src: usize,
    dst: usize,
    mut input: String,
    mut output: String,
) -> (usize, usize, String, String) {
    let (new_src, new_dst) = if src == 0 {
        let old_dst = dst;
        (old_dst + 1, if old_dst == 0 { 1 } else { 0 })
    } else {
        (dst + 1, src - 1)
    };
    if !output.trim().is_empty() {
        std::mem::swap(&mut input, &mut output);
    }
    (new_src, new_dst, input, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_non_auto_languages_and_text() {
        let (src, dst, input, output) = swap_state(2, 0, "hello".to_string(), "你好".to_string());
        assert_eq!(src, 1); // zh-CN
        assert_eq!(dst, 1); // en
        assert_eq!(input, "你好");
        assert_eq!(output, "hello");
    }

    #[test]
    fn swaps_auto_source_language() {
        let (src, dst, _, _) = swap_state(0, 0, "".to_string(), "".to_string());
        assert_eq!(src, 1); // zh-CN
        assert_eq!(dst, 1); // en
    }
}
