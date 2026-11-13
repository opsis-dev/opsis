pub fn float_offset(frame: f64, fps: f64, amplitude: f64, start: f64, period: f64) -> f64 {
    let local = (frame - start).max(0.0);
    let value = (local / (fps * period) * std::f64::consts::PI * 2.0).sin() * amplitude;
    if value == 0.0 { 0.0 } else { -value }
}
