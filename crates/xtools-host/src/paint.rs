use gtk4::cairo;
use gtk4::gdk::prelude::*;
use gtk4::gdk_pixbuf::Pixbuf;
use xtools_ui::{MARK_PX, ORB_FILL, ORB_MARK, ToolId, func_radius};

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
    cr.select_font_face(
        "sans-serif",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Bold,
    );
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

const XTOOLS_SVG: &[u8] = include_bytes!("../../../xtools.svg");

thread_local! {
    static BASE_ICON: Option<Pixbuf> = Pixbuf::from_read(std::io::Cursor::new(XTOOLS_SVG)).ok();
}

pub fn draw_main(cr: &cairo::Context, cx: f64, cy: f64, scale: f64) {
    let icon_size = (32.0 * scale).round() as i32;
    BASE_ICON.with(|base| {
        if let Some(pixbuf) = base.as_ref().and_then(|pb| {
            pb.scale_simple(icon_size, icon_size, gtk4::gdk_pixbuf::InterpType::Bilinear)
        }) {
            let px = cx - f64::from(icon_size) / 2.0;
            let py = cy - f64::from(icon_size) / 2.0;
            cr.set_source_pixbuf(&pixbuf, px, py);
            cr.paint().ok();
        } else {
            draw_text_mark(cr, cx, cy, "x", scale);
        }
    });
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
    cr.set_source_rgba(ORB_MARK.r, ORB_MARK.g, ORB_MARK.b, 0.20 * t.clamp(0.0, 1.0));
    cr.set_line_width((1.0 * scale).max(1.0));
    cr.new_sub_path();
    cr.arc(cx, cy, fr - 0.5, 0.0, std::f64::consts::TAU);
    cr.stroke().ok();
    cr.set_source_rgba(ORB_MARK.r, ORB_MARK.g, ORB_MARK.b, t.clamp(0.0, 1.0));
    match id {
        ToolId::Time => draw_clock(cr, cx, cy, fr),
        ToolId::Json => draw_text_mark(cr, cx, cy, "{}", scale),
        ToolId::Trans => draw_text_mark(cr, cx, cy, "文", scale),
    }
    cr.restore().ok();
}
