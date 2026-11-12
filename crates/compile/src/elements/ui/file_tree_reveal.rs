use opsis_macros::settings;
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, SvgElement, div, span, svg};
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, Display, FlexDirection, NumericVariant, Overflow, Position, Style, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::sprung;
use crate::elements::code_theme::{Palette, code_palette, code_tokens};
use crate::elements::code_window::{CodeWindow, bottom_line, code_window};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::oklch_mix::with_alpha;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export)]
pub struct FileTreeReveal {
    #[setting(
        "Files",
        default = vec![
            "src/components/cart.tsx".into(),
            "src/components/checkout.tsx".into(),
            "src/lib/api.ts".into(),
            "src/index.ts".into(),
            "public/logo.svg".into(),
            "README.md".into(),
            "package.json".into(),
        ],
        help = "One path per item, like src/app.ts. End a folder with / to show it empty."
    )]
    pub nodes: Vec<String>,
    #[setting(
        "Title",
        default = "acme-app",
        help = "Title bar over the tree. Omit to drop the header."
    )]
    pub title: String,
    #[setting(
        "Highlighted file",
        default = "src/components/checkout.tsx",
        help = "Slash-joined path of the file that lights up at the end. Omit to select nothing."
    )]
    pub selected_path: String,
    #[setting(
        "Row stagger",
        default = 0.18,
        step = 0.01,
        help = "Seconds between one row appearing and the next.",
        advanced
    )]
    pub row_stagger: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Selected row's bar, tint and label.",
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
    #[setting("File count word", default = "files", advanced)]
    pub count_label: String,
    #[setting(
        "Panel width",
        default = 480,
        range(min = 280, max = 1200),
        step = 1,
        unit = "px",
        advanced
    )]
    pub panel_width: u32,
    #[setting("Font", widget = "font", advanced)]
    pub font_family: String,
}

pub fn definition() -> ElementMeta {
    meta::<FileTreeReveal>(
        element(
            ElementKind::Component,
            "file-tree-reveal",
            "File tree",
            "A folder tree that opens one item at a time and highlights a file.",
            ElementCategory::Ui,
            120,
        )
        .fonts(&["Inter"])
        .look(NodeLook::of("lucide:folder-tree")),
    )
}

const PANEL_FOR: f64 = 0.46;
const HEADER_AT: f64 = 0.16;
const ROWS_AT: f64 = 0.42;
const ROW_FOR: f64 = 0.34;
const BEFORE_SELECT: f64 = 0.24;
const SELECT_FOR: f64 = 0.4;
const HEADER_U: f64 = 42.0;
const ROW_U: f64 = 31.0;
const PAD_U: f64 = 24.0;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);
const PANEL_SPRING: SpringConfig = SpringConfig {
    damping: 18.0,
    stiffness: 130.0,
    mass: 0.85,
    overshoot_clamping: false,
};
const FOLDER: &str = "M3.4 6.6a1.8 1.8 0 0 1 1.8-1.8h3.6l2 2.4h8a1.8 1.8 0 0 1 1.8 1.8v8.4a1.8 1.8 0 0 1-1.8 1.8H5.2a1.8 1.8 0 0 1-1.8-1.8V6.6Z";
const FILE: &str = "M13.6 3.4H7a1.8 1.8 0 0 0-1.8 1.8v13.6A1.8 1.8 0 0 0 7 20.6h10a1.8 1.8 0 0 0 1.8-1.8V8.6l-5.2-5.2Z";
const FILE_FOLD: &str = "M13.4 3.6v5.2h5.2";
const CHEVRON: &str = "M9 6.5L15.5 12 9 17.5";

#[derive(Debug, Clone, Default)]
struct TreeNode {
    name: String,
    children: Option<Vec<Self>>,
}

fn tree_from_paths(paths: &[String]) -> Vec<TreeNode> {
    let mut root = TreeNode::default();
    for path in paths {
        let parts: Vec<&str> = path.split('/').map(str::trim).collect();
        let folder = parts.last().is_some_and(|p| p.is_empty());
        let names: Vec<&str> = parts.into_iter().filter(|p| !p.is_empty()).collect();
        let mut node = &mut root;
        for (i, name) in names.iter().enumerate() {
            let more = i < names.len() - 1;
            let children = node.children.get_or_insert_with(Vec::new);
            let at = if let Some(at) = children.iter().position(|c| c.name == *name) {
                if more {
                    children[at].children.get_or_insert_with(Vec::new);
                }
                at
            } else {
                children.push(TreeNode {
                    name: (*name).to_owned(),
                    children: (more || folder).then(Vec::new),
                });
                children.len() - 1
            };
            node = &mut children[at];
        }
    }
    root.children.unwrap_or_default()
}

#[derive(Debug, Clone)]
struct FlatRow {
    name: String,
    path: String,
    depth: f64,
    is_folder: bool,
    first_child: Option<usize>,
}

fn flatten(nodes: &[TreeNode], depth: f64, parent: &str, out: &mut Vec<FlatRow>) {
    for node in nodes {
        let path = if parent.is_empty() {
            node.name.clone()
        } else {
            format!("{parent}/{}", node.name)
        };
        let at = out.len();
        out.push(FlatRow {
            name: node.name.clone(),
            path: path.clone(),
            depth,
            is_folder: node.children.is_some(),
            first_child: None,
        });
        if let Some(children) = node.children.as_ref().filter(|c| !c.is_empty()) {
            out[at].first_child = Some(out.len());
            flatten(children, depth + 1.0, &path, out);
        }
    }
}

fn extension_color<'p>(name: &str, palette: &'p Palette) -> &'p str {
    let extension = name.rfind('.').map_or(name, |dot| &name[dot + 1..]);
    match extension {
        "tsx" | "jsx" => palette.get("type"),
        "ts" | "js" | "mjs" => palette.get("call"),
        "json" => palette.get("number"),
        "css" => palette.get("prop"),
        "md" | "mdx" => palette.get("string"),
        "svg" | "png" => palette.get("keyword"),
        _ => palette.get("fg"),
    }
}

fn glyph(size: f64, paths: &[(&str, &str, f64, bool)]) -> Node {
    let root = paths.iter().fold(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none"),
        |root, (d, color, width, rounded)| {
            let mut path = SvgElement::new("path")
                .attr("d", *d)
                .attr("stroke", *color)
                .attr("stroke-width", *width);
            if *rounded {
                path = path.attr("stroke-linejoin", "round");
            }
            root.child(path)
        },
    );
    svg(root, Style::default()).into()
}

fn chevron(size: f64, color: &str) -> Node {
    svg(
        SvgElement::new("svg")
            .attr("width", size)
            .attr("height", size)
            .attr("viewBox", "0 0 24 24")
            .attr("fill", "none")
            .child(
                SvgElement::new("path")
                    .attr("d", CHEVRON)
                    .attr("stroke", color)
                    .attr("stroke-width", 2.0)
                    .attr("stroke-linecap", "round")
                    .attr("stroke-linejoin", "round"),
            ),
        Style::default(),
    )
    .into()
}

fn wanted(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_lowercase()
}

pub fn draw(props: &FileTreeReveal, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (width, height) = (cx.at.stage.width, cx.at.stage.height);
    let (frame, fps) = (cx.frame(), cx.fps());
    let palette = code_palette(
        props.theme.mode,
        code_tokens(props.theme.mode),
        &props.theme.colors,
    );
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
    let mut rows = Vec::new();
    flatten(&tree_from_paths(&props.nodes), 0.0, "", &mut rows);
    let count = float(rows.len());
    let fit_u = if height > width {
        (width / 560.0).min(height / 1000.0)
    } else {
        (width / 640.0).min(height / 360.0)
    };
    let u = fit_u.min(f64::mul_add(edge, -2.0, height) / count.mul_add(ROW_U, HEADER_U + PAD_U));
    let reveal_at = |index: f64| index.mul_add(props.row_stagger, ROWS_AT);
    let last_end = if rows.is_empty() {
        ROWS_AT
    } else {
        reveal_at(count - 1.0) + ROW_FOR
    };
    let select_at = last_end + BEFORE_SELECT;
    let select = if props.selected_path.is_empty() {
        0.0
    } else {
        ease(select_at, select_at + SELECT_FOR, EDITORIAL)?
    };
    let target = wanted(&props.selected_path);
    let panel = sprung(frame - at(0.0), fps, PANEL_SPRING)?;
    let open = ease(0.0, PANEL_FOR, EDITORIAL)?;
    let header_in = ease(HEADER_AT, HEADER_AT + 0.4, easing_enter())?;
    let panel_w = f64::mul_add(side, -2.0, width).min(f64::from(props.panel_width) * u);
    let row_h = ROW_U * u;
    let indent = 19.0 * u;
    let accent = props.accent_color.as_str();
    let mut list = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let start = reveal_at(float(index));
        let row_in = ease(start, start + ROW_FOR, easing_enter())?;
        let turn = match row.first_child {
            Some(child) => {
                let child_at = reveal_at(float(child));
                ease(child_at - 0.08, child_at + 0.22, easing_enter())?
            }
            None => 0.0,
        };
        let selected = !target.is_empty() && row.path.to_lowercase() == target;
        let shine = if selected { select } else { 0.0 };
        let name_color = if row.is_folder {
            palette.get("fg")
        } else {
            extension_color(&row.name, &palette)
        };
        let mut row_style = Style::default()
            .position(Position::Relative)
            .height(Length::px(row_h * row_in))
            .display(Display::Flex)
            .align_items(Align::Center)
            .gap(8.0 * u)
            .padding_left(Length::px(row.depth.mul_add(indent, 8.0 * u)))
            .padding_right(Length::px(10.0 * u))
            .border_radius_all(8.0 * u)
            .opacity(row_in)
            .overflow(Overflow::Hidden);
        if selected {
            row_style =
                row_style.background_color(tinted(accent, if shine > 0.4 { 11.0 } else { 0.0 }));
        }
        let mut line = div(row_style);
        if selected {
            let grow = interpolate(shine, &[0.0, 1.0], &[0.3, 1.0], &Interpolation::default())?;
            line = line.child(span(
                Style::default()
                    .position(Position::Absolute)
                    .left(Length::ZERO)
                    .top(Length::px(4.0 * u))
                    .bottom(Length::px(4.0 * u))
                    .width(Length::px(2.5 * u))
                    .border_radius_all(999.0)
                    .background_color(css_color(accent))
                    .opacity(shine)
                    .scale(1.0, grow),
            ));
        }
        line = line.child(
            span(
                Style::default()
                    .display(Display::Flex)
                    .width(Length::px(13.0 * u))
                    .justify_content(Align::Center)
                    .opacity(if row.is_folder { 1.0 } else { 0.0 })
                    .rotate(Angle::deg(turn * 90.0)),
            )
            .child(chevron(13.0 * u, palette.get("faint"))),
        );
        line = line.child(if row.is_folder {
            glyph(16.0 * u, &[(FOLDER, palette.get("dim"), 1.6, true)])
        } else {
            let faded = with_alpha(name_color, 0.7);
            glyph(
                16.0 * u,
                &[(FILE, &faded, 1.6, true), (FILE_FOLD, &faded, 1.6, false)],
            )
        });
        line = line.child(
            span(
                Style::default()
                    .color(css_color(if selected && shine > 0.4 {
                        accent
                    } else {
                        name_color
                    }))
                    .font_size(16.0 * u)
                    .font_weight(if row.is_folder { 600.0 } else { 500.0 })
                    .white_space(WhiteSpace::Nowrap)
                    .translate(Length::px((1.0 - row_in) * -14.0 * u), Length::ZERO),
            )
            .child(row.name.clone()),
        );
        list.push(line.into());
    }
    let mut window = div(code_window(
        &palette,
        CodeWindow {
            u,
            width: panel_w,
            radius: 18.0 * u,
            open,
            rise: (1.0 - panel) * 22.0 * u,
        },
    ));
    if !props.title.is_empty() {
        let files = rows.iter().filter(|r| !r.is_folder).count();
        window = window.child(
            div(Style::default()
                .height(Length::px(HEADER_U * u))
                .display(Display::Flex)
                .align_items(Align::Center)
                .gap(9.0 * u)
                .padding(Edges::axes(Length::ZERO, Length::px(16.0 * u)))
                .background_color(css_color(palette.get("header")))
                .border(bottom_line(palette.get("border")))
                .opacity(header_in))
            .child(glyph(17.0 * u, &[(FOLDER, accent, 1.6, true)]))
            .child(
                span(
                    Style::default()
                        .color(css_color(palette.get("fg")))
                        .font_size(15.0 * u)
                        .font_weight(600.0)
                        .letter_spacing(Length::em(0.01)),
                )
                .child(props.title.clone()),
            )
            .child(
                span(
                    Style::default()
                        .margin_left(Length::Auto)
                        .color(css_color(palette.get("faint")))
                        .font_size(13.0 * u)
                        .font_weight(500.0)
                        .font_variant_numeric(NumericVariant {
                            tabular: true,
                            ..NumericVariant::default()
                        }),
                )
                .child(format!("{files} {}", props.count_label)),
            ),
        );
    }
    let window = window.child(
        div(Style::default()
            .padding(Edges {
                top: Length::px(10.0 * u),
                right: Length::px(12.0 * u),
                bottom: Length::px(14.0 * u),
                left: Length::px(12.0 * u),
            })
            .min_height(Length::px(count * row_h))
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column))
        .children(list),
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
    .child(window)
    .into())
}
