use opsis_compile::nodes::image::{
    BeforeAfter, CropImage, ImageGrid, Magnify, Rect, ResizeImage, RotateImage, area_rect,
    before_after_plan, crop_rect, grid_plan, magnify_plan, resized_size, rotated_size,
};
use opsis_compile::nodes::image_frame::{FrameImage, framed_placement};
use opsis_types::numbers::exact_eq;
use proptest::prelude::*;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn settings<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn picture() -> impl Strategy<Value = (f64, f64)> {
    (1u32..4000, 1u32..4000).prop_map(|(w, h)| (f64::from(w), f64::from(h)))
}

fn area() -> impl Strategy<Value = Value> {
    (-0.5..1.5f64, -0.5..1.5f64, 0.0..1.5f64, 0.0..1.5f64).prop_map(
        |(x, y, width, height)| json!({ "x": x, "y": y, "width": width, "height": height }),
    )
}

fn kept_inside(source: (f64, f64), rect: &Rect) -> bool {
    rect.x >= 0.0
        && rect.y >= 0.0
        && rect.width >= 1.0
        && rect.height >= 1.0
        && rect.x + rect.width <= source.0 + 1e-9
        && rect.y + rect.height <= source.1 + 1e-9
}

fn whole(rect: &Rect) -> bool {
    [rect.x, rect.y, rect.width, rect.height]
        .iter()
        .all(|v| v.fract() == 0.0)
}

fn apart(a: &Rect, b: &Rect) -> bool {
    a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y
}

proptest! {
    #[test]
    fn keeps_every_drawn_area_and_crop_inside_the_picture(
        source in picture(),
        drawn in area(),
        margins in (0u32..3000, 0u32..3000, 0u32..3000, 0u32..3000),
    ) {
        let rect = area_rect(source, &settings(drawn.clone()));
        prop_assert!(kept_inside(source, &rect) && whole(&rect), "{rect:?}");
        let by_area: CropImage = settings(json!({ "mode": "area", "area": [drawn] }));
        prop_assert!(kept_inside(source, &crop_rect(source, &by_area)));
        let (left, top, right, bottom) = margins;
        let by_margins: CropImage = settings(json!({ "mode": "margins", "left": left, "top": top, "right": right, "bottom": bottom }));
        let rect = crop_rect(source, &by_margins);
        prop_assert!(kept_inside(source, &rect) && whole(&rect), "{rect:?}");
    }

    #[test]
    fn resizes_to_the_asked_side_and_keeps_the_proportion(
        source in picture(),
        width in 0u32..5000,
        height in 0u32..5000,
        fit in prop::sample::select(vec!["inside", "cover", "exact"]),
    ) {
        let p: ResizeImage = settings(json!({ "width": width, "height": height, "fit": fit }));
        let (w, h) = resized_size(source, &p);
        let (want_w, want_h) = (f64::from(width), f64::from(height));
        prop_assert!(w >= 1.0 && h >= 1.0 && w.fract() == 0.0 && h.fract() == 0.0);
        match (width > 0, height > 0) {
            (false, false) => prop_assert_eq!((w, h), source),
            (true, false) => prop_assert!(exact_eq(w, want_w) && (h - source.1 * want_w / source.0).abs() <= 0.5),
            (false, true) => prop_assert!(exact_eq(h, want_h) && (w - source.0 * want_h / source.1).abs() <= 0.5),
            (true, true) if fit == "inside" => {
                prop_assert!(w <= want_w + 0.5 && h <= want_h + 0.5);
                prop_assert!((w - want_w).abs() <= 0.5 || (h - want_h).abs() <= 0.5);
            }
            (true, true) => prop_assert_eq!((w, h), (want_w, want_h)),
        }
    }

    #[test]
    fn scales_by_percent_and_never_enlarges_when_asked(
        source in picture(),
        scale in 1u32..1000,
        width in 0u32..5000,
        height in 0u32..5000,
    ) {
        let by_percent: ResizeImage = settings(json!({ "by": "percent", "scale": scale }));
        let (w, h) = resized_size(source, &by_percent);
        let k = f64::from(scale) / 100.0;
        prop_assert!((w - (source.0 * k).round().max(1.0)).abs() < 1e-9);
        prop_assert!((h - (source.1 * k).round().max(1.0)).abs() < 1e-9);
        let capped: ResizeImage = settings(json!({ "width": width, "height": height, "shrinkOnly": true }));
        let (w, h) = resized_size(source, &capped);
        prop_assert!(w <= source.0.max(1.0) + 0.5 && h <= source.1.max(1.0) + 0.5, "{w} {h} {source:?}");
    }

    #[test]
    fn cuts_off_the_corners_to_a_part_inside_the_turned_picture(
        source in picture(),
        angle in -45.0..45.0f64,
    ) {
        prop_assume!(angle.abs() > 0.01);
        let p: RotateImage = settings(json!({ "turn": "none", "angle": angle, "cropCorners": true }));
        let (w, h) = rotated_size(source, &p);
        prop_assert!(w <= source.0 && h <= source.1, "{w} {h} {source:?}");
        let (sin, cos) = angle.to_radians().sin_cos();
        let (sin, cos) = (sin.abs(), cos.abs());
        prop_assert!(f64::mul_add(h, sin, w * cos) <= source.0 + 1.0 && f64::mul_add(h, cos, w * sin) <= source.1 + 1.0);
    }

    #[test]
    fn turns_onto_a_canvas_that_just_holds_the_turned_picture(
        source in picture(),
        turn in prop::sample::select(vec!["none", "right", "half", "left"]),
        angle in -45.0..45.0f64,
    ) {
        let p: RotateImage = settings(json!({ "turn": turn, "angle": angle }));
        let (w, h) = rotated_size(source, &p);
        let (tw, th) = if turn == "right" || turn == "left" { (source.1, source.0) } else { source };
        let (sin, cos) = angle.to_radians().sin_cos();
        let exact = (f64::mul_add(th, sin.abs(), tw * cos.abs()), f64::mul_add(th, cos.abs(), tw * sin.abs()));
        prop_assert!(w >= exact.0 - 1e-6 && w < exact.0 + 1.0, "{w} {exact:?}");
        prop_assert!(h >= exact.1 - 1e-6 && h < exact.1 + 1.0, "{h} {exact:?}");
    }

    #[test]
    fn lays_images_out_apart_inside_the_canvas_and_never_enlarged(
        sizes in prop::collection::vec(picture(), 1..9),
        layout in prop::sample::select(vec!["row", "column", "grid"]),
        fit in prop::sample::select(vec!["same-height", "same-width", "cells"]),
        columns in 1u32..5,
        gap in 0u32..60,
        padding in 0u32..60,
    ) {
        let p: ImageGrid = settings(json!({ "layout": layout, "fit": fit, "columns": columns, "gap": gap, "padding": padding }));
        let plan = grid_plan(&sizes, &p);
        prop_assert_eq!(plan.cells.len(), sizes.len());
        prop_assert!(plan.columns * plan.rows >= sizes.len());
        for (cell, size) in plan.cells.iter().zip(&sizes) {
            prop_assert!(cell.width <= size.0 && cell.height <= size.1, "{cell:?} {size:?}");
            prop_assert!(cell.x >= f64::from(padding) - 0.5 && cell.y >= f64::from(padding) - 0.5);
            prop_assert!(cell.x + cell.width <= plan.canvas.0 - f64::from(padding) + 0.5);
            prop_assert!(cell.y + cell.height <= plan.canvas.1 - f64::from(padding) + 0.5);
        }
        for (i, a) in plan.cells.iter().enumerate() {
            for b in &plan.cells[i + 1..] {
                prop_assert!(apart(a, b), "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn frames_a_picture_with_its_padding_on_a_canvas_of_the_chosen_shape(
        source in picture(),
        padding in 0.0..50.0f64,
        shape in prop::sample::select(vec!["keep", "wide", "square", "portrait", "story"]),
    ) {
        let p: FrameImage = settings(json!({ "padding": padding, "shape": shape }));
        let ((w, h), (x, y)) = framed_placement(source, &p);
        let pad = (source.0.min(source.1) * padding / 100.0).round();
        prop_assert!(x >= pad - 0.5 && y >= pad - 0.5);
        prop_assert!(x + source.0 <= w - pad + 0.5 && y + source.1 <= h - pad + 0.5);
        let ratio = match shape {
            "wide" => Some(16.0 / 9.0),
            "square" => Some(1.0),
            "portrait" => Some(4.0 / 5.0),
            "story" => Some(9.0 / 16.0),
            _ => None,
        };
        if let Some(ratio) = ratio {
            prop_assert!((w / h - ratio).abs() <= 1.0 / h.min(w), "{w}×{h}");
        }
    }

    #[test]
    fn places_both_pictures_inside_the_before_and_after_canvas(
        before in picture(),
        after in picture(),
        layout in prop::sample::select(vec!["split", "side-by-side", "stacked"]),
        divider in 0u32..40,
    ) {
        let p: BeforeAfter = settings(json!({ "layout": layout, "dividerWidth": divider }));
        let plan = before_after_plan(before, after, &p);
        let canvas = (f64::from(plan.size.0), f64::from(plan.size.1));
        prop_assert!(kept_inside(canvas, &plan.before) && kept_inside(canvas, &plan.after));
        prop_assert_eq!((plan.before.width, plan.before.height), before);
        if layout != "split" {
            prop_assert!(apart(&plan.before, &plan.after));
        }
    }

    #[test]
    fn magnifies_an_area_inside_the_picture_into_a_view_inside_it_too_and_never_shrinks_it(
        source in (50u32..4000, 50u32..4000).prop_map(|(w, h)| (f64::from(w), f64::from(h))),
        drawn in area(),
        shown in area(),
        zoom in 0.5..8.0f64,
        size_from in prop::sample::select(vec!["two-boxes", "area-and-zoom", "shown-and-zoom"]),
        shape in prop::sample::select(vec!["circle", "rounded"]),
    ) {
        let p: Magnify = settings(json!({ "sizeFrom": size_from, "area": [drawn], "shown": [shown], "zoom": zoom, "shape": shape }));
        let plan = magnify_plan(source, &p);
        prop_assert!(plan.zoom >= 1.0 - 1e-9, "{}", plan.zoom);
        if size_from != "two-boxes" {
            prop_assert!(plan.zoom <= zoom.max(1.0) + 1e-9, "{}", plan.zoom);
        }
        let inside = |r: &Rect| r.x >= -0.5 && r.y >= -0.5 && r.x + r.width <= source.0 + 0.5 && r.y + r.height <= source.1 + 0.5;
        prop_assert!(inside(&plan.area), "{:?}", plan.area);
        prop_assert!(inside(&plan.shown), "{:?}", plan.shown);
    }
}
