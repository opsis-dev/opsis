use opsis_layout::fonts::Fonts;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Cache, Resources, paint};
use opsis_scene::color::Color;
use opsis_scene::node::{Node, div};
use opsis_scene::paint::Transform;
use opsis_scene::style::{Position, Style, WillChange};
use opsis_scene::values::{Length, Num};
use opsis_skia::{Rgba, Surface};
use proptest::prelude::*;

const SIDE: f64 = 160.0;
const PIXELS: u32 = 160;

fn placed(x: f64, y: f64, size: f64) -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(Length::px(x))
        .top(Length::px(y))
        .width(Length::px(size))
        .height(Length::px(size))
}

fn scene(scale: f64, shift: f64, will_change: WillChange) -> Node {
    let layer = placed(0.0, 0.0, SIDE)
        .transform(vec![
            Transform::Translate(Length::px(shift), Length::px(-shift), Length::ZERO),
            Transform::Scale(Num(scale), Num(scale), Num(1.0)),
        ])
        .will_change(will_change);
    div(placed(0.0, 0.0, SIDE))
        .child(
            div(layer)
                .child(div(
                    placed(10.0, 10.0, 40.0).background_color(Color::rgb8(249, 115, 22))
                ))
                .child(div(
                    placed(70.0, 30.0, 60.0).background_color(Color::rgb8(14, 165, 233))
                ))
                .child(div(
                    placed(40.0, 90.0, 50.0).background_color(Color::rgb8(34, 197, 94))
                )),
        )
        .into()
}

fn drawn(root: &Node, cache: Cache) -> (Vec<u8>, Cache) {
    let files = |_: &str| None;
    let mut fonts = Fonts::new();
    let viewport = Viewport {
        width: SIDE,
        height: SIDE,
    };
    let laid = lay_out(root, viewport, &mut fonts, &Images { size_of: &|_| None });
    let mut surface = Surface::new(PIXELS, PIXELS).expect("a surface");
    surface.clear(Rgba::new(0.0, 0.0, 0.0, 1.0));
    let mut resources = Resources::with_cache(&files, fonts.faces(), cache);
    paint(root, &laid, viewport, &mut surface, &mut resources);
    let mut cache = resources.into_cache();
    cache.next_frame();
    (surface.rgba(), cache)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn a_zooming_layer_draws_the_same_from_a_warm_cache_as_from_a_cold_one(
        first in 0.6f64..2.5,
        second in 0.6f64..2.5,
        first_shift in -20.0f64..20.0,
        second_shift in -20.0f64..20.0,
    ) {
        let (_, warm) = drawn(&scene(first, first_shift, WillChange::Scale), Cache::default());
        let later = scene(second, second_shift, WillChange::Scale);
        let (from_warm, _) = drawn(&later, warm);
        let (from_cold, _) = drawn(&later, Cache::default());
        prop_assert!(from_warm == from_cold);
    }

    #[test]
    fn a_still_layer_draws_the_same_from_a_warm_cache_as_from_a_cold_one(
        scale in 0.6f64..2.5,
        shift in -20.0f64..20.0,
    ) {
        let still = scene(scale, shift, WillChange::Transform);
        let (_, warm) = drawn(&still, Cache::default());
        let (from_warm, _) = drawn(&still, warm);
        let (from_cold, _) = drawn(&still, Cache::default());
        prop_assert!(from_warm == from_cold);
    }
}

#[test]
fn a_zoom_inside_one_scale_step_draws_its_layer_once() {
    let mut cache = Cache::default();
    for scale in [1.02, 1.04, 1.06, 1.08] {
        cache = drawn(&scene(scale, 0.0, WillChange::Scale), cache).1;
    }
    assert_eq!(cache.layers_drawn(), 1);
}

#[test]
fn a_still_layer_is_drawn_once_while_it_holds() {
    let mut cache = Cache::default();
    for _ in 0..4 {
        cache = drawn(&scene(1.5, 4.0, WillChange::Transform), cache).1;
    }
    assert_eq!(cache.layers_drawn(), 1);
}
