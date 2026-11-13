pub mod boxes;
pub mod effects;
pub mod flex;
pub mod media;
pub mod paints;
pub mod size;
pub mod svg;
pub mod text;
pub mod transforms;
pub mod values;

use std::path::PathBuf;

use opsis_golden::{dev_cache, repo};
use opsis_scene::color::Color;
use opsis_scene::node::{Element, Node, div, span};
use opsis_scene::style::{Overflow, Position, Style};
use opsis_scene::values::Length;

pub const WIDTH: f64 = 640.0;
pub const HEIGHT: f64 = 360.0;
const ORACLE_ENV: &str = "OPSIS_ORACLE";
const GUTTER: f64 = 24.0;
const GAP: f64 = 16.0;
const CELL_WIDTH: f64 = 136.0;
const CELL_HEIGHT: f64 = 148.0;
const COLUMNS: usize = 4;

pub fn oracle_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(ORACLE_ENV) {
        return PathBuf::from(dir);
    }
    let root = dev_cache("oracle");
    std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{}: {e}. Run bun run oracle first.", root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("cases.json").is_file())
        .min()
        .unwrap_or_else(|| panic!("no oracle with cases in {}", root.display()))
}

pub fn case_font_files() -> Vec<(String, PathBuf)> {
    let file = oracle_dir().join("cases.json");
    let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let cases: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let mut out = Vec::new();
    for (name, family) in cases["fonts"]["families"].as_object().into_iter().flatten() {
        for face in family["faces"].as_array().into_iter().flatten() {
            let Some(source) = face["file"].as_str() else {
                continue;
            };
            out.push((
                name.clone(),
                repo()
                    .join("public")
                    .join(source.trim_start_matches("app:")),
            ));
        }
    }
    out
}

pub const FALLBACK_FOLDERS: [&str; 2] = [
    "public/fonts/bundled/fallback/opsis-fallback",
    "public/fonts/bundled/terminal/opsis-terminal-symbols-2",
];

pub fn fallback_font_files() -> Vec<(String, PathBuf)> {
    FALLBACK_FOLDERS
        .iter()
        .flat_map(|folder| subset_files(&repo().join(folder)))
        .collect()
}

fn subset_files(folder: &std::path::Path) -> Vec<(String, PathBuf)> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(folder)
        .unwrap_or_else(|e| panic!("{}: {e}", folder.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "woff2"))
        .collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let name = path.file_stem()?.to_str()?.to_owned();
            let subset = name
                .trim_start_matches("noto-sans-")
                .rsplitn(3, '-')
                .nth(2)?
                .to_owned();
            Some((subset, path))
        })
        .collect()
}

pub fn app_file(source: &str) -> Option<PathBuf> {
    source
        .strip_prefix("app:")
        .map(|file| repo().join("public").join(file))
}

pub fn cell(index: usize, style: &Style) -> Style {
    let across = opsis_types::numbers::float(index % COLUMNS) * (CELL_WIDTH + GAP);
    let down = opsis_types::numbers::float(index / COLUMNS) * (CELL_HEIGHT + GAP);
    Style::default()
        .position(Position::Absolute)
        .left(GUTTER + across)
        .top(GUTTER + down)
        .width(CELL_WIDTH)
        .height(CELL_HEIGHT)
        .merged(style)
}

pub fn ink() -> Color {
    Color::rgb8(248, 250, 252)
}

pub fn paper() -> Color {
    Color::rgb8(15, 23, 42)
}

pub fn accent() -> Color {
    Color::rgb8(124, 58, 237)
}

pub fn warm() -> Color {
    Color::rgb8(249, 115, 22)
}

pub fn cool() -> Color {
    Color::rgb8(14, 165, 233)
}

pub fn leaf() -> Color {
    Color::rgb8(34, 197, 94)
}

pub fn rose() -> Color {
    Color::rgb8(244, 63, 94)
}

pub fn slate() -> Color {
    Color::rgb8(51, 65, 85)
}

pub fn fill() -> Color {
    warm()
}

pub fn tinted(nodes: impl IntoIterator<Item = Node>, colors: &[Color]) -> Vec<Node> {
    nodes
        .into_iter()
        .zip(colors.iter().cycle())
        .map(|(node, color)| match node {
            Node::Element(mut element) => {
                element.style = element.style.background_color(color.clone());
                Node::Element(element)
            }
            text @ Node::Text(_) => text,
        })
        .collect()
}

fn tint_at(node: &mut Node, path: &[usize], color: &Color) {
    let Node::Element(element) = node else {
        return;
    };
    match path.split_first() {
        None => element.style = element.style.clone().background_color(color.clone()),
        Some((first, rest)) => {
            if let Some(child) = element.children.get_mut(*first) {
                tint_at(child, rest, color);
            }
        }
    }
}

fn merge_at(node: &mut Node, path: &[usize], style: &Style) {
    let Node::Element(element) = node else {
        return;
    };
    match path.split_first() {
        None => element.style = element.style.clone().merged(style),
        Some((first, rest)) => {
            if let Some(child) = element.children.get_mut(*first) {
                merge_at(child, rest, style);
            }
        }
    }
}

pub fn styled(mut root: Node, list: &[(&[usize], Style)]) -> Node {
    for (path, style) in list {
        merge_at(&mut root, path, style);
    }
    root
}

pub fn colored(mut root: Node, list: &[(&[usize], Color)]) -> Node {
    for (path, color) in list {
        tint_at(&mut root, path, color);
    }
    root
}

pub fn rainbow() -> [Color; 4] {
    [warm(), cool(), leaf(), rose()]
}

pub const fn pct(value: f64) -> Length {
    Length::percent(value)
}

pub const fn d(style: Style) -> Element {
    div(style)
}

pub const fn s(style: Style) -> Element {
    span(style)
}

pub fn stage(children: Vec<Node>) -> Node {
    let film = Style::default()
        .position(Position::Absolute)
        .left(0.0)
        .top(0.0)
        .width(WIDTH)
        .height(HEIGHT)
        .overflow(Overflow::Hidden);
    let stage = Style::default()
        .position(Position::Absolute)
        .inset(Length::ZERO)
        .overflow(Overflow::Hidden)
        .background_color(paper())
        .color(ink())
        .font_family(vec!["Inter".to_owned()])
        .font_size(16.0);
    div(film).child(div(stage).children(children)).into()
}

pub fn scene(id: &str) -> Option<Node> {
    Some(match id {
        "box/border" => boxes::border(),
        "box/padding" => boxes::padding(),
        "box/overflow" => boxes::overflow(),
        "box/object-fit" => boxes::object_fit(),
        "flex/display" => flex::display(),
        "flex/flex-direction" => flex::flex_direction(),
        "flex/justify-content" => flex::justify_content(),
        "flex/gap" => flex::gap(),
        "flex/flex" => flex::flex(),
        "flex/flex-wrap" => flex::flex_wrap(),
        "flex/grid-template-columns" => flex::grid_template_columns(),
        "flex/align-items" => flex::align_items(),
        "flex/flex-shrink" => flex::flex_shrink(),
        "flex/place-items" => flex::place_items(),
        "media/img" => media::img_case(),
        "size/width-height" => size::width_height(),
        "size/min-max-width" => size::min_max_width(),
        "size/position" => size::position(),
        "size/inset" => size::inset(),
        "size/z-index" => size::z_index(),
        "text/font-family" => text::font_family(),
        "text/font-size" => text::font_size(),
        "text/letter-spacing" => text::letter_spacing(),
        "text/line-height" => text::line_height(),
        "text/text-align" => text::text_align(),
        "text/white-space" => text::white_space(),
        "text/color" => text::color(),
        "text/font-weight" => text::font_weight(),
        "text/font-variant-numeric" => text::font_variant_numeric(),
        "text/font-variation-settings" => text::font_variation_settings(),
        "text/text-transform" => text::text_transform(),
        "text/text-shadow" => text::text_shadow(),
        "text/text-stroke" => text::text_stroke(),
        "text/text-overflow" => text::text_overflow(),
        "text/wrapping" => text::wrapping(),
        "text/spans" => text::spans(),
        "text/vertical-align" => text::vertical_align(),
        "effects/translate" => transforms::translate_case(),
        "effects/rotate" => transforms::rotate_case(),
        "effects/scale" => transforms::scale_case(),
        "effects/transform" => transforms::transform_case(),
        "effects/transform-origin" => transforms::transform_origin_case(),
        "effects/perspective" => transforms::perspective_case(),
        "effects/matrix3d" => transforms::matrix3d_case(),
        "values/calc" => values::calc(),
        "values/color-mix" => values::color_mix(),
        "values/oklab" => values::oklab(),
        "values/oklch" => values::oklch(),
        "values/container-units" => values::container_units(),
        "values/relative-units" => values::relative_units(),
        "box/background-color" => boxes::background_color(),
        "box/background-layers" => boxes::background_layers(),
        "box/border-radius" => boxes::border_radius(),
        "box/box-shadow" => boxes::box_shadow(),
        "box/linear-gradient" => boxes::linear_gradient(),
        "box/radial-gradient" => boxes::radial_gradient(),
        "box/conic-gradient" => boxes::conic_gradient(),
        "box/repeating-gradient" => boxes::repeating_gradient(),
        "effects/opacity" => effects::opacity(),
        "effects/filter-blur" => effects::filter_blur(),
        "effects/filter-color" => effects::filter_color(),
        "effects/filter-drop-shadow" => effects::filter_drop_shadow(),
        "effects/mix-blend-mode" => effects::mix_blend_mode(),
        "effects/backdrop-filter" => effects::backdrop_filter(),
        "effects/background-clip-text" => effects::background_clip_text(),
        "effects/clip-path" => effects::clip_path(),
        "effects/clip-path-url" => effects::clip_path_url(),
        "effects/mask-image" => effects::mask_image(),
        "svg/view-box" => svg::view_box(),
        "media/canvas" => media::canvas_case(),
        "svg-filters/css-url-filter" => svg::css_url_filter(),
        other => return svg::scene(other),
    })
}

pub fn ids() -> Vec<&'static str> {
    IDS.iter()
        .copied()
        .chain(svg::MARKUPS.iter().map(|(id, _)| *id))
        .collect()
}

pub const IDS: &[&str] = &[
    "media/canvas",
    "svg/view-box",
    "svg-filters/css-url-filter",
    "effects/opacity",
    "effects/filter-blur",
    "effects/filter-color",
    "effects/filter-drop-shadow",
    "effects/mix-blend-mode",
    "effects/backdrop-filter",
    "effects/background-clip-text",
    "effects/clip-path",
    "effects/clip-path-url",
    "effects/mask-image",
    "box/background-color",
    "box/background-layers",
    "box/border-radius",
    "box/box-shadow",
    "box/linear-gradient",
    "box/radial-gradient",
    "box/conic-gradient",
    "box/repeating-gradient",
    "box/border",
    "box/padding",
    "box/overflow",
    "box/object-fit",
    "flex/display",
    "flex/flex-direction",
    "flex/justify-content",
    "flex/gap",
    "flex/flex",
    "flex/flex-wrap",
    "flex/grid-template-columns",
    "flex/align-items",
    "flex/flex-shrink",
    "flex/place-items",
    "media/img",
    "size/width-height",
    "size/min-max-width",
    "size/position",
    "size/inset",
    "size/z-index",
    "text/font-family",
    "text/font-size",
    "text/letter-spacing",
    "text/line-height",
    "text/text-align",
    "text/white-space",
    "text/color",
    "text/font-weight",
    "text/font-variant-numeric",
    "text/font-variation-settings",
    "text/text-transform",
    "text/text-shadow",
    "text/text-stroke",
    "text/text-overflow",
    "text/wrapping",
    "text/spans",
    "text/vertical-align",
    "effects/translate",
    "effects/rotate",
    "effects/scale",
    "effects/transform",
    "effects/transform-origin",
    "effects/perspective",
    "effects/matrix3d",
    "values/calc",
    "values/color-mix",
    "values/oklab",
    "values/oklch",
    "values/container-units",
    "values/relative-units",
];
