//! Shared egui chrome. Compiled only with feature `egui-chrome`.

use egui::{
    Align2, Color32, Context, FontFamily, FontId, Response, RichText, Sense, Stroke, TextEdit, Ui,
    Vec2, Visuals,
};

use crate::theme::{Color, ORB_FILL, ORB_MARK};

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

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    let ground = c32(ORB_FILL);
    let mark = c32(ORB_MARK);
    visuals.panel_fill = ground;
    visuals.window_fill = ground;
    visuals.extreme_bg_color = secondary();
    visuals.faint_bg_color = secondary();
    visuals.override_text_color = Some(mark);
    visuals.widgets.inactive.bg_fill = secondary();
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, muted());
    visuals.widgets.hovered.bg_fill = secondary();
    visuals.widgets.active.bg_fill = mark;
    visuals.selection.stroke = Stroke::new(1.0, muted());
    ctx.set_visuals(visuals);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
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
            FontId::new(20.0, FontFamily::Proportional),
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

pub fn title_strip(ui: &mut Ui, title: &str) {
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), 32.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(RichText::new(title).heading().strong().color(c32(ORB_MARK)));
        },
    );
}

pub fn labeled_field(ui: &mut Ui, caption: &str, text: &mut String) -> Response {
    ui.label(RichText::new(caption).small().color(muted()));
    ui.add(
        TextEdit::singleline(text)
            .font(egui::TextStyle::Monospace)
            .desired_width(ui.available_width()),
    )
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
    let size = Vec2::new(64.0, 32.0);
    if !enabled {
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        ui.painter().rect_filled(rect, 4.0, secondary());
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(16.0),
            muted(),
        );
        return response;
    }
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    ui.painter().rect_filled(rect, 4.0, c32(ORB_MARK));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(16.0),
        c32(ORB_FILL),
    );
    response
}

pub fn inline_error(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.colored_label(destructive(), RichText::new(text).size(16.0));
}
