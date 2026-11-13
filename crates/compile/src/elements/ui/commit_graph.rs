use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::random::js_number_text;
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, Overflow, Position, Style, TextOverflow, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::{NodeLook, NodeMotion};
use opsis_types::numbers::{exact_eq, float, truncate};

use crate::elements::anim::{clamped, sprung};
use crate::elements::code_theme::{Palette, code_palette};
use crate::elements::code_window::{CodeWindow, bottom_line, code_window};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::path_utils::path_length;
use crate::elements::shared::Colour;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct CommitGraphCommit {
    #[setting("Message", default = "Merge retry backoff")]
    pub message: String,
    #[setting("Hash", default = "b8c4e05")]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Lane", "type" = "integer")]
    pub lane: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Parent", "type" = "integer")]
    pub parent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Merge from", "type" = "integer")]
    pub merge_from: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Ref", "type" = "string")]
    pub r#ref: Option<String>,
}

#[settings(export)]
pub struct CommitGraph {
    #[setting(
        "Commits",
        default = vec![
            CommitGraphCommit {
                message: "Seed the registry manifest".into(),
                hash: "9c41f0a".into(),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Add render queue worker".into(),
                hash: "3ab77e2".into(),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Branch: retry backoff".into(),
                hash: "d0e91c4".into(),
                lane: Some(1),
                parent: Some(1),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Cap retries at five".into(),
                hash: "51b2fa8".into(),
                lane: Some(1),
                ..Default::default()
            },
            CommitGraphCommit {
                message: "Ship caption presets".into(),
                hash: "7f30dd1".into(),
                lane: Some(0),
                parent: Some(1),
                ..Default::default()
            },
            CommitGraphCommit {
                lane: Some(0),
                parent: Some(4),
                merge_from: Some(3),
                r#ref: Some("main".into()),
                ..Default::default()
            },
        ]
    )]
    pub commits: Vec<CommitGraphCommit>,
    #[setting(
        "Window title",
        default = "git log --graph --oneline",
        help = "Text in the title bar. Omit to drop the chrome header."
    )]
    pub window_title: String,
    #[setting(
        "Start at (seconds)",
        default = 0.34,
        step = 0.01,
        help = "Second the first commit lands.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Step (seconds)",
        default = 0.36,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "Seconds between commits. Each edge draws over this same gap.",
        advanced
    )]
    pub step_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Trunk colour, and the branch ref chip.",
        check = colour
    )]
    pub accent_color: String,
    #[setting(
        "Branch colors",
        default = vec!["#7DD3E8".into(), "#C99BE8".into(), "#9BD4A0".into(), "#E8B86D".into()],
        length(min = 1),
        help = "Colours for branch lanes 1 and up.",
        advanced
    )]
    pub lane_colors: Vec<Colour>,
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
    #[setting("Hash font", widget = "font", advanced)]
    pub code_font: String,
    #[setting("Font", widget = "font", advanced)]
    pub font: String,
    #[setting(
        "Window width",
        default = 860,
        range(min = 400, max = 1400),
        step = 1,
        unit = "px",
        advanced
    )]
    pub window_width: u32,
    #[setting(
        "Row spacing",
        default = 48,
        range(min = 28, max = 96),
        step = 1,
        unit = "px",
        advanced
    )]
    pub row_spacing: u32,
    #[setting(
        "Lane spacing",
        default = 30,
        range(min = 16, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub lane_spacing: u32,
    #[setting(
        "Dot size",
        default = 6.5,
        range(min = 3, max = 16),
        unit = "px",
        advanced
    )]
    pub dot_size: f64,
    #[setting(
        "Line width",
        default = 2.2,
        range(min = 1, max = 8),
        unit = "px",
        advanced
    )]
    pub line_width: f64,
    #[setting(
        "Text size",
        default = 15.5,
        range(min = 10, max = 28),
        unit = "px",
        advanced
    )]
    pub text_size: f64,
    #[setting(
        "Corner radius",
        default = 18,
        range(min = 0, max = 40),
        step = 1,
        unit = "px",
        advanced
    )]
    pub corner_radius: u32,
    #[setting("Window dots", default = true, advanced)]
    pub window_dots: bool,
}

pub fn definition() -> ElementMeta {
    meta::<CommitGraph>(
        element(
            ElementKind::Component,
            "commit-graph",
            "Commit graph",
            "A version history graph that draws its branches and merges commit by commit.",
            ElementCategory::Ui,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:git-graph").moves(NodeMotion::Draw)),
    )
}

const WINDOW_FOR: f64 = 0.46;
const CHROME_AT: f64 = 0.14;
const CHROME_FOR: f64 = 0.38;
const FIRST_LEAD: f64 = 0.24;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const WINDOW_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 130.0,
    mass: 0.85,
    overshoot_clamping: false,
};
const SMALL_TEXT: f64 = 14.5 / 15.5;

#[derive(Debug, Clone)]
struct Commit<'a> {
    source: &'a CommitGraphCommit,
    lane: f64,
    parent: Option<usize>,
    merge_from: Option<usize>,
}

fn earlier(reference: Option<u32>, index: usize) -> Option<usize> {
    reference
        .and_then(|r| usize::try_from(r).ok())
        .filter(|r| *r < index)
}

struct Graph<'a> {
    props: &'a CommitGraph,
    commits: Vec<Commit<'a>>,
    u: f64,
    row_h: f64,
    lane_w: f64,
    gutter: f64,
    frame: f64,
    fps: f64,
}

impl Graph<'_> {
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

    fn lane_color(&self, lane: f64) -> &str {
        if lane <= 0.0 {
            return &self.props.accent_color;
        }
        let colors = &self.props.lane_colors;
        let index: usize = truncate(lane - 1.0);
        colors
            .get(index % colors.len().max(1))
            .map_or("", |c| c.0.as_str())
    }

    fn row_y(&self, index: usize) -> f64 {
        float(index).mul_add(self.row_h, self.row_h / 2.0)
    }

    const fn lane_x(&self, lane: f64) -> f64 {
        lane.mul_add(self.lane_w, self.gutter)
    }

    fn arrival(&self, index: usize) -> Result<f64, DrawFailure> {
        let p = self.props;
        let i = float(index);
        self.ease(
            i.mul_add(p.step_seconds, p.start_at_seconds),
            (i + 1.0).mul_add(p.step_seconds, p.start_at_seconds),
            EDITORIAL,
        )
    }

    fn progress(&self, index: usize) -> Result<f64, DrawFailure> {
        if index == 0 {
            let start = self.props.start_at_seconds;
            return self.ease(start - FIRST_LEAD, start, easing_enter());
        }
        self.arrival(index)
    }

    fn edge_path(&self, from: usize, to: usize) -> String {
        let n = js_number_text;
        let (x1, y1) = (self.lane_x(self.commits[from].lane), self.row_y(from));
        let (x2, y2) = (self.lane_x(self.commits[to].lane), self.row_y(to));
        if exact_eq(x1, x2) {
            return format!("M {} {} L {} {}", n(x1), n(y1), n(x2), n(y2));
        }
        let bend = (y2 - y1) * 0.55;
        format!(
            "M {} {} C {} {}, {} {}, {} {}",
            n(x1),
            n(y1),
            n(x1),
            n(y1 + bend),
            n(x2),
            n(y2 - bend),
            n(x2),
            n(y2)
        )
    }
}

fn graph_svg(
    g: &Graph<'_>,
    palette: &Palette,
    width: f64,
    height: f64,
) -> Result<SvgElement, DrawFailure> {
    let p = g.props;
    let mut root = SvgElement::new("svg")
        .attr("width", width)
        .attr("height", height)
        .attr(
            "viewBox",
            format!("0 0 {} {}", js_number_text(width), js_number_text(height)),
        );
    for (index, commit) in g.commits.iter().enumerate().skip(1) {
        let progress = g.arrival(index)?;
        if progress <= 0.0 {
            continue;
        }
        let parent = commit.parent.unwrap_or(index - 1);
        let mut edges = vec![(parent, g.lane_color(commit.lane))];
        if let Some(merge) = commit.merge_from {
            edges.push((merge, g.lane_color(g.commits[merge].lane)));
        }
        for (from, stroke) in edges {
            let d = g.edge_path(from, index);
            let length = path_length(&d);
            root = root.child(
                SvgElement::new("path")
                    .attr("d", d)
                    .attr("fill", "none")
                    .attr("stroke", stroke)
                    .attr("stroke-width", p.line_width * g.u)
                    .attr("stroke-linecap", "round")
                    .attr("opacity", 0.85)
                    .attr("stroke-dasharray", length)
                    .attr("stroke-dashoffset", (1.0 - progress) * length),
            );
        }
    }
    for (index, commit) in g.commits.iter().enumerate() {
        let pop = clamped(g.progress(index)?, [0.82, 1.0], [0.0, 1.0])?;
        if pop <= 0.0 {
            continue;
        }
        root = root.child(
            SvgElement::new("circle")
                .attr("cx", g.lane_x(commit.lane))
                .attr("cy", g.row_y(index))
                .attr("r", p.dot_size * g.u * pop.mul_add(0.4, 0.6))
                .attr("fill", palette.get("window"))
                .attr("stroke", g.lane_color(commit.lane))
                .attr("stroke-width", p.line_width * (2.6 / 2.2) * g.u)
                .attr("opacity", pop),
        );
    }
    Ok(root)
}

fn row(
    g: &Graph<'_>,
    palette: &Palette,
    index: usize,
    graph_w: f64,
    mono: &[String],
) -> Result<Node, DrawFailure> {
    let (p, u) = (g.props, g.u);
    let commit = &g.commits[index];
    let text_in = clamped(g.progress(index)?, [0.86, 1.0], [0.0, 1.0])?;
    let mut line = div(Style::default()
        .position(Position::Absolute)
        .left(Length::px(14.0f64.mul_add(u, 18.0f64.mul_add(u, graph_w))))
        .right(Length::px(20.0 * u))
        .top(Length::px(
            12.0f64.mul_add(-u, 10.0f64.mul_add(u, g.row_y(index))),
        ))
        .height(Length::px(24.0 * u))
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(12.0 * u)
        .opacity(text_in)
        .translate(Length::px((1.0 - text_in) * 14.0 * u), Length::ZERO))
    .child(
        span(
            Style::default()
                .color(css_color(g.lane_color(commit.lane)))
                .font_size(p.text_size * SMALL_TEXT * u)
                .font_weight(600.0)
                .font_family(mono.to_vec())
                .flex_shrink(0.0),
        )
        .child(commit.source.hash.clone()),
    )
    .child(
        span(
            Style::default()
                .color(css_color(palette.get("fg")))
                .font_size(p.text_size * u)
                .font_weight(500.0)
                .white_space(WhiteSpace::Nowrap)
                .overflow(Overflow::Hidden)
                .text_overflow(TextOverflow::Ellipsis),
        )
        .child(commit.source.message.clone()),
    );
    if let Some(name) = commit.source.r#ref.as_deref().filter(|r| !r.is_empty()) {
        line = line.child(
            span(
                Style::default()
                    .flex_shrink(0.0)
                    .padding(Edges::axes(Length::px(2.5 * u), Length::px(8.0 * u)))
                    .border_radius_all(6.0 * u)
                    .background_color(tinted(&p.accent_color, 13.0))
                    .border(Edges::all(BorderSide {
                        width: Length::px(1.0),
                        style: BorderStyle::Solid,
                        color: Some(tinted(&p.accent_color, 40.0)),
                    }))
                    .color(css_color(&p.accent_color))
                    .font_size(p.text_size * SMALL_TEXT * u)
                    .font_weight(600.0),
            )
            .child(name.to_owned()),
        );
    }
    Ok(line.into())
}

fn header(g: &Graph<'_>, palette: &Palette, chrome_in: f64, mono: &[String]) -> Node {
    let (p, u) = (g.props, g.u);
    let dots: Vec<Node> = (0..if p.window_dots { 3 } else { 0 })
        .map(|_| {
            span(
                Style::default()
                    .width(Length::px(9.0 * u))
                    .height(Length::px(9.0 * u))
                    .border_radius_all(999.0)
                    .background_color(css_color(palette.get("faint"))),
            )
            .into()
        })
        .collect();
    div(Style::default()
        .height(Length::px(40.0 * u))
        .display(Display::Flex)
        .align_items(Align::Center)
        .gap(8.0 * u)
        .padding(Edges::axes(Length::ZERO, Length::px(16.0 * u)))
        .background_color(css_color(palette.get("header")))
        .border(bottom_line(palette.get("border")))
        .opacity(chrome_in))
    .children(dots)
    .child(
        span(
            Style::default()
                .margin_left(Length::px(10.0 * u))
                .color(css_color(palette.get("dim")))
                .font_size(p.text_size * SMALL_TEXT * u)
                .font_weight(500.0)
                .font_family(mono.to_vec()),
        )
        .child(p.window_title.clone()),
    )
    .into()
}

pub fn draw(props: &CommitGraph, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let palette = code_palette(props.theme.mode, &[], &props.theme.colors);
    let (side, edge) = safe_padding(width, height);
    let design_u = if height > width {
        (width / 496.0).min(height / 896.0)
    } else {
        (width / 1024.0).min(height / 576.0)
    };
    let commits: Vec<Commit<'_>> = props
        .commits
        .iter()
        .enumerate()
        .map(|(index, source)| Commit {
            source,
            lane: f64::from(source.lane.unwrap_or(0)),
            parent: earlier(source.parent, index),
            merge_from: earlier(source.merge_from, index),
        })
        .collect();
    let title_room = if props.window_title.is_empty() {
        0.0
    } else {
        40.0
    };
    let u = design_u.min(
        f64::mul_add(edge, -2.0, height) / float(commits.len()).mul_add(48.0, title_room + 20.0),
    );
    let lanes = commits.iter().map(|c| c.lane).fold(0.0, f64::max) + 1.0;
    let g = Graph {
        props,
        u,
        row_h: f64::from(props.row_spacing) * u,
        lane_w: f64::from(props.lane_spacing) * u,
        gutter: 26.0 * u,
        frame: cx.frame(),
        fps: cx.fps(),
        commits,
    };
    let window_in = sprung(g.frame - g.at(0.0), g.fps, WINDOW_SPRING)?;
    let open = g.ease(0.0, WINDOW_FOR, EDITORIAL)?;
    let chrome_in = g.ease(CHROME_AT, CHROME_AT + CHROME_FOR, easing_enter())?;
    let window_w = f64::mul_add(side, -2.0, width).min(f64::from(props.window_width) * u);
    let graph_w = lanes.mul_add(g.lane_w, g.gutter);
    let body_h = g.row_h * float(g.commits.len());
    let font_family = font_or(&props.font, FontRole::Main);
    let mono = font_or(&props.code_font, FontRole::Code);
    let mut body = div(Style::default()
        .position(Position::Relative)
        .height(Length::px(body_h))
        .padding(Edges::axes(Length::px(10.0 * u), Length::ZERO)))
    .child(svg(
        graph_svg(&g, &palette, graph_w, body_h)?,
        Style::default()
            .position(Position::Absolute)
            .left(Length::px(18.0 * u))
            .top(Length::px(10.0 * u)),
    ));
    for index in 0..g.commits.len() {
        body = body.child(row(&g, &palette, index, graph_w, &mono)?);
    }
    let mut window = div(code_window(
        &palette,
        CodeWindow {
            width: window_w,
            radius: f64::from(props.corner_radius) * u,
            open,
            rise: (1.0 - window_in) * 22.0 * u,
            clip_round: 18.0 * u,
            shadow: (24.0 * u, 60.0 * u),
        },
    ));
    if !props.window_title.is_empty() {
        window = window.child(header(&g, &palette, chrome_in, &mono));
    }
    let page = if props.background_color.is_empty() {
        palette.get("page")
    } else {
        &props.background_color
    };
    Ok(div(with_background(
        absolute_fill()
            .font_family(font_family)
            .display(Display::Flex)
            .align_items(Align::Center)
            .justify_content(Align::Center)
            .padding(Edges::axes(Length::px(edge), Length::px(side))),
        page,
    ))
    .child(window.child(body))
    .into())
}
