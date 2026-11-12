pub mod computed;
mod drawing;
pub mod fonts;
mod inline;
pub mod measure;
pub mod output;
mod style;
pub mod transform;
mod tree;

use std::collections::HashMap;

use opsis_scene::node::Node;
use opsis_types::numbers::narrow;
use taffy::compute_root_layout;
use taffy::prelude::*;

use crate::computed::{Context, ROOT_FONT_SIZE, TextStyle};
use crate::fonts::Fonts;
use crate::output::{Frame, Laid, Walk};
use crate::tree::Tree;
pub use crate::tree::{Images, Tag};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
}

const CONTAINER_PASSES: usize = 4;

pub fn lay_out(root: &Node, viewport: Viewport, fonts: &mut Fonts, images: &Images<'_>) -> Laid {
    let Node::Element(element) = root else {
        return Laid::default();
    };
    let mut containers = HashMap::new();
    for pass in 1..=CONTAINER_PASSES {
        let mut tree = Tree {
            nodes: Vec::new(),
            formulas: Vec::new(),
            fonts,
            context: Context {
                viewport_width: viewport.width,
                viewport_height: viewport.height,
                root_font_size: ROOT_FONT_SIZE,
                container: None,
            },
            text_layouts: parley::LayoutContext::new(),
            extents: HashMap::new(),
            containers: std::mem::take(&mut containers),
        };
        let Some(root_id) = tree.build_element(element, &[], &TextStyle::initial(), images, false)
        else {
            return Laid::default();
        };
        let size = Size {
            width: AvailableSpace::Definite(narrow(viewport.width)),
            height: AvailableSpace::Definite(narrow(viewport.height)),
        };
        compute_root_layout(&mut tree, NodeId::from(root_id), size);
        let measured = tree.measured_containers();
        if pass < CONTAINER_PASSES && measured != tree.containers {
            containers = measured;
            continue;
        }
        let mut walk = Walk {
            tree: &mut tree,
            out: Vec::new(),
        };
        walk.node(root_id, &Frame::ROOT);
        return Laid { boxes: walk.out };
    }
    Laid::default()
}
