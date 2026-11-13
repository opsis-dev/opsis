use opsis_scene::clock::js_round;
use opsis_scene::keyed::percent;
use opsis_scene::measure::Measure;
use opsis_scene::node::{Node, div, img};
use opsis_scene::paint::{Filter, Shadow, Transform};
use opsis_scene::style::{
    Align, ContainerType, Display, ObjectFit, Overflow, Position, Style, WillChange,
};
use opsis_scene::timeline::{At, absolute_fill};
use opsis_scene::values::{Calc, ContainerUnit, Length, Num};
use opsis_types::actions::{ActionKind, ActionLog};
use opsis_types::recording::{AddressMode, CameraStyle, RecordingProps, ScreenProps};
use opsis_types::timeline::ReframeMode;

use crate::elements::camera::annotations::{Annotation, Shown, annotations, resolve_annotations};
use crate::elements::camera::click_log::{ClickLog, design_scale_for, to_click_log};
use crate::elements::camera::cursor::cursor;
use crate::elements::camera::keys::{
    TYPED_LINGER_S, keycap_hud, shown_keys, typed_beats, typed_chip,
};
use crate::elements::camera::stage::{backdrop, is_light_backdrop, rim_light, stage};
use crate::elements::camera::track::{
    BASE_POSE, CameraKey, Tuning, ZoomState, build_camera_track, is_narrow, js_clamp, log_time_s,
    pose_at, pose_to_css, zoom_at,
};
use crate::elements::camera::window::{CHROME_H, Chrome, Frame, window_frame};
use crate::elements::draw::{DrawFailure, css_color};
use crate::elements::fonts::{FontRole, font_or};
use crate::elements::terminal::screen::{Fit, TerminalScreen, terminal_screen};
use crate::recording::{DEFAULT_ADDRESS_BAR, DEFAULT_FRAME_THEME, DEFAULT_WINDOW_STYLE};
use crate::text::display_address;

const PAGE_BG: &str = "#fcfcfc";
const BLEED_BG: &str = "#0b0b10";
const STAGE_COLOR: &str = "#000";
const WINDOW_FIT: f64 = 0.86;
const STAGE_FIT: f64 = 0.84;
const STAGE_GROUND: &str = "#faf6f1";
const BLUR_SHUTTER_ANGLE: f64 = 180.0;
const BLUR_SAMPLES: u32 = 8;
const SAME_SHAPE_TOLERANCE: f64 = 0.02;
const US_PER_S: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Shot {
    Window {
        fit: f64,
        ground: Option<&'static str>,
        zoom: bool,
    },
    FullBleed,
}

const fn shot_of(style: CameraStyle) -> Shot {
    match style {
        CameraStyle::Classic => Shot::Window {
            fit: WINDOW_FIT,
            ground: None,
            zoom: true,
        },
        CameraStyle::Stage => Shot::Window {
            fit: STAGE_FIT,
            ground: Some(STAGE_GROUND),
            zoom: false,
        },
        CameraStyle::Proof | CameraStyle::Ledger => Shot::FullBleed,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Size {
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Reframe {
    Same,
    Contain { box_size: Size, narrow: Option<f64> },
    Fit { height: f64 },
    Cover { box_size: Size, follow: bool },
}

fn contain_box(outer: Size, aspect: f64) -> Size {
    if outer.width / outer.height > aspect {
        Size {
            width: outer.height * aspect,
            height: outer.height,
        }
    } else {
        Size {
            width: outer.width,
            height: outer.width / aspect,
        }
    }
}

fn reframe_layout(source: Size, canvas: Size, reframe: ReframeMode, full_page: bool) -> Reframe {
    let source_aspect = source.width / source.height;
    let out_aspect = canvas.width / canvas.height;
    if (source_aspect - out_aspect).abs() <= SAME_SHAPE_TOLERANCE {
        return Reframe::Same;
    }
    let narrower = out_aspect < source_aspect;
    if reframe == ReframeMode::Follow && (narrower || full_page) {
        return Reframe::Contain {
            box_size: contain_box(canvas, source_aspect),
            narrow: narrower.then(|| js_clamp((source_aspect / out_aspect).sqrt(), 1.0, 2.0)),
        };
    }
    if reframe == ReframeMode::Fit {
        return Reframe::Fit {
            height: canvas.width / source_aspect,
        };
    }
    let width = canvas.width.max(canvas.height * source_aspect);
    Reframe::Cover {
        box_size: Size {
            width,
            height: width / source_aspect,
        },
        follow: reframe == ReframeMode::Follow,
    }
}

fn page_at(log: &ActionLog, ms: f64) -> Option<&str> {
    log.addresses
        .iter()
        .rev()
        .find(|a| a.t <= ms)
        .or_else(|| log.addresses.first())
        .map(|a| a.url.as_str())
}

fn first_page(log: &ActionLog) -> Option<&str> {
    log.actions
        .iter()
        .find(|a| a.kind == ActionKind::Navigate)
        .and_then(|a| a.label.as_deref())
}

fn path_of(address: &str) -> String {
    url::Url::parse(address)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .map_or_else(String::new, |u| {
            format!(
                "{}{}",
                u.path(),
                u.fragment().map_or_else(String::new, |f| format!("#{f}"))
            )
        })
}

fn host_of(typed: &str) -> &str {
    let rest = typed
        .find("://")
        .filter(|at| typed[..*at].chars().all(|c| c.is_ascii_alphabetic()) && *at > 0)
        .map_or(typed, |at| &typed[at + 3..]);
    rest.split(['/', '?', '#']).next().unwrap_or_default()
}

fn frame_address(
    log: &ActionLog,
    ms: f64,
    typed: Option<&str>,
    mode: Option<AddressMode>,
) -> Option<String> {
    let address = typed.map(str::trim).filter(|a| !a.is_empty());
    let Some(address) = address else {
        return first_page(log).map(|page| display_address(page, false));
    };
    if mode != Some(AddressMode::Host) {
        return Some(display_address(address, false));
    }
    let page = page_at(log, ms).or_else(|| first_page(log));
    Some(display_address(
        &format!(
            "{}{}",
            host_of(address),
            page.map_or_else(String::new, path_of)
        ),
        false,
    ))
}

fn terminal_title(log: &ActionLog, ms: f64, typed: Option<&str>) -> Option<String> {
    if let Some(text) = typed.map(str::trim).filter(|t| !t.is_empty()) {
        return Some(text.to_owned());
    }
    log.titles
        .iter()
        .rev()
        .find(|t| t.t <= ms)
        .or_else(|| log.titles.first())
        .map(|t| t.title.clone())
        .filter(|t| !t.is_empty())
}

struct Clip<'a> {
    props: &'a RecordingProps,
    measure: &'a dyn Measure,
    log: ClickLog,
    track: Vec<CameraKey>,
    tuning: Tuning,
    title: Option<String>,
    box_size: Option<Size>,
    stage: Size,
    marks: Vec<Annotation>,
}

fn screen(clip: &Clip<'_>, t: f64, fill: bool, area: Size) -> Option<Node> {
    let props = clip.props;
    let url = match &props.screen {
        ScreenProps::Footage {
            footage_url: Some(url),
            ..
        } => url,
        ScreenProps::Terminal {
            frames_url: Some(url),
            look,
            effects,
            ..
        } => {
            return terminal_screen(
                &TerminalScreen {
                    frames_url: url,
                    look,
                    effects: effects.as_ref(),
                    time_s: t,
                    fit: if fill { Fit::Fill } else { Fit::Cover },
                    area: (area.width, area.height),
                },
                clip.measure,
            );
        }
        ScreenProps::Footage { .. } | ScreenProps::Terminal { .. } => return None,
    };
    let micros = js_round(t * US_PER_S).max(0.0);
    Some(
        img(
            format!("{url}{}", opsis_scene::random::js_number_text(micros)),
            Style::default()
                .width(Length::percent(100.0))
                .height(Length::percent(100.0))
                .object_fit(if fill {
                    ObjectFit::Fill
                } else {
                    ObjectFit::Cover
                })
                .display(Display::Block),
        )
        .into(),
    )
}

const fn has_screen(props: &RecordingProps) -> bool {
    match &props.screen {
        ScreenProps::Footage { footage_url, .. } => footage_url.is_some(),
        ScreenProps::Terminal { frames_url, .. } => frames_url.is_some(),
    }
}

fn camera_transform(z: ZoomState) -> Vec<Transform> {
    vec![
        Transform::Translate(percent(z.translate_x), percent(z.translate_y), Length::ZERO),
        Transform::Scale(Num(z.scale), Num(z.scale), Num(1.0)),
    ]
}

struct Framing {
    chrome: bool,
    fit: f64,
    zoom: bool,
}

fn camera(clip: &Clip<'_>, framing: &Framing, frame: f64, fps: f64, k: f64) -> (ZoomState, Style) {
    let chrome_h = if framing.chrome { CHROME_H * k } else { 0.0 };
    let base_h = clip.box_size.map_or(clip.stage.height, |b| b.height);
    let chrome_frac = chrome_h / (base_h + chrome_h);
    let z = if framing.zoom {
        zoom_at(
            &clip.log,
            &clip.track,
            log_time_s(frame, fps, clip.props.speed),
            &clip.tuning,
            chrome_frac,
            framing.fit,
        )
    } else {
        pose_to_css(BASE_POSE, chrome_frac, framing.fit)
    };
    let (width, height) = clip.box_size.map_or_else(
        || {
            (
                Length::percent(100.0),
                Length::Calc(Box::new(Calc::Sum(vec![
                    Length::percent(100.0),
                    Length::px(chrome_h),
                ]))),
            )
        },
        |b| (Length::px(b.width), Length::px(b.height + chrome_h)),
    );
    let style = Style::default()
        .width(width)
        .height(height)
        .flex_shrink(0.0)
        .transform(camera_transform(z))
        .position(Position::Relative)
        .will_change(WillChange::Transform);
    (z, style)
}

fn window_framing(clip: &Clip<'_>, fit: f64, zoom: bool) -> Framing {
    Framing {
        chrome: clip.props.window.chrome,
        fit: clip.props.shot.window_fit.unwrap_or(fit),
        zoom,
    }
}

fn overlays(
    clip: &Clip<'_>,
    t: f64,
    scale: f64,
    k: f64,
    area: Size,
) -> Result<Vec<Node>, DrawFailure> {
    let shown = Shown {
        viewport: clip.log.viewport,
        area: (area.width, area.height),
        t,
        camera_scale: scale,
        k,
    };
    let mut out: Vec<Node> = annotations(&clip.marks, &shown)?.into_iter().collect();
    out.extend(cursor(&clip.log, &clip.props.pointer, t, scale, k));
    Ok(out)
}

fn window_group(
    clip: &Clip<'_>,
    fit: f64,
    zoom: bool,
    frame: f64,
    fps: f64,
    k: f64,
) -> Result<Node, DrawFailure> {
    let (z, style) = camera(clip, &window_framing(clip, fit, zoom), frame, fps, k);
    let t = log_time_s(frame, fps, clip.props.speed);
    let window = &clip.props.window;
    let chrome = window.chrome.then(|| Chrome {
        style: window.style.unwrap_or(DEFAULT_WINDOW_STYLE),
        theme: window.theme.unwrap_or(DEFAULT_FRAME_THEME),
        address_bar: window.bar.unwrap_or(DEFAULT_ADDRESS_BAR),
        address: clip.title.as_deref().unwrap_or(""),
        font: window.font.as_deref().map_or_else(
            || font_or("", FontRole::Main),
            |f| font_or(f, FontRole::Main),
        ),
    });
    let mut children: Vec<Node> = screen(clip, t, true, clip.box_size.unwrap_or(clip.stage))
        .into_iter()
        .collect();
    children.extend(overlays(
        clip,
        t,
        z.scale,
        k,
        clip.box_size.unwrap_or(clip.stage),
    )?);
    Ok(stage(vec![
        div(style)
            .child(window_frame(
                &Frame {
                    chrome,
                    radius: clip.props.shot.radius,
                    style: window.style.unwrap_or(DEFAULT_WINDOW_STYLE),
                    lift: clip.props.shot.lift_colours.unwrap_or(true),
                },
                children,
                k,
            ))
            .into(),
    ]))
}

fn shadow_group(
    clip: &Clip<'_>,
    fit: f64,
    zoom: bool,
    light: bool,
    frame: f64,
    fps: f64,
    k: f64,
) -> Node {
    let (_, style) = camera(clip, &window_framing(clip, fit, zoom), frame, fps, k);
    stage(vec![
        div(style)
            .children(rim_light(light, clip.props.shot.rim_glow.unwrap_or(1.0), k))
            .into(),
    ])
}

fn shutter_offsets(angle: f64, samples: u32) -> Vec<f64> {
    let open = angle.clamp(0.0, 360.0) / 360.0;
    if open == 0.0 || samples <= 1 {
        return vec![0.0];
    }
    (0..samples)
        .map(|i| open * f64::from(i) / f64::from(samples))
        .collect()
}

fn demo_clip(clip: &Clip<'_>, at: &At, k: f64) -> Result<Node, DrawFailure> {
    let (frame, fps) = (at.clock.frame, f64::from(at.clock.fps));
    let shot = shot_of(clip.props.shot.style);
    let Shot::Window { fit, ground, zoom } = shot else {
        return if clip.box_size.is_some() {
            narrow_bleed(clip, frame, fps, k)
        } else {
            full_bleed(clip, frame, fps, k)
        };
    };
    let chosen = clip.props.shot.backdrop_chosen.unwrap_or(false);
    let name = if chosen {
        clip.props.shot.backdrop.as_str()
    } else {
        ground.unwrap_or(&clip.props.shot.backdrop)
    };
    let mut root = div(absolute_fill())
        .child(backdrop(name))
        .child(shadow_group(
            clip,
            fit,
            zoom,
            is_light_backdrop(name),
            frame,
            fps,
            k,
        ));
    if zoom && clip.props.shot.motion_blur {
        let angle = clip.props.shot.shutter_angle.unwrap_or(BLUR_SHUTTER_ANGLE);
        let mut layers = Vec::new();
        for (i, offset) in shutter_offsets(angle, BLUR_SAMPLES).into_iter().enumerate() {
            layers.push(
                div(absolute_fill().opacity(1.0 / (opsis_types::numbers::float(i) + 1.0)))
                    .child(window_group(
                        clip,
                        fit,
                        zoom,
                        (frame - offset).max(0.0),
                        fps,
                        k,
                    )?)
                    .into(),
            );
        }
        root = root.child(div(absolute_fill()).children(layers));
    } else {
        root = root.child(window_group(clip, fit, zoom, frame, fps, k)?);
    }
    Ok(root.into())
}

fn full_bleed(clip: &Clip<'_>, frame: f64, fps: f64, k: f64) -> Result<Node, DrawFailure> {
    let t = log_time_s(frame, fps, clip.props.speed);
    let mut children: Vec<Node> = screen(clip, t, false, clip.stage).into_iter().collect();
    children.extend(overlays(clip, t, 1.0, k, clip.stage)?);
    Ok(div(absolute_fill()
        .overflow(Overflow::Hidden)
        .background_color(css_color(PAGE_BG)))
    .children(children)
    .into())
}

fn narrow_bleed(clip: &Clip<'_>, frame: f64, fps: f64, k: f64) -> Result<Node, DrawFailure> {
    let framing = Framing {
        chrome: false,
        fit: 1.0,
        zoom: is_narrow(&clip.tuning),
    };
    let (z, style) = camera(clip, &framing, frame, fps, k);
    let t = log_time_s(frame, fps, clip.props.speed);
    let shot = &clip.props.shot;
    let blurred = div(absolute_fill()
        .filter(vec![
            Filter::Blur(Length::px(shot.bleed_blur.unwrap_or(48.0))),
            Filter::Brightness(Num(shot.bleed_dim.unwrap_or(0.55))),
            Filter::Saturate(Num(1.2)),
        ])
        .transform(vec![Transform::Scale(Num(1.15), Num(1.15), Num(1.0))]))
    .children(screen(clip, t, false, clip.stage));
    let mut inner: Vec<Node> = screen(clip, t, true, clip.box_size.unwrap_or(clip.stage))
        .into_iter()
        .collect();
    inner.extend(overlays(
        clip,
        t,
        z.scale,
        k,
        clip.box_size.unwrap_or(clip.stage),
    )?);
    Ok(div(absolute_fill()
        .overflow(Overflow::Hidden)
        .background_color(css_color(BLEED_BG)))
    .child(blurred)
    .child(stage(vec![
        div(style.box_shadow(vec![Shadow {
            inset: false,
            x: Length::ZERO,
            y: Length::px(30.0),
            blur: Length::px(80.0),
            spread: Length::ZERO,
            color: css_color("rgba(0,0,0,0.45)"),
        }]))
        .children(inner)
        .into(),
    ]))
    .into())
}

fn screen_clip(clip: &Clip<'_>, at: &At, k: f64) -> Result<Node, DrawFailure> {
    let (frame, fps) = (at.clock.frame, f64::from(at.clock.fps));
    let zoom = matches!(
        shot_of(clip.props.shot.style),
        Shot::Window { zoom: true, .. }
    );
    let (z, style) = camera(
        clip,
        &Framing {
            chrome: false,
            fit: 1.0,
            zoom,
        },
        frame,
        fps,
        k,
    );
    let t = log_time_s(frame, fps, clip.props.speed);
    let vp = clip.log.viewport;
    let aspect = f64::from(vp.width) / f64::from(vp.height);
    let area = Size {
        width: clip.stage.width.max(clip.stage.height * aspect),
        height: clip.stage.height.max(clip.stage.width / aspect),
    };
    let mut inner: Vec<Node> = screen(clip, t, true, area).into_iter().collect();
    inner.extend(overlays(clip, t, z.scale, k, area)?);
    let cover = |a: Length, b: Length| Length::Calc(Box::new(Calc::Max(vec![a, b])));
    Ok(div(absolute_fill()
        .overflow(Overflow::Hidden)
        .container_type(ContainerType::Size)
        .background_color(css_color(PAGE_BG)))
    .child(
        div(Style::default()
            .position(Position::Absolute)
            .left(Length::percent(50.0))
            .top(Length::percent(50.0))
            .width(cover(
                Length::Container(ContainerUnit::Cqw, Num(100.0)),
                Length::Container(ContainerUnit::Cqh, Num(100.0 * aspect)),
            ))
            .height(cover(
                Length::Container(ContainerUnit::Cqh, Num(100.0)),
                Length::Container(ContainerUnit::Cqw, Num(100.0 / aspect)),
            ))
            .transform(vec![Transform::Translate(
                Length::percent(-50.0),
                Length::percent(-50.0),
                Length::ZERO,
            )]))
        .child(div(style).children(inner)),
    )
    .into())
}

fn letterboxed(height: f64, width: f64, child: Node) -> Node {
    div(absolute_fill()
        .align_items(Align::Center)
        .justify_content(Align::Center)
        .background_color(css_color(STAGE_COLOR)))
    .child(
        div(Style::default()
            .position(Position::Relative)
            .width(Length::px(width))
            .height(Length::px(height)))
        .child(child),
    )
    .into()
}

#[expect(
    clippy::suboptimal_flops,
    reason = "the film page's JavaScript rounds every step, and the pictures have to match it"
)]
fn cropped(box_size: Size, cx: f64, stage: Size, child: Node) -> Node {
    div(absolute_fill().overflow(Overflow::Hidden))
        .child(
            div(Style::default()
                .position(Position::Absolute)
                .left(Length::px(js_clamp(
                    stage.width / 2.0 - cx * box_size.width,
                    stage.width - box_size.width,
                    0.0,
                )))
                .top(Length::px((stage.height - box_size.height) / 2.0))
                .width(Length::px(box_size.width))
                .height(Length::px(box_size.height)))
            .child(child),
        )
        .into()
}

pub fn recording(
    props: &RecordingProps,
    at: &At,
    reframe: ReframeMode,
    measure: &dyn Measure,
) -> Result<Option<Node>, DrawFailure> {
    if !has_screen(props) {
        return Ok(None);
    }
    let stage = Size {
        width: at.stage.width,
        height: at.stage.height,
    };
    let (frame, fps) = (at.clock.frame, f64::from(at.clock.fps));
    let log = to_click_log(
        &props.log,
        &props.name,
        &props.extra_beats,
        props.camera.zoom,
    );
    let t = log_time_s(frame, fps, props.speed);
    let viewport = Size {
        width: f64::from(props.log.viewport.width),
        height: f64::from(props.log.viewport.height),
    };
    let tuning = Tuning {
        camera: props.camera.tuning.clone(),
        narrow: None,
    };
    if props.shot.in_device.unwrap_or(false) {
        let tuning = Tuning::default();
        let track = build_camera_track(&log, props.speed, &tuning);
        let clip = Clip {
            props,
            measure,
            log,
            track,
            tuning,
            title: None,
            box_size: None,
            stage,
            marks: resolve_annotations(&props.log, &props.annotations),
        };
        return Ok(Some(screen_clip(&clip, at, design_scale_for(stage.width))?));
    }
    let full_page = !matches!(shot_of(props.shot.style), Shot::Window { .. });
    let layout = reframe_layout(viewport, stage, reframe, full_page);
    let (box_size, narrow) = match layout {
        Reframe::Contain { box_size, narrow } => (Some(box_size), narrow),
        _ => (None, None),
    };
    let tuning = Tuning { narrow, ..tuning };
    let ms = t * 1000.0;
    let title = match &props.screen {
        ScreenProps::Terminal { .. } => {
            terminal_title(&props.log, ms, props.window.text.as_deref())
        }
        ScreenProps::Footage { .. } => frame_address(
            &props.log,
            ms,
            props.window.text.as_deref(),
            props.window.mode,
        ),
    };
    let track = build_camera_track(&log, props.speed, &tuning);
    let clip = Clip {
        props,
        measure,
        log,
        track,
        tuning,
        title,
        box_size,
        stage,
        marks: resolve_annotations(&props.log, &props.annotations),
    };
    let node = match layout {
        Reframe::Same => demo_clip(&clip, at, design_scale_for(stage.width))?,
        Reframe::Contain { box_size, .. } => {
            demo_clip(&clip, at, design_scale_for(box_size.width))?
        }
        Reframe::Fit { height } => letterboxed(
            height,
            stage.width,
            demo_clip(&clip, at, design_scale_for(stage.width))?,
        ),
        Reframe::Cover { box_size, follow } => {
            let cx = if follow {
                pose_at(&clip.track, frame, fps, props.speed, &clip.tuning).cx
            } else {
                0.5
            };
            cropped(
                box_size,
                cx,
                stage,
                demo_clip(&clip, at, design_scale_for(box_size.width))?,
            )
        }
    };
    let mut out = div(absolute_fill()).child(node);
    if let Some(look) = &props.keystrokes {
        let k = design_scale_for(stage.width.max(stage.height));
        let at_stage = opsis_scene::timeline::Stage {
            width: stage.width,
            height: stage.height,
        };
        if look.chords {
            let keys = shown_keys(&clip.log.keys, &look.left_out);
            if !keys.is_empty()
                && let Some(hud) = keycap_hud(&keys, t, look, k, at_stage)
            {
                out = out.child(hud);
            }
        }
        if look.typed {
            let beats = typed_beats(&props.log, look.typed_hold_s.unwrap_or(TYPED_LINGER_S));
            if let Some(chip) = typed_chip(&beats, t, look, at_stage)? {
                out = out.child(chip);
            }
        }
    }
    Ok(Some(out.into()))
}
