use opsis_macros::{choice, settings};
use opsis_scene::color::Color;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::paint::{Shadow, Transform};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, Overflow, Position, Style, TextAlign,
    TextOverflow, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length, Num};
use opsis_types::looks::NodeLook;

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::neutral_theme::{NeutralTheme, neutral_theme};
use crate::elements::oklch_mix::mix_oklch;
use crate::elements::theme::{Theme, neutral_themes};
use crate::elements::units::{utf16_len, utf16_slice};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum MenuState {
    #[label("Open")]
    Opened,
    #[label("Closed")]
    Closed,
}

#[choice(export)]
pub enum MenuIcon {
    Search,
    Settings,
    User,
    File,
}

#[settings(export, eq)]
pub struct CommandMenuItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Icon", "type" = "string", "enum" = ["search", "settings", "user", "file"])]
    pub icon: Option<MenuIcon>,
    #[setting("Label", default = "New item")]
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Shortcut", "type" = "string")]
    pub shortcut: Option<String>,
}

#[settings(export)]
pub struct CommandMenu {
    #[setting(
        "Menu",
        default = MenuState::Opened,
        show_if = { "autoplay": ["false"] }
    )]
    pub state: MenuState,
    #[setting("Search", default = "new")]
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Letters typed",
        range(min = 0, max = 200),
        zero_label = "All",
        step = 1,
        show_if = { "autoplay": ["false"] },
        "type" = "integer",
        advanced
    )]
    pub reveal_count: Option<u32>,
    #[setting(
        "Selected item",
        default = -1,
        range(min = -1, max = 20),
        step = 1,
        help = "Counting from 0. -1 selects none.",
        show_if = { "autoplay": ["false"] }
    )]
    pub selected_index: i32,
    #[setting(
        "Hovered item",
        default = -1,
        range(min = -1, max = 20),
        step = 1,
        help = "Counting from 0. -1 hovers none.",
        show_if = { "autoplay": ["false"] },
        advanced
    )]
    pub highlighted_index: i32,
    #[setting("Theme", transform = neutral_themes, widget = "theme")]
    pub theme: Theme,
    #[setting(
        "Items",
        default = vec![
            CommandMenuItem {
                icon: Some(MenuIcon::User),
                label: "Profile".into(),
                shortcut: Some("⌘ P".into()),
            },
            CommandMenuItem {
                icon: Some(MenuIcon::Settings),
                label: "Settings".into(),
                shortcut: Some("⌘ S".into()),
            },
            CommandMenuItem {
                icon: Some(MenuIcon::File),
                label: "New File".into(),
                shortcut: Some("⌘ N".into()),
            },
            CommandMenuItem {
                icon: Some(MenuIcon::Search),
                label: "Search docs".into(),
                ..Default::default()
            },
        ]
    )]
    pub items: Vec<CommandMenuItem>,
    #[setting(
        "Play it out",
        default = true,
        help = "Opens the menu, types the search, then picks the first match. Off shows one moment you set below."
    )]
    pub autoplay: bool,
    #[setting(
        "Press item",
        default = -1,
        range(min = -1, max = 20),
        step = 1,
        help = "The item that gets pressed, counting from 0. -1 presses none.",
        show_if = { "autoplay": ["false"] },
        advanced
    )]
    pub pressed_index: i32,
    #[setting(
        "Time per letter",
        default = 0.0833,
        range(min = 0.0333, max = 0.4),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub letter_seconds: f64,
    #[setting(
        "Pause before picking",
        default = 0.6,
        range(min = 0, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub pause_seconds: f64,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4))]
    pub speed: f64,
    #[setting(
        "Match picked",
        default = 0,
        range(min = 0, max = 20),
        step = 1,
        help = "Which of the matching items gets picked, counting from 0.",
        show_if = { "autoplay": ["true"] },
        advanced
    )]
    pub pick_index: u32,
    #[setting("Placeholder", default = "Type a command or search…", advanced)]
    pub placeholder: String,
    #[setting("No match text", default = "No results found.", advanced)]
    pub empty_text: String,
    #[setting("Backdrop dimming", default = 0.5, range(min = 0, max = 1), advanced)]
    pub backdrop_opacity: f64,
    #[setting(
        "Menu width",
        default = 440,
        range(min = 280, max = 900),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_width: u32,
    #[setting(
        "Distance from top",
        default = 18.0,
        range(min = 0, max = 60),
        unit = "%",
        advanced
    )]
    pub top: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
    #[setting(
        "Corner radius",
        default = 10,
        range(min = 0, max = 32),
        step = 1,
        unit = "px",
        advanced
    )]
    pub radius: u32,
}

pub fn definition() -> ElementMeta {
    meta::<CommandMenu>(
        element(
            ElementKind::Component,
            "command-menu",
            "Command menu",
            "A command palette that opens, filters and picks an item.",
            ElementCategory::Scene,
            240,
        )
        .authored_clock()
        .exact_design(1280, 720)
        .look(NodeLook::of("lucide:search-code")),
    )
}

const OPEN_AT: f64 = 6.0;
const OPEN_FOR: f64 = 12.0;
const TYPE_AT: f64 = 24.0;
const HOVER_AFTER_TYPING: f64 = 8.0;
const PRESS_FOR: f64 = 6.0;
const SEARCH_PATH: &str = "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14ZM20 20l-3.5-3.5";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowState {
    Idle,
    Hover,
    Press,
    Selected,
}

struct Look {
    backdrop: f64,
    opacity: f64,
    scale: f64,
    rise: f64,
}

const OPENED: Look = Look {
    backdrop: 1.0,
    opacity: 1.0,
    scale: 1.0,
    rise: 0.0,
};

const CLOSED: Look = Look {
    backdrop: 0.0,
    opacity: 0.0,
    scale: 0.96,
    rise: 8.0,
};

#[expect(
    clippy::suboptimal_flops,
    reason = "the film blends the two looks with separate multiplies and adds"
)]
fn blend(a: &Look, b: &Look, t: f64) -> Look {
    Look {
        backdrop: a.backdrop + (b.backdrop - a.backdrop) * t,
        opacity: a.opacity + (b.opacity - a.opacity) * t,
        scale: a.scale + (b.scale - a.scale) * t,
        rise: a.rise + (b.rise - a.rise) * t,
    }
}

fn revealed(full: &str, count: f64) -> String {
    let shown = count
        .floor()
        .clamp(0.0, opsis_types::numbers::float(utf16_len(full)));
    utf16_slice(
        full,
        0,
        usize::try_from(opsis_types::numbers::whole(shown)).unwrap_or(0),
    )
}

const fn icon_path(icon: MenuIcon) -> &'static str {
    match icon {
        MenuIcon::Search => SEARCH_PATH,
        MenuIcon::Settings => {
            "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6ZM12 2v3 M12 19v3 M4.2 4.2l2.1 2.1 M17.7 17.7l2.1 2.1 M2 12h3 M19 12h3 M4.2 19.8l2.1-2.1 M17.7 6.3l2.1-2.1"
        }
        MenuIcon::User => "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM5 20a7 7 0 0 1 14 0",
        MenuIcon::File => "M7 3h7l4 4v14H7ZM14 3v4h4",
    }
}

fn icon(path: &str, size: f64, color: &str) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr("d", path)
                    .attr("stroke", color)
                    .attr("stroke-width", "2")
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round"),
            ),
        Style::default()
            .width(Length::px(size))
            .height(Length::px(size)),
    )
    .into()
}

fn solid(color: &str) -> Edges<BorderSide> {
    Edges::all(BorderSide {
        width: Length::px(1.0),
        style: BorderStyle::Solid,
        color: Some(css_color(color)),
    })
}

fn row(
    item: &CommandMenuItem,
    state: RowState,
    theme: &NeutralTheme,
    width: f64,
    radius: f64,
) -> Node {
    let (background, label, mark, scale) = match state {
        RowState::Hover => (
            theme.get("accent").to_owned(),
            theme.get("popoverForeground"),
            theme.get("foreground"),
            1.0,
        ),
        RowState::Press => (
            mix_oklch(theme.get("accent"), theme.get("foreground"), 0.08),
            theme.get("popoverForeground"),
            theme.get("foreground"),
            0.98,
        ),
        RowState::Selected => (
            theme.get("accent").to_owned(),
            theme.get("accentForeground"),
            theme.get("foreground"),
            1.0,
        ),
        RowState::Idle => (
            theme.get("popover").to_owned(),
            theme.get("popoverForeground"),
            theme.get("mutedForeground"),
            1.0,
        ),
    };
    let mut lead = span(
        Style::default()
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(10.0)
            .min_width(Length::ZERO),
    );
    if let Some(kind) = item.icon {
        lead = lead.child(
            span(Style::default().display(Display::Flex).flex_shrink(0.0)).child(icon(
                icon_path(kind),
                16.0,
                mark,
            )),
        );
    }
    lead = lead.child(
        span(
            Style::default()
                .overflow(Overflow::Hidden)
                .text_overflow(TextOverflow::Ellipsis)
                .white_space(WhiteSpace::Nowrap),
        )
        .child(item.label.clone()),
    );
    let mut out = div(Style::default()
        .width(Length::px(width))
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::SpaceBetween)
        .gap(12.0)
        .padding(Edges::axes(Length::px(8.0), Length::px(12.0)))
        .border_radius_all(radius)
        .transform(vec![Transform::Scale(Num(scale), Num(scale), Num(1.0))])
        .background_color(css_color(&background))
        .color(css_color(label))
        .font_size(14.0)
        .letter_spacing(Length::em(-0.01)))
    .child(lead);
    if let Some(shortcut) = &item.shortcut {
        out = out.child(
            span(
                Style::default()
                    .flex_shrink(0.0)
                    .display(Display::InlineFlex)
                    .align_items(Align::Center)
                    .gap(2.0)
                    .height(Length::px(20.0))
                    .padding(Edges::axes(Length::ZERO, Length::px(6.0)))
                    .font_size(12.0)
                    .letter_spacing(Length::em(0.05))
                    .color(css_color(theme.get("mutedForeground")))
                    .background_color(css_color(theme.get("muted")))
                    .border(solid(theme.get("border")))
                    .border_radius_all(4.0f64.max(radius - 4.0)),
            )
            .child(shortcut.clone()),
        );
    }
    out.into()
}

fn index_of(items: &[CommandMenuItem], item: Option<&CommandMenuItem>) -> i64 {
    item.and_then(|wanted| items.iter().position(|i| std::ptr::eq(i, wanted)))
        .and_then(|i| i64::try_from(i).ok())
        .unwrap_or(-1)
}

#[expect(
    clippy::too_many_lines,
    reason = "one pass over the menu like the film's component"
)]
pub fn draw(props: &CommandMenu, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let frame = cx.authored_frame() * props.speed;
    let theme = neutral_theme(&props.theme);
    let radius = f64::from(props.radius);
    let opened = interpolate(
        frame,
        &[OPEN_AT, OPEN_AT + OPEN_FOR],
        &[0.0, 1.0],
        &Interpolation {
            easing: Some(Easing::ease_out(Easing::Cubic)),
            ..Interpolation::clamped()
        },
    )?;
    let letters = opsis_types::numbers::float(props.query.chars().count());
    let typed = 0.0f64.max(((frame - TYPE_AT) / props.letter_seconds).floor() + 1.0);
    let shown = if props.autoplay {
        Some(typed.min(letters))
    } else {
        props.reveal_count.map(f64::from)
    };
    let look = if props.autoplay {
        blend(&CLOSED, &OPENED, opened)
    } else if props.state == MenuState::Opened {
        OPENED
    } else {
        CLOSED
    };
    let visible = shown.map_or_else(
        || props.query.clone(),
        |count| revealed(&props.query, count),
    );
    let needle = visible.trim().to_lowercase();
    let filtered: Vec<&CommandMenuItem> = props
        .items
        .iter()
        .filter(|item| needle.is_empty() || item.label.to_lowercase().contains(&needle))
        .collect();
    let typing_end = letters.mul_add(props.letter_seconds, TYPE_AT);
    let chosen = filtered
        .len()
        .checked_sub(1)
        .and_then(|last| filtered.get(usize::try_from(props.pick_index).unwrap_or(0).min(last)))
        .copied();
    let pick = index_of(&props.items, chosen);
    let hover_at = typing_end + HOVER_AFTER_TYPING;
    let press_at = hover_at + props.pause_seconds;
    let (highlighted, pressed, selected) = if props.autoplay {
        (
            if frame >= hover_at { pick } else { -1 },
            if frame >= press_at && frame < press_at + PRESS_FOR {
                pick
            } else {
                -1
            },
            if frame >= press_at + PRESS_FOR {
                pick
            } else {
                -1
            },
        )
    } else {
        (
            i64::from(props.highlighted_index),
            i64::from(props.pressed_index),
            i64::from(props.selected_index),
        )
    };
    let width = f64::from(props.panel_width);
    let mut list = div(Style::default()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .gap(2.0)
        .padding(Edges::axes(Length::px(4.0), Length::ZERO)));
    if filtered.is_empty() {
        list = list.child(
            div(Style::default()
                .padding(Edges::axes(Length::px(20.0), Length::px(12.0)))
                .text_align(TextAlign::Center)
                .font_size(14.0)
                .color(css_color(theme.get("mutedForeground"))))
            .child(props.empty_text.clone()),
        );
    } else {
        for item in &filtered {
            let index = index_of(&props.items, Some(item));
            let state = if index == pressed {
                RowState::Press
            } else if index == selected {
                RowState::Selected
            } else if index == highlighted {
                RowState::Hover
            } else {
                RowState::Idle
            };
            list = list.child(row(item, state, &theme, width - 16.0, radius));
        }
    }
    let query_color = if visible.is_empty() {
        theme.get("mutedForeground")
    } else {
        theme.get("popoverForeground")
    };
    let input = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(10.0)
        .padding(Edges::axes(Length::px(8.0), Length::px(10.0))))
    .child(icon(SEARCH_PATH, 18.0, theme.get("mutedForeground")))
    .child(
        span(
            Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .font_size(15.0)
                .letter_spacing(Length::em(-0.01))
                .color(css_color(query_color)),
        )
        .child(if visible.is_empty() {
            props.placeholder.clone()
        } else {
            visible
        })
        .child(span(
            Style::default()
                .display(Display::InlineBlock)
                .width(Length::px(1.5))
                .height(Length::px(18.0))
                .margin_left(Length::px(1.0))
                .background_color(css_color(theme.get("foreground"))),
        )),
    );
    let panel = div(Style::default()
        .position(Position::Relative)
        .width(Length::px(width))
        .transform_origin((Length::percent(50.0), Length::ZERO, Length::ZERO))
        .transform(vec![
            Transform::Translate(Length::ZERO, Length::px(look.rise), Length::ZERO),
            Transform::Scale(Num(look.scale), Num(look.scale), Num(1.0)),
        ])
        .opacity(look.opacity)
        .background_color(css_color(theme.get("popover")))
        .border(solid(theme.get("border")))
        .border_radius_all(radius + 6.0)
        .padding(Edges::all(Length::px(8.0)))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(24.0),
            blur: Length::px(48.0),
            spread: Length::px(-12.0),
            color: Color::srgb_alpha(0.0, 0.0, 0.0, 0.25),
        }]))
    .child(input)
    .child(div(Style::default()
        .height(Length::px(1.0))
        .background_color(css_color(theme.get("border")))
        .margin(Edges::axes(Length::px(4.0), Length::ZERO))))
    .child(list);
    Ok(div(Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::FlexStart)
        .justify_content(Align::Center)
        .padding_top(Length::percent(props.top))
        .font_family(font_or(&props.font_family, FontRole::Main)))
    .child(div(absolute_fill().background_color(Color::srgb_alpha(
        0.0,
        0.0,
        0.0,
        props.backdrop_opacity * look.backdrop,
    ))))
    .child(panel)
    .into())
}
