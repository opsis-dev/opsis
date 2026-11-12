use opsis_skia::{
    Axis, FontSpec, Gradient, GradientSpace, Image, ImageFilter, Lottie, Paint, PointF, RectF,
    Rgba, Sampling, Shader, Surface, TileMode, Typeface,
};

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

const fn pixel(rgba: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let at = (y * width + x) * 4;
    [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
}

#[test]
fn a_rectangle_is_drawn_over_the_cleared_picture() {
    let mut surface = Surface::new(64, 32).expect("a surface");
    surface.clear(Rgba::new(0.0, 0.0, 1.0, 1.0));
    surface.fill_rect(
        RectF::new(10.0, 8.0, 20.0, 16.0),
        Rgba::new(1.0, 0.0, 0.0, 1.0),
    );
    let rgba = surface.rgba();
    assert_eq!(rgba.len(), 64 * 32 * 4);
    assert_eq!(pixel(&rgba, 64, 20, 16), [255, 0, 0, 255]);
    assert_eq!(pixel(&rgba, 64, 2, 2), [0, 0, 255, 255]);
    assert_eq!(pixel(&rgba, 64, 40, 16), [0, 0, 255, 255]);
    assert!(surface.png().starts_with(&PNG_SIGNATURE));
}

#[test]
fn an_empty_picture_is_refused() {
    assert!(Surface::new(0, 10).is_err());
}

fn repo_file(path: &str) -> Vec<u8> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read(root.join(path)).expect("a repository file")
}

fn coverage(rgba: &[u8]) -> usize {
    rgba.chunks(4).filter(|p| p[0] > 128).count()
}

#[test]
fn glyphs_from_font_bytes_are_drawn() {
    let bytes = repo_file("public/fonts/bundled/terminal/opsis-terminal/JetBrainsMono-wght.ttf");
    let typeface = Typeface::from_bytes(&bytes, 0, &[Axis::new(*b"wght", 700.0)])
        .expect("a typeface from bytes");
    let mut surface = Surface::new(64, 32).expect("a surface");
    surface.clear(Rgba::new(0.0, 0.0, 0.0, 1.0));
    let glyphs = [36_u16, 37, 38];
    let positions = [
        PointF::new(0.0, 0.0),
        PointF::new(14.0, 0.0),
        PointF::new(28.0, 0.0),
    ];
    surface.draw_glyphs(
        &typeface,
        FontSpec::sized(24.0),
        (&glyphs, &positions),
        PointF::new(4.0, 24.0),
        &Paint::filled(Rgba::new(1.0, 1.0, 1.0, 1.0)),
    );
    assert!(coverage(&surface.rgba()) > 50);
    let outlines = typeface.outlines(
        FontSpec::sized(24.0),
        &glyphs,
        &positions,
        PointF::new(4.0, 24.0),
    );
    assert!(outlines.bounds().width > 20.0);
}

#[test]
fn an_unreadable_font_is_refused() {
    assert!(Typeface::from_bytes(b"not a font", 0, &[]).is_none());
}

#[test]
fn a_gradient_runs_from_its_first_colour_to_its_last() {
    let mut surface = Surface::new(100, 10).expect("a surface");
    let shader = Shader::linear(
        PointF::new(0.0, 0.0),
        PointF::new(100.0, 0.0),
        &[Rgba::new(1.0, 0.0, 0.0, 1.0), Rgba::new(0.0, 0.0, 1.0, 1.0)],
        &[],
        Gradient::new(GradientSpace::Oklch),
    )
    .expect("a gradient");
    let mut paint = Paint::new();
    paint.set_shader(&shader);
    surface.draw_rect(RectF::new(0.0, 0.0, 100.0, 10.0), &paint);
    let rgba = surface.rgba();
    let first = pixel(&rgba, 100, 0, 5);
    let last = pixel(&rgba, 100, 99, 5);
    assert!(first[0] > 240 && first[2] < 20);
    assert!(last[2] > 240 && last[0] < 20);
}

#[test]
fn a_decoded_picture_is_drawn_and_blurred() {
    let png = {
        let mut source = Surface::new(8, 8).expect("a surface");
        source.clear(Rgba::new(0.0, 1.0, 0.0, 1.0));
        source.png()
    };
    let image = Image::decode(&png).expect("a decoded picture");
    assert_eq!((image.width(), image.height()), (8, 8));
    let mut surface = Surface::new(32, 32).expect("a surface");
    let mut paint = Paint::new();
    paint.set_image_filter(&ImageFilter::blur(
        2.0,
        2.0,
        TileMode::Decal,
        &ImageFilter::source(),
    ));
    surface.draw_image_rect(
        &image,
        (
            RectF::new(0.0, 0.0, 8.0, 8.0),
            RectF::new(8.0, 8.0, 16.0, 16.0),
        ),
        Sampling::Linear,
        &paint,
    );
    let rgba = surface.rgba();
    assert_eq!(pixel(&rgba, 32, 16, 16)[1], 255);
    let edge = pixel(&rgba, 32, 7, 16);
    assert!(edge[3] > 0 && edge[3] < 255);
}

const SQUARE_ANIMATION: &str = r#"{"v":"5.7.4","fr":30,"ip":0,"op":30,"w":100,"h":100,"layers":[{"ty":4,"ip":0,"op":30,"st":0,"ks":{"o":{"a":0,"k":100},"r":{"a":0,"k":0},"p":{"a":0,"k":[50,50]},"a":{"a":0,"k":[0,0]},"s":{"a":0,"k":[100,100]}},"shapes":[{"ty":"rc","p":{"a":0,"k":[0,0]},"s":{"a":0,"k":[40,40]},"r":{"a":0,"k":0}},{"ty":"fl","c":{"a":0,"k":[1,0,0,1]},"o":{"a":0,"k":100}}]}]}"#;

#[test]
fn an_animation_file_is_played() {
    let mut animation = Lottie::from_json(SQUARE_ANIMATION.as_bytes()).expect("an animation");
    assert_eq!(animation.size(), (100.0, 100.0));
    assert_eq!(animation.frames(), (0.0, 30.0));
    animation.seek(10.0);
    let mut surface = Surface::new(100, 100).expect("a surface");
    surface.draw_lottie(&animation, RectF::new(0.0, 0.0, 100.0, 100.0));
    let rgba = surface.rgba();
    assert_eq!(pixel(&rgba, 100, 50, 50), [255, 0, 0, 255]);
    assert_eq!(pixel(&rgba, 100, 5, 5)[3], 0);
}

#[test]
fn a_file_that_is_not_an_animation_is_refused() {
    assert!(Lottie::from_json(b"{}").is_none());
}
