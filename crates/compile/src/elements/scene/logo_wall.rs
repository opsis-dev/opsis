use opsis_macros::settings;
use opsis_scene::color::Interpolation as ColorInterpolation;
use opsis_scene::css::with_background;
use opsis_scene::easing::Easing;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, img, span};
use opsis_scene::paint::{ColorStop, Filter, Gradient, LinearDirection};
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexWrap, ObjectFit, Overflow, Style, TextOverflow,
    TextTransform, WhiteSpace,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Angle, Edges, Length, Num};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::{float, whole};

use crate::elements::anim::clamped;
use crate::elements::code_theme::{Palette, code_palette};
use crate::elements::draw::{Cx, DrawFailure, css_color, tinted};
use crate::elements::entry::{element, meta};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::geometry::safe_padding;
use crate::elements::text_split::easing_enter;
use crate::elements::theme::{Theme, ThemeMode, code_themes};
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct LogoWallLogo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting(
        "Image",
        widget = "asset",
        accept = "image/*",
        zero_label = "Name and glyph",
        "type" = "string"
    )]
    pub image: Option<String>,
    #[setting("Name", default = "Marrow")]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Color", widget = "color", zero_label = "None", "type" = "string")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[setting("Glyph", "type" = "string")]
    pub glyph: Option<String>,
}

#[settings(export)]
pub struct LogoWall {
    #[setting(
        "Logos",
        default = vec![
            LogoWallLogo {
                name: "Northstar".into(),
                color: Some("#E8B86D".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Halcyon".into(),
                color: Some("#7DD3E8".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Fernweh".into(),
                color: Some("#9BD4A0".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Orbital".into(),
                color: Some("#C99BE8".into()),
                ..Default::default()
            },
            LogoWallLogo { name: "Lumen".into(), color: Some("#8FB8F0".into()), ..Default::default() },
            LogoWallLogo {
                name: "Tidewater".into(),
                color: Some("#E89B9B".into()),
                ..Default::default()
            },
            LogoWallLogo {
                name: "Kestrel".into(),
                color: Some("#E8B86D".into()),
                ..Default::default()
            },
            LogoWallLogo { color: Some("#7DD3E8".into()), ..Default::default() },
        ],
        help = "Wordmark, brand colour and the glyph in the mark beside it, or a logo image in their place."
    )]
    pub logos: Vec<LogoWallLogo>,
    #[setting(
        "Small heading",
        default = "Trusted by",
        help = "Small line above the heading. Omit to drop it."
    )]
    pub eyebrow: String,
    #[setting(
        "Title",
        default = "Teams that ship with us",
        help = "Heading above the wall. Omit to drop it."
    )]
    pub title: String,
    #[setting(
        "Columns",
        default = 4,
        range(min = 1),
        step = 1,
        help = "Tiles per row."
    )]
    pub columns: u32,
    #[setting(
        "Start at (seconds)",
        default = 0.32,
        step = 0.01,
        help = "Second the first tile arrives.",
        advanced
    )]
    pub start_at_seconds: f64,
    #[setting(
        "Arrive stagger (seconds)",
        default = 0.06,
        step = 0.01,
        help = "Seconds between tiles arriving, in reading order. Deliberately fast.",
        advanced
    )]
    pub arrive_stagger_seconds: f64,
    #[setting(
        "Stagger (seconds)",
        default = 0.24,
        step = 0.01,
        help = "Seconds between diagonals of the colour sweep.",
        advanced
    )]
    pub stagger_seconds: f64,
    #[setting(
        "Colour fade (seconds)",
        default = 0.6,
        range(min = 0.05, max = 10),
        step = 0.01,
        help = "How long one tile takes to go from grey to brand colour.",
        advanced
    )]
    pub warm_seconds: f64,
    #[setting(
        "Accent color",
        default = "#E8B86D",
        help = "Eyebrow, and the fallback for logos with no colour.",
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
    #[setting(
        "Mark text color",
        widget = "color",
        zero_label = "Theme",
        check = colour_or_empty,
        advanced
    )]
    pub glyph_color: String,
    #[setting(
        "Font",
        widget = "font",
        help = "Project font follows the fonts set for the whole project.",
        advanced
    )]
    pub font_family: String,
    #[setting(
        "Wall width",
        default = 900,
        range(min = 300, max = 1800),
        step = 10,
        unit = "px",
        advanced
    )]
    pub wall_width: u32,
    #[setting(
        "Gap",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub gap: u32,
    #[setting(
        "Tile height",
        default = 88,
        range(min = 40, max = 240),
        step = 1,
        unit = "px",
        advanced
    )]
    pub tile_height: u32,
    #[setting(
        "Tile corners",
        default = 14,
        range(min = 0, max = 60),
        step = 1,
        unit = "px",
        advanced
    )]
    pub tile_radius: u32,
    #[setting(
        "Title size",
        default = 30,
        range(min = 12, max = 80),
        step = 1,
        unit = "px",
        advanced
    )]
    pub title_size: u32,
    #[setting("Uppercase small heading", default = true, advanced)]
    pub eyebrow_uppercase: bool,
    #[setting(
        "Name size",
        default = 19,
        range(min = 10, max = 48),
        step = 1,
        unit = "px",
        advanced
    )]
    pub name_size: u32,
    #[setting(
        "Logo image height",
        default = 46,
        range(min = 16, max = 120),
        step = 1,
        unit = "px",
        advanced
    )]
    pub image_size: u32,
    #[setting(
        "Mark",
        default = true,
        help = "The small badge with the first letter or glyph.",
        advanced
    )]
    pub show_mark: bool,
    #[setting(
        "Starting brightness",
        default = 0.72,
        range(min = 0, max = 1),
        step = 0.01,
        help = "How dim the grey logos are before they warm up.",
        advanced
    )]
    pub start_brightness: f64,
}

pub fn definition() -> ElementMeta {
    meta::<LogoWall>(
        element(
            ElementKind::Component,
            "logo-wall",
            "Logo wall",
            "A grid of customer logos that light up from grey to colour.",
            ElementCategory::Scene,
            120,
        )
        .look(NodeLook::of("lucide:layout-grid")),
    )
}

const HEAD_AT: f64 = 0.08;
const HEAD_FOR: f64 = 0.4;
const EDITORIAL: Easing = Easing::Bezier(0.45, 0.0, 0.55, 1.0);

fn tile(
    props: &LogoWall,
    logo: &LogoWallLogo,
    size: (f64, f64),
    u: f64,
    (arrive, warm): (f64, f64),
    dark: bool,
    palette: &Palette,
) -> Result<Node, DrawFailure> {
    let (tile_w, tile_h) = size;
    let color = logo.color.as_deref().unwrap_or(&props.accent_color);
    let mark_warm = clamped(warm, [0.0, 0.7], [0.0, 1.0])?;
    let grow = arrive.mul_add(0.06, 0.94);
    let border = if warm > 0.5 {
        tinted(color, 27.0)
    } else {
        css_color(palette.get("border"))
    };
    let mut out = div(Style::default()
        .width(Length::px(tile_w))
        .height(Length::px(tile_h))
        .border_radius_all(f64::from(props.tile_radius) * u)
        .background_color(css_color(palette.get("band")))
        .border(Edges::all(BorderSide {
            width: Length::px(1.0),
            style: BorderStyle::Solid,
            color: Some(border),
        }))
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .gap(10.0 * u)
        .opacity(arrive)
        .scale(grow, grow)
        .filter(vec![
            Filter::Saturate(Num(warm)),
            Filter::Brightness(Num(
                warm.mul_add(1.0 - props.start_brightness, props.start_brightness)
            )),
        ]));
    if let Some(image) = logo.image.as_deref().filter(|i| !i.is_empty()) {
        return Ok(out
            .child(img(
                image.to_owned(),
                Style::default()
                    .max_width(Length::percent(78.0))
                    .max_height(Length::px(f64::from(props.image_size) * u))
                    .object_fit(ObjectFit::Contain),
            ))
            .into());
    }
    if props.show_mark {
        let glyph = logo
            .glyph
            .clone()
            .unwrap_or_else(|| logo.name.chars().next().map_or_default(String::from));
        let fill = if !props.glyph_color.is_empty() {
            props.glyph_color.as_str()
        } else if dark {
            "#12100C"
        } else {
            "#FFFFFF"
        };
        out = out.child(
            div(Style::default()
                .width(Length::px(30.0 * u))
                .height(Length::px(30.0 * u))
                .flex_shrink(0.0)
                .border_radius_all(9.0 * u)
                .display(Display::Grid)
                .align_items(Align::Center)
                .justify_items(Align::Center)
                .background(vec![
                    Gradient::Linear {
                        direction: LinearDirection::Angle(Angle::deg(140.0)),
                        stops: vec![
                            ColorStop::new(css_color(color)),
                            ColorStop::new(tinted(color, 53.0)),
                        ],
                        repeating: false,
                        interpolation: ColorInterpolation::DEFAULT,
                    }
                    .layer(),
                ])
                .color(css_color(fill))
                .font_size(16.0 * u)
                .font_weight(800.0)
                .opacity(mark_warm.mul_add(0.55, 0.45)))
            .child(glyph),
        );
    }
    let room = 24.0f64.mul_add(-u, tile_w - if props.show_mark { 40.0 * u } else { 0.0 });
    Ok(out
        .child(
            span(
                Style::default()
                    .color(css_color(color))
                    .font_size(f64::from(props.name_size) * u)
                    .font_weight(700.0)
                    .letter_spacing(Length::em(-0.02))
                    .min_width(Length::ZERO)
                    .max_width(Length::px(room))
                    .overflow(Overflow::Hidden)
                    .text_overflow(TextOverflow::Ellipsis)
                    .white_space(WhiteSpace::Nowrap),
            )
            .child(logo.name.clone()),
        )
        .into())
}

pub fn draw(props: &LogoWall, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
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
    let head_in = ease(HEAD_AT, HEAD_AT + HEAD_FOR, easing_enter())?;
    let wall_w = f64::mul_add(side, -2.0, width).min(f64::from(props.wall_width) * u);
    let gap = f64::from(props.gap) * u;
    let per_row = whole(f64::from(props.columns)).max(1);
    let per = f64::from(per_row);
    let tile_w = gap.mul_add(-(per - 1.0), wall_w) / per;
    let tile_h = f64::from(props.tile_height) * u;
    let dark = props.theme.mode == ThemeMode::Dark;
    let mut wall = div(Style::default().width(Length::px(wall_w)));
    if !props.eyebrow.is_empty() {
        wall = wall.child(
            div(Style::default()
                .color(css_color(&props.accent_color))
                .font_size(14.5 * u)
                .font_weight(600.0)
                .letter_spacing(Length::em(0.14))
                .text_transform(if props.eyebrow_uppercase {
                    TextTransform::Uppercase
                } else {
                    TextTransform::None
                })
                .opacity(head_in))
            .child(props.eyebrow.clone()),
        );
    }
    if !props.title.is_empty() {
        wall = wall.child(
            div(Style::default()
                .margin_top(Length::px(8.0 * u))
                .margin_bottom(Length::px(24.0 * u))
                .color(css_color(palette.get("fg")))
                .font_size(f64::from(props.title_size) * u)
                .font_weight(700.0)
                .letter_spacing(Length::em(-0.02))
                .opacity(head_in)
                .translate(Length::ZERO, Length::px((1.0 - head_in) * 12.0 * u)))
            .child(props.title.clone()),
        );
    }
    let mut tiles = Vec::new();
    for (index, logo) in props.logos.iter().enumerate() {
        let i = u32::try_from(index).unwrap_or(u32::MAX);
        let (row, column) = (f64::from(i / per_row), f64::from(i % per_row));
        let arrive_at = float(index).mul_add(props.arrive_stagger_seconds, props.start_at_seconds);
        let warm_at = (row + column).mul_add(props.stagger_seconds, props.start_at_seconds + 0.3);
        let arrive = ease(arrive_at, arrive_at + 0.36, easing_enter())?;
        let warm = ease(warm_at, warm_at + props.warm_seconds, EDITORIAL)?;
        tiles.push(tile(
            props,
            logo,
            (tile_w, tile_h),
            u,
            (arrive, warm),
            dark,
            &palette,
        )?);
    }
    let wall = wall.child(
        div(Style::default()
            .display(Display::Flex)
            .flex_wrap(FlexWrap::Wrap)
            .gap(gap))
        .children(tiles),
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
    .child(wall)
    .into())
}
