use opsis_macros::settings;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::Node;
use opsis_scene::paint::Transform;
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use opsis_types::looks::NodeLook;

use crate::elements::background::shader_grain_gradient::{Grain, grain_field};
use crate::elements::draw::{Cx, DrawFailure, Transition};
use crate::elements::entry::{element, meta};
use crate::elements::shared::{Colour, VerticalDirection};
use crate::elements::transition::present::filled;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct WaveWipe {
    #[setting("Intensity", default = 0.2, range(min = 0, max = 1), step = 0.05)]
    pub intensity: f64,
    #[setting(
        "Softness",
        default = 0.7,
        range(min = 0, max = 1),
        step = 0.05,
        advanced
    )]
    pub softness: f64,
    #[setting("Noise", default = 0.4, range(min = 0, max = 1), step = 0.05, advanced)]
    pub noise: f64,
    #[setting("Background", default = "#141318", check = colour)]
    pub color_back: String,
    #[setting(
        "Colours",
        default = vec!["#3a3a52".into(), "#4a4a68".into(), "#8f88ae".into()],
        length(min = 2, max = 7)
    )]
    pub colors: Vec<Colour>,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        advanced
    )]
    pub speed: f64,
    #[setting(
        "Zoom",
        default = 1.16,
        range(min = 0.5, max = 4),
        step = 0.01,
        advanced
    )]
    pub zoom: f64,
    #[setting(
        "Direction",
        default = VerticalDirection::Up
    )]
    pub direction: VerticalDirection,
}

pub fn definition() -> ElementMeta {
    meta::<WaveWipe>(
            element(
                ElementKind::Presentation,
                "wave-wipe",
                "Wave wipe",
                "A grain-gradient wave that sweeps up across the frame, covering the outgoing scene and revealing the next one from below.",
                ElementCategory::Transition,
                60,
            )
                .look(NodeLook::of("lucide:waves-horizontal")),
        )
}

const IN_OUT: Easing = Easing::Bezier(0.42, 0.0, 0.58, 1.0);

fn eased(p: f64, from: &[f64], to: &[f64], easing: Easing) -> Result<f64, DrawFailure> {
    Ok(interpolate(
        p,
        from,
        to,
        &Interpolation::clamped().eased(easing),
    )?)
}

const WAVE: f64 = 1.0;

pub fn present(props: &WaveWipe, side: Transition, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let sign = if props.direction == VerticalDirection::Up {
        1.0
    } else {
        -1.0
    };
    let p = side.progress;
    if !side.entering {
        let lift = eased(
            p,
            &[0.0, 0.7],
            &[0.0, -70.0 * sign],
            Easing::ease_in(Easing::Cubic),
        )?;
        let style = Style::default()
            .opacity(eased(p, &[0.3, 0.5], &[1.0, 0.0], IN_OUT)?)
            .transform(vec![Transform::Translate(
                Length::ZERO,
                Length::percent(lift),
                Length::ZERO,
            )]);
        return Ok(filled(&style, side.children));
    }
    let field_opacity = eased(p, &[0.18, 0.45], &[0.0, 1.0], IN_OUT)?;
    let drift = eased(
        p,
        &[0.0, 1.0],
        &[0.0, 0.7 * sign],
        Easing::Bezier(0.45, 0.0, 0.55, 1.0),
    )?;
    let rise = eased(
        p,
        &[0.4, 0.82, 1.0],
        &[100.0 * sign, -3.5 * sign, 0.0],
        Easing::ease_out(Easing::Cubic),
    )?;
    let field = grain_field(&Grain {
        time: cx.frame() / cx.fps() * props.speed,
        colors: props.colors.iter().map(|c| c.0.clone()).collect(),
        color_back: props.color_back.clone(),
        softness: props.softness,
        intensity: props.intensity,
        noise: props.noise,
        shape: WAVE,
        scale: props.zoom,
        rotation: 0.0,
        offset: (0.0, drift),
    });
    let risen = Style::default().transform(vec![Transform::Translate(
        Length::ZERO,
        Length::percent(rise),
        Length::ZERO,
    )]);
    Ok(filled(
        &Style::default(),
        vec![
            filled(&Style::default().opacity(field_opacity), vec![field]),
            filled(&risen, side.children),
        ],
    ))
}
