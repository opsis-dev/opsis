use opsis_scene::prop;
use opsis_scene::style::{
    Align, BorderStyle, BoxSizing as SceneBoxSizing, Display as SceneDisplay,
    FlexDirection as SceneFlexDirection, FlexWrap as SceneFlexWrap, GridLine,
    Overflow as SceneOverflow, Position as ScenePosition, RepeatCount, Style as SceneStyle, Track,
};
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::numbers::narrow;
use taffy::prelude::*;
use taffy::style::{
    GridTemplateComponent, MaxTrackSizingFunction, MinTrackSizingFunction, Overflow,
    RepetitionCount,
};

use crate::computed::{Context, Formula, Resolved, TextStyle, resolve};
use crate::tree::calc_token;

pub const SVG_DEFAULT_WIDTH: f64 = 300.0;
pub const SVG_DEFAULT_HEIGHT: f64 = 150.0;

pub struct Lengths<'a> {
    pub text: &'a TextStyle,
    pub context: &'a Context,
    pub formulas: &'a mut Vec<Formula>,
}

impl Lengths<'_> {
    fn resolved(&self, length: &Length) -> Resolved {
        resolve(length, &self.text.units(), self.context)
    }

    fn token(&mut self, formula: Formula) -> *const () {
        self.formulas.push(formula);
        calc_token(self.formulas.len() - 1)
    }

    pub fn lp(&mut self, length: &Length) -> LengthPercentage {
        match self.resolved(length) {
            Resolved::Auto => LengthPercentage::length(0.0),
            Resolved::Px(px) => LengthPercentage::length(narrow(px)),
            Resolved::Percent(p) => LengthPercentage::percent(narrow(p / 100.0)),
            Resolved::Calc(formula) => LengthPercentage::calc(self.token(formula)),
        }
    }

    pub fn lpa(&mut self, length: &Length) -> LengthPercentageAuto {
        match self.resolved(length) {
            Resolved::Auto => LengthPercentageAuto::auto(),
            Resolved::Px(px) => LengthPercentageAuto::length(narrow(px)),
            Resolved::Percent(p) => LengthPercentageAuto::percent(narrow(p / 100.0)),
            Resolved::Calc(formula) => LengthPercentageAuto::calc(self.token(formula)),
        }
    }

    pub fn dimension(&mut self, length: &Length) -> Dimension {
        match self.resolved(length) {
            Resolved::Auto => Dimension::auto(),
            Resolved::Px(px) => Dimension::length(narrow(px)),
            Resolved::Percent(p) => Dimension::percent(narrow(p / 100.0)),
            Resolved::Calc(formula) => Dimension::calc(self.token(formula)),
        }
    }

    fn edges_lp(&mut self, edges: &Edges<Length>) -> Rect<LengthPercentage> {
        Rect {
            left: self.lp(&edges.left),
            right: self.lp(&edges.right),
            top: self.lp(&edges.top),
            bottom: self.lp(&edges.bottom),
        }
    }

    fn edges_lpa(&mut self, edges: &Edges<Length>) -> Rect<LengthPercentageAuto> {
        Rect {
            left: self.lpa(&edges.left),
            right: self.lpa(&edges.right),
            top: self.lpa(&edges.top),
            bottom: self.lpa(&edges.bottom),
        }
    }

    fn min_track(&mut self, track: &Track) -> MinTrackSizingFunction {
        match track {
            Track::Length(length) => match self.resolved(length) {
                Resolved::Auto => MinTrackSizingFunction::auto(),
                Resolved::Px(px) => MinTrackSizingFunction::length(narrow(px)),
                Resolved::Percent(p) => MinTrackSizingFunction::percent(narrow(p / 100.0)),
                Resolved::Calc(formula) => MinTrackSizingFunction::calc(self.token(formula)),
            },
            Track::MinContent => MinTrackSizingFunction::min_content(),
            Track::MaxContent => MinTrackSizingFunction::max_content(),
            Track::Fraction(_) | Track::Auto | Track::MinMax(..) | Track::Repeat(..) => {
                MinTrackSizingFunction::auto()
            }
        }
    }

    fn max_track(&mut self, track: &Track) -> MaxTrackSizingFunction {
        match track {
            Track::Length(length) => match self.resolved(length) {
                Resolved::Auto => MaxTrackSizingFunction::auto(),
                Resolved::Px(px) => MaxTrackSizingFunction::length(narrow(px)),
                Resolved::Percent(p) => MaxTrackSizingFunction::percent(narrow(p / 100.0)),
                Resolved::Calc(formula) => MaxTrackSizingFunction::calc(self.token(formula)),
            },
            Track::Fraction(Num(n)) => MaxTrackSizingFunction::fr(narrow(*n)),
            Track::MinContent => MaxTrackSizingFunction::min_content(),
            Track::MaxContent => MaxTrackSizingFunction::max_content(),
            Track::Auto | Track::MinMax(..) | Track::Repeat(..) => MaxTrackSizingFunction::auto(),
        }
    }

    fn track(&mut self, track: &Track) -> TrackSizingFunction {
        match track {
            Track::MinMax(min, max) => minmax(self.min_track(min), self.max_track(max)),
            Track::Fraction(_) => minmax(MinTrackSizingFunction::auto(), self.max_track(track)),
            other => minmax(self.min_track(other), self.max_track(other)),
        }
    }

    fn template(&mut self, tracks: &[Track]) -> Vec<GridTemplateComponent<String>> {
        tracks
            .iter()
            .map(|track| match track {
                Track::Repeat(count, inner) => {
                    let count = match count {
                        RepeatCount::Times(n) => RepetitionCount::Count(*n),
                        RepeatCount::AutoFill => RepetitionCount::AutoFill,
                        RepeatCount::AutoFit => RepetitionCount::AutoFit,
                    };
                    let sized: Vec<TrackSizingFunction> =
                        inner.iter().map(|t| self.track(t)).collect();
                    repeat(count, sized)
                }
                other => GridTemplateComponent::Single(self.track(other)),
            })
            .collect()
    }
}

const fn align_items(align: Align) -> Option<AlignItems> {
    Some(match align {
        Align::Start => AlignItems::START,
        Align::End => AlignItems::END,
        Align::FlexStart => AlignItems::FLEX_START,
        Align::FlexEnd => AlignItems::FLEX_END,
        Align::Center => AlignItems::CENTER,
        Align::Baseline => AlignItems::BASELINE,
        Align::Stretch => AlignItems::STRETCH,
        Align::Normal | Align::SpaceBetween | Align::SpaceAround | Align::SpaceEvenly => {
            return None;
        }
    })
}

const fn align_content(align: Align) -> Option<AlignContent> {
    Some(match align {
        Align::Start => AlignContent::START,
        Align::End => AlignContent::END,
        Align::FlexStart => AlignContent::FLEX_START,
        Align::FlexEnd => AlignContent::FLEX_END,
        Align::Center => AlignContent::CENTER,
        Align::Stretch => AlignContent::STRETCH,
        Align::SpaceBetween => AlignContent::SPACE_BETWEEN,
        Align::SpaceAround => AlignContent::SPACE_AROUND,
        Align::SpaceEvenly => AlignContent::SPACE_EVENLY,
        Align::Normal | Align::Baseline => return None,
    })
}

fn grid_line(line: GridLine) -> GridPlacement<String> {
    match line {
        GridLine::Auto => GridPlacement::Auto,
        GridLine::Line(n) => line_placement(n),
        GridLine::Span(n) => GridPlacement::Span(n),
    }
}

fn line_placement(n: i16) -> GridPlacement<String> {
    taffy::style_helpers::line(n)
}

const fn overflow(value: SceneOverflow) -> Overflow {
    match value {
        SceneOverflow::Visible => Overflow::Visible,
        SceneOverflow::Hidden => Overflow::Hidden,
        SceneOverflow::Clip => Overflow::Clip,
    }
}

fn makes_containing_block(style: &SceneStyle) -> bool {
    prop!(style, Transform).is_some_and(|t| !t.is_empty())
        || prop!(style, Translate).is_some()
        || prop!(style, Rotate).is_some()
        || prop!(style, Scale).is_some()
        || prop!(style, Filter).is_some_and(|f| !f.is_empty())
        || prop!(style, Perspective).is_some()
        || prop!(style, ContainerType)
            .is_some_and(|c| *c != opsis_scene::style::ContainerType::Normal)
}

const fn display(value: SceneDisplay) -> Display {
    match value {
        SceneDisplay::Flex | SceneDisplay::InlineFlex => Display::Flex,
        SceneDisplay::Grid | SceneDisplay::InlineGrid => Display::Grid,
        SceneDisplay::None => Display::None,
        SceneDisplay::Block
        | SceneDisplay::Inline
        | SceneDisplay::InlineBlock
        | SceneDisplay::Contents
        | SceneDisplay::WebkitBox => Display::Block,
    }
}

pub fn border_widths(style: &SceneStyle, lengths: &mut Lengths<'_>) -> Rect<LengthPercentage> {
    let Some(border) = prop!(style, Border) else {
        return Rect::zero();
    };
    let mut side = |side: &opsis_scene::style::BorderSide| {
        if side.style == BorderStyle::None {
            LengthPercentage::length(0.0)
        } else {
            lengths.lp(&side.width)
        }
    };
    Rect {
        left: side(&border.left),
        right: side(&border.right),
        top: side(&border.top),
        bottom: side(&border.bottom),
    }
}

pub fn taffy_style(
    scene: &SceneStyle,
    scene_display: SceneDisplay,
    _blockify: bool,
    text: &TextStyle,
    context: &Context,
    formulas: &mut Vec<Formula>,
) -> Style {
    let mut lengths = Lengths {
        text,
        context,
        formulas,
    };
    let mut style = Style {
        display: display(scene_display),
        box_sizing: match prop!(scene, BoxSizing) {
            Some(SceneBoxSizing::ContentBox) => BoxSizing::ContentBox,
            _ => BoxSizing::BorderBox,
        },
        ..Style::default()
    };
    style.position = match prop!(scene, Position) {
        Some(ScenePosition::Absolute | ScenePosition::Fixed) => Position::Absolute,
        Some(ScenePosition::Relative) => Position::Relative,
        Some(ScenePosition::Static) | None => {
            if makes_containing_block(scene) {
                Position::Relative
            } else {
                Position::Static
            }
        }
    };
    let positioned = style.position != Position::Static;
    let inset = |l: Option<&Length>, lengths: &mut Lengths<'_>| {
        l.filter(|_| positioned)
            .map_or(LengthPercentageAuto::auto(), |l| lengths.lpa(l))
    };
    style.inset = Rect {
        left: inset(prop!(scene, Left), &mut lengths),
        right: inset(prop!(scene, Right), &mut lengths),
        top: inset(prop!(scene, Top), &mut lengths),
        bottom: inset(prop!(scene, Bottom), &mut lengths),
    };
    let dimension = |l: Option<&Length>, lengths: &mut Lengths<'_>| {
        l.map_or(Dimension::auto(), |l| lengths.dimension(l))
    };
    style.size = Size {
        width: dimension(prop!(scene, Width), &mut lengths),
        height: dimension(prop!(scene, Height), &mut lengths),
    };
    let bound = |l: Option<&Length>, lengths: &mut Lengths<'_>| {
        l.map_or(LengthPercentageAuto::auto(), |l| lengths.lpa(l))
    };
    style.min_size = Size {
        width: bound(prop!(scene, MinWidth), &mut lengths),
        height: bound(prop!(scene, MinHeight), &mut lengths),
    };
    style.max_size = Size {
        width: bound(prop!(scene, MaxWidth), &mut lengths),
        height: bound(prop!(scene, MaxHeight), &mut lengths),
    };
    style.aspect_ratio = prop!(scene, AspectRatio).map(|Num(r)| narrow(*r));
    if let Some(margin) = prop!(scene, Margin) {
        style.margin = lengths.edges_lpa(margin);
    }
    if let Some(padding) = prop!(scene, Padding) {
        style.padding = lengths.edges_lp(padding);
    }
    style.border = border_widths(scene, &mut lengths);
    style.overflow = taffy::Point {
        x: prop!(scene, OverflowX)
            .copied()
            .map_or(Overflow::Visible, overflow),
        y: prop!(scene, OverflowY)
            .copied()
            .map_or(Overflow::Visible, overflow),
    };
    style.flex_direction = match prop!(scene, FlexDirection) {
        Some(SceneFlexDirection::Column) => FlexDirection::Column,
        Some(SceneFlexDirection::RowReverse) => FlexDirection::RowReverse,
        Some(SceneFlexDirection::ColumnReverse) => FlexDirection::ColumnReverse,
        Some(SceneFlexDirection::Row) | None => FlexDirection::Row,
    };
    style.flex_wrap = match prop!(scene, FlexWrap) {
        Some(SceneFlexWrap::Wrap) => FlexWrap::Wrap,
        Some(SceneFlexWrap::WrapReverse) => FlexWrap::WrapReverse,
        Some(SceneFlexWrap::Nowrap) | None => FlexWrap::NoWrap,
    };
    if let Some(Num(grow)) = prop!(scene, FlexGrow) {
        style.flex_grow = narrow(*grow);
    }
    if let Some(Num(shrink)) = prop!(scene, FlexShrink) {
        style.flex_shrink = narrow(*shrink);
    }
    if let Some(basis) = prop!(scene, FlexBasis) {
        style.flex_basis = lengths.dimension(basis);
    }
    style.justify_content = prop!(scene, JustifyContent)
        .copied()
        .and_then(align_content)
        .unwrap_or(AlignContent::NORMAL);
    style.align_content = prop!(scene, AlignContent)
        .copied()
        .and_then(align_content)
        .unwrap_or(AlignContent::NORMAL);
    style.align_items = prop!(scene, AlignItems)
        .copied()
        .and_then(align_items)
        .unwrap_or(AlignItems::NORMAL);
    style.align_self = prop!(scene, AlignSelf).copied().and_then(align_items);
    style.justify_items = prop!(scene, JustifyItems)
        .copied()
        .and_then(align_items)
        .unwrap_or(AlignItems::NORMAL);
    style.justify_self = prop!(scene, JustifySelf).copied().and_then(align_items);
    style.gap = Size {
        width: prop!(scene, ColumnGap).map_or(LengthPercentage::length(0.0), |g| lengths.lp(g)),
        height: prop!(scene, RowGap).map_or(LengthPercentage::length(0.0), |g| lengths.lp(g)),
    };
    if let Some(tracks) = prop!(scene, GridTemplateColumns) {
        style.grid_template_columns = lengths.template(tracks);
    }
    if let Some(tracks) = prop!(scene, GridTemplateRows) {
        style.grid_template_rows = lengths.template(tracks);
    }
    if let Some(tracks) = prop!(scene, GridAutoRows) {
        style.grid_auto_rows = tracks.iter().map(|t| lengths.track(t)).collect();
    }
    if let Some(tracks) = prop!(scene, GridAutoColumns) {
        style.grid_auto_columns = tracks.iter().map(|t| lengths.track(t)).collect();
    }
    if let Some((start, end)) = prop!(scene, GridColumn) {
        style.grid_column = Line {
            start: grid_line(*start),
            end: grid_line(*end),
        };
    }
    if let Some((start, end)) = prop!(scene, GridRow) {
        style.grid_row = Line {
            start: grid_line(*start),
            end: grid_line(*end),
        };
    }
    style
}
