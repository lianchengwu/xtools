use xtools_ui::POP_MS;

pub fn ease_out_cubic(t: f64) -> f64 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

/// Progress 0..1 from a start timestamp in microseconds.
pub fn progress(now_us: i64, start_us: i64) -> f64 {
    let dur_us = i64::from(POP_MS) * 1000;
    if dur_us <= 0 {
        return 1.0;
    }
    ((now_us - start_us) as f64 / dur_us as f64).clamp(0.0, 1.0)
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
