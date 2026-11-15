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

fn panned(scale: f64, shift: f64) -> Node {
    let layer = placed(40.0, 40.0, 60.0)
        .transform(vec![
            Transform::Translate(Length::px(shift), Length::px(shift / 2.0), Length::ZERO),
            Transform::Scale(Num(scale), Num(scale), Num(1.0)),
        ])
        .will_change(WillChange::Transform);
    div(placed(0.0, 0.0, SIDE))
        .child(
            div(layer)
                .child(div(
                    placed(5.0, 5.0, 20.0).background_color(Color::rgb8(249, 115, 22))
                ))
                .child(div(
                    placed(30.0, 25.0, 25.0).background_color(Color::rgb8(14, 165, 233))
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
    fn a_panning_layer_draws_the_same_from_a_warm_cache_as_from_a_cold_one(
        scale in 0.6f64..1.4,
        first in -8.0f64..8.0,
        second in -8.0f64..8.0,
    ) {
        let (_, warm) = drawn(&panned(scale, first), Cache::default());
        let later = panned(scale, second);
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

#[test]
fn a_pan_at_one_scale_draws_its_layer_once() {
    let mut cache = Cache::default();
    for shift in [0.0, 1.3, 2.7, 4.1, -3.4] {
        cache = drawn(&panned(1.25, shift), cache).1;
    }
    assert_eq!(cache.layers_drawn(), 1);
}

#[test]
fn a_pan_zoomed_in_past_the_picture_draws_its_layer_once() {
    let mut cache = Cache::default();
    for shift in [0.0, -2.5, -5.0, -7.5, -10.0, -12.5, -15.0, -17.5] {
        cache = drawn(&panned(3.0, shift), cache).1;
    }
    assert_eq!(cache.layers_drawn(), 1);
}

fn stacked_sample(tag: i32) -> Node {
    let window = placed(30.0, 30.0, 90.0)
        .border_radius_all(14.0)
        .background_color(Color::rgb8(250, 250, 250))
        .z_index(tag)
        .transform(vec![Transform::Rotate(opsis_scene::values::Angle::Deg(
            Num(7.0),
        ))])
        .will_change(WillChange::Transform);
    div(placed(0.0, 0.0, SIDE))
        .child(div(window).child(div(
            placed(10.0, 10.0, 30.0).background_color(Color::rgb8(14, 165, 233)),
        )))
        .into()
}

fn shutter(tags: &[i32]) -> Node {
    let samples: Vec<Vec<Node>> = tags.iter().map(|t| vec![stacked_sample(*t)]).collect();
    div(placed(0.0, 0.0, SIDE).background_color(Color::rgb8(20, 20, 28)))
        .children(opsis_scene::shutter::shutter_layers(&samples))
        .into()
}

#[test]
fn samples_that_share_one_raster_stack_to_the_same_edges_as_eight_rasters() {
    let (shared, cache) = drawn(&shutter(&[0; 8]), Cache::default());
    assert_eq!(cache.layers_drawn(), 1);
    let (apart, cache) = drawn(&shutter(&[0, 1, 2, 3, 4, 5, 6, 7]), Cache::default());
    assert_eq!(cache.layers_drawn(), 8);
    assert!(shared == apart);
}
