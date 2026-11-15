use opsis_types::numbers::{exact_eq, float};

use crate::node::{Node, div};
use crate::timeline::{At, Nodes, absolute_fill};

pub fn shutter_offsets(shutter_angle: f64, samples: u32) -> Vec<f64> {
    let open = shutter_angle.clamp(0.0, 360.0) / 360.0;
    if exact_eq(open, 0.0) || samples <= 1 {
        return vec![0.0];
    }
    (0..samples)
        .map(|i| (open * f64::from(i)) / f64::from(samples))
        .collect()
}

pub fn sample_opacity(index: usize) -> f64 {
    1.0 / (float(index) + 1.0)
}

pub fn shutter_layers(samples: &[Vec<Node>]) -> Vec<Node> {
    samples
        .iter()
        .enumerate()
        .map(|(i, sample)| {
            div(absolute_fill().opacity(sample_opacity(i)))
                .children(sample.clone())
                .into()
        })
        .collect()
}

pub fn motion_blur(
    shutter_angle: f64,
    samples: u32,
    at: &At,
    draw: &dyn Fn(&At) -> Nodes,
) -> Result<Node, crate::timeline::DrawError> {
    let mut drawn = Vec::new();
    for offset in shutter_offsets(shutter_angle, samples) {
        let frozen = At {
            clock: at.clock.freeze(0.0_f64.max(at.clock.frame - offset)),
            ..*at
        };
        drawn.push(draw(&frozen)?);
    }
    Ok(div(absolute_fill()).children(shutter_layers(&drawn)).into())
}
