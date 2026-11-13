use opsis_scene::color::Color;
use opsis_scene::paint::Shadow;
use opsis_scene::prop;
use opsis_scene::style::{
    DecorationLine, DecorationStyle, Direction, FontStyle, LineHeight, NumericVariant,
    OverflowWrap, PaintOrder, Style, Tag, TextAlign, TextRendering, TextStroke, TextTransform,
    TextWrap, Visibility, WhiteSpace, WordBreak,
};
use opsis_scene::values::{Calc, ContainerUnit, Length, Num};

pub const ROOT_FONT_SIZE: f64 = 16.0;
pub const ROOT_TAB_SIZE: f64 = 4.0;
const NORMAL_WEIGHT: f64 = 400.0;
const BOLDER_STEP: f64 = 300.0;
const CH_SHARE: f64 = 0.5;
const EX_SHARE: f64 = 0.5;
const NORMAL_LINE_HEIGHT: f64 = 1.2;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Container {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Context {
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub root_font_size: f64,
    pub container: Option<Container>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    Px(f64),
    Percent(f64),
    Sum(Vec<Self>),
    Scaled(Box<Self>, f64),
    Min(Vec<Self>),
    Max(Vec<Self>),
    Clamp(Box<Self>, Box<Self>, Box<Self>),
}

impl Formula {
    pub fn at(&self, basis: f64) -> f64 {
        match self {
            Self::Px(px) => *px,
            Self::Percent(p) => basis * p / 100.0,
            Self::Sum(terms) => terms.iter().map(|t| t.at(basis)).sum(),
            Self::Scaled(inner, by) => inner.at(basis) * by,
            Self::Min(terms) => terms
                .iter()
                .map(|t| t.at(basis))
                .fold(f64::INFINITY, f64::min),
            Self::Max(terms) => terms
                .iter()
                .map(|t| t.at(basis))
                .fold(f64::NEG_INFINITY, f64::max),
            Self::Clamp(low, preferred, high) => {
                preferred.at(basis).min(high.at(basis)).max(low.at(basis))
            }
        }
    }

    fn has_percent(&self) -> bool {
        match self {
            Self::Px(_) => false,
            Self::Percent(_) => true,
            Self::Sum(terms) | Self::Min(terms) | Self::Max(terms) => {
                terms.iter().any(Self::has_percent)
            }
            Self::Scaled(inner, _) => inner.has_percent(),
            Self::Clamp(a, b, c) => a.has_percent() || b.has_percent() || c.has_percent(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Resolved {
    Auto,
    Px(f64),
    Percent(f64),
    Calc(Formula),
}

impl Resolved {
    pub fn px_or(&self, basis: f64, auto: f64) -> f64 {
        match self {
            Self::Auto => auto,
            Self::Px(px) => *px,
            Self::Percent(p) => basis * p / 100.0,
            Self::Calc(formula) => formula.at(basis),
        }
    }
}

fn container_basis(unit: ContainerUnit, context: &Context) -> f64 {
    let size = context.container.unwrap_or(Container {
        width: context.viewport_width,
        height: context.viewport_height,
    });
    match unit {
        ContainerUnit::Cqw | ContainerUnit::Cqi => size.width,
        ContainerUnit::Cqh | ContainerUnit::Cqb => size.height,
        ContainerUnit::Cqmin => size.width.min(size.height),
        ContainerUnit::Cqmax => size.width.max(size.height),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Units {
    pub font_size: f64,
    pub line_height: f64,
    pub ch: f64,
    pub ex: f64,
}

impl Units {
    pub fn of_size(font_size: f64, line_height: f64) -> Self {
        Self {
            font_size,
            line_height,
            ch: font_size * CH_SHARE,
            ex: font_size * EX_SHARE,
        }
    }
}

pub fn formula(length: &Length, units: &Units, context: &Context) -> Option<Formula> {
    let Units {
        font_size,
        line_height,
        ch,
        ex,
    } = *units;
    Some(match length {
        Length::Auto => return None,
        Length::Px(px) => Formula::Px(px.0),
        Length::Percent(Num(p)) => Formula::Percent(*p),
        Length::Em(Num(n)) => Formula::Px(n * font_size),
        Length::Rem(Num(n)) => Formula::Px(n * context.root_font_size),
        Length::Ch(Num(n)) => Formula::Px(n * ch),
        Length::Ex(Num(n)) => Formula::Px(n * ex),
        Length::Lh(Num(n)) => Formula::Px(n * line_height),
        Length::Vw(Num(n)) => Formula::Px(n * context.viewport_width / 100.0),
        Length::Vh(Num(n)) => Formula::Px(n * context.viewport_height / 100.0),
        Length::Container(unit, Num(n)) => Formula::Px(n * container_basis(*unit, context) / 100.0),
        Length::Calc(calc) => {
            let terms = |list: &[Length]| -> Option<Vec<Formula>> {
                list.iter().map(|l| formula(l, units, context)).collect()
            };
            match calc.as_ref() {
                Calc::Sum(list) => Formula::Sum(terms(list)?),
                Calc::Min(list) => Formula::Min(terms(list)?),
                Calc::Max(list) => Formula::Max(terms(list)?),
                Calc::Scaled(inner, Num(by)) => {
                    Formula::Scaled(Box::new(formula(inner, units, context)?), *by)
                }
                Calc::Clamp(low, preferred, high) => Formula::Clamp(
                    Box::new(formula(low, units, context)?),
                    Box::new(formula(preferred, units, context)?),
                    Box::new(formula(high, units, context)?),
                ),
            }
        }
    })
}

pub fn resolve(length: &Length, units: &Units, context: &Context) -> Resolved {
    match formula(length, units, context) {
        None => Resolved::Auto,
        Some(Formula::Px(px)) => Resolved::Px(px),
        Some(Formula::Percent(p)) => Resolved::Percent(p),
        Some(other) if other.has_percent() => Resolved::Calc(other),
        Some(other) => Resolved::Px(other.at(0.0)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineSpacing {
    Normal,
    Number(f64),
    Px(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decoration {
    pub lines: Vec<DecorationLine>,
    pub style: DecorationStyle,
    pub color: Color,
    pub thickness: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub color: Color,
    pub families: Vec<String>,
    pub font_size: f64,
    pub font_weight: f64,
    pub font_style: FontStyle,
    pub line_height: LineSpacing,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub text_align: TextAlign,
    pub text_transform: TextTransform,
    pub white_space: WhiteSpace,
    pub text_rendering: TextRendering,
    pub word_break: WordBreak,
    pub overflow_wrap: OverflowWrap,
    pub text_wrap: TextWrap,
    pub direction: Direction,
    pub numeric: NumericVariant,
    pub features: Vec<(Tag, i32)>,
    pub variations: Vec<(Tag, Num)>,
    pub text_shadow: Vec<Shadow>,
    pub text_stroke: Option<TextStroke>,
    pub paint_order: PaintOrder,
    pub visibility: Visibility,
    pub tab_size: f64,
    pub decorations: Vec<Decoration>,
    pub ch: f64,
    pub ex: f64,
}

impl TextStyle {
    pub fn initial() -> Self {
        Self {
            color: Color::BLACK,
            families: Vec::new(),
            font_size: ROOT_FONT_SIZE,
            font_weight: NORMAL_WEIGHT,
            font_style: FontStyle::Normal,
            line_height: LineSpacing::Number(1.5),
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_align: TextAlign::Start,
            text_transform: TextTransform::None,
            white_space: WhiteSpace::Normal,
            text_rendering: TextRendering::Auto,
            word_break: WordBreak::Normal,
            overflow_wrap: OverflowWrap::Normal,
            text_wrap: TextWrap::Wrap,
            direction: Direction::Ltr,
            numeric: NumericVariant::default(),
            features: Vec::new(),
            variations: Vec::new(),
            text_shadow: Vec::new(),
            text_stroke: None,
            paint_order: PaintOrder::Normal,
            visibility: Visibility::Visible,
            tab_size: ROOT_TAB_SIZE,
            decorations: Vec::new(),
            ch: ROOT_FONT_SIZE * CH_SHARE,
            ex: ROOT_FONT_SIZE * EX_SHARE,
        }
    }

    pub fn units(&self) -> Units {
        Units {
            font_size: self.font_size,
            line_height: self.line_height_px(),
            ch: self.ch,
            ex: self.ex,
        }
    }

    pub fn line_height_px(&self) -> f64 {
        match self.line_height {
            LineSpacing::Normal => self.font_size * NORMAL_LINE_HEIGHT,
            LineSpacing::Number(n) => self.font_size * n,
            LineSpacing::Px(px) => px,
        }
    }

    #[must_use]
    pub fn child(&self, style: &Style, context: &Context) -> Self {
        let mut out = self.clone();
        if let Some(size) = prop!(style, FontSize) {
            out.font_size =
                resolve(size, &self.units(), context).px_or(self.font_size, self.font_size);
            out.ch = out.font_size * CH_SHARE;
            out.ex = out.font_size * EX_SHARE;
        }
        let font_size = out.font_size;
        let own = Units {
            line_height: out.line_height_px(),
            ..out.units()
        };
        let px = |l: &Length| resolve(l, &own, context).px_or(font_size, 0.0);
        if let Some(color) = prop!(style, Color)
            && *color != Color::CurrentColor
        {
            out.color = color.clone();
        }
        if let Some(families) = prop!(style, FontFamily) {
            out.families.clone_from(families);
        }
        if let Some(Num(weight)) = prop!(style, FontWeight) {
            out.font_weight = *weight;
        }
        if let Some(font_style) = prop!(style, FontStyle) {
            out.font_style = *font_style;
        }
        if let Some(line_height) = prop!(style, LineHeight) {
            out.line_height = match line_height {
                LineHeight::Normal => LineSpacing::Normal,
                LineHeight::Number(Num(n)) => LineSpacing::Number(*n),
                LineHeight::Length(Length::Percent(Num(p))) => {
                    LineSpacing::Px(font_size * p / 100.0)
                }
                LineHeight::Length(l) => LineSpacing::Px(px(l)),
            };
        }
        if let Some(spacing) = prop!(style, LetterSpacing) {
            out.letter_spacing = px(spacing);
        }
        if let Some(spacing) = prop!(style, WordSpacing) {
            out.word_spacing = px(spacing);
        }
        if let Some(Num(size)) = prop!(style, TabSize) {
            out.tab_size = *size;
        }
        macro_rules! copy {
            ($($variant:ident => $field:ident),*) => {$(
                if let Some(value) = prop!(style, $variant) {
                    out.$field = value.clone();
                }
            )*};
        }
        copy! {
            TextAlign => text_align,
            TextTransform => text_transform,
            WhiteSpace => white_space,
            TextRendering => text_rendering,
            WordBreak => word_break,
            OverflowWrap => overflow_wrap,
            TextWrap => text_wrap,
            Direction => direction,
            FontVariantNumeric => numeric,
            FontFeatureSettings => features,
            FontVariationSettings => variations,
            TextShadow => text_shadow,
            PaintOrder => paint_order,
            Visibility => visibility
        }
        if let Some(decoration) = prop!(style, TextDecoration)
            && decoration.lines.iter().any(|l| *l != DecorationLine::None)
        {
            out.decorations.push(Decoration {
                lines: decoration
                    .lines
                    .iter()
                    .copied()
                    .filter(|l| *l != DecorationLine::None)
                    .collect(),
                style: decoration.style,
                color: decoration
                    .color
                    .clone()
                    .unwrap_or_else(|| out.color.clone()),
                thickness: decoration.thickness.as_ref().map(px),
            });
        }
        if let Some(stroke) = prop!(style, TextStroke) {
            out.text_stroke = Some(stroke.clone());
        }
        out
    }

    pub fn bolder(weight: f64) -> f64 {
        if weight < NORMAL_WEIGHT {
            NORMAL_WEIGHT
        } else {
            (weight + BOLDER_STEP).min(900.0)
        }
    }
}
