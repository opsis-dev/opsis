use opsis_scene::node::Node;
use opsis_scene::shutter::{sample_opacity, shutter_layers, shutter_offsets};
use opsis_types::numbers::float;
use proptest::prelude::*;

proptest! {
    #[test]
    fn offsets_start_at_zero_and_stay_inside_the_open_shutter(angle in 0.0f64..720.0, samples in 0u32..32) {
        let offsets = shutter_offsets(angle, samples);
        let open = angle.clamp(0.0, 360.0) / 360.0;
        prop_assert!(offsets[0].abs() < f64::EPSILON);
        prop_assert!(offsets.iter().all(|&o| o >= 0.0 && (o < open || offsets.len() == 1)));
        prop_assert!(offsets.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn stacked_samples_weigh_the_same(samples in 1usize..32) {
        let weights: Vec<f64> = (0..samples)
            .map(|k| sample_opacity(k) * ((k + 1)..samples).map(|j| 1.0 - sample_opacity(j)).product::<f64>())
            .collect();
        for weight in weights {
            prop_assert!((weight - 1.0 / float(samples)).abs() < 1e-12);
        }
    }

    #[test]
    fn copies_of_a_partly_covered_pixel_grow_denser_as_they_stack(samples in 2usize..32, alpha in 0.01f64..0.99) {
        let stacked = (0..samples).fold(0.0, |under: f64, i| {
            let a = sample_opacity(i) * alpha;
            (1.0 - a).mul_add(under, a)
        });
        let expected = 1.0 - (0..samples).map(|i| sample_opacity(i).mul_add(-alpha, 1.0)).product::<f64>();
        prop_assert!((stacked - expected).abs() < 1e-12);
        prop_assert!(stacked > alpha);
    }

    #[test]
    fn every_sample_gets_its_own_layer_in_order(samples in 1usize..32) {
        let drawn: Vec<Vec<Node>> = (0..samples).map(|_| vec![Node::Text(String::new())]).collect();
        let layers = shutter_layers(&drawn);
        prop_assert_eq!(layers.len(), samples);
        for (i, layer) in layers.iter().enumerate() {
            let Node::Element(layer) = layer else {
                return Err(TestCaseError::fail("a layer is text"));
            };
            let opacity = opsis_scene::prop!(&layer.style, Opacity).map(|a| a.0);
            prop_assert_eq!(opacity, Some(sample_opacity(i)));
        }
    }
}
