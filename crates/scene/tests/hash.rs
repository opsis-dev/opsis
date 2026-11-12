use opsis_scene::color::Color;
use opsis_scene::hash::{content_hash, full_hash};
use opsis_scene::node::{Node, div, span};
use opsis_scene::style::Style;
use opsis_scene::values::Length;
use proptest::prelude::*;

fn card(width: f64, label: &str) -> Style {
    Style::default()
        .width(width)
        .height(40.0)
        .background_color(Color::rgb8(249, 115, 22))
        .font_family(vec![label.to_owned()])
}

fn tree(width: f64, label: &str, x: f64, opacity: f64) -> Node {
    div(card(width, label).left(x).opacity(opacity))
        .child(span(Style::default()).child(label))
        .into()
}

proptest! {
    #[test]
    fn moving_or_fading_keeps_the_content(width in 1.0f64..500.0, x in -500.0f64..500.0, dx in -500.0f64..500.0, opacity in 0.0f64..1.0) {
        let label = "Opsis";
        prop_assert_eq!(
            content_hash(&tree(width, label, x, 1.0)),
            content_hash(&tree(width, label, x + dx, opacity))
        );
    }

    #[test]
    fn drift_below_a_hundred_and_twenty_eighth_of_a_pixel_hashes_the_same(whole in -2_000i32..2_000, drift in -0.007f64..0.007) {
        let width = f64::from(whole) / 8.0;
        prop_assert_eq!(full_hash(&tree(width, "a", 0.0, 1.0)), full_hash(&tree(width + drift, "a", 0.0, 1.0)));
    }

    #[test]
    fn a_change_inside_changes_the_content(width in 1.0f64..500.0, grown in 0.5f64..100.0, label in "[a-z]{1,8}") {
        let base = content_hash(&tree(width, &label, 0.0, 1.0));
        prop_assert_ne!(base, content_hash(&tree(width + grown, &label, 0.0, 1.0)));
        prop_assert_ne!(base, content_hash(&tree(width, &format!("{label}!"), 0.0, 1.0)));
    }

    #[test]
    fn opacity_steps_follow_eight_bits(step in 0u8..=254) {
        let a = f64::from(step) / 255.0;
        let b = f64::from(step + 1) / 255.0;
        let node = |o: f64| -> Node { div(Style::default().opacity(o).width(Length::px(10.0))).into() };
        prop_assert_ne!(full_hash(&node(a)), full_hash(&node(b)));
        prop_assert_eq!(full_hash(&node(a)), full_hash(&node(a + 0.0005)));
    }
}
