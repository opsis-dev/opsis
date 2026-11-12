use opsis_scene::random::js_number_text;

pub fn polar(cx: f64, cy: f64, radius: f64, degrees: f64) -> (f64, f64) {
    let radians = (degrees - 90.0).to_radians();
    (
        radius.mul_add(radians.cos(), cx),
        radius.mul_add(radians.sin(), cy),
    )
}

pub fn arc_path(cx: f64, cy: f64, radius: f64, start: f64, end: f64) -> String {
    if end - start <= 0.001 {
        return String::new();
    }
    let n = js_number_text;
    let a = polar(cx, cy, radius, start);
    let b = polar(cx, cy, radius, f64::midpoint(start, end));
    let c = polar(cx, cy, radius, end);
    format!(
        "M {} {} A {r} {r} 0 0 1 {} {} A {r} {r} 0 0 1 {} {}",
        n(a.0),
        n(a.1),
        n(b.0),
        n(b.1),
        n(c.0),
        n(c.1),
        r = n(radius)
    )
}

pub fn safe_padding(width: f64, height: f64) -> (f64, f64) {
    (
        (80.0 / 1080.0 * width).round(),
        (100.0 / 1080.0 * height).round(),
    )
}
