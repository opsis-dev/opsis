use opsis_compile::elements::camera::annotations::{Annotation, Shown, annotations};
use opsis_compile::elements::camera::window::lift_filters;
use opsis_scene::node::Node;
use opsis_scene::prop;
use opsis_types::actions::{Rect, Viewport};
use opsis_types::recording::{AnnotationKind, AnnotationSpec, HighlightLook, LabelLook};

fn annotation(kind: AnnotationKind, from_s: f64, to_s: f64) -> Annotation {
    Annotation {
        id: format!("{kind:?}"),
        kind,
        rect: Rect {
            x: 100.0,
            y: 80.0,
            w: 120.0,
            h: 40.0,
        },
        from_s,
        to_s,
        spec: AnnotationSpec {
            kind,
            look: None,
            roughness: None,
            match_text: String::new(),
            text: "Save".to_owned(),
            color: "#f97316".to_owned(),
            padding: 4.0,
            lead_ms: 0.0,
            hold_ms: 0.0,
            text_color: None,
            font_size: Some(16.0),
            font: None,
            label_look: Some(LabelLook {
                font_weight: 600.0,
                box_padding: 8.0,
                radius: 8.0,
                pointer_size: 6.0,
                shadow: true,
                arrow_size: 1.0,
                line_width: 3.0,
                tilt: -4.0,
            }),
            mark_look: None,
        },
        look: HighlightLook::Clean,
        roughness: 1.0,
        seed: 1,
    }
}

const fn shown(t: f64, lift: bool) -> Shown {
    Shown {
        viewport: Viewport {
            width: 1280,
            height: 800,
        },
        area: (1280.0, 800.0),
        t,
        camera_scale: 1.0,
        k: 1.0,
        lift,
    }
}

fn filters_of(node: &Node) -> Vec<opsis_scene::paint::Filter> {
    match node {
        Node::Element(element) => prop!(&element.style, Filter).cloned().unwrap_or_default(),
        Node::Text(_) => Vec::new(),
    }
}

fn marks() -> Vec<Annotation> {
    [
        AnnotationKind::Highlight,
        AnnotationKind::Tooltip,
        AnnotationKind::Callout,
    ]
    .into_iter()
    .map(|kind| annotation(kind, 1.0, 3.0))
    .collect()
}

#[test]
fn every_shown_mark_carries_the_lift_and_the_box_around_them_none() {
    let drawn = annotations(&marks(), &shown(2.0, true)).unwrap().unwrap();
    assert_eq!(filters_of(&drawn).len(), 0);
    let Node::Element(around) = &drawn else {
        panic!("the marks have no box");
    };
    assert_eq!(around.children.len(), 3);
    for mark in &around.children {
        assert!(filters_of(mark).ends_with(&lift_filters()));
    }
    let plain = annotations(&marks(), &shown(2.0, false)).unwrap().unwrap();
    let Node::Element(plain) = &plain else {
        panic!("the marks have no box");
    };
    assert!(
        plain
            .children
            .iter()
            .all(|mark| filters_of(mark).is_empty())
    );
}

#[test]
fn marks_that_are_all_hidden_add_nothing() {
    assert_eq!(annotations(&marks(), &shown(8.0, true)).unwrap(), None);
    assert_eq!(annotations(&marks(), &shown(0.0, true)).unwrap(), None);
}
