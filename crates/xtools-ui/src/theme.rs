/// Logical (surface-local) color. Host passes these to cairo `set_source_rgba`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}

/// Dark opaque disk. Not an accent wash. Not translucent.
pub const ORB_FILL: Color = Color::rgba(0.12, 0.12, 0.14, 1.0);

/// Light mark on the disk.
pub const ORB_MARK: Color = Color::rgba(0.92, 0.93, 0.95, 1.0);

/// Pango mark size in logical px.
pub const MARK_PX: f64 = 16.0;

/// Main orb diameter.
pub const MAIN_D: f64 = 40.0;

/// Function orb diameter.
pub const FUNC_D: f64 = 32.0;

/// Gap between main and function disks. Disks must not overlap.
pub const GAP: f64 = 8.0;

/// Click vs drag slop in logical px (D-14).
pub const SLOP: f64 = 8.0;

/// Expand / collapse pop duration in milliseconds (D-10).
pub const POP_MS: u32 = 120;

pub fn main_radius() -> f64 {
    MAIN_D / 2.0
}

pub fn func_radius() -> f64 {
    FUNC_D / 2.0
}

pub fn orbit_radius() -> f64 {
    main_radius() + func_radius() + GAP
}
