use opsis_types::numbers::{float, truncate};

use crate::elements::draw::Cx;
use crate::elements::units::{utf16_len, utf16_slice};

const DEFAULT_CPS: f64 = 20.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typed {
    pub text: String,
    pub count: usize,
    pub done: bool,
    pub typing: bool,
}

fn reveal_count(local: f64, fps: f64, len: usize, cps: f64) -> usize {
    let over = float(len) / cps * fps;
    if over <= 0.0 {
        return len;
    }
    let shown: f64 = (local / over * float(len)).floor().clamp(0.0, float(len));
    truncate(shown)
}

pub fn typewriter(full: &str, cx: &Cx<'_>, cps: Option<f64>, speed: f64, start: f64) -> Typed {
    let local = cx.authored_frame().mul_add(speed, -start);
    let len = utf16_len(full);
    let count = if local <= 0.0 {
        0
    } else {
        reveal_count(local, Cx::authored_fps(), len, cps.unwrap_or(DEFAULT_CPS))
    };
    Typed {
        text: utf16_slice(full, 0, count),
        count,
        done: count >= len,
        typing: count > 0 && count < len,
    }
}
