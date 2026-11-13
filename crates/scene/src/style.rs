use serde::{Deserialize, Serialize};

use crate::color::Color;
use crate::paint::{
    BlendMode, BoxArea, ClipPath, Filter, ImageLayer, MaskLayer, Shadow, Transform,
};
use crate::values::{Alpha, Angle, Corners, Edges, Length, Num, Position2, Px};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Display {
    #[default]
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    InlineGrid,
    Contents,
    None,
    WebkitBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlexWrap {
    #[default]
    Nowrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Align {
    Normal,
    Stretch,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Track {
    Length(Length),
    Fraction(Num),
    Auto,
    MinContent,
    MaxContent,
    MinMax(Box<Self>, Box<Self>),
    Repeat(RepeatCount, Vec<Self>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepeatCount {
    Times(u16),
    AutoFill,
    AutoFit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GridLine {
    #[default]
    Auto,
    Line(i16),
    Span(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContainerType {
    #[default]
    Normal,
    Size,
    InlineSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BorderStyle {
    None,
    #[default]
    Solid,
    Dashed,
    Dotted,
    Double,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct BorderSide {
    pub width: Length,
    pub style: BorderStyle,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct Outline {
    pub width: Length,
    pub style: BorderStyle,
    pub color: Option<Color>,
    pub offset: Length,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransformStyle {
    #[default]
    Flat,
    Preserve3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectFit {
    #[default]
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageRendering {
    #[default]
    Auto,
    Pixelated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextRendering {
    #[default]
    Auto,
    GeometricPrecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineHeight {
    Normal,
    Number(Num),
    Length(Length),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextAlign {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhiteSpace {
    #[default]
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WordBreak {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverflowWrap {
    #[default]
    Normal,
    Anywhere,
    BreakWord,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Sub,
    Super,
    TextTop,
    TextBottom,
    Middle,
    Top,
    Bottom,
    Length(Length),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextWrap {
    #[default]
    Wrap,
    Balance,
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecorationLine {
    #[default]
    None,
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecorationStyle {
    #[default]
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct TextDecoration {
    pub lines: Vec<DecorationLine>,
    pub style: DecorationStyle,
    pub color: Option<Color>,
    pub thickness: Option<Length>,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
pub struct TextStroke {
    pub width: Length,
    pub color: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaintOrder {
    #[default]
    Normal,
    StrokeFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one keyword of font-variant-numeric"
)]
pub struct NumericVariant {
    pub tabular: bool,
    pub proportional: bool,
    pub slashed_zero: bool,
    pub oldstyle: bool,
    pub lining: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(pub [u8; 4]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoxSizing {
    #[default]
    BorderBox,
    ContentBox,
}

macro_rules! properties {
    ($($name:ident / $variant:ident: $kind:ty),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum Prop {
            $($variant($kind),)*
        }

        impl Style {
            $(
                #[must_use]
                pub fn $name(self, value: impl Into<$kind>) -> Self {
                    self.with(Prop::$variant(value.into()))
                }
            )*
        }
    };
}

properties! {
    display / Display: Display,
    position / Position: Position,
    box_sizing / BoxSizing: BoxSizing,
    top / Top: Length,
    right / Right: Length,
    bottom / Bottom: Length,
    left / Left: Length,
    width / Width: Length,
    height / Height: Length,
    min_width / MinWidth: Length,
    min_height / MinHeight: Length,
    max_width / MaxWidth: Length,
    max_height / MaxHeight: Length,
    aspect_ratio / AspectRatio: Num,
    margin / Margin: Edges<Length>,
    padding / Padding: Edges<Length>,
    border / Border: Edges<BorderSide>,
    border_radius / BorderRadius: Corners<(Length, Length)>,
    z_index / ZIndex: i32,
    overflow_x / OverflowX: Overflow,
    overflow_y / OverflowY: Overflow,
    container_type / ContainerType: ContainerType,
    isolation / Isolation: bool,
    visibility / Visibility: Visibility,
    flex_direction / FlexDirection: FlexDirection,
    flex_wrap / FlexWrap: FlexWrap,
    flex_grow / FlexGrow: Num,
    flex_shrink / FlexShrink: Num,
    flex_basis / FlexBasis: Length,
    order / Order: i32,
    justify_content / JustifyContent: Align,
    justify_items / JustifyItems: Align,
    justify_self / JustifySelf: Align,
    align_items / AlignItems: Align,
    align_self / AlignSelf: Align,
    align_content / AlignContent: Align,
    row_gap / RowGap: Length,
    column_gap / ColumnGap: Length,
    grid_template_columns / GridTemplateColumns: Vec<Track>,
    grid_template_rows / GridTemplateRows: Vec<Track>,
    grid_auto_rows / GridAutoRows: Vec<Track>,
    grid_auto_columns / GridAutoColumns: Vec<Track>,
    grid_column / GridColumn: (GridLine, GridLine),
    grid_row / GridRow: (GridLine, GridLine),
    background_color / BackgroundColor: Color,
    background / Background: Vec<ImageLayer>,
    background_clip_text / BackgroundClipText: bool,
    box_shadow / BoxShadow: Vec<Shadow>,
    outline / Outline: Outline,
    opacity / Opacity: Alpha,
    mix_blend_mode / MixBlendMode: BlendMode,
    filter / Filter: Vec<Filter>,
    backdrop_filter / BackdropFilter: Vec<Filter>,
    clip_path / ClipPath: ClipPath,
    mask / Mask: Vec<MaskLayer>,
    transform / Transform: Vec<Transform>,
    translate3d / Translate: (Length, Length, Length),
    rotate3d / Rotate: (Num, Num, Num, Angle),
    scale3d / Scale: (Num, Num, Num),
    transform_origin / TransformOrigin: (Length, Length, Length),
    perspective / Perspective: Px,
    perspective_origin / PerspectiveOrigin: Position2,
    transform_style / TransformStyle: TransformStyle,
    backface_visibility / BackfaceVisibility: Visibility,
    object_fit / ObjectFit: ObjectFit,
    object_position / ObjectPosition: Position2,
    image_rendering / ImageRendering: ImageRendering,
    color / Color: Color,
    font_family / FontFamily: Vec<String>,
    font_size / FontSize: Length,
    font_weight / FontWeight: Num,
    font_style / FontStyle: FontStyle,
    font_variant_numeric / FontVariantNumeric: NumericVariant,
    font_feature_settings / FontFeatureSettings: Vec<(Tag, i32)>,
    font_variation_settings / FontVariationSettings: Vec<(Tag, Num)>,
    line_height / LineHeight: LineHeight,
    letter_spacing / LetterSpacing: Length,
    word_spacing / WordSpacing: Length,
    text_align / TextAlign: TextAlign,
    text_transform / TextTransform: TextTransform,
    white_space / WhiteSpace: WhiteSpace,
    text_rendering / TextRendering: TextRendering,
    word_break / WordBreak: WordBreak,
    overflow_wrap / OverflowWrap: OverflowWrap,
    text_overflow / TextOverflow: TextOverflow,
    text_wrap / TextWrap: TextWrap,
    line_clamp / LineClamp: u16,
    direction / Direction: Direction,
    text_decoration / TextDecoration: TextDecoration,
    text_shadow / TextShadow: Vec<Shadow>,
    text_stroke / TextStroke: TextStroke,
    paint_order / PaintOrder: PaintOrder,
    vertical_align / VerticalAlign: VerticalAlign,
    tab_size / TabSize: Num,
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Style(Vec<Prop>);

#[macro_export]
macro_rules! prop {
    ($style:expr, $variant:ident) => {
        $style.props().iter().rev().find_map(|p| match p {
            $crate::style::Prop::$variant(value) => Some(value),
            _ => None,
        })
    };
}

impl Prop {
    pub const fn places(&self) -> bool {
        matches!(
            self,
            Self::Opacity(_)
                | Self::Transform(_)
                | Self::Translate(_)
                | Self::Rotate(_)
                | Self::Scale(_)
                | Self::TransformOrigin(_)
                | Self::Top(_)
                | Self::Right(_)
                | Self::Bottom(_)
                | Self::Left(_)
        )
    }
}

impl Style {
    pub fn props(&self) -> &[Prop] {
        &self.0
    }

    #[must_use]
    pub fn with(mut self, prop: Prop) -> Self {
        let kind = std::mem::discriminant(&prop);
        self.0.retain(|p| std::mem::discriminant(p) != kind);
        self.0.push(prop);
        self
    }

    #[must_use]
    pub fn merged(mut self, other: &Self) -> Self {
        for prop in &other.0 {
            self = self.with(prop.clone());
        }
        self
    }

    #[must_use]
    pub fn without_placement(&self) -> Self {
        Self(self.0.iter().filter(|p| !p.places()).cloned().collect())
    }

    #[must_use]
    pub fn inset(self, value: impl Into<Length>) -> Self {
        let value = value.into();
        self.top(value.clone())
            .right(value.clone())
            .bottom(value.clone())
            .left(value)
    }

    #[must_use]
    pub fn gap(self, value: impl Into<Length>) -> Self {
        let value = value.into();
        self.row_gap(value.clone()).column_gap(value)
    }

    #[must_use]
    pub fn overflow(self, value: Overflow) -> Self {
        self.overflow_x(value).overflow_y(value)
    }

    #[must_use]
    pub fn translate(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        self.translate3d((x.into(), y.into(), Length::ZERO))
    }

    #[must_use]
    pub fn scale(self, x: f64, y: f64) -> Self {
        self.scale3d((Num(x), Num(y), Num(1.0)))
    }

    #[must_use]
    pub fn rotate(self, angle: Angle) -> Self {
        self.rotate3d((Num(0.0), Num(0.0), Num(1.0), angle))
    }

    #[must_use]
    pub fn border_radius_all(self, radius: impl Into<Length>) -> Self {
        let radius = radius.into();
        self.border_radius(Corners::all((radius.clone(), radius)))
    }

    fn edges_of(&self, padding: bool) -> Edges<Length> {
        let found = if padding {
            prop!(self, Padding)
        } else {
            prop!(self, Margin)
        };
        found.cloned().unwrap_or_else(|| Edges::all(Length::ZERO))
    }

    #[must_use]
    pub fn margin_top(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            top: value.into(),
            ..self.edges_of(false)
        };
        self.margin(edges)
    }

    #[must_use]
    pub fn margin_right(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            right: value.into(),
            ..self.edges_of(false)
        };
        self.margin(edges)
    }

    #[must_use]
    pub fn margin_bottom(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            bottom: value.into(),
            ..self.edges_of(false)
        };
        self.margin(edges)
    }

    #[must_use]
    pub fn margin_left(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            left: value.into(),
            ..self.edges_of(false)
        };
        self.margin(edges)
    }

    #[must_use]
    pub fn padding_top(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            top: value.into(),
            ..self.edges_of(true)
        };
        self.padding(edges)
    }

    #[must_use]
    pub fn padding_right(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            right: value.into(),
            ..self.edges_of(true)
        };
        self.padding(edges)
    }

    #[must_use]
    pub fn padding_bottom(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            bottom: value.into(),
            ..self.edges_of(true)
        };
        self.padding(edges)
    }

    #[must_use]
    pub fn padding_left(self, value: impl Into<Length>) -> Self {
        let edges = Edges {
            left: value.into(),
            ..self.edges_of(true)
        };
        self.padding(edges)
    }

    #[must_use]
    pub fn background_area(self, area: BoxArea) -> Self {
        self.background_clip_text(area == BoxArea::Text)
    }
}
