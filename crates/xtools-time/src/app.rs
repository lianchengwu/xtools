use std::os::unix::net::UnixListener;
use std::time::Duration;

use eframe::egui::{self, CentralPanel, Context, ViewportCommand};
use xtools_ui::chrome::{
    apply_theme, copy_button_enabled, inline_error, labeled_field, now_button, title_strip,
};
use xtools_ui::{accept_raise, raise_instance, TIME_INSTANCE};

use crate::convert;

#[derive(Clone, Copy)]
enum Field {
    Seconds,
    Millis,
    Local,
}

pub struct TimeApp {
    _lock: UnixListener,
    seconds: String,
    millis: String,
    local: String,
    good_seconds: String,
    good_millis: String,
    good_local: String,
    error: Option<(Field, String)>,
    copied_sec_until: f64,
    copied_ms_until: f64,
    centered: bool,
}

impl TimeApp {
    pub fn new(lock: UnixListener) -> Self {
        let (s, ms, local) = convert::from_now();
        let seconds = s.to_string();
        let millis = ms.to_string();
        Self {
            _lock: lock,
            seconds: seconds.clone(),
            millis: millis.clone(),
            local: local.clone(),
            good_seconds: seconds,
            good_millis: millis,
            good_local: local,
            error: None,
            copied_sec_until: 0.0,
            copied_ms_until: 0.0,
            centered: false,
        }
    }

    fn apply_triple(&mut self, s: i64, ms: i64, local: String) {
        self.seconds = s.to_string();
        self.millis = ms.to_string();
        self.local = local;
        self.good_seconds = self.seconds.clone();
        self.good_millis = self.millis.clone();
        self.good_local = self.local.clone();
        self.error = None;
    }

    fn on_seconds(&mut self) {
        let trimmed = self.seconds.trim();
        if trimmed.is_empty() {
            self.error = Some((
                Field::Seconds,
                "这一栏是空的\n输入一个值，或点「现在」填入当前时间。".into(),
            ));
            return;
        }
        match trimmed.parse::<i64>().ok().and_then(|n| convert::from_seconds(n).ok()) {
            Some((s, ms, local)) => self.apply_triple(s, ms, local),
            None => {
                self.error = Some((
                    Field::Seconds,
                    "秒数无效。输入 Unix 秒，或点「现在」。".into(),
                ));
            }
        }
    }

    fn on_millis(&mut self) {
        let trimmed = self.millis.trim();
        if trimmed.is_empty() {
            self.error = Some((
                Field::Millis,
                "这一栏是空的\n输入一个值，或点「现在」填入当前时间。".into(),
            ));
            return;
        }
        match trimmed.parse::<i64>().ok().and_then(|n| convert::from_millis(n).ok()) {
            Some((s, ms, local)) => self.apply_triple(s, ms, local),
            None => {
                self.error = Some((
                    Field::Millis,
                    "毫秒无效。输入 Unix 毫秒，或点「现在」。".into(),
                ));
            }
        }
    }

    fn on_local(&mut self) {
        let trimmed = self.local.trim();
        if trimmed.is_empty() {
            self.error = Some((
                Field::Local,
                "这一栏是空的\n输入一个值，或点「现在」填入当前时间。".into(),
            ));
            return;
        }
        match convert::from_local(trimmed) {
            Ok((s, ms, local)) => self.apply_triple(s, ms, local),
            Err(_) => {
                self.error = Some((
                    Field::Local,
                    "本地时间无效。按 年-月-日 时:分:秒 填写，或点「现在」。".into(),
                ));
            }
        }
    }

    fn poll_raise(&mut self, ctx: &Context) {
        if accept_raise(&self._lock).is_some() {
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }
}

impl eframe::App for TimeApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);
        self.poll_raise(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        apply_theme(&ctx);
        if !self.centered {
            if let Some(cmd) = ViewportCommand::center_on_screen(&ctx) {
                ctx.send_viewport_cmd(cmd);
            }
            self.centered = true;
        }
        self.poll_raise(&ctx);

        let now = ctx.input(|i| i.time);
        let sec_copied = now < self.copied_sec_until;
        let ms_copied = now < self.copied_ms_until;
        let sec_ok = self.seconds.trim().parse::<i64>().is_ok();
        let ms_ok = self.millis.trim().parse::<i64>().is_ok();

        CentralPanel::default().show(ui, |ui| {
            title_strip(ui, "时间戳");
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(ui.available_width() - 80.0);
                    let r = labeled_field(ui, "秒", &mut self.seconds);
                    if r.changed() {
                        self.on_seconds();
                    }
                    if matches!(self.error, Some((Field::Seconds, _))) {
                        if let Some((_, ref msg)) = self.error {
                            inline_error(ui, msg);
                        }
                    }
                });
                if copy_button_enabled(ui, sec_copied, sec_ok).clicked() && sec_ok {
                    ctx.copy_text(self.seconds.trim().to_string());
                    self.copied_sec_until = now + 1.0;
                    ctx.request_repaint_after(Duration::from_secs(1));
                }
            });

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(ui.available_width() - 80.0);
                    let r = labeled_field(ui, "毫秒", &mut self.millis);
                    if r.changed() {
                        self.on_millis();
                    }
                    if matches!(self.error, Some((Field::Millis, _))) {
                        if let Some((_, ref msg)) = self.error {
                            inline_error(ui, msg);
                        }
                    }
                });
                if copy_button_enabled(ui, ms_copied, ms_ok).clicked() && ms_ok {
                    ctx.copy_text(self.millis.trim().to_string());
                    self.copied_ms_until = now + 1.0;
                    ctx.request_repaint_after(Duration::from_secs(1));
                }
            });

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("本地时间")
                        .small()
                        .color(egui::Color32::from_rgb(0x8F, 0x8F, 0x94)),
                );
                if now_button(ui).clicked() {
                    let (s, ms, local) = convert::from_now();
                    self.apply_triple(s, ms, local);
                }
            });
            let r = labeled_field(ui, "", &mut self.local);
            if r.changed() {
                self.on_local();
            }
            if matches!(self.error, Some((Field::Local, _))) {
                if let Some((_, ref msg)) = self.error {
                    inline_error(ui, msg);
                }
            }
        });
    }
}

/// Used only so a losing child can raise without mapping a window.
#[allow(dead_code)]
pub fn forward_raise(token: Option<&str>) {
    let _ = raise_instance(TIME_INSTANCE, token);
}
