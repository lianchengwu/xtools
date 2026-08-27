#[cfg(unix)]
use gtk4::gdk;
use xtools_ui::{GAP, ToolId, func_radius, main_radius, orbit_radius};

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    #[cfg(unix)]
    #[allow(dead_code)]
    pub fn from_monitor(geo: &gdk::Rectangle) -> Self {
        Self {
            x: f64::from(geo.x()),
            y: f64::from(geo.y()),
            w: f64::from(geo.width()),
            h: f64::from(geo.height()),
        }
    }
}

/// Widget-local output. Layer-shell surface origin is (0,0), not monitor.x/y.
pub fn surface_rect(w: f64, h: f64) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: w.max(1.0),
        h: h.max(1.0),
    }
}

/// Unscaled 4K (and similar) makes a 40px disk vanish. Scale paint/hit only.
pub fn vis_scale(surface: Rect) -> f64 {
    if surface.w >= 2560.0 || surface.h >= 1600.0 {
        2.0
    } else {
        1.0
    }
}
/// D-16: mid-right, vertically centered, full disk on-screen.
#[allow(dead_code)]
pub fn default_main_center(surface: Rect, main_r: f64) -> (f64, f64) {
    let inset = main_r.max(8.0);
    let cx = surface.w - inset;
    let cy = surface.h / 2.0;
    clamp_main(cx, cy, main_r, surface)
}

/// D-17: entire main disk stays inside the output.
pub fn clamp_main(cx: f64, cy: f64, main_r: f64, surface: Rect) -> (f64, f64) {
    let min_x = surface.x + main_r;
    let max_x = surface.x + surface.w - main_r;
    let min_y = surface.y + main_r;
    let max_y = surface.y + surface.h - main_r;
    (
        cx.clamp(min_x.min(max_x), max_x.max(min_x)),
        cy.clamp(min_y.min(max_y), max_y.max(min_y)),
    )
}

/// Rest angles in radians: left-up, up, right-up. 0 = +x, CCW.
const REST_DEG: [f64; 3] = [-150.0, -90.0, -30.0];

fn deg_to_rad(d: f64) -> f64 {
    d * std::f64::consts::PI / 180.0
}

fn seat_at(main: (f64, f64), angle: f64, r: f64) -> (f64, f64) {
    (main.0 + r * angle.cos(), main.1 + r * angle.sin())
}

fn disk_inside(c: (f64, f64), r: f64, mon: Rect) -> bool {
    c.0 - r >= mon.x && c.0 + r <= mon.x + mon.w && c.1 - r >= mon.y && c.1 + r <= mon.y + mon.h
}

/// Fan-above seats for `ToolId::ALL`. Rotate/shrink so every function disk stays on the output.
pub fn fan_seats(main: (f64, f64), monitor: Rect, scale: f64) -> [(ToolId, f64, f64); 3] {
    let fr = func_radius() * scale;
    let min_r = (main_radius() + func_radius() + GAP) * scale;
    let mut radius = orbit_radius() * scale;
    let mut offset = 0.0_f64;

    let seats_for = |off: f64, rad: f64| {
        let mut out = [(ToolId::Time, 0.0, 0.0); 3];
        for (i, id) in ToolId::ALL.iter().enumerate() {
            let ang = deg_to_rad(REST_DEG[i] + off);
            let (x, y) = seat_at(main, ang, rad);
            out[i] = (*id, x, y);
        }
        out
    };

    let all_fit = |off: f64, rad: f64| {
        seats_for(off, rad)
            .iter()
            .all(|(_, x, y)| disk_inside((*x, *y), fr, monitor))
    };

    if !all_fit(0.0, radius) {
        let mut found = false;
        for step in 0..=12 {
            let off = 15.0 * f64::from(step);
            for sign in [1.0, -1.0] {
                if all_fit(sign * off, radius) {
                    offset = sign * off;
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            let mut rad = radius;
            while rad > min_r && !all_fit(offset, rad) {
                rad -= 2.0;
            }
            radius = rad.max(min_r);
            while radius > fr + 2.0 && !all_fit(offset, radius) {
                radius -= 2.0;
            }
        }
    }

    seats_for(offset, radius)
}

pub fn hit_disk(px: f64, py: f64, cx: f64, cy: f64, r: f64) -> bool {
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= r * r
}
