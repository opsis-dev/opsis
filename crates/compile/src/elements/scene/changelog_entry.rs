use opsis_macros::{choice, settings};
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, span};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, NumericVariant, Style, TextAlign,
    TextTransform,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::{clamped, sprung};
use crate::elements::code_theme::code_palette;
use crate::elements::code_window::{CodeWindow, code_window};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[choice(export)]
pub enum ChangeKind {
    Added,
    Fixed,
    Changed,
    Removed,
}

#[settings(export, eq)]
pub struct ChangelogEntryChange {
    #[setting("Kind", default = ChangeKind::Removed)]
    pub kind: ChangeKind,
    #[setting("Text", default = "Legacy waveform atom")]
    pub text: String,
}

#[settings(export)]
pub struct ChangelogEntry {
    #[setting(
        "Changes",
        default = vec![
            ChangelogEntryChange {
                kind: ChangeKind::Added,
                text: "Caption presets for nine languages".into(),
            },
            ChangelogEntryChange {
                kind: ChangeKind::Added,
                text: "Render API webhooks on job completion".into(),
            },
            ChangelogEntryChange {
                kind: ChangeKind::Fixed,
                text: "Retry backoff no longer stalls the queue".into(),
            },
            ChangelogEntryChange {
                kind: ChangeKind::Changed,
                text: "Theme tokens moved to a single source".into(),
            },
            ChangelogEntryChange::default(),
        ],
        help = "A kind (added, fixed, changed or removed) and the sentence beside it."
    )]
    pub changes: Vec<ChangelogEntryChange>,
    #[setting(
        "Version",
        default = "v2.4.0",
        help = "Version string, set at display size."
    )]
    pub version: String,
    #[setting("Date", default = "16 August", help = "Date beside the version.")]
    pub date: String,
    #[setting(
        "Summary",
        default = "Captions, webhooks, and a queue that stops sulking.",
        help = "Headline under the version. Omit for releases without one."
    )]
    pub summary: String,
    #[setting(
        "Start at (seconds)",
        default = 0.66,
        step = 0.01,
        help = "Second the first change row arrives.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Stagger (seconds)",
        default = 0.28,
        step = 0.01,
        help = "Seconds between rows. Each sentence trails its own tag by 0.08s.",
        advanced
    )]
    pub stagger_seconds: f64,
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
    #[setting("Added colour", widget = "color", zero_label = "Default", advanced)]
    pub kind_added: String,
    #[setting("Fixed colour", widget = "color", zero_label = "Default")]
    pub kind_fixed: String,
    #[setting("Changed colour", widget = "color", zero_label = "Default")]
    pub kind_changed: String,
    #[setting("Removed colour", widget = "color", zero_label = "Default")]
    pub kind_removed: String,
    #[setting("Added tag", default = "Added", advanced)]
    pub added_label: String,
    #[setting("Fixed tag", default = "Fixed", advanced)]
    pub fixed_label: String,
    #[setting("Changed tag", default = "Changed", advanced)]
    pub changed_label: String,
    #[setting("Removed tag", default = "Removed", advanced)]
    pub removed_label: String,
    #[setting(
        "Card width",
        default = 760,
        range(min = 320, max = 1600),
        step = 1,
        unit = "px",
        advanced
    )]
    pub card_width: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<ChangelogEntry>(
        element(
            ElementKind::Component,
            "changelog-entry",
            "Changelog",
            "A release note: version and date, then each change appears with its own tag.",
            ElementCategory::Scene,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:rotate-ccw-clock")),
    )
}

const CARD_FOR: f64 = 0.46;
const VERSION_AT: f64 = 0.12;
const VERSION_FOR: f64 = 0.44;
const RULE_AT: f64 = 0.34;
const RULE_FOR: f64 = 0.5;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const CARD_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 130.0,
    mass: 0.85,
    overshoot_clamping: false,
};

fn kind_style(props: &ChangelogEntry, kind: ChangeKind) -> (&str, &str) {
    let (chosen, fallback, label) = match kind {
        ChangeKind::Added => (&props.kind_added, "#9BD4A0", &props.added_label),
        ChangeKind::Fixed => (&props.kind_fixed, "#7DD3E8", &props.fixed_label),
        ChangeKind::Changed => (&props.kind_changed, "#E8B86D", &props.changed_label),
        ChangeKind::Removed => (&props.kind_removed, "#E89B9B", &props.removed_label),
    };
    (if chosen.is_empty() { fallback } else { chosen }, label)
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film computes this with a separate multiply and add"
)]
pub fn draw(props: &ChangelogEntry, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height, frame, fps) = (cx.at.stage.width, cx.at.stage.height, cx.frame(), cx.fps());
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let at = |seconds: f64| seconds * fps / props.speed;
    let ease = |from: f64, to: f64, easing: Easing| -> Result<f64, DrawFailure> {
        Ok(interpolate(
            frame,
            &[at(from), at(to)],
            &[0.0, 1.0],
            &Interpolation {
                easing: Some(easing),
                ..Interpolation::clamped()
            },
        )?)
    };
    let u = if height > width {
        (width / 496.0).min(height / 896.0)
    } else {
        (width / 1024.0).min(height / 576.0)
    };
    let card_in = sprung(frame - at(0.0), fps, CARD_SPRING)?;
    let open = ease(0.0, CARD_FOR, EDITORIAL)?;
    let version_in = ease(VERSION_AT, VERSION_AT + VERSION_FOR, easing_enter())?;
    let rule_in = ease(RULE_AT, RULE_AT + RULE_FOR, EDITORIAL)?;
    let card_w = (width - side * 2.0).min(f64::from(props.card_width) * u);
    let mut head = div(Style::default()
        .display(Display::Flex)
        .align_items(Align::Baseline)
        .gap(12.0 * u)
        .opacity(version_in)
        .translate(Length::ZERO, Length::px((1.0 - version_in) * 12.0 * u)))
    .child(
        span(
            Style::default()
                .color(css_color(palette.get("fg")))
                .font_size(38.0 * u)
                .font_weight(800.0)
                .letter_spacing(Length::em(-0.03))
                .font_variant_numeric(NumericVariant {
                    tabular: true,
                    ..NumericVariant::default()
                }),
        )
        .child(props.version.clone()),
    );
    if !props.date.is_empty() {
        head = head.child(
            span(
                Style::default()
                    .color(css_color(palette.get("dim")))
                    .font_size(15.0 * u)
                    .font_weight(500.0),
            )
            .child(props.date.clone()),
        );
    }
    let mut card = div(code_window(
        &palette,
        CodeWindow {
            width: card_w,
            radius: 20.0 * u,
            open,
            rise: (1.0 - card_in) * 22.0 * u,
            clip_round: 20.0 * u,
            shadow: (24.0 * u, 60.0 * u),
        },
    )
    .padding(Edges::all(Length::px(30.0 * u))))
    .child(head);
    if !props.summary.is_empty() {
        card = card.child(
            div(Style::default()
                .margin_top(Length::px(6.0 * u))
                .color(css_color(palette.get("dim")))
                .font_size(16.5 * u)
                .font_weight(500.0)
                .opacity(clamped(version_in, [0.4, 1.0], [0.0, 1.0])?))
            .child(props.summary.clone()),
        );
    }
    card = card.child(div(Style::default()
        .margin_top(Length::px(18.0 * u))
        .margin_bottom(Length::px(18.0 * u))
        .height(Length::px(1.0))
        .width(Length::percent(rule_in * 100.0))
        .background_color(css_color(palette.get("border")))));
    let mut rows = Vec::new();
    for (index, change) in props.changes.iter().enumerate() {
        let row_at = float(index).mul_add(props.stagger_seconds, props.start_at_seconds);
        let row_in = ease(row_at, row_at + 0.4, easing_enter())?;
        let text_in = ease(row_at + 0.08, row_at + 0.46, easing_enter())?;
        let (color, label) = kind_style(props, change.kind);
        let grow = row_in.mul_add(0.1, 0.9);
        rows.push(
            div(Style::default()
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(14.0 * u)
                .opacity(row_in))
            .child(
                span(
                    Style::default()
                        .width(Length::px(74.0 * u))
                        .flex_shrink(0.0)
                        .text_align(TextAlign::Center)
                        .padding(Edges::axes(Length::px(4.0 * u), Length::ZERO))
                        .border_radius_all(7.0 * u)
                        .background_color(tinted(color, 12.0))
                        .border(Edges::all(BorderSide {
                            width: Length::px(1.0),
                            style: BorderStyle::Solid,
                            color: Some(tinted(color, 33.0)),
                        }))
                        .color(css_color(color))
                        .font_size(14.5 * u)
                        .font_weight(700.0)
                        .letter_spacing(Length::em(0.08))
                        .text_transform(TextTransform::Uppercase)
                        .scale(grow, grow),
                )
                .child(label.to_owned()),
            )
            .child(
                span(
                    Style::default()
                        .color(css_color(palette.get("fg")))
                        .font_size(17.0 * u)
                        .font_weight(500.0)
                        .opacity(text_in)
                        .translate(Length::px((1.0 - text_in) * 10.0 * u), Length::ZERO),
                )
                .child(change.text.clone()),
            )
            .into(),
        );
    }
    let card = card.child(
        div(Style::default()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap(12.0 * u))
        .children(rows),
    );
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
    .child(card)
    .into())
}
