use gtk4::cairo;
use xtools_ui::{func_radius, main_radius, theme, ToolId, MARK_PX, ORB_FILL, ORB_MARK};

fn fill_disk(cr: &cairo::Context, cx: f64, cy: f64, r: f64) {
    cr.set_source_rgba(ORB_FILL.r, ORB_FILL.g, ORB_FILL.b, ORB_FILL.a);
    cr.new_sub_path();
    cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
    cr.fill().ok();
}

fn mark_color(cr: &cairo::Context) {
    cr.set_source_rgba(ORB_MARK.r, ORB_MARK.g, ORB_MARK.b, ORB_MARK.a);
}

fn draw_clock(cr: &cairo::Context, cx: f64, cy: f64, fr: f64) {
    mark_color(cr);
    cr.set_line_width((1.6 * fr / func_radius()).max(1.0));
    cr.new_sub_path();
    cr.arc(cx, cy, fr * 0.38, 0.0, std::f64::consts::TAU);
    cr.stroke().ok();
    cr.set_line_width((1.4 * fr / func_radius()).max(1.0));
    cr.move_to(cx, cy);
    cr.line_to(cx, cy - fr * 0.22);
    cr.stroke().ok();
    cr.move_to(cx, cy);
    cr.line_to(cx + fr * 0.16, cy + fr * 0.04);
    cr.stroke().ok();
}

fn draw_text_mark(cr: &cairo::Context, cx: f64, cy: f64, text: &str, scale: f64) {
    cr.select_font_face("sans-serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(MARK_PX * scale);
    let ext = cr
        .text_extents(text)
        .unwrap_or_else(|_| cairo::TextExtents::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
    mark_color(cr);
    cr.move_to(
        cx - ext.width() / 2.0 - ext.x_bearing(),
        cy - ext.height() / 2.0 - ext.y_bearing(),
    );
    cr.show_text(text).ok();
}

pub fn draw_main(cr: &cairo::Context, cx: f64, cy: f64, scale: f64) {
    let _ = theme::MAIN_D;
    let r = main_radius() * scale;
    fill_disk(cr, cx, cy, r);
    cr.set_source_rgba(ORB_MARK.r, ORB_MARK.g, ORB_MARK.b, 0.55);
    cr.set_line_width((1.5 * scale).max(1.0));
    cr.new_sub_path();
    cr.arc(cx, cy, r - 1.0, 0.0, std::f64::consts::TAU);
    cr.stroke().ok();
    draw_text_mark(cr, cx, cy, "x", scale);
}

pub fn draw_func(cr: &cairo::Context, id: ToolId, cx: f64, cy: f64, t: f64, scale: f64) {
    if t <= 0.0 {
        return;
    }
    let fr = func_radius() * scale;
    cr.save().ok();
    cr.set_source_rgba(ORB_FILL.r, ORB_FILL.g, ORB_FILL.b, t.clamp(0.0, 1.0));
    cr.new_sub_path();
    cr.arc(cx, cy, fr, 0.0, std::f64::consts::TAU);
    cr.fill().ok();
    cr.set_source_rgba(ORB_MARK.r, ORB_MARK.g, ORB_MARK.b, t.clamp(0.0, 1.0));
    match id {
        ToolId::Time => draw_clock(cr, cx, cy, fr),
        ToolId::Json => draw_text_mark(cr, cx, cy, "{}", scale),
        ToolId::Trans => draw_text_mark(cr, cx, cy, "文", scale),
    }
    cr.restore().ok();
}
