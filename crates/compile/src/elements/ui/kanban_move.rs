use std::f64::consts::PI;

use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, NumericVariant, Overflow, Position, Style,
    TextOverflow, TextTransform, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float};

use crate::elements::anim::sprung;
use crate::elements::code_theme::{Palette, code_palette};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::units::to_fixed;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct KanbanMoveCard {
    #[setting("Title", default = "CSV export")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Meta", "type" = "string")]
    pub meta: Option<String>,
    #[setting("Column", default = 2)]
    pub column: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Tint", widget = "color", zero_label = "None", "type" = "string")]
    pub tint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Move to", "type" = "integer")]
    pub move_to: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Move at seconds", "type" = "number")]
    pub move_at_seconds: Option<f64>,
}

#[settings(export)]
pub struct KanbanMove {
    #[setting(
        "Columns",
        default = vec!["Backlog".into(), "In progress".into(), "Shipped".into()],
        help = "Column headings, left to right. Cards address columns by index."
    )]
    pub columns: Vec<String>,
    #[setting(
        "Cards",
        default = vec![
            KanbanMoveCard {
                title: "Onboarding checklist".into(),
                meta: Some("APP-218 · Ana".into()),
                column: 0,
                tint: Some("#8FB8F0".into()),
                ..Default::default()
            },
            KanbanMoveCard {
                title: "Retry failed payments".into(),
                meta: Some("APP-204 · Piotr".into()),
                column: 0,
                tint: Some("#E8B86D".into()),
                move_to: Some(1),
                move_at_seconds: Some(0.95),
            },
            KanbanMoveCard {
                title: "Dark mode".into(),
                meta: Some("APP-231 · Dai".into()),
                column: 0,
                tint: Some("#C99BE8".into()),
                ..Default::default()
            },
            KanbanMoveCard {
                title: "Search filters".into(),
                meta: Some("APP-190 · Ana".into()),
                column: 1,
                tint: Some("#7DD3E8".into()),
                move_to: Some(2),
                move_at_seconds: Some(1.72),
            },
            KanbanMoveCard {
                title: "Team invites".into(),
                meta: Some("APP-177 · Kit".into()),
                column: 1,
                tint: Some("#9BD4A0".into()),
                ..Default::default()
            },
            KanbanMoveCard {
                meta: Some("APP-166 · Dai".into()),
                tint: Some("#9BD4A0".into()),
                ..Default::default()
            },
        ],
        help = "Title, meta line, starting column, rail tint, and the one move the card makes."
    )]
    pub cards: Vec<KanbanMoveCard>,
    #[setting(
        "Move time (seconds)",
        default = 0.62,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "How long a card takes to arc from column to column.",
        advanced
    )]
    pub move_seconds: f64,
    #[setting(
        "Card stagger (seconds)",
        default = 0.08,
        step = 0.01,
        help = "Seconds between each card's arrival during the opening deal.",
        advanced
    )]
    pub deal_stagger_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Destination column glow, and the fallback card rail.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting(
        "Speed",
        default = 1.0,
        range(min = 0.25, max = 4),
        step = 0.25,
        help = "Animation speed multiplier."
    )]
    pub speed: f64,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "Theme",
        help = "Leave empty to follow the theme."
    )]
    pub background_color: String,
    #[setting("Show counts", default = true, advanced)]
    pub show_counts: bool,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<KanbanMove>(
        element(
            ElementKind::Component,
            "kanban-move",
            "Kanban board",
            "A task board where cards move from one column to the next.",
            ElementCategory::Ui,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:square-kanban")),
    )
}

const BOARD_FOR: f64 = 0.5;
const DEAL_AT: f64 = 0.18;
const DEAL_FOR: f64 = 0.42;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const BOARD_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 130.0,
    mass: 0.9,
    overshoot_clamping: false,
};

#[derive(Debug, Clone)]
struct Card<'a> {
    source: &'a KanbanMoveCard,
    column: f64,
    move_to: Option<f64>,
    move_at: f64,
}

struct Board<'a> {
    props: &'a KanbanMove,
    cards: Vec<Card<'a>>,
    frame: f64,
    fps: f64,
}

fn same(a: Option<f64>, b: f64) -> bool {
    a.is_some_and(|a| exact_eq(a, b))
}

impl Board<'_> {
    fn at(&self, seconds: f64) -> f64 {
        seconds * self.fps / self.props.speed
    }

    fn ease(&self, from: f64, to: f64, easing: Easing) -> Result<f64, DrawFailure> {
        Ok(interpolate(
            self.frame,
            &[self.at(from), self.at(to)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    }

    fn start_slot(&self, index: usize) -> f64 {
        let column = self.cards[index].column;
        float(
            self.cards[..index]
                .iter()
                .filter(|c| exact_eq(c.column, column))
                .count(),
        )
    }

    fn landing_slot(&self, index: usize) -> f64 {
        let card = &self.cards[index];
        let Some(target) = card.move_to else {
            return 0.0;
        };
        let arrives = card.move_at;
        let resident = self
            .cards
            .iter()
            .filter(|o| exact_eq(o.column, target))
            .count();
        let gone = self
            .cards
            .iter()
            .filter(|o| exact_eq(o.column, target) && o.move_to.is_some() && o.move_at < arrives)
            .count();
        let earlier = self
            .cards
            .iter()
            .enumerate()
            .filter(|(i, o)| *i != index && same(o.move_to, target) && o.move_at < arrives)
            .count();
        float(resident) - float(gone) + float(earlier)
    }

    fn progress(&self, card: &Card<'_>) -> Result<f64, DrawFailure> {
        if card.move_to.is_none() {
            return Ok(0.0);
        }
        self.ease(
            card.move_at,
            card.move_at + self.props.move_seconds,
            EDITORIAL,
        )
    }

    fn closed_up_by(
        &self,
        column: f64,
        own: f64,
        present_from: f64,
        index: usize,
    ) -> Result<f64, DrawFailure> {
        let mut sum = 0.0;
        for (i, other) in self.cards.iter().enumerate() {
            if i == index
                || !exact_eq(other.column, column)
                || other.move_to.is_none()
                || other.move_at < present_from
                || self.start_slot(i) >= own
            {
                continue;
            }
            sum += self.progress(other)?;
        }
        Ok(sum)
    }
}

struct Layout {
    u: f64,
    gutter: f64,
    column_w: f64,
    card_h: f64,
    card_gap: f64,
    header_h: f64,
    list_h: f64,
}

impl Layout {
    fn column_x(&self, column: f64) -> f64 {
        column * (self.column_w + self.gutter)
    }

    fn slot_y(&self, slot: f64) -> f64 {
        slot.mul_add(self.card_h + self.card_gap, self.header_h)
    }
}

fn column_node(
    b: &Board<'_>,
    l: &Layout,
    palette: &Palette,
    index: usize,
    name: &str,
) -> Result<Node, DrawFailure> {
    let (p, u) = (b.props, l.u);
    let at = float(index);
    let mut settled = 0;
    let mut incoming = 0.0f64;
    for (i, card) in b.cards.iter().enumerate() {
        let progress = b.progress(card)?;
        let place = if progress > 0.5 {
            card.move_to
        } else {
            Some(card.column)
        };
        let deal = float(i).mul_add(p.deal_stagger_seconds, DEAL_AT);
        if same(place, at) && b.ease(deal, deal + DEAL_FOR, easing_enter())? > 0.5 {
            settled += 1;
        }
        if same(card.move_to, at) {
            incoming = incoming.max((progress * (1.0 - progress) * 4.0).min(1.0));
        }
    }
    let lit = incoming > 0.02;
    let border = if lit {
        tinted(
            &p.accent_color,
            to_fixed(incoming.mul_add(120.0, 40.0) / 2.55, 1),
        )
    } else {
        css_color(palette.get("border"))
    };
    let mut style = Style::default()
        .position(Position::Absolute)
        .left(Length::px(l.column_x(at)))
        .top(Length::ZERO)
        .width(Length::px(l.column_w))
        .height(Length::px(l.header_h + l.list_h))
        .border_radius_all(14.0 * u)
        .background_color(css_color(palette.get("band")))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(border),
        }));
    if lit {
        style = style.box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::ZERO,
            blur: Length::px(incoming * 26.0 * u),
            spread: Length::ZERO,
            color: tinted(&p.accent_color, 20.0),
        }]);
    }
    let mut count = span(
        Style::default()
            .color(css_color(palette.get("faint")))
            .font_size(14.5 * u)
            .font_weight(600.0)
            .font_variant_numeric(NumericVariant {
                tabular: true,
                ..NumericVariant::default()
            }),
    );
    if p.show_counts {
        count = count.child(settled.to_string());
    }
    Ok(div(style)
        .child(
            div(Style::default()
                .height(Length::px(l.header_h))
                .display(Display::Flex)
                .align_items(Align::Center)
                .justify_content(Align::SpaceBetween)
                .padding(Edges::axes(Length::ZERO, Length::px(12.0 * u))))
            .child(
                span(
                    Style::default()
                        .color(css_color(palette.get("dim")))
                        .font_size(14.5 * u)
                        .font_weight(600.0)
                        .letter_spacing(Length::em(0.06))
                        .text_transform(TextTransform::Uppercase),
                )
                .child(name.to_owned()),
            )
            .child(count),
        )
        .into())
}

fn ellipsized(style: Style) -> Style {
    style
        .white_space(WhiteSpace::Nowrap)
        .overflow(Overflow::Hidden)
        .text_overflow(TextOverflow::Ellipsis)
}

fn card_node(
    board: &Board<'_>,
    grid: &Layout,
    palette: &Palette,
    index: usize,
) -> Result<Node, DrawFailure> {
    let (p, u) = (board.props, grid.u);
    let card = &board.cards[index];
    let deal = float(index).mul_add(p.deal_stagger_seconds, DEAL_AT);
    let dealt = board.ease(deal, deal + DEAL_FOR, easing_enter())?;
    let progress = board.progress(card)?;
    let home = board.start_slot(index);
    let from_slot = home - board.closed_up_by(card.column, home, f64::NEG_INFINITY, index)?;
    let to_slot = match card.move_to {
        None => from_slot,
        Some(target) => {
            let landing = board.landing_slot(index);
            landing - board.closed_up_by(target, landing, card.move_at, index)?
        }
    };
    let line = |from: f64, to: f64| {
        interpolate(
            progress,
            &[0.0, 1.0],
            &[from, to],
            &Interpolation::default(),
        )
    };
    let x = line(
        grid.column_x(card.column),
        grid.column_x(card.move_to.unwrap_or(card.column)),
    )?;
    let y = line(grid.slot_y(from_slot), grid.slot_y(to_slot))?;
    let lift = (PI * progress).sin();
    let tint = card.source.tint.as_deref().unwrap_or(&p.accent_color);
    let grow = dealt.mul_add(0.06, 0.94) * lift.mul_add(0.05, 1.0);
    let mut text = div(Style::default().min_width(Length::ZERO)).child(
        div(ellipsized(
            Style::default()
                .color(css_color(palette.get("fg")))
                .font_size(16.0 * u)
                .font_weight(600.0)
                .letter_spacing(Length::em(-0.01)),
        ))
        .child(card.source.title.clone()),
    );
    if let Some(meta) = card.source.meta.as_deref().filter(|m| !m.is_empty()) {
        text = text.child(
            div(ellipsized(
                Style::default()
                    .margin_top(Length::px(4.0 * u))
                    .color(css_color(palette.get("dim")))
                    .font_size(14.5 * u)
                    .font_weight(500.0),
            ))
            .child(meta.to_owned()),
        );
    }
    Ok(div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(8.0f64.mul_add(u, x)))
        .top(Length::px(lift.mul_add(-22.0 * u, y)))
        .width(Length::px(16.0f64.mul_add(-u, grid.column_w)))
        .height(Length::px(grid.card_h))
        .z_index(if lift > 0.01 { 20 } else { 1 })
        .border_radius_all(11.0 * u)
        .background_color(css_color(palette.get("window")))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(if lift > 0.05 {
                tinted(tint, 40.0)
            } else {
                css_color(palette.get("border"))
            }),
        }))
        .box_shadow(vec![
            Shadow {
                inset: true,
                x: Length::ZERO,
                y: Length::px(1.0),
                blur: Length::ZERO,
                spread: Length::ZERO,
                color: css_color(palette.get("highlight")),
            },
            Shadow {
                inset: false,
                x: Length::ZERO,
                y: Length::px(lift.mul_add(20.0, 4.0) * u),
                blur: Length::px(lift.mul_add(34.0, 10.0) * u),
                spread: Length::ZERO,
                color: css_color(palette.get("shadow")),
            },
        ])
        .opacity(dealt)
        .transform(vec![
            Transform::Scale(Num(grow), Num(grow), Num(1.0)),
            Transform::Rotate(Angle::deg(lift * -1.6)),
        ])
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(10.0 * u)
        .padding(Edges::axes(Length::ZERO, Length::px(12.0 * u)))
        .overflow(Overflow::Hidden))
    .child(div(Style::default()
        .width(Length::px(3.0 * u))
        .height(Length::px(40.0 * u))
        .border_radius_all(999.0)
        .background_color(css_color(tint))
        .opacity(0.85)
        .flex_shrink(0.0)))
    .child(text)
    .into())
}

pub fn draw(props: &KanbanMove, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let design_u = if height > width {
        (width / 496.0).min(height / 896.0)
    } else {
        (width / 1024.0).min(height / 576.0)
    };
    let last = float(props.columns.len().saturating_sub(1));
    let on_board = |column: f64| ((column + 0.5).floor()).clamp(0.0, last);
    let cards: Vec<Card<'_>> = props
        .cards
        .iter()
        .map(|source| {
            let column = on_board(f64::from(source.column));
            let move_to = match (source.move_to, source.move_at_seconds) {
                (Some(to), Some(_)) => {
                    Some(on_board(f64::from(to))).filter(|t| !exact_eq(*t, column))
                }
                _ => None,
            };
            Card {
                source,
                column,
                move_to,
                move_at: source.move_at_seconds.unwrap_or(0.0),
            }
        })
        .collect();
    let board = Board {
        props,
        cards,
        frame: cx.frame(),
        fps: cx.fps(),
    };
    let rows = (0..board.cards.len())
        .map(|i| {
            let landing = if board.cards[i].move_to.is_some() {
                board.landing_slot(i)
            } else {
                0.0
            };
            board.start_slot(i).max(landing) + 1.0
        })
        .fold(3.0f64, f64::max);
    let u = design_u.min(
        f64::mul_add(edge, -2.0, height) / (rows - 1.0).mul_add(10.0, rows.mul_add(74.0, 34.0)),
    );
    let board_in = sprung(board.frame - board.at(0.0), board.fps, BOARD_SPRING)?;
    let open = board.ease(0.0, BOARD_FOR, EDITORIAL)?;
    let board_w = f64::mul_add(side, -2.0, width).min(1000.0 * u);
    let gutter = 18.0 * u;
    let columns = float(props.columns.len());
    let layout = Layout {
        u,
        gutter,
        column_w: f64::mul_add(gutter, -(columns - 1.0), board_w) / columns,
        card_h: 74.0 * u,
        card_gap: 10.0 * u,
        header_h: 34.0 * u,
        list_h: (74.0 * u).mul_add(rows, 10.0 * u * (rows - 1.0)),
    };
    let mut nodes = Vec::new();
    for (i, name) in props.columns.iter().enumerate() {
        nodes.push(column_node(&board, &layout, &palette, i, name)?);
    }
    for i in 0..board.cards.len() {
        nodes.push(card_node(&board, &layout, &palette, i)?);
    }
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    Ok(div(with_background(
        absolute_fill()
            .font_family(font_or(&props.font_family, FontRole::Main))
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .padding(Edges::axes(Length::px(edge), Length::px(side))),
        page,
    ))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .width(Length::px(board_w))
            .height(Length::px(layout.header_h + layout.list_h))
            .opacity(open)
            .transform(vec![Transform::Translate(
                Length::ZERO,
                Length::px((1.0 - board_in) * 22.0 * u),
                Length::ZERO,
            )]))
        .children(nodes),
    )
    .into())
}
