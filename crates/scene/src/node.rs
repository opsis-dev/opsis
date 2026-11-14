use serde::{Deserialize, Serialize};

use crate::paint::AssetRef;
use crate::random::js_number_text;
use crate::style::Style;
use crate::values::Num;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Element {
    pub kind: Kind,
    pub style: Style,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Div,
    Span,
    Image(AssetRef),
    Svg(SvgElement),
    Footage(FootageFrame),
    Lottie(LottieFrame),
    Effect(Effect),
    Stage3d(Box<Stage3d>),
    Drawing(crate::drawing::Drawing),
    Slot(String),
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct SvgElement {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<SvgChild>,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SvgChild {
    Element(SvgElement),
    Text(String),
    Html(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttrValue(pub String);

impl From<&str> for AttrValue {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for AttrValue {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<f64> for AttrValue {
    fn from(value: f64) -> Self {
        Self(js_number_text(value))
    }
}

impl From<i32> for AttrValue {
    fn from(value: i32) -> Self {
        Self(value.to_string())
    }
}

impl From<u32> for AttrValue {
    fn from(value: u32) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FootageFrame {
    pub source: AssetRef,
    pub frame: u32,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct LottieFrame {
    pub source: AssetRef,
    pub frame: Num,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Effect {
    pub program: String,
    pub uniforms: Vec<EffectUniform>,
    pub scale: Num,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lighting3d {
    Studio,
    Soft,
    Dramatic,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape3d {
    Sphere {
        radius: Num,
        width_segments: u32,
        height_segments: u32,
    },
    Torus {
        radius: Num,
        tube: Num,
        radial_segments: u32,
        tubular_segments: u32,
    },
    RoundedBox {
        size: [Num; 3],
        segments: u32,
        radius: Num,
    },
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Material3d {
    pub color: [Num; 4],
    pub metalness: Num,
    pub roughness: Num,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Object3d {
    pub shape: Shape3d,
    pub matrix: [Num; 16],
    pub material: Material3d,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Stage3d {
    pub fov: Num,
    pub distance: Num,
    pub lighting: Lighting3d,
    pub objects: Vec<Object3d>,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct EffectUniform {
    pub name: String,
    pub values: Vec<Num>,
    pub columns: u32,
}

impl Element {
    pub const fn new(kind: Kind, style: Style) -> Self {
        Self {
            kind,
            style,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn child(mut self, node: impl Into<Node>) -> Self {
        self.children.push(node.into());
        self
    }

    #[must_use]
    pub fn children(mut self, nodes: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(nodes);
        self
    }
}

impl From<Element> for Node {
    fn from(element: Element) -> Self {
        Self::Element(element)
    }
}

impl From<&str> for Node {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<String> for Node {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

pub const fn div(style: Style) -> Element {
    Element::new(Kind::Div, style)
}

pub const fn span(style: Style) -> Element {
    Element::new(Kind::Span, style)
}

pub fn img(source: impl Into<String>, style: Style) -> Element {
    Element::new(Kind::Image(AssetRef(source.into())), style)
}

impl SvgElement {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<AttrValue>) -> Self {
        self.attributes.push((name.into(), value.into().0));
        self
    }

    #[must_use]
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(SvgChild::Element(child));
        self
    }

    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.children.push(SvgChild::Text(text.into()));
        self
    }
}

pub const fn drawing(canvas: crate::drawing::Drawing, style: Style) -> Element {
    Element::new(Kind::Drawing(canvas), style)
}

pub const fn svg(root: SvgElement, style: Style) -> Element {
    Element::new(Kind::Svg(root), style)
}
