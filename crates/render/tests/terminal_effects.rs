use opsis_compile::elements::terminal::effects::perlin_tables;
use opsis_film::renderer_program;
use opsis_layout::fonts::Fonts;
use opsis_layout::{Images, Viewport, lay_out};
use opsis_paint::{Pass, Programs, Resources, paint};
use opsis_renderer::{Driver, Renderer, Setting};
use opsis_scene::node::{Effect, EffectUniform, Element, Kind, Node, SvgElement, div, svg};
use opsis_scene::style::{Position, Style};
use opsis_scene::values::{Length, Num};
use opsis_skia::{Rgba, Surface};

const SIDE: (u32, u32) = (320, 180);
const SEED: f64 = 7.0;
const FREQUENCY: f64 = 0.85;
const MEAN_LIMIT: f32 = 0.02;

struct Child(Renderer);

impl Programs for Child {
    fn run(&mut self, pass: &Pass<'_>) -> Option<Vec<u8>> {
        let inputs: Vec<&[u8]> = pass.inputs.iter().map(Vec::as_slice).collect();
        let settings: Vec<Setting> = pass
            .uniforms
            .iter()
            .map(|s| Setting {
                name: s.name.clone(),
                values: s.values.clone(),
                columns: s.columns,
            })
            .collect();
        self.0.run(pass.program, &inputs, pass.size, &settings).ok()
    }
}

fn filled() -> Style {
    Style::default()
        .position(Position::Absolute)
        .left(Length::ZERO)
        .top(Length::ZERO)
        .width(Length::px(f64::from(SIDE.0)))
        .height(Length::px(f64::from(SIDE.1)))
}

fn turbulence_filter() -> Node {
    let filter = SvgElement::new("filter")
        .attr("id", "grain")
        .attr("x", 0.0)
        .attr("y", 0.0)
        .attr("width", "100%")
        .attr("height", "100%")
        .child(
            SvgElement::new("feTurbulence")
                .attr("type", "fractalNoise")
                .attr("baseFrequency", FREQUENCY)
                .attr("numOctaves", 1.0)
                .attr("seed", SEED)
                .attr("stitchTiles", "stitch"),
        )
        .child(
            SvgElement::new("feColorMatrix")
                .attr("type", "saturate")
                .attr("values", "0"),
        );
    let root = SvgElement::new("svg")
        .attr("width", f64::from(SIDE.0))
        .attr("height", f64::from(SIDE.1))
        .child(SvgElement::new("defs").child(filter))
        .child(
            SvgElement::new("rect")
                .attr("x", 0.0)
                .attr("y", 0.0)
                .attr("width", f64::from(SIDE.0))
                .attr("height", f64::from(SIDE.1))
                .attr("filter", "url(#grain)"),
        );
    div(filled()).child(svg(root, filled())).into()
}

fn uniform(name: &str, values: Vec<f64>, columns: u32) -> EffectUniform {
    EffectUniform {
        name: name.to_owned(),
        values: values.into_iter().map(Num).collect(),
        columns,
    }
}

fn grain_effect() -> Node {
    let tables = perlin_tables(SEED);
    let effect = Element::new(
        Kind::Effect(Effect {
            program: "terminal-grain".to_owned(),
            uniforms: vec![
                uniform("frequency", vec![FREQUENCY, FREQUENCY], 2),
                uniform("colour", vec![0.0], 1),
                uniform("lattice", tables.lattice, 4),
                uniform("gradients", tables.gradients, 4),
            ],
            scale: Num(1.0),
        }),
        filled(),
    );
    div(filled()).child(effect).into()
}

fn drawn(root: &Node, programs: Option<&mut Child>) -> Vec<u8> {
    let files = |_: &str| None;
    let mut fonts = Fonts::new();
    let viewport = Viewport {
        width: f64::from(SIDE.0),
        height: f64::from(SIDE.1),
    };
    let laid = lay_out(root, viewport, &mut fonts, &Images { size_of: &|_| None });
    let mut surface = Surface::new(SIDE.0, SIDE.1).unwrap();
    surface.clear(Rgba::new(0.0, 0.0, 0.0, 1.0));
    let mut resources = Resources::new(&files, fonts.faces());
    if let Some(programs) = programs {
        resources = resources.with_programs(programs);
    }
    paint(root, &laid, viewport, &mut surface, &mut resources);
    surface.rgba()
}

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|[r, g, b, _]| [*r, *g, *b])
        .collect()
}

#[test]
#[ignore = "needs the opsis-renderer program and a graphics driver"]
fn the_grain_pass_draws_skias_turbulence() {
    let program = renderer_program().expect("build opsis-renderer first");
    let temp = opsis_golden::dev_cache("terminal-effects");
    let mut child = Child(Renderer::start(&program, &Driver::Card, &temp).unwrap());
    let want = drawn(&turbulence_filter(), None);
    let got = drawn(&grain_effect(), Some(&mut child));
    std::fs::create_dir_all(&temp).unwrap();
    let a = nv_flip::FlipImageRgb8::with_data(SIDE.0, SIDE.1, &rgb(&want));
    let b = nv_flip::FlipImageRgb8::with_data(SIDE.0, SIDE.1, &rgb(&got));
    let mean =
        nv_flip::FlipPool::from_image(&nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE))
            .mean();
    image::RgbaImage::from_raw(SIDE.0, SIDE.1, want)
        .unwrap()
        .save(temp.join("skia.png"))
        .unwrap();
    image::RgbaImage::from_raw(SIDE.0, SIDE.1, got)
        .unwrap()
        .save(temp.join("pass.png"))
        .unwrap();
    assert!(mean <= MEAN_LIMIT, "FLIP mean {mean}");
}

const BULGE: f64 = 40.0;
const SHIFT: (f64, f64) = (3.0, 1.0);
const RED_ONLY: &str = "1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0";
const GREEN_ONLY: &str = "0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0";
const BLUE_ONLY: &str = "0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0";

fn picture() -> Node {
    let stripes = (0..8).map(|i| {
        let colours = [
            "#f43f5e", "#22c55e", "#3b82f6", "#facc15", "#ffffff", "#a855f7", "#0ea5e9", "#f97316",
        ];
        SvgElement::new("rect")
            .attr("x", f64::from(i) * 40.0)
            .attr("y", f64::from(i % 3).mul_add(30.0, 20.0))
            .attr("width", 24.0)
            .attr("height", 120.0)
            .attr("fill", colours[usize::try_from(i).unwrap()])
    });
    let mut root = SvgElement::new("svg")
        .attr("width", f64::from(SIDE.0))
        .attr("height", f64::from(SIDE.1));
    for stripe in stripes {
        root = root.child(stripe);
    }
    svg(root, filled()).into()
}

fn lens_filter() -> SvgElement {
    let channel = |values: &str, result: &str| {
        SvgElement::new("feColorMatrix")
            .attr("in", "bent")
            .attr("values", values)
            .attr("result", result)
    };
    let offset = |input: &str, x: f64, y: f64, result: &str| {
        SvgElement::new("feOffset")
            .attr("in", input)
            .attr("dx", x)
            .attr("dy", y)
            .attr("result", result)
    };
    SvgElement::new("filter")
        .attr("id", "lens")
        .attr("x", 0.0)
        .attr("y", 0.0)
        .attr("width", f64::from(SIDE.0))
        .attr("height", f64::from(SIDE.1))
        .attr("filterUnits", "userSpaceOnUse")
        .attr("color-interpolation-filters", "sRGB")
        .child(
            SvgElement::new("feImage")
                .attr(
                    "href",
                    opsis_compile::elements::terminal::effects::bulge_map(0.5, 0.5),
                )
                .attr("x", 0.0)
                .attr("y", 0.0)
                .attr("width", f64::from(SIDE.0))
                .attr("height", f64::from(SIDE.1))
                .attr("preserveAspectRatio", "none")
                .attr("result", "map"),
        )
        .child(
            SvgElement::new("feDisplacementMap")
                .attr("in", "SourceGraphic")
                .attr("in2", "map")
                .attr("scale", BULGE)
                .attr("xChannelSelector", "R")
                .attr("yChannelSelector", "G")
                .attr("result", "bent"),
        )
        .child(channel(RED_ONLY, "red"))
        .child(offset("red", SHIFT.0, SHIFT.1, "redShifted"))
        .child(channel(GREEN_ONLY, "green"))
        .child(channel(BLUE_ONLY, "blue"))
        .child(offset("blue", -SHIFT.0, -SHIFT.1, "blueShifted"))
        .child(
            SvgElement::new("feBlend")
                .attr("in", "redShifted")
                .attr("in2", "green")
                .attr("mode", "screen")
                .attr("result", "redGreen"),
        )
        .child(
            SvgElement::new("feBlend")
                .attr("in", "redGreen")
                .attr("in2", "blueShifted")
                .attr("mode", "screen"),
        )
}

fn lensed_by_filter() -> Node {
    let defs = SvgElement::new("svg")
        .attr("width", 0.0)
        .attr("height", 0.0)
        .child(SvgElement::new("defs").child(lens_filter()));
    div(filled())
        .child(svg(defs, Style::default().position(Position::Absolute)))
        .child(
            div(filled().filter(vec![opsis_scene::paint::Filter::Url("#lens".to_owned())]))
                .child(picture()),
        )
        .into()
}

fn lensed_by_pass() -> Node {
    let map = opsis_scene::node::img(
        opsis_compile::elements::terminal::effects::bulge_map(0.5, 0.5),
        filled().object_fit(opsis_scene::style::ObjectFit::Fill),
    );
    let effect = Element::new(
        Kind::Effect(Effect {
            program: "terminal-lens".to_owned(),
            uniforms: vec![
                uniform("shift", vec![SHIFT.0, SHIFT.1], 2),
                uniform("bulge", vec![BULGE], 1),
            ],
            scale: Num(1.0),
        }),
        filled(),
    )
    .child(picture())
    .child(map);
    div(filled()).child(effect).into()
}

#[test]
#[ignore = "needs the opsis-renderer program and a graphics driver"]
fn the_lens_pass_draws_the_svg_lens() {
    let program = renderer_program().expect("build opsis-renderer first");
    let temp = opsis_golden::dev_cache("terminal-effects");
    let mut child = Child(Renderer::start(&program, &Driver::Card, &temp).unwrap());
    let want = drawn(&lensed_by_filter(), None);
    let got = drawn(&lensed_by_pass(), Some(&mut child));
    std::fs::create_dir_all(&temp).unwrap();
    let a = nv_flip::FlipImageRgb8::with_data(SIDE.0, SIDE.1, &rgb(&want));
    let b = nv_flip::FlipImageRgb8::with_data(SIDE.0, SIDE.1, &rgb(&got));
    let mean =
        nv_flip::FlipPool::from_image(&nv_flip::flip(a, b, nv_flip::DEFAULT_PIXELS_PER_DEGREE))
            .mean();
    image::RgbaImage::from_raw(SIDE.0, SIDE.1, want)
        .unwrap()
        .save(temp.join("lens-skia.png"))
        .unwrap();
    image::RgbaImage::from_raw(SIDE.0, SIDE.1, got)
        .unwrap()
        .save(temp.join("lens-pass.png"))
        .unwrap();
    assert!(mean <= MEAN_LIMIT, "FLIP mean {mean}");
}
