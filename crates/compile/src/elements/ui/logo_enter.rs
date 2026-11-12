use opsis_macros::settings;
use opsis_scene::color::Color;
use opsis_scene::css::with_background;
use opsis_scene::interpolate::{Interpolation, interpolate};
use opsis_scene::node::{Node, div, img};
use opsis_scene::paint::Shadow;
use opsis_scene::spring::SpringConfig;
use opsis_scene::style::{
    Align, BorderSide, BorderStyle, Display, FlexDirection, ObjectFit, Style,
};
use opsis_scene::timeline::absolute_fill;
use opsis_scene::values::{Edges, Length};
use opsis_types::looks::NodeLook;
use opsis_types::numbers::float;

use crate::elements::anim::sprung;
use crate::elements::draw::{Cx, DrawFailure, css_color};
use crate::elements::entry::{element, meta};
use crate::elements::shared::Orientation;
use crate::elements::{ElementCategory, ElementKind, ElementMeta};

#[settings(export, eq)]
pub struct LogoEnterLogo {
    #[setting(
        "Image",
        default = "app:placeholders/logo-mark.svg",
        widget = "asset",
        accept = "image/*"
    )]
    pub image: String,
    #[setting("Background", default = "#2563eb", check = colour)]
    pub background: String,
}

#[settings(export)]
pub struct LogoEnter {
    #[setting(
        "Diameter",
        default = 118,
        range(min = 48, max = 320),
        step = 1,
        unit = "px"
    )]
    pub diameter: u32,
    #[setting(
        "Overlap",
        default = 38,
        range(min = 0, max = 200),
        step = 1,
        unit = "px"
    )]
    pub overlap: u32,
    #[setting("Ring", default = "#fff", check = colour)]
    pub ring_color: String,
    #[setting("Orientation", default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[setting(
        "Stagger",
        default = 0.2333,
        range(min = 0),
        step = 0.01,
        unit = "s",
        frames_at = 30,
        advanced
    )]
    pub stagger: f64,
    #[setting(
        "Logos",
        default = vec![
            LogoEnterLogo::default(),
            LogoEnterLogo { background: "#0f766e".into(), ..Default::default() },
            LogoEnterLogo { background: "#b45309".into(), ..Default::default() },
            LogoEnterLogo { background: "#7c3aed".into(), ..Default::default() },
        ],
        length(min = 1, max = 16)
    )]
    pub logos: Vec<LogoEnterLogo>,
    #[setting("Speed", default = 1.0, range(min = 0.25, max = 4), step = 0.25)]
    pub speed: f64,
    #[setting(
        "Bounce damping",
        default = 13,
        range(min = 4, max = 40),
        step = 1,
        help = "Lower bounces more as each logo lands.",
        advanced
    )]
    pub bounce: u32,
    #[setting(
        "Ring width",
        default = 5,
        range(min = 0, max = 20),
        step = 1,
        unit = "px",
        advanced
    )]
    pub ring_width: u32,
    #[setting(
        "Shadow",
        default = 0.5,
        range(min = 0, max = 1),
        step = 0.01,
        advanced
    )]
    pub shadow: f64,
    #[setting(
        "Logo size",
        default = 52,
        range(min = 20, max = 100),
        step = 1,
        unit = "%",
        advanced
    )]
    pub logo_scale: u32,
    #[setting("First logo on top", default = true, advanced)]
    pub first_on_top: bool,
}

pub fn definition() -> ElementMeta {
    meta::<LogoEnter>(
            element(
                ElementKind::Component,
                "logo-enter",
                "Logo enter",
                "A stacked group of round brand chips that spring in one-by-one, then hold. Swap the logos to feature any set of marks.",
                ElementCategory::Ui,
                90,
            )
                .authored_clock()
                .design(1920, 1080)
                .look(NodeLook::of("lucide:layout-grid-circles")),
        )
}

const VERTICAL_ROOM: f64 = 980.0;
const HORIZONTAL_ROOM: f64 = 1800.0;
const ENTRY: SpringConfig = SpringConfig {
    damping: 13.0,
    stiffness: 130.0,
    mass: 0.8,
    overshoot_clamping: false,
};

fn logo(
    props: &LogoEnter,
    index: usize,
    entry: &LogoEnterLogo,
    frame: f64,
    fps: f64,
) -> Result<Node, DrawFailure> {
    let vertical = props.orientation == Orientation::Vertical;
    let s = sprung(
        float(index).mul_add(-props.stagger, frame * props.speed),
        fps,
        SpringConfig {
            damping: f64::from(props.bounce),
            ..ENTRY
        },
    )?;
    let scale = interpolate(s, &[0.0, 1.0], &[0.5, 1.0], &Interpolation::default())?;
    let offset = interpolate(s, &[0.0, 1.0], &[22.0, 0.0], &Interpolation::default())?;
    let diameter = f64::from(props.diameter);
    let lead = if index == 0 {
        0.0
    } else {
        -f64::from(props.overlap)
    };
    let count = props.logos.len();
    let inner = diameter * f64::from(props.logo_scale) / 100.0;
    let mut mark = div(Style::default()
        .width(Length::px(inner))
        .height(Length::px(inner))
        .display(Display::Flex));
    if !entry.image.is_empty() {
        mark = mark.child(img(
            entry.image.clone(),
            Style::default()
                .width(Length::percent(100.0))
                .height(Length::percent(100.0))
                .object_fit(ObjectFit::Contain),
        ));
    }
    let (shift_x, shift_y) = if vertical {
        (offset, 0.0)
    } else {
        (0.0, offset)
    };
    let style = Style::default()
        .width(Length::px(diameter))
        .height(Length::px(diameter))
        .border_radius_all(Length::percent(50.0))
        .margin_left(Length::px(if vertical { 0.0 } else { lead }))
        .margin_top(Length::px(if vertical { lead } else { 0.0 }))
        .z_index(
            i32::try_from(if props.first_on_top {
                count - index
            } else {
                index + 1
            })
            .unwrap_or(0),
        )
        .border(Edges::all(BorderSide {
            width: Length::px(f64::from(props.ring_width)),
            style: BorderStyle::Solid,
            color: Some(css_color(&props.ring_color)),
        }))
        .box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(20.0),
            blur: Length::px(48.0),
            spread: Length::ZERO,
            color: Color::srgb_alpha(0.0, 0.0, 0.0, props.shadow),
        }])
        .display(Display::Flex)
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .opacity(s)
        .translate(Length::px(shift_x), Length::px(shift_y))
        .scale(scale, scale);
    Ok(div(with_background(style, &entry.background))
        .child(mark)
        .into())
}

pub fn draw(props: &LogoEnter, cx: &Cx<'_>) -> Result<Node, DrawFailure> {
    let (frame, fps) = (cx.authored_frame(), Cx::authored_fps());
    let vertical = props.orientation == Orientation::Vertical;
    let room = if vertical {
        VERTICAL_ROOM
    } else {
        HORIZONTAL_ROOM
    };
    let count = float(props.logos.len());
    let span = (count - 1.0)
        .max(0.0)
        .mul_add(-f64::from(props.overlap), count * f64::from(props.diameter));
    let fit = if span > room { room / span } else { 1.0 };
    let logos = props
        .logos
        .iter()
        .enumerate()
        .map(|(i, entry)| logo(props, i, entry, frame, fps))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center))
    .child(
        div(Style::default()
            .display(Display::Flex)
            .flex_direction(if vertical {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            })
            .scale(fit, fit))
        .children(logos),
    )
    .into())
}
