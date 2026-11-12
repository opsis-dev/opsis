use indexmap::IndexMap;
use opsis_macros::settings;
use opsis_scene::color::{Color, ColorSpace, Interpolation as ColorInterpolation};
use opsis_scene::css::with_background;
use opsis_scene::node::{Node, div, span};
use opsis_scene::paint::{Filter, Transform};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, BoxSizing, Display, LineHeight, Overflow, Position, Style,
    TextAlign, WhiteSpace,
};
use opsis_scene::values::{Corners, Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{exact_eq, float};

use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::morph_diff::{MorphKind, Version, morph_of, state_at, timeline, version_of};
use crate::elements::theme::{Theme, ThemeMode, code_morph_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct CodeMorphStep {
    #[setting("Code", widget = "code")]
    pub code: String,
    #[setting(
        "Show at",
        range(min = 0, max = 120),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub at: f64,
}

#[settings(export)]
pub struct CodeMorph {
    #[setting("Filename", default = "invoices.tsx")]
    pub filename: String,
    #[setting(
        "Theme",
        default = Theme::of(ThemeMode::Dark),
        transform = code_morph_themes,
        widget = "theme"
    )]
    pub theme: Theme,
    #[setting("Accent", default = "#0ea5e9", check = colour)]
    pub accent_color: String,
    #[setting(
        "Morph time",
        default = 1.2,
        range(min = 0.4, max = 3),
        step = 0.01,
        unit = "s",
        frames_at = 30
    )]
    pub morph_duration: f64,
    #[setting("Font size", default = 20, range(min = 12, max = 32), step = 1)]
    pub font_size: u32,
    #[setting("Highlight changes", default = true)]
    pub highlight: bool,
    #[setting("Line numbers", default = true)]
    pub line_numbers: bool,
    #[setting("Window dots", default = true, advanced)]
    pub window_dots: bool,
    #[setting(
        "Versions",
        default = vec![
            CodeMorphStep {
                code: "import { loadInvoices } from \"./api\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  const invoices = await loadInvoices();\n  list.textContent = \"\";\n  for (const invoice of invoices) {\n    const row = document.createElement(\"li\");\n    row.textContent = invoice.title;\n    list.append(row);\n  }\n}".into(),
                ..Default::default()
            },
            CodeMorphStep {
                at: 1.6,
                code: "import { loadInvoices } from \"./api\";\nimport { renderRows } from \"./rows\";\n\nexport async function showInvoices(list: HTMLElement) {\n  list.textContent = \"Loading…\";\n  renderRows(list, await loadInvoices());\n}".into(),
            },
        ],
        help = "Each version of the code and when it starts to show."
    )]
    pub steps: Vec<CodeMorphStep>,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting("Line height", default = 1.55, range(min = 1.2, max = 2.4), advanced)]
    pub line_height: f64,
    #[setting(
        "Corner radius",
        default = 14,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Largest size", default = 0.9, range(min = 0.3, max = 1), advanced)]
    pub max_size: f64,
    #[setting(
        "Highlight for",
        default = 0.8,
        range(min = 0, max = 5),
        step = 0.0333,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub highlight_hold: f64,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
    #[setting(
        "Background",
        widget = "color",
        zero_label = "None",
        check = colour_or_empty,
        advanced
    )]
    pub background_color: String,
}

pub fn definition() -> ElementMeta {
    meta::<CodeMorph>(
            element(
                ElementKind::Component,
                "code-morph",
                "Code morph",
                "A flat code editor that morphs between versions of a file, gliding each surviving token to its new line and column while removed code dissolves and new code fades in.",
                ElementCategory::Ui,
                138,
            )
                .authored_clock()
                .look(NodeLook::of("lucide:code-xml").morphs("lucide:braces")),
        )
}

const REFERENCE: (f64, f64) = (1280.0, 720.0);
const MAX_WIDTH: f64 = 1152.0;
const MAX_HEIGHT: f64 = 640.0;
const MAX_SHARE: f64 = 0.9;
const ADVANCE: f64 = 0.6;
const HEADER: f64 = 42.0;
const PAD_TOP: f64 = 20.0;
const PAD_BOTTOM: f64 = 22.0;
const PAD_LEFT: f64 = 22.0;
const PAD_RIGHT: f64 = 28.0;
const GUTTER_GAP: f64 = 2.0;
const DOT: f64 = 11.0;
const DOT_GAP: f64 = 7.0;
const DOT_LEFT: f64 = 16.0;
const TAB_OFFSET: f64 = 14.0;
const TAB_LEFT: f64 = 10.0;
const TAB_TOP: f64 = 8.0;
const TAB_PAD: f64 = 14.0;
const TAB_RADIUS: f64 = 8.0;
const TAB_FONT: f64 = 13.0;
const BAR: f64 = 2.0;
const BLUR: f64 = 3.5;
const MIN_FONT: f64 = 10.0;
const MAX_FONT: f64 = 40.0;

struct Palette<'a> {
    base: [(&'static str, &'static str); 13],
    overrides: &'a IndexMap<String, String>,
    share: f64,
    band: f64,
}

impl Palette<'_> {
    fn get(&self, key: &str) -> &str {
        self.overrides.get(key).map_or_else(
            || {
                self.base
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map_or("", |(_, v)| *v)
            },
            String::as_str,
        )
    }
}

const fn palette_for(theme: &Theme) -> Palette<'_> {
    match theme.mode {
        ThemeMode::Dark => Palette {
            base: [
                ("panel", "#111113"),
                ("chrome", "#0c0c0e"),
                ("border", "#27272a"),
                ("dot", "#3f3f46"),
                ("label", "#e4e4e7"),
                ("gutter", "#52525b"),
                ("keyword", "#a1a1aa"),
                ("function", "#fafafa"),
                ("type", "#fafafa"),
                ("text", "#d4d4d8"),
                ("punct", "#71717a"),
                ("comment", "#5b5b63"),
                ("mix", "#ffffff"),
            ],
            overrides: &theme.colors,
            share: 80.0,
            band: 0.16,
        },
        ThemeMode::Light => Palette {
            base: [
                ("panel", "#ffffff"),
                ("chrome", "#f7f7f8"),
                ("border", "#e4e4e7"),
                ("dot", "#d4d4d8"),
                ("label", "#27272a"),
                ("gutter", "#b4b4bb"),
                ("keyword", "#71717a"),
                ("function", "#09090b"),
                ("type", "#09090b"),
                ("text", "#3f3f46"),
                ("punct", "#909098"),
                ("comment", "#a1a1aa"),
                ("mix", "#000000"),
            ],
            overrides: &theme.colors,
            share: 70.0,
            band: 0.12,
        },
    }
}

const fn kind_key(kind: MorphKind) -> &'static str {
    match kind {
        MorphKind::Keyword => "keyword",
        MorphKind::String => "string",
        MorphKind::Number => "number",
        MorphKind::Comment => "comment",
        MorphKind::Function => "function",
        MorphKind::Type => "type",
        MorphKind::Text => "text",
        MorphKind::Punct => "punct",
    }
}

fn ordered_steps(steps: &[CodeMorphStep]) -> Vec<(f64, &str)> {
    let mut ordered: Vec<(usize, f64, &str)> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.at.max(0.0), s.code.as_str()))
        .collect();
    ordered.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    if ordered.is_empty() {
        return vec![(0.0, "")];
    }
    ordered
        .into_iter()
        .map(|(_, at, code)| (at, code))
        .collect()
}

const fn ch(value: f64) -> Length {
    Length::Ch(Num(value))
}

#[expect(
    clippy::too_many_lines,
    reason = "one panel laid out in the film's order, which keeps the port checkable against it"
)]
pub fn draw(props: &CodeMorph, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let steps = ordered_steps(&props.steps);
    let versions: Vec<Version> = steps.iter().map(|(_, code)| version_of(code)).collect();
    let morphs: Vec<_> = versions
        .windows(2)
        .map(|w| morph_of(&w[0], &w[1]))
        .collect();
    let size = f64::from(props.font_size).clamp(MIN_FONT, MAX_FONT);
    let rows = versions.iter().map(|v| v.lines).max().unwrap_or(0);
    let columns = versions.iter().map(|v| v.columns).max().unwrap_or(0);
    let digits = if props.line_numbers {
        2.max(rows.to_string().len())
    } else {
        0
    };
    let gutter = if props.line_numbers {
        float(digits) + GUTTER_GAP
    } else {
        0.0
    };
    let filename = props.filename.trim();
    let header = if props.window_dots || !filename.is_empty() {
        HEADER
    } else {
        0.0
    };
    let leading = size * props.line_height;
    let panel_width = (gutter + float(columns)).mul_add(ADVANCE * size, PAD_LEFT + PAD_RIGHT);
    let panel_height = float(rows).mul_add(leading, header + PAD_TOP + PAD_BOTTOM);
    let share = props.max_size / MAX_SHARE;
    let fit = (MAX_WIDTH * share / panel_width)
        .min(MAX_HEIGHT * share / panel_height)
        .min(1.0);
    let unit = (width / REFERENCE.0).min(height / REFERENCE.1) * fit;
    let line_height = leading * unit;
    let header_h = header * unit;
    let at: Vec<f64> = steps.iter().map(|(at, _)| *at).collect();
    let plan = timeline(
        &at,
        props.morph_duration,
        props.highlight,
        props.highlight_hold,
    );
    let time = cx.authored_frame().max(0.0) * props.speed.max(0.0);
    let state = state_at(time, &plan, &versions, &morphs, BLUR * unit);
    let palette = palette_for(&props.theme);
    let hairline = unit.max(1.0);
    let edge = BorderSide {
        width: Length::px(hairline),
        style: BorderStyle::Solid,
        color: Some(css_color(palette.get("border"))),
    };
    let top = PAD_TOP.mul_add(unit, header_h);
    let accent_text = Color::mix(
        ColorInterpolation {
            space: ColorSpace::Oklab,
            ..ColorInterpolation::DEFAULT
        },
        (css_color(&props.accent_color), Some(palette.share)),
        (css_color(palette.get("mix")), None),
    );
    let color_of = |kind: MorphKind| match kind {
        MorphKind::String | MorphKind::Number => accent_text.clone(),
        other => css_color(palette.get(kind_key(other))),
    };
    let mut panel = div(Style::default()
        .position(Position::Relative)
        .flex_shrink(0.0)
        .box_sizing(BoxSizing::ContentBox)
        .width(Length::sum([
            ch(gutter + float(columns)),
            Length::px((PAD_LEFT + PAD_RIGHT) * unit),
        ]))
        .height(Length::px(
            PAD_BOTTOM.mul_add(unit, float(rows).mul_add(line_height, top)),
        ))
        .background_color(css_color(palette.get("panel")))
        .color(css_color(palette.get("text")))
        .border(Edges::all(edge.clone()))
        .border_radius_all(f64::from(props.corner_radius) * unit)
        .overflow(Overflow::Hidden)
        .font_family(font_or(&props.font, FontRole::Code))
        .font_size(size * unit)
        .line_height(LineHeight::Length(Length::px(line_height))));
    if header_h > 0.0 {
        let mut bar = div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .top(Length::ZERO)
            .right(Length::ZERO)
            .height(Length::px(header_h))
            .background_color(css_color(palette.get("chrome"))))
        .child(div(Style::default()
            .position(Position::Absolute)
            .left(Length::ZERO)
            .right(Length::ZERO)
            .bottom(Length::ZERO)
            .height(Length::px(hairline))
            .background_color(css_color(palette.get("border")))));
        if props.window_dots {
            for dot in 0..3 {
                bar = bar.child(div(Style::default()
                    .position(Position::Absolute)
                    .left(Length::px(
                        float(dot).mul_add(DOT + DOT_GAP, DOT_LEFT) * unit,
                    ))
                    .top(Length::px((HEADER - DOT) / 2.0 * unit))
                    .width(Length::px(DOT * unit))
                    .height(Length::px(DOT * unit))
                    .border_radius_all(Length::percent(50.0))
                    .background_color(css_color(palette.get("dot")))));
            }
        }
        if !filename.is_empty() {
            let after_dots = DOT_GAP.mul_add(2.0, DOT.mul_add(3.0, DOT_LEFT)) + TAB_OFFSET;
            let tab_left = if props.window_dots {
                after_dots
            } else {
                TAB_LEFT
            } * unit;
            let radius = (Length::px(TAB_RADIUS * unit), Length::px(TAB_RADIUS * unit));
            bar = bar.child(
                div(Style::default()
                    .position(Position::Absolute)
                    .left(Length::px(tab_left))
                    .bottom(Length::ZERO)
                    .height(Length::px((HEADER - TAB_TOP) * unit))
                    .box_sizing(BoxSizing::BorderBox)
                    .display(Display::Flex)
                    .align_items(Align::Center)
                    .padding(Edges::axes(Length::ZERO, Length::px(TAB_PAD * unit)))
                    .background_color(css_color(palette.get("panel")))
                    .border(Edges {
                        top: edge.clone(),
                        right: edge.clone(),
                        bottom: BorderSide::default(),
                        left: edge,
                    })
                    .border_radius(Corners {
                        top_left: radius.clone(),
                        top_right: radius,
                        bottom_right: (Length::ZERO, Length::ZERO),
                        bottom_left: (Length::ZERO, Length::ZERO),
                    })
                    .color(css_color(palette.get("label")))
                    .font_size(TAB_FONT * unit)
                    .line_height(LineHeight::Number(Num(1.0)))
                    .white_space(WhiteSpace::Pre))
                .child(filename.to_owned()),
            );
        }
        panel = panel.child(bar);
    }
    for band in &state.highlights {
        panel = panel.child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .right(Length::ZERO)
                .top(Length::px(float(band.row).mul_add(line_height, top)))
                .height(Length::px(line_height)))
            .child(div(Style::default()
                .position(Position::Absolute)
                .inset(Length::ZERO)
                .background_color(css_color(&props.accent_color))
                .opacity(band.opacity * palette.band)))
            .child(div(Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::ZERO)
                .bottom(Length::ZERO)
                .width(Length::px(BAR * unit))
                .background_color(css_color(&props.accent_color))
                .opacity(band.opacity))),
        );
    }
    let mut code = div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(PAD_LEFT * unit))
        .top(Length::px(top)));
    if props.line_numbers {
        for row in &state.line_numbers {
            code = code.child(
                div(Style::default()
                    .position(Position::Absolute)
                    .left(Length::ZERO)
                    .top(Length::px(float(row.row) * line_height))
                    .width(ch(float(digits)))
                    .text_align(TextAlign::Right)
                    .white_space(WhiteSpace::Pre)
                    .color(css_color(palette.get("gutter")))
                    .opacity(row.opacity))
                .child((row.row + 1).to_string()),
            );
        }
    }
    let spans: Vec<Node> = state
        .tokens
        .iter()
        .map(|token| {
            let mut moves = vec![Transform::Translate(
                ch(token.col),
                Length::px(token.line * line_height),
                Length::ZERO,
            )];
            if !exact_eq(token.scale, 1.0) {
                moves.push(Transform::Scale(
                    Num(token.scale),
                    Num(token.scale),
                    Num(1.0),
                ));
            }
            let mut style = Style::default()
                .position(Position::Absolute)
                .left(Length::ZERO)
                .top(Length::ZERO)
                .display(Display::Block)
                .white_space(WhiteSpace::Pre)
                .color(color_of(token.kind))
                .opacity(token.opacity)
                .transform(moves);
            if token.blur > 0.0 {
                style = style.filter(vec![Filter::Blur(Length::px(token.blur))]);
            }
            span(style).child(token.text.clone()).into()
        })
        .collect();
    code = code.child(
        div(Style::default()
            .position(Position::Absolute)
            .left(ch(gutter))
            .top(Length::ZERO))
        .children(spans),
    );
    let mut holder = Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center);
    if !props.background_color.is_empty() {
        holder = with_background(holder, &props.background_color);
    }
    Ok(div(holder).child(panel.child(code)).into())
}
