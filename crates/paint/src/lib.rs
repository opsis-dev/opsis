mod boxes;
mod canvas;
mod clips;
pub mod color;
mod effects;
mod filters;
mod geometry;
mod gradients;
pub mod lottie;
mod resources;
mod svg;
mod text;

use std::collections::HashMap;

use opsis_layout::Viewport;
use opsis_layout::computed::{Context, ROOT_FONT_SIZE};
use opsis_layout::output::{Laid, Placed, PlacedKind};
use opsis_layout::transform::Matrix;
use opsis_scene::node::{Effect, Element, Kind, Node};
use opsis_scene::paint::{BlendMode, Filter};
use opsis_scene::prop;
use opsis_scene::style::{Overflow, Position, Style, Visibility, WillChange};
use opsis_scene::values::Length;
use opsis_skia::{Paint, RectF, Rgba, Sampling, Surface};
use opsis_types::numbers::exact_eq;

use crate::boxes::{Areas, draw_box, draw_replaced};
use crate::geometry::{is_rounded, matrix44, rect_f, round_rect};
pub use crate::resources::{Frames, Pass, Picture, Programs, Resources, Setting};

pub(crate) struct Painter<'s, 'r, 'l> {
    pub root: &'l Node,
    pub surface: &'s mut Surface,
    pub resources: &'s mut Resources<'r>,
    pub laid: &'l Laid,
    pub index: &'l HashMap<Vec<usize>, usize>,
    pub context: Context,
    pub offscreen: bool,
    pub base: Matrix,
    pub slots: &'l Slots,
}

#[derive(Debug, Default)]
pub(crate) struct Slots {
    sources: HashMap<String, Vec<usize>>,
    referenced: std::collections::HashSet<String>,
}

impl Slots {
    fn gather(&mut self, node: &Node, path: &mut Vec<usize>) {
        let Node::Element(element) = node else {
            return;
        };
        if let Kind::Slot(name) = &element.kind {
            if element.children.is_empty() {
                self.referenced.insert(name.clone());
            } else {
                self.sources.insert(name.clone(), path.clone());
            }
        }
        for (i, child) in element.children.iter().enumerate() {
            path.push(i);
            self.gather(child, path);
            path.pop();
        }
    }

    fn of(root: &Node) -> Self {
        let mut slots = Self::default();
        slots.gather(root, &mut Vec::new());
        slots
    }

    fn hidden(&self, element: &Element) -> bool {
        matches!(&element.kind, Kind::Slot(name) if !element.children.is_empty() && self.referenced.contains(name))
    }
}

fn element_at<'n>(root: &'n Node, path: &[usize]) -> Option<&'n Element> {
    let Node::Element(first) = root else {
        return None;
    };
    let mut element = first;
    for &i in path {
        let Node::Element(next) = element.children.get(i)? else {
            return None;
        };
        element = next;
    }
    Some(element)
}

#[derive(Debug, Clone)]
enum Entry<'n> {
    Box {
        element: &'n Element,
        path: Vec<usize>,
        clips: Vec<usize>,
    },
    Text {
        path: Vec<usize>,
        clips: Vec<usize>,
    },
    Pseudo {
        element: &'n Element,
        path: Vec<usize>,
        clips: Vec<usize>,
        flow: Vec<Self>,
    },
}

#[derive(Debug, Default)]
struct Layers<'n> {
    negative: Vec<(i32, Entry<'n>)>,
    zero: Vec<Entry<'n>>,
    positive: Vec<(i32, Entry<'n>)>,
}

fn is_positioned(style: &Style) -> bool {
    prop!(style, Position).is_some_and(|p| *p != Position::Static)
}

fn has_transform(style: &Style) -> bool {
    prop!(style, Transform).is_some_and(|t| !t.is_empty())
        || prop!(style, Translate).is_some()
        || prop!(style, Rotate).is_some()
        || prop!(style, Scale).is_some()
}

const ASPECT_UNIFORM: &str = "aspect";
const SIZE_UNIFORM: &str = "size";
const SCALE_UNIFORM: &str = "scale";
const PROGRESS_UNIFORM: &str = "progress";
const MAX_EFFECT_PIXELS: f64 = 1920.0 * 1080.0 * 4.0;

fn unpremultiplied(mut rgba: Vec<u8>) -> Vec<u8> {
    for pixel in rgba.as_chunks_mut::<4>().0 {
        let alpha = pixel[3];
        if alpha == 0 || alpha == u8::MAX {
            continue;
        }
        for channel in &mut pixel[..3] {
            let straight = u32::from(*channel) * u32::from(u8::MAX) / u32::from(alpha);
            *channel = u8::try_from(straight.min(u32::from(u8::MAX))).unwrap_or(u8::MAX);
        }
    }
    rgba
}

fn makes_context(style: &Style) -> bool {
    (is_positioned(style) && prop!(style, ZIndex).is_some())
        || effects::needs_layer(style)
        || effects::has_mask(style)
        || has_transform(style)
        || prop!(style, BackdropFilter).is_some_and(|f| !f.is_empty())
        || prop!(style, ClipPath).is_some()
        || prop!(style, Mask).is_some_and(|m| !m.is_empty())
        || prop!(style, Perspective).is_some()
        || prop!(style, WillChange) == Some(&WillChange::Transform)
}

struct Affine {
    xx: f64,
    yx: f64,
    xy: f64,
    yy: f64,
    tx: f64,
    ty: f64,
}

impl Affine {
    fn of(matrix: &Matrix) -> Option<Self> {
        let m = &matrix.0;
        let zero = [2, 3, 6, 7, 8, 9, 11, 14];
        let flat = zero.iter().all(|&i| exact_eq(m[i], 0.0)) && exact_eq(m[15], 1.0);
        flat.then_some(Self {
            xx: m[0],
            yx: m[1],
            xy: m[4],
            yy: m[5],
            tx: m[12],
            ty: m[13],
        })
    }

    fn scale(&self) -> f64 {
        self.xx.hypot(self.yx).max(self.xy.hypot(self.yy))
    }

    fn inverse(&self) -> Option<Matrix> {
        let det = self.xx.mul_add(self.yy, -(self.yx * self.xy));
        if det.abs() <= f64::EPSILON {
            return None;
        }
        let (xx, yx, xy, yy) = (self.yy / det, -self.yx / det, -self.xy / det, self.xx / det);
        let mut out = Matrix::IDENTITY;
        out.0[0] = xx;
        out.0[1] = yx;
        out.0[4] = xy;
        out.0[5] = yy;
        out.0[12] = -xx.mul_add(self.tx, xy * self.ty);
        out.0[13] = -yx.mul_add(self.tx, yy * self.ty);
        Some(out)
    }
}

fn clips_overflow(style: &Style) -> bool {
    let hidden = |o: Option<&Overflow>| o.is_some_and(|o| *o != Overflow::Visible);
    hidden(prop!(style, OverflowX)) || hidden(prop!(style, OverflowY))
}

impl<'n> Painter<'_, '_, '_> {
    fn collect(
        &self,
        element: &'n Element,
        path: &[usize],
        clips: &[usize],
        into: &mut Layers<'n>,
        flow: &mut Vec<Entry<'n>>,
    ) {
        let mut inner = clips.to_vec();
        if clips_overflow(&element.style)
            && let Some(&i) = self.index.get(path)
        {
            inner.push(i);
        }
        for (i, child) in element.children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            match child {
                Node::Text(_) => {
                    if self.index.contains_key(&child_path) {
                        flow.push(Entry::Text {
                            path: child_path,
                            clips: inner.clone(),
                        });
                    }
                }
                Node::Element(el) => {
                    if !self.index.contains_key(&child_path) {
                        self.collect(el, &child_path, &inner, into, flow);
                        continue;
                    }
                    let entry = Entry::Box {
                        element: el,
                        path: child_path.clone(),
                        clips: inner.clone(),
                    };
                    let style = &el.style;
                    let z = prop!(style, ZIndex).copied();
                    let context = makes_context(style) || matches!(el.kind, Kind::Effect(_));
                    match z {
                        Some(z) if z < 0 && context => into.negative.push((z, entry)),
                        Some(z) if z > 0 && context => into.positive.push((z, entry)),
                        _ if context => into.zero.push(entry),
                        _ if is_positioned(style) => {
                            let at = into.zero.len();
                            into.zero.push(Entry::Pseudo {
                                element: el,
                                path: child_path.clone(),
                                clips: inner.clone(),
                                flow: Vec::new(),
                            });
                            let mut own = Vec::new();
                            self.collect(el, &child_path, &inner, into, &mut own);
                            if let Some(Entry::Pseudo { flow, .. }) = into.zero.get_mut(at) {
                                *flow = own;
                            }
                        }
                        _ => {
                            flow.push(entry);
                            self.collect(el, &child_path, &inner, into, flow);
                        }
                    }
                }
            }
        }
    }

    fn apply_clips(&mut self, clips: &[usize]) {
        for &i in clips {
            let placed = &self.laid.boxes[i];
            let Some(style) = placed.style.as_ref() else {
                continue;
            };
            let areas = Areas::of(placed, style, self);
            self.surface.reset_matrix();
            self.surface.concat(matrix44(&placed.matrix));
            if is_rounded(&areas.inner) {
                self.surface
                    .clip_round_rect(round_rect(areas.padding, &areas.inner), true);
            } else {
                self.surface.clip_rect(rect_f(areas.padding), true);
            }
        }
    }

    fn with_matrix(&mut self, placed: &Placed) {
        self.surface.reset_matrix();
        self.surface.concat(matrix44(&placed.matrix));
    }

    fn draw_own(&mut self, element: &Element, path: &[usize]) {
        let Some(&i) = self.index.get(path) else {
            return;
        };
        let laid = self.laid;
        let placed = &laid.boxes[i];
        if placed.text.visibility == Visibility::Hidden {
            return;
        }
        self.with_matrix(placed);
        draw_box(self, placed, &element.style);
        match &element.kind {
            Kind::Image(asset) => draw_replaced(self, placed, &element.style, &asset.0),
            Kind::Svg(svg) => self.draw_svg(placed, &element.style, svg),
            Kind::Drawing(drawing) => canvas::draw(self, placed, &element.style, drawing),
            Kind::Lottie(frame) => boxes::draw_lottie(self, placed, &element.style, frame),
            Kind::Stage3d(stage) => self.draw_stage(placed, &element.style, stage),
            _ => {}
        }
    }

    fn draw_stage(&mut self, placed: &Placed, style: &Style, stage: &opsis_scene::node::Stage3d) {
        let content = Areas::of(placed, style, self).content;
        let size: (u32, u32) = (
            opsis_types::numbers::whole(content.width.round()),
            opsis_types::numbers::whole(content.height.round()),
        );
        if size.0 == 0 || size.1 == 0 {
            return;
        }
        let textures: Vec<Option<resources::Picture>> = stage
            .textures
            .iter()
            .map(|texture| match texture {
                opsis_scene::node::Texture3d::Picture(source) => self.resources.texture(&source.0),
                opsis_scene::node::Texture3d::Glow { .. } => None,
            })
            .collect();
        let Some(out) = self.resources.stage(stage, size, &textures) else {
            return;
        };
        let Some(image) = opsis_skia::Image::from_rgba(size.0, size.1, &unpremultiplied(out))
        else {
            return;
        };
        let (w, h) = (
            opsis_types::numbers::narrow(f64::from(size.0)),
            opsis_types::numbers::narrow(f64::from(size.1)),
        );
        self.surface.draw_image_rect(
            &image,
            (RectF::new(0.0, 0.0, w, h), rect_f(content)),
            Sampling::Linear,
            &Paint::new(),
        );
    }

    fn draw_svg(&mut self, placed: &Placed, style: &Style, svg: &opsis_scene::node::SvgElement) {
        let areas = Areas::of(placed, style, self);
        let content = areas.content;
        let mut rooted = svg.clone();
        if !rooted.attributes.iter().any(|(k, _)| k == "color") {
            let [r, g, b, a] = color::resolve(&placed.text.color, &placed.text.color).srgb();
            rooted.attributes.push((
                "color".to_owned(),
                format!(
                    "rgba({}, {}, {}, {})",
                    f64::from(r.clamp(0.0, 1.0)) * 255.0,
                    f64::from(g.clamp(0.0, 1.0)) * 255.0,
                    f64::from(b.clamp(0.0, 1.0)) * 255.0,
                    f64::from(a.clamp(0.0, 1.0))
                ),
            ));
        }
        let document = clips::markup(&rooted, Some((content.width, content.height)));
        let base = placed
            .matrix
            .then(&opsis_layout::transform::Matrix::translation(
                content.x, content.y, 0.0,
            ));
        let Painter {
            surface, resources, ..
        } = self;
        let visible = prop!(style, OverflowX) == Some(&Overflow::Visible)
            || svg
                .attributes
                .iter()
                .any(|(k, v)| k == "overflow" && v == "visible");
        let Some(tree) = resources.markup(document) else {
            return;
        };
        let saved = surface.save();
        if !visible {
            surface.clip_rect(geometry::rect_f(content), true);
        }
        let mut decode = |bytes: &[u8]| opsis_skia::Image::decode(bytes);
        svg::SvgPainter {
            surface,
            decode: &mut decode,
        }
        .group(tree.root(), &base);
        surface.restore_to(saved);
    }

    fn draw_text(&mut self, path: &[usize]) {
        let Some(&i) = self.index.get(path) else {
            return;
        };
        let laid = self.laid;
        let placed = &laid.boxes[i];
        if matches!(placed.kind, PlacedKind::Text { .. }) {
            self.with_matrix(placed);
            text::draw_text(self, placed);
        }
    }

    fn paint_entry(&mut self, entry: &Entry<'n>) {
        match entry {
            Entry::Text { path, clips } => {
                let saved = self.surface.save();
                self.apply_clips(clips);
                self.draw_text(path);
                self.surface.restore_to(saved);
            }
            Entry::Box {
                element,
                path,
                clips,
            } => {
                let saved = self.surface.save();
                self.apply_clips(clips);
                self.draw_own(element, path);
                self.surface.restore_to(saved);
            }
            Entry::Pseudo {
                element,
                path,
                clips,
                flow,
            } => {
                let saved = self.surface.save();
                self.apply_clips(clips);
                self.draw_own(element, path);
                self.surface.restore_to(saved);
                for inner in flow {
                    self.paint_entry(inner);
                }
            }
        }
    }

    fn faces_back(&self, element: &Element, path: &[usize]) -> bool {
        prop!(&element.style, BackfaceVisibility) == Some(&Visibility::Hidden)
            && self.index.get(path).is_some_and(|&i| {
                self.laid.boxes[i]
                    .matrix
                    .faces_back(self.laid.boxes[i].border_box)
            })
    }

    fn filter_reach(&self, element: &Element, path: &[usize]) -> f64 {
        let style = &element.style;
        if prop!(style, MixBlendMode).is_some_and(|b| *b != BlendMode::Normal)
            || prop!(style, BackdropFilter).is_some_and(|f| !f.is_empty())
        {
            return 0.0;
        }
        let (Some(filters), Some(&i)) = (prop!(style, Filter), self.index.get(path)) else {
            return 0.0;
        };
        let text = &self.laid.boxes[i].text;
        let length = |l: &Length| geometry::px(l, text, &self.context, 0.0);
        filters
            .iter()
            .map(|filter| match filter {
                Filter::Blur(radius) => 3.0 * length(radius),
                Filter::DropShadow(shadow) => 3.0f64.mul_add(
                    length(&shadow.blur),
                    length(&shadow.x).abs().max(length(&shadow.y).abs()),
                ),
                _ => 0.0,
            })
            .sum()
    }

    fn paint_offscreen(
        &mut self,
        element: &'n Element,
        path: &[usize],
        clips: &[usize],
        reach: f64,
    ) {
        let margin = reach.ceil();
        let grow: u32 = opsis_types::numbers::whole(margin * 2.0);
        let (width, height) = (self.surface.width() + grow, self.surface.height() + grow);
        let Ok(surface) = Surface::new(width, height) else {
            return;
        };
        let shift = opsis_types::numbers::narrow(margin);
        let base = Matrix::translation(margin, margin, 0.0).then(&self.base);
        let mut surface = surface.with_base(matrix44(&base));
        surface.clear(Rgba::TRANSPARENT);
        {
            let mut sub = Painter {
                root: self.root,
                surface: &mut surface,
                resources: self.resources,
                laid: self.laid,
                index: self.index,
                context: self.context,
                offscreen: true,
                base,
                slots: self.slots,
            };
            sub.paint_context(element, path, &[]);
        }
        let Some(image) = surface.snapshot() else {
            return;
        };
        let saved = self.surface.save();
        self.apply_clips(clips);
        self.surface.reset_device_matrix();
        let (w, h) = (
            opsis_types::numbers::narrow(f64::from(width)),
            opsis_types::numbers::narrow(f64::from(height)),
        );
        self.surface.draw_image_rect(
            &image,
            (RectF::new(0.0, 0.0, w, h), RectF::new(-shift, -shift, w, h)),
            Sampling::Nearest,
            &Paint::new(),
        );
        self.surface.restore_to(saved);
    }

    fn paint_composited(&mut self, element: &'n Element, path: &[usize], clips: &[usize]) -> bool {
        let Some(&i) = self.index.get(path) else {
            return false;
        };
        let placed = &self.laid.boxes[i];
        let Some(affine) = Affine::of(&placed.matrix) else {
            return false;
        };
        let Some(inverse) = affine.inverse() else {
            return false;
        };
        let Some(raster) = Affine::of(&self.base.then(&placed.matrix)).map(|a| a.scale()) else {
            return false;
        };
        let Some(from_device) = Affine::of(&self.base).and_then(|a| a.inverse()) else {
            return false;
        };
        if raster <= f64::EPSILON {
            return false;
        }
        let origin = (placed.border_box.x, placed.border_box.y);
        let to_layer = Matrix::scaling(raster, raster, 1.0)
            .then(&Matrix::translation(-origin.0, -origin.1, 0.0))
            .then(&inverse);
        let view = opsis_layout::output::Rect {
            x: 0.0,
            y: 0.0,
            width: f64::from(self.surface.width()),
            height: f64::from(self.surface.height()),
        };
        let seen = to_layer.then(&from_device).bounds(view);
        let (left, top) = ((seen.x - 2.0).floor(), (seen.y - 2.0).floor());
        let width: u32 = opsis_types::numbers::whole((seen.x + seen.width + 2.0).ceil() - left);
        let height: u32 = opsis_types::numbers::whole((seen.y + seen.height + 2.0).ceil() - top);
        let Ok(surface) = Surface::new(width.max(1), height.max(1)) else {
            return false;
        };
        let base = Matrix::translation(-left, -top, 0.0).then(&to_layer);
        let mut surface = surface.with_base(matrix44(&base));
        surface.clear(Rgba::TRANSPARENT);
        {
            let mut sub = Painter {
                root: self.root,
                surface: &mut surface,
                resources: self.resources,
                laid: self.laid,
                index: self.index,
                context: self.context,
                offscreen: self.offscreen,
                base,
                slots: self.slots,
            };
            sub.paint_context_here(element, path, &[]);
        }
        let Some(image) = surface.snapshot() else {
            return true;
        };
        let back = placed
            .matrix
            .then(&Matrix::translation(origin.0, origin.1, 0.0))
            .then(&Matrix::scaling(1.0 / raster, 1.0 / raster, 1.0))
            .then(&Matrix::translation(left, top, 0.0));
        let saved = self.surface.save();
        self.apply_clips(clips);
        self.surface.reset_matrix();
        self.surface.concat(matrix44(&back));
        let (w, h) = (
            opsis_types::numbers::narrow(f64::from(width)),
            opsis_types::numbers::narrow(f64::from(height)),
        );
        self.surface.draw_image_rect(
            &image,
            (RectF::new(0.0, 0.0, w, h), RectF::new(0.0, 0.0, w, h)),
            Sampling::Linear,
            &Paint::new(),
        );
        self.surface.restore_to(saved);
        true
    }

    fn effect_inputs(
        &mut self,
        element: &'n Element,
        path: &[usize],
        placed: &Placed,
        size: (u32, u32),
        scale: f64,
    ) -> Option<Vec<Option<Vec<u8>>>> {
        let inverse = Affine::of(&placed.matrix)?.inverse()?;
        let b = placed.border_box;
        let base = Matrix::scaling(scale, scale, 1.0)
            .then(&Matrix::translation(-b.x, -b.y, 0.0))
            .then(&inverse);
        let mut inputs = Vec::new();
        for (c, child) in element.children.iter().enumerate() {
            let Node::Element(child) = child else {
                continue;
            };
            let own: Vec<usize> = path.iter().copied().chain([c]).collect();
            let referenced = match &child.kind {
                Kind::Slot(name) if child.children.is_empty() => Some(name),
                _ => None,
            };
            let (source, child_path, revealed) = match referenced {
                None => (child, own, false),
                Some(name) => {
                    let found = self
                        .slots
                        .sources
                        .get(name)
                        .and_then(|p| Some((element_at(self.root, p)?, p.clone())));
                    let Some((found, found_path)) = found else {
                        inputs.push(None);
                        continue;
                    };
                    (found, found_path, true)
                }
            };
            let surface = Surface::new(size.0, size.1).ok()?;
            let mut surface = surface.with_base(matrix44(&base));
            surface.clear(Rgba::TRANSPARENT);
            {
                let mut sub = Painter {
                    root: self.root,
                    surface: &mut surface,
                    resources: self.resources,
                    laid: self.laid,
                    index: self.index,
                    context: self.context,
                    offscreen: false,
                    base,
                    slots: self.slots,
                };
                if revealed {
                    sub.paint_context_here(source, &child_path, &[]);
                } else {
                    sub.paint_context(source, &child_path, &[]);
                }
            }
            inputs.push(Some(surface.rgba()));
        }
        Some(inputs)
    }

    fn paint_effect(
        &mut self,
        element: &'n Element,
        effect: &Effect,
        path: &[usize],
        clips: &[usize],
    ) -> bool {
        let Some(&i) = self.index.get(path) else {
            return false;
        };
        let laid = self.laid;
        let placed = &laid.boxes[i];
        let b = placed.border_box;
        let base = (b.width.round(), b.height.round());
        let wanted = effect.scale.0.max(1.0);
        let fitted = (MAX_EFFECT_PIXELS / (base.0 * wanted * base.1 * wanted).max(1.0))
            .sqrt()
            .min(1.0);
        let size: (u32, u32) = (
            opsis_types::numbers::whole((base.0 * wanted * fitted).round()),
            opsis_types::numbers::whole((base.1 * wanted * fitted).round()),
        );
        if size.0 == 0 || size.1 == 0 {
            return true;
        }
        let scale = f64::from(size.0) / base.0.max(1.0);
        let Some(inputs) = self.effect_inputs(element, path, placed, size, scale) else {
            return false;
        };
        let narrow = opsis_types::numbers::narrow;
        let mut uniforms: Vec<Setting> = effect
            .uniforms
            .iter()
            .map(|u| Setting {
                name: u.name.clone(),
                values: u.values.iter().map(|v| narrow(v.0)).collect(),
                columns: usize::try_from(u.columns).unwrap_or(1),
            })
            .collect();
        let (width, height) = (f64::from(size.0), f64::from(size.1));
        uniforms.push(Setting::of(ASPECT_UNIFORM, vec![narrow(width / height)]));
        uniforms.push(Setting::of(
            SIZE_UNIFORM,
            vec![narrow(width), narrow(height)],
        ));
        uniforms.push(Setting::of(SCALE_UNIFORM, vec![narrow(scale)]));
        if inputs.iter().any(Option::is_none) {
            for u in uniforms.iter_mut().filter(|u| u.name == PROGRESS_UNIFORM) {
                u.values = vec![1.0];
            }
        }
        let blank =
            vec![0u8; usize::try_from(u64::from(size.0) * u64::from(size.1) * 4).unwrap_or(0)];
        let inputs: Vec<Vec<u8>> = inputs
            .into_iter()
            .map(|input| input.unwrap_or_else(|| blank.clone()))
            .collect();
        let Some(out) = self.resources.run(&Pass {
            program: &effect.program,
            inputs: &inputs,
            size,
            uniforms: &uniforms,
        }) else {
            return false;
        };
        let Some(image) = opsis_skia::Image::from_rgba(size.0, size.1, &unpremultiplied(out))
        else {
            return true;
        };
        let saved = self.surface.save();
        self.apply_clips(clips);
        self.with_matrix(placed);
        let (w, h) = (
            opsis_types::numbers::narrow(f64::from(size.0)),
            opsis_types::numbers::narrow(f64::from(size.1)),
        );
        self.surface.draw_image_rect(
            &image,
            (RectF::new(0.0, 0.0, w, h), rect_f(b)),
            Sampling::Linear,
            &Paint::new(),
        );
        self.surface.restore_to(saved);
        true
    }

    fn paint_context(&mut self, element: &'n Element, path: &[usize], clips: &[usize]) {
        if self.faces_back(element, path) || self.slots.hidden(element) {
            return;
        }
        if let Kind::Effect(effect) = &element.kind
            && self.paint_effect(element, effect, path, clips)
        {
            return;
        }
        if prop!(&element.style, WillChange) == Some(&WillChange::Transform)
            && self.paint_composited(element, path, clips)
        {
            return;
        }
        if !self.offscreen {
            let reach = self.filter_reach(element, path);
            if reach > 0.0 {
                self.paint_offscreen(element, path, clips, reach);
                return;
            }
        }
        let was_offscreen = self.offscreen;
        self.offscreen = false;
        self.paint_context_here(element, path, clips);
        self.offscreen = was_offscreen;
    }

    fn paint_context_here(&mut self, element: &'n Element, path: &[usize], clips: &[usize]) {
        let saved = self.surface.save();
        self.apply_clips(clips);
        let laid = self.laid;
        let root = self.root;
        let opened = match self.index.get(path).copied() {
            Some(i) => {
                let placed = &laid.boxes[i];
                self.with_matrix(placed);
                Some((
                    placed,
                    effects::open_layer(self, placed, &element.style, root),
                ))
            }
            None => None,
        };
        self.draw_own(element, path);
        let mut layers = Layers::default();
        let mut flow = Vec::new();
        self.collect(element, path, &[], &mut layers, &mut flow);
        layers.negative.sort_by_key(|(z, _)| *z);
        layers.positive.sort_by_key(|(z, _)| *z);
        for (_, entry) in &layers.negative {
            self.paint_nested(entry);
        }
        for entry in &flow {
            self.paint_entry(entry);
        }
        for entry in &layers.zero {
            self.paint_nested(entry);
        }
        for (_, entry) in &layers.positive {
            self.paint_nested(entry);
        }
        if let Some((placed, opened)) = opened {
            effects::close_layer(self, placed, &element.style, opened);
        }
        self.surface.restore_to(saved);
    }

    fn paint_nested(&mut self, entry: &Entry<'n>) {
        match entry {
            Entry::Box {
                element,
                path,
                clips,
            } => self.paint_context(element, path, clips),
            Entry::Text { .. } | Entry::Pseudo { .. } => self.paint_entry(entry),
        }
    }
}

pub fn paint(
    root: &Node,
    laid: &Laid,
    viewport: Viewport,
    surface: &mut Surface,
    resources: &mut Resources<'_>,
) {
    let Node::Element(element) = root else {
        return;
    };
    let index = laid
        .boxes
        .iter()
        .enumerate()
        .map(|(i, placed)| (placed.path.clone(), i))
        .collect();
    let slots = Slots::of(root);
    let mut painter = Painter {
        root,
        surface,
        resources,
        laid,
        index: &index,
        offscreen: false,
        base: Matrix::translation(0.0, 0.0, 0.0),
        slots: &slots,
        context: Context {
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            root_font_size: ROOT_FONT_SIZE,
            container: None,
        },
    };
    painter.paint_context(element, &[], &[]);
}
