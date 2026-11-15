use opsis_layout::Tag;
use opsis_layout::output::{Placed, PlacedKind, Rect};
use opsis_scene::node::Kind;
use opsis_scene::paint::Filter;
use opsis_scene::prop;
use opsis_scene::style::{Overflow, Style};
use opsis_scene::values::Length;

use crate::geometry::{outset, px};
use crate::{Affine, Painter, element_at};

const INK_MARGIN: f64 = 2.0;

enum SvgInk {
    Inside,
    Drawn(Rect),
    Unknown,
}
const SHADOW_BLURS: f64 = 2.0;
const BLUR_REACH: f64 = 3.0;

impl Painter<'_, '_, '_> {
    fn length(&self, placed: &Placed, length: &Length) -> f64 {
        px(length, &placed.text, &self.context, 0.0)
    }

    fn reach_of(&self, placed: &Placed, style: &Style) -> f64 {
        let Some(filters) = prop!(style, Filter) else {
            return 0.0;
        };
        filters
            .iter()
            .map(|filter| match filter {
                Filter::Blur(radius) => BLUR_REACH * self.length(placed, radius),
                Filter::DropShadow(shadow) => BLUR_REACH.mul_add(
                    self.length(placed, &shadow.blur),
                    self.length(placed, &shadow.x)
                        .abs()
                        .max(self.length(placed, &shadow.y).abs()),
                ),
                _ => 0.0,
            })
            .sum()
    }

    fn overhang(&self, placed: &Placed, style: &Style) -> f64 {
        let shadows = prop!(style, BoxShadow).map_or(0.0, |shadows| {
            shadows
                .iter()
                .filter(|s| !s.inset)
                .map(|s| {
                    let offset = self
                        .length(placed, &s.x)
                        .abs()
                        .max(self.length(placed, &s.y).abs());
                    SHADOW_BLURS.mul_add(
                        self.length(placed, &s.blur).max(0.0),
                        offset + self.length(placed, &s.spread).max(0.0),
                    )
                })
                .fold(0.0, f64::max)
        });
        let outline = prop!(style, Outline).map_or(0.0, |o| {
            self.length(placed, &o.width).max(0.0) + self.length(placed, &o.offset).abs()
        });
        shadows.max(outline)
    }

    fn svg_overflow(&mut self, placed: &Placed) -> SvgInk {
        let Some(element) = element_at(self.root, &placed.path) else {
            return SvgInk::Unknown;
        };
        let Kind::Svg(svg) = &element.kind else {
            return SvgInk::Unknown;
        };
        let visible = prop!(&element.style, OverflowX) == Some(&Overflow::Visible)
            || svg
                .attributes
                .iter()
                .any(|(k, v)| k == "overflow" && v == "visible");
        if !visible {
            return SvgInk::Inside;
        }
        let (document, content) = self.svg_document(placed, &element.style, svg);
        let Some(tree) = self.resources.markup(document) else {
            return SvgInk::Unknown;
        };
        let drawn = tree.root().abs_layer_bounding_box();
        let local = Rect {
            x: content.x + f64::from(drawn.x()),
            y: content.y + f64::from(drawn.y()),
            width: f64::from(drawn.width()),
            height: f64::from(drawn.height()),
        };
        SvgInk::Drawn(placed.matrix.bounds(outset(local, INK_MARGIN)))
    }

    fn drawn_within_box(&self, placed: &Placed) -> Option<f64> {
        match &placed.kind {
            PlacedKind::Text { .. } => {
                let shadows: f64 = placed
                    .text
                    .text_shadow
                    .iter()
                    .map(|s| {
                        let offset = self
                            .length(placed, &s.x)
                            .abs()
                            .max(self.length(placed, &s.y).abs());
                        SHADOW_BLURS.mul_add(self.length(placed, &s.blur).max(0.0), offset)
                    })
                    .fold(0.0, f64::max);
                Some(placed.text.font_size + shadows)
            }
            PlacedKind::Element(Tag::Div | Tag::Span | Tag::Image | Tag::Svg) => Some(0.0),
            PlacedKind::Element(Tag::Canvas | Tag::Leaf) => None,
        }
    }

    pub(crate) fn ink(&mut self, path: &[usize]) -> Option<Rect> {
        let mut seen: Option<Rect> = None;
        let mut reach = 0.0f64;
        let laid = self.laid;
        for placed in laid.boxes.iter().filter(|b| b.path.starts_with(path)) {
            let mut grow = self.drawn_within_box(placed)?;
            let scale = Affine::of(&placed.matrix)?.scale();
            if let Some(style) = placed.style.as_ref() {
                if prop!(style, BackdropFilter).is_some_and(|f| !f.is_empty()) {
                    return None;
                }
                grow = grow.max(self.overhang(placed, style));
                reach = self.reach_of(placed, style).mul_add(scale, reach);
            }
            let mut rect = placed
                .matrix
                .bounds(outset(placed.border_box, grow + INK_MARGIN));
            if matches!(placed.kind, PlacedKind::Element(Tag::Svg)) {
                match self.svg_overflow(placed) {
                    SvgInk::Inside => {}
                    SvgInk::Drawn(drawn) => rect = rect.union(drawn),
                    SvgInk::Unknown => return None,
                }
            }
            seen = Some(seen.map_or(rect, |s| s.union(rect)));
        }
        seen.map(|s| outset(s, reach + INK_MARGIN))
    }
}

#[cfg(test)]
mod tests {
    use opsis_layout::computed::{Context, ROOT_FONT_SIZE};
    use opsis_layout::fonts::Fonts;
    use opsis_layout::transform::Matrix;
    use opsis_layout::{Images, Viewport, lay_out};
    use opsis_scene::node::{Node, SvgElement, div, svg};
    use opsis_scene::paint::Filter;
    use opsis_scene::style::{Overflow, Position, Style};
    use opsis_scene::values::{Length, Num};
    use opsis_skia::Surface;

    use crate::{Painter, Resources, Slots};

    const SIDE: f64 = 400.0;

    fn placed(x: f64, y: f64, size: f64) -> Style {
        Style::default()
            .position(Position::Absolute)
            .left(Length::px(x))
            .top(Length::px(y))
            .width(Length::px(size))
            .height(Length::px(size))
    }

    fn scene() -> Node {
        let line = SvgElement::new("svg")
            .attr("width", 20.0)
            .attr("height", 20.0)
            .attr("viewBox", "0 0 20 20")
            .child(
                SvgElement::new("path")
                    .attr("d", "M0 0 L80 60")
                    .attr("stroke", "red")
                    .attr("stroke-width", 4.0),
            );
        div(placed(0.0, 0.0, SIDE))
            .child(div(placed(30.0, 40.0, 20.0)
                .background_color(opsis_scene::color::Color::rgb8(1, 2, 3))
                .filter(vec![Filter::Saturate(Num(1.05))])))
            .child(svg(
                line,
                placed(200.0, 200.0, 20.0).overflow(Overflow::Visible),
            ))
            .into()
    }

    fn inks() -> Vec<opsis_layout::output::Rect> {
        let root = scene();
        let viewport = Viewport {
            width: SIDE,
            height: SIDE,
        };
        let mut fonts = Fonts::new();
        let laid = lay_out(&root, viewport, &mut fonts, &Images { size_of: &|_| None });
        let files = |_: &str| None;
        let mut resources = Resources::new(&files, fonts.faces());
        let mut surface = Surface::new(400, 400).expect("a surface");
        let index = laid
            .boxes
            .iter()
            .enumerate()
            .map(|(i, b)| (b.path.clone(), i))
            .collect();
        let slots = Slots::default();
        let mut painter = Painter {
            root: &root,
            surface: &mut surface,
            resources: &mut resources,
            laid: &laid,
            index: &index,
            context: Context {
                viewport_width: SIDE,
                viewport_height: SIDE,
                root_font_size: ROOT_FONT_SIZE,
                container: None,
            },
            offscreen: false,
            base: Matrix::translation(0.0, 0.0, 0.0),
            slots: &slots,
        };
        [vec![0], vec![1]]
            .iter()
            .map(|path| painter.ink(path).expect("an ink"))
            .collect()
    }

    #[test]
    fn a_small_filtered_box_has_a_small_ink() {
        let ink = inks()[0];
        assert!(ink.x <= 30.0 && ink.y <= 40.0);
        assert!(ink.x + ink.width >= 50.0 && ink.y + ink.height >= 60.0);
        assert!(ink.width < 40.0 && ink.height < 40.0, "{ink:?}");
    }

    #[test]
    fn an_svg_drawing_past_its_box_counts_what_it_draws() {
        let ink = inks()[1];
        assert!(ink.x + ink.width >= 280.0, "{ink:?}");
        assert!(ink.y + ink.height >= 260.0, "{ink:?}");
        assert!(ink.width < 120.0 && ink.height < 100.0, "{ink:?}");
    }
}
