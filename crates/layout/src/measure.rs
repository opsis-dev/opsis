use std::cell::RefCell;
use std::collections::HashMap;

use opsis_scene::drawing::Font;
use opsis_scene::measure::{Measure, MeasuredBox};
use opsis_scene::node::Node;
use parley::PositionedLayoutItem;
use skrifa::instance::{LocationRef, NormalizedCoord, Size};
use skrifa::{FontRef, GlyphId, MetadataProvider};

use crate::computed::{Context, ROOT_FONT_SIZE};
use crate::drawing::style_of;
use crate::fonts::Fonts;
use crate::inline::probe;
use crate::tree::Tree;
use crate::{Images, Viewport, lay_out};

pub struct Measurer<'f> {
    fonts: RefCell<&'f mut Fonts>,
    viewport: Viewport,
}

impl std::fmt::Debug for Measurer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Measurer")
            .field("viewport", &self.viewport)
            .finish_non_exhaustive()
    }
}

impl<'f> Measurer<'f> {
    pub const fn new(fonts: &'f mut Fonts, viewport: Viewport) -> Self {
        Self {
            fonts: RefCell::new(fonts),
            viewport,
        }
    }
}

impl Measure for Measurer<'_> {
    fn text_width(&self, text: &str, font: &Font, letter_spacing: f64) -> f64 {
        let mut fonts = self.fonts.borrow_mut();
        let mut tree = Tree {
            nodes: Vec::new(),
            formulas: Vec::new(),
            fonts: &mut fonts,
            context: Context {
                viewport_width: self.viewport.width,
                viewport_height: self.viewport.height,
                root_font_size: ROOT_FONT_SIZE,
                container: None,
            },
            text_layouts: parley::LayoutContext::new(),
            extents: HashMap::new(),
            containers: HashMap::new(),
        };
        let mut style = style_of(font);
        style.letter_spacing = letter_spacing;
        f64::from(probe(&mut tree, &style, text).width())
    }

    fn text_ink(&self, text: &str, font: &Font) -> (f64, f64) {
        let mut fonts = self.fonts.borrow_mut();
        let mut tree = Tree {
            nodes: Vec::new(),
            formulas: Vec::new(),
            fonts: &mut fonts,
            context: Context {
                viewport_width: self.viewport.width,
                viewport_height: self.viewport.height,
                root_font_size: ROOT_FONT_SIZE,
                container: None,
            },
            text_layouts: parley::LayoutContext::new(),
            extents: HashMap::new(),
            containers: HashMap::new(),
        };
        let paragraph = probe(&mut tree, &style_of(font), text);
        let (mut ascent, mut descent) = (0.0_f64, 0.0_f64);
        for line in paragraph.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let data = run.font();
                let Ok(face) = FontRef::from_index(data.data.data(), data.index) else {
                    continue;
                };
                let coords: Vec<NormalizedCoord> = run
                    .normalized_coords()
                    .iter()
                    .map(|c| NormalizedCoord::from_bits(c.to_bits()))
                    .collect();
                let metrics =
                    face.glyph_metrics(Size::new(run.font_size()), LocationRef::new(&coords));
                for glyph in glyph_run.glyphs() {
                    if let Some(bounds) = metrics.bounds(GlyphId::new(glyph.id)) {
                        ascent = ascent.max(f64::from(bounds.y_max - glyph.y));
                        descent = descent.max(f64::from(glyph.y - bounds.y_min));
                    }
                }
            }
        }
        (ascent.ceil(), descent.ceil())
    }

    fn boxes(&self, root: &Node) -> Vec<MeasuredBox> {
        let mut fonts = self.fonts.borrow_mut();
        let laid = lay_out(
            root,
            self.viewport,
            &mut fonts,
            &Images { size_of: &|_| None },
        );
        laid.boxes
            .into_iter()
            .map(|placed| MeasuredBox {
                path: placed.path,
                x: placed.border_box.x,
                y: placed.border_box.y,
                width: placed.border_box.width,
                height: placed.border_box.height,
            })
            .collect()
    }
}
