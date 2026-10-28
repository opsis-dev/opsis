use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use opsis_compile::nodes::image::{Mark, MarkKind};
use opsis_image::marks::{MarkFonts, annotate};

const RED: [u8; 3] = [239, 68, 68];
const WHITE: [u8; 3] = [255, 255, 255];

fn blank() -> DynamicImage {
    DynamicImage::ImageRgb8(RgbImage::from_pixel(400, 200, Rgb(WHITE)))
}

fn mark(kind: MarkKind, from: (f64, f64), to: (f64, f64)) -> Mark {
    Mark {
        kind,
        x: from.0,
        y: from.1,
        x2: to.0,
        y2: to.1,
        ..Mark::default()
    }
}

fn at(image: &DynamicImage, x: u32, y: u32) -> [u8; 3] {
    let p = image.get_pixel(x, y).0;
    [p[0], p[1], p[2]]
}

fn fonts() -> MarkFonts {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../public/fonts/bundled/inter/inter-latin-wght-normal.woff2");
    let font = opsis_image::text::load_font(std::fs::read(file).unwrap(), 700.0).unwrap();
    MarkFonts::from([((String::new(), 700), font)])
}

#[test]
fn draws_an_arrow_with_a_shaft_and_a_head_at_its_tip() {
    let out = annotate(
        &blank(),
        &[mark(MarkKind::Arrow, (0.1, 0.5), (0.9, 0.5))],
        &MarkFonts::new(),
    );
    assert_eq!(at(&out, 200, 100), RED, "the shaft");
    assert_eq!(at(&out, 200, 92), WHITE, "the shaft is the line width");
    assert_eq!(at(&out, 340, 105), RED, "the head is wider than the shaft");
    assert_eq!(at(&out, 30, 100), WHITE, "nothing before the tail");
    assert_eq!(at(&out, 365, 100), WHITE, "nothing past the tip");
}

#[test]
fn draws_box_and_oval_outlines_at_their_line_width() {
    let area = ((0.25, 0.25), (0.75, 0.75));
    let boxed = annotate(
        &blank(),
        &[mark(MarkKind::Rectangle, area.0, area.1)],
        &MarkFonts::new(),
    );
    assert_eq!(at(&boxed, 100, 100), RED, "the left edge");
    assert_eq!(at(&boxed, 101, 50), RED, "the corner is square");
    assert_eq!(at(&boxed, 200, 100), WHITE, "inside stays");
    assert_eq!(at(&boxed, 92, 100), WHITE, "outside stays");
    let oval = annotate(
        &blank(),
        &[mark(MarkKind::Oval, area.0, area.1)],
        &MarkFonts::new(),
    );
    assert_eq!(at(&oval, 200, 50), RED, "the top of the oval");
    assert_eq!(at(&oval, 100, 100), RED, "the left of the oval");
    assert_eq!(at(&oval, 104, 54), WHITE, "the corner is round");
}

#[test]
fn numbers_steps_in_order_and_writes_text_in_the_chosen_font() {
    let step = |x: f64| Mark {
        size: 60,
        ..mark(MarkKind::Step, (x, 0.5), (x, 0.5))
    };
    let out = annotate(&blank(), &[step(0.25), step(0.75)], &fonts());
    assert_eq!(at(&out, 100 + 24, 100), RED, "inside the circle");
    assert_eq!(at(&out, 100 + 34, 100), WHITE, "outside the circle");
    let white_in = |cx: u32| {
        (cx - 12..cx + 12)
            .flat_map(|x| (88..112).map(move |y| (x, y)))
            .filter(|&(x, y)| at(&out, x, y) == WHITE)
            .count()
    };
    let (one, two) = (white_in(100), white_in(300));
    assert!(one > 20 && two > one, "a 1 then a wider 2: {one} {two}");
    let text = Mark {
        kind: MarkKind::Text,
        text: "Hello".to_owned(),
        color: "#000000".to_owned(),
        size: 40,
        ..mark(MarkKind::Text, (0.1, 0.1), (0.1, 0.1))
    };
    let written = annotate(&blank(), &[text], &fonts());
    let ink = (40..200)
        .flat_map(|x| (20..70).map(move |y| (x, y)))
        .filter(|&(x, y)| at(&written, x, y)[0] < 60)
        .count();
    assert!(ink > 300, "the text is written: {ink}");
    assert_eq!(at(&written, 300, 150), WHITE);
}
