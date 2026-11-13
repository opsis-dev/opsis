use opsis_scene::node::{Effect, Element, Kind, Node};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::Num;

use crate::elements::oklch_mix::parsed_or_black;

pub fn rgb_uniform(css: &str) -> Vec<f64> {
    let rgb = parsed_or_black(css);
    vec![rgb.r, rgb.g, rgb.b]
}

pub fn filtered(program: &str, uniforms: Vec<(&str, Vec<f64>)>, child: Node) -> Node {
    Element::new(
        Kind::Effect(Effect {
            program: program.to_owned(),
            uniforms: uniforms
                .into_iter()
                .map(|(name, values)| (name.to_owned(), values.into_iter().map(Num).collect()))
                .collect(),
        }),
        absolute_fill(),
    )
    .child(child)
    .into()
}
