use opsis_scene::node::{Node, div};
use opsis_scene::style::Style;
use opsis_scene::timeline::absolute_fill;

pub fn filled(style: &Style, children: Vec<Node>) -> Node {
    div(absolute_fill().merged(style)).children(children).into()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phase {
    pub entering: bool,
    pub displace: f64,
    pub opacity: f64,
}

pub fn transition_phase(
    progress: f64,
    entering: bool,
    lead: f64,
    fade: bool,
) -> Result<Phase, crate::elements::draw::DrawFailure> {
    let spans = lead >= 1.0;
    let ranged = |from: [f64; 3], to: [f64; 3]| {
        opsis_scene::interpolate::interpolate(
            progress,
            &from,
            &to,
            &opsis_scene::interpolate::Interpolation::clamped(),
        )
    };
    if entering {
        let arrived = if spans {
            progress
        } else {
            ranged([0.0, lead, 1.0], [0.0, 1.0, 1.0])?
        };
        return Ok(Phase {
            entering,
            displace: 1.0 - arrived,
            opacity: if fade {
                ranged([0.0, 0.42, 1.0], [0.0, 0.72, 1.0])?
            } else {
                1.0
            },
        });
    }
    let left = if spans {
        progress
    } else {
        ranged([0.0, 1.0 - lead, 1.0], [0.0, 0.0, 1.0])?
    };
    Ok(Phase {
        entering,
        displace: left,
        opacity: if fade {
            ranged([0.0, 0.58, 1.0], [1.0, 0.42, 0.0])?
        } else {
            1.0
        },
    })
}

pub fn encoded_svg(markup: &str) -> String {
    let escaped: String = markup
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect();
    format!("data:image/svg+xml,{escaped}")
}
