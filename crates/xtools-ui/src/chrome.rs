//! Shared egui chrome. Compiled only with feature `egui-chrome`.

use std::sync::Arc;

use egui::{
    Align2, Button, Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId,
    Frame, Key, Margin, Pos2, Response, RichText, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2,
    ViewportCommand, Visuals,
};

use crate::theme::{Color, ORB_FILL, ORB_MARK};

const CJK_FONT: &str = "xtools-cjk";

const CJK_CANDIDATES: &[&str] = &[
    "/usr/share/fonts/truetype/SourceHanSansCN-Regular.otf",
    "/usr/share/fonts/truetype/SourceHanSansCN-Normal.otf",
    "/usr/share/fonts/opentype/source-han-sans/SourceHanSansCN-Regular.otf",
    "/usr/share/fonts/noto-cjk/NotoSansCJKsc-Regular.otf",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
    "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
];

fn c32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8,
    )
}

fn secondary() -> Color32 {
    Color32::from_rgb(0x2E, 0x2E, 0x33)
}

fn muted() -> Color32 {
    Color32::from_rgb(0x8F, 0x8F, 0x94)
}

fn destructive() -> Color32 {
    Color32::from_rgb(0xC4, 0x5C, 0x5C)
}

fn hairline() -> Color32 {
    Color32::from_rgb(0x3A, 0x3A, 0x40)
}

pub fn cjk_font_path() -> Option<&'static str> {
    CJK_CANDIDATES
        .iter()
        .copied()
        .find(|path| std::path::Path::new(path).is_file())
}

/// Install a system CJK face so Han in labels is not tofu.
pub fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    if let Some(path) = cjk_font_path()
        && let Ok(bytes) = std::fs::read(path)
    {
        fonts
            .font_data
            .insert(CJK_FONT.to_owned(), Arc::new(FontData::from_owned(bytes)));
        if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
            proportional.insert(0, CJK_FONT.to_owned());
        }
        if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
            mono.push(CJK_FONT.to_owned());
        }
    }
    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    let ground = c32(ORB_FILL);
    let mark = c32(ORB_MARK);
    let radius = CornerRadius::same(8);
    visuals.panel_fill = Color32::TRANSPARENT;
    visuals.window_fill = ground;
    visuals.extreme_bg_color = secondary();
    visuals.faint_bg_color = secondary();
    visuals.override_text_color = Some(mark);
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.widgets.noninteractive.corner_radius = radius;
    visuals.widgets.inactive.bg_fill = secondary();
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, muted());
    visuals.widgets.inactive.corner_radius = radius;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x3A, 0x3A, 0x40);
    visuals.widgets.hovered.corner_radius = radius;
    visuals.widgets.active.bg_fill = mark;
    visuals.widgets.active.corner_radius = radius;
    visuals.selection.stroke = Stroke::new(1.0, muted());
    ctx.set_visuals(visuals);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.spacing.button_padding = Vec2::new(10.0, 6.0);
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(16.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::new(18.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            FontId::new(16.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(16.0, FontFamily::Proportional),
        );
    });
}

/// Rounded frameless card: drag title, painted close, padded body.
pub fn tool_shell(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    let rect = ui.max_rect();
    let rounding = CornerRadius::same(12);
    ui.painter().rect_filled(rect, rounding, c32(ORB_FILL));
    ui.painter().rect_stroke(
        rect,
        rounding,
        Stroke::new(1.0, hairline()),
        StrokeKind::Inside,
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        title_bar(ui, title);
        Frame::new()
            .inner_margin(Margin::symmetric(20, 16))
            .show(ui, add_contents);
    });
}

fn title_bar(ui: &mut Ui, title: &str) {
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        std::process::exit(0);
    }
    let height = 40.0;
    let close_w = 40.0;
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), height),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let close = ui.add_sized(
                    Vec2::new(close_w, height),
                    Button::new(RichText::new("×").size(20.0).color(muted()))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE),
                );
                if close.clicked() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    std::process::exit(0);
                }

                let (drag, drag_resp) = ui.allocate_exact_size(ui.available_size(), Sense::drag());
                ui.painter().text(
                    Pos2::new(drag.left() + 20.0, drag.center().y),
                    Align2::LEFT_CENTER,
                    title,
                    FontId::proportional(18.0),
                    c32(ORB_MARK),
                );
                if drag_resp.drag_started() {
                    ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
                }
            },
        );
    });
    let bottom = ui.min_rect().bottom();
    let left = ui.max_rect().left();
    let right = ui.max_rect().right();
    ui.painter().line_segment(
        [
            Pos2::new(left + 12.0, bottom),
            Pos2::new(right - 12.0, bottom),
        ],
        Stroke::new(1.0, hairline()),
    );
}

pub fn labeled_field(ui: &mut Ui, caption: &str, text: &mut String) -> Response {
    if !caption.is_empty() {
        ui.label(RichText::new(caption).small().color(muted()));
        ui.add_space(4.0);
    }
    value_field(ui, text)
}

pub fn value_field(ui: &mut Ui, text: &mut String) -> Response {
    value_field_width(ui, text, ui.available_width())
}

pub fn value_field_width(ui: &mut Ui, text: &mut String, width: f32) -> Response {
    ui.add(
        TextEdit::singleline(text)
            .font(egui::TextStyle::Monospace)
            .desired_width(width.max(80.0))
            .min_size(Vec2::new(0.0, 32.0))
            .margin(Margin::symmetric(10, 8)),
    )
}

/// Caption, then field | action on one row so the button never overlaps the well.
pub fn field_with_action(
    ui: &mut Ui,
    caption: &str,
    text: &mut String,
    action: impl FnOnce(&mut Ui),
) -> Response {
    if !caption.is_empty() {
        ui.label(RichText::new(caption).small().color(muted()));
        ui.add_space(4.0);
    }
    ui.horizontal(|ui| {
        let reserved = 76.0;
        let r = value_field_width(ui, text, ui.available_width() - reserved);
        ui.add_space(8.0);
        action(ui);
        r
    })
    .inner
}

pub fn copy_button(ui: &mut Ui, copied: bool) -> Response {
    copy_button_enabled(ui, copied, true)
}

pub fn copy_button_enabled(ui: &mut Ui, copied: bool, enabled: bool) -> Response {
    let label = if copied { "已复制" } else { "复制" };
    accent_button(ui, label, enabled)
}

pub fn now_button(ui: &mut Ui) -> Response {
    accent_button(ui, "现在", true)
}

fn accent_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    let size = Vec2::new(68.0, 32.0);
    let rounding = CornerRadius::same(8);
    if !enabled {
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().rect_filled(rect, rounding, secondary());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(15.0),
            muted(),
        );
        return response;
    }
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let fill = if response.is_pointer_button_down_on() {
        Color32::from_rgb(0xD4, 0xD6, 0xDC)
    } else if response.hovered() {
        Color32::from_rgb(0xF4, 0xF5, 0xF8)
    } else {
        c32(ORB_MARK)
    };
    ui.painter().rect_filled(rect, rounding, fill);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(15.0),
        c32(ORB_FILL),
    );
    response
}

pub fn inline_error(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.colored_label(destructive(), RichText::new(text).size(13.0));
}

#[cfg(test)]
mod tests {
    use super::cjk_font_path;

    #[test]
    fn finds_system_cjk_face() {
        assert!(
            cjk_font_path().is_some(),
            "need a CJK otf/ttf on this machine so labels are not tofu"
        );
    }
}
