use std::sync::Arc;
use std::time::Duration;

use opsis_compile::recording::point_rect;
use opsis_types::actions::{CursorSample, Point, Rect, Viewport};
use serde_json::json;
use tokio::sync::Mutex;

use crate::cdp::{CaptureError, Result, Tab};
use crate::drag_path::{
    Curve, bounds_of, bulged_curve, drag_duration, glide_plan, nudge_toward, offset_path,
    polyline_at, thin_path,
};
use crate::ease::camera_ease;
use crate::input::Input;
use crate::page_scripts::{CURSOR_UNDER, CursorUnder, DEFAULT_CURSOR, call, cursor_keyword};
use crate::selectors::Target;

const DEFAULT_PAUSE_MS: f64 = 700.0;
const SAMPLE_MS: u64 = 8;
const GLIDE_MIN_PX: f64 = 6.0;
const SETTLE_MS: f64 = 220.0;
const OVERSHOOT_RETURN_MS: f64 = 130.0;
const CLICK_HOLD_MS: f64 = 80.0;
const FIELD_FOCUS_MS: f64 = 150.0;
const KEY_DELAY_MS: u64 = 30;
const DRAG_HOLD_MS: f64 = 150.0;
const DRAG_NUDGE_PX: f64 = 6.0;
const DROP_SETTLE_MS: f64 = 120.0;
const QUICK_DRAG_STEPS: u32 = 20;
const VISIBLE_TIMEOUT: Duration = Duration::from_secs(20);
const SCROLL_MOVED_PX: f64 = 8.0;
const SCROLL_SETTLE_MS: f64 = 450.0;
const MIN_TARGET_H: f64 = 48.0;
const GROWN_TARGET_H: f64 = 72.0;
const TARGET_PAD_X: f64 = 24.0;
const TARGET_LIFT: f64 = 8.0;
const DESIGN_WIDTH: f64 = 1920.0;
const NOT_ON_SCREEN: &str =
    "The place to drop it isn't on screen. Scroll to it first, or use a bigger window.";

#[derive(Debug, Clone, Default)]
pub struct TakeClick {
    pub label: Option<String>,
    pub t_ms: f64,
    pub t_depart_ms: Option<f64>,
    pub t_down_ms: Option<f64>,
    pub type_end_ms: Option<f64>,
    pub x: f64,
    pub y: f64,
    pub rect: Option<Rect>,
    pub cluster: Option<String>,
    pub zoom: Option<bool>,
    pub zoom_scale: Option<f64>,
    pub t_up_ms: Option<f64>,
    pub path: Option<Vec<Point>>,
    pub drop: Option<Rect>,
    pub selector: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub t_ms: f64,
    pub chord: String,
    pub label: Option<String>,
}

pub trait TakeSink: Send + Sync {
    fn aim(&self, label: &str, at: Point);
    fn click(&self, event: TakeClick) -> usize;
    fn type_end(&self, click: usize, t_ms: f64);
    fn cursor(&self, sample: CursorSample);
    fn key(&self, event: KeyEvent);
}

#[derive(Debug, Clone, Default)]
pub struct ActionOpts {
    pub selector: Option<String>,
    pub cluster: Option<String>,
    pub zoom: bool,
    pub frame: Option<Target>,
    pub chunk_pause_ms: f64,
    pub zoom_scale: Option<f64>,
    pub travel_ms: Option<f64>,
    pub hover_ms: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragMethod {
    Smooth,
    Quick,
}

#[derive(Debug, Clone)]
pub struct DragOpts {
    pub action: ActionOpts,
    pub hold_ms: Option<f64>,
    pub drag_ms: f64,
    pub method: DragMethod,
}

pub enum DragTo {
    Target(Target),
    By(Point),
    Path(Vec<Point>),
}

struct TargetBox {
    cx: f64,
    cy: f64,
    rect: Rect,
}

struct Aim {
    at: Point,
    rect: Rect,
}

struct DragPlan {
    points: Vec<Point>,
    end: Point,
    drop: Rect,
}

fn round_rect(r: Rect) -> Rect {
    Rect {
        x: r.x.round(),
        y: r.y.round(),
        w: r.w.round(),
        h: r.h.round(),
    }
}

fn pad_small_target(rect: Rect, design_scale: f64) -> Rect {
    if rect.h >= MIN_TARGET_H * design_scale {
        return rect;
    }
    let height = GROWN_TARGET_H * design_scale;
    let grow = (height - rect.h) / 2.0;
    Rect {
        x: (rect.x - TARGET_PAD_X * design_scale).max(0.0),
        y: (rect.y - grow - TARGET_LIFT * design_scale).max(0.0),
        w: rect.w + 2.0 * TARGET_PAD_X * design_scale,
        h: height,
    }
}

pub async fn pause(ms: f64) {
    tokio::time::sleep(Duration::from_secs_f64(ms.max(0.0) / 1000.0)).await;
}

fn onto_plan(origin: Point, drop: &TargetBox) -> DragPlan {
    let end = Point {
        x: drop.cx,
        y: drop.cy,
    };
    DragPlan {
        points: vec![origin, end],
        end,
        drop: drop.rect,
    }
}

struct CursorState {
    last: Point,
    last_depart_ms: f64,
    shown: String,
}

pub struct CursorDriver {
    tab: Arc<Tab>,
    input: Arc<Input>,
    elapsed: Arc<dyn Fn() -> f64 + Send + Sync>,
    sink: Arc<dyn TakeSink>,
    reads_shapes: bool,
    viewport: Viewport,
    design_scale: f64,
    state: Mutex<CursorState>,
}

impl CursorDriver {
    pub async fn new(
        tab: Arc<Tab>,
        input: Arc<Input>,
        elapsed: Arc<dyn Fn() -> f64 + Send + Sync>,
        sink: Arc<dyn TakeSink>,
        reads_shapes: bool,
        viewport: Viewport,
    ) -> Self {
        let center = Point {
            x: f64::from(viewport.width) / 2.0,
            y: f64::from(viewport.height) / 2.0,
        };
        let driver = Self {
            tab,
            input,
            elapsed,
            sink,
            reads_shapes,
            viewport,
            design_scale: f64::from(viewport.width) / DESIGN_WIDTH,
            state: Mutex::new(CursorState {
                last: center,
                last_depart_ms: 0.0,
                shown: DEFAULT_CURSOR.to_owned(),
            }),
        };
        driver.mark(center, None).await;
        driver
    }

    fn now(&self) -> f64 {
        (self.elapsed)()
    }

    pub async fn pause(&self, ms: Option<f64>) {
        pause(ms.unwrap_or(DEFAULT_PAUSE_MS)).await;
    }

    pub async fn press_key(&self, chord: &str, label: Option<&str>) -> Result<()> {
        let t_ms = self.now();
        self.input.press(&self.tab, chord, None).await?;
        self.sink.key(KeyEvent {
            t_ms,
            chord: chord.to_owned(),
            label: label.map(str::to_owned),
        });
        Ok(())
    }

    pub async fn focus(
        &self,
        target: &Target,
        label: Option<&str>,
        opts: &ActionOpts,
    ) -> Result<()> {
        let label = label.unwrap_or("focus");
        let aim = self.approach(target, label, opts).await?;
        self.log_event(&aim, Some(label), opts, TakeClick::default(), None)
            .await;
        Ok(())
    }

    pub async fn move_and_click(
        &self,
        target: &Target,
        label: Option<&str>,
        opts: &ActionOpts,
    ) -> Result<()> {
        let aim = self
            .approach(target, label.unwrap_or("click"), opts)
            .await?;
        self.click_at(&aim, label, opts, TakeClick::default())
            .await?;
        Ok(())
    }

    pub async fn type_into(
        &self,
        target: &Target,
        chunks: &[String],
        label: Option<&str>,
        opts: &ActionOpts,
    ) -> Result<()> {
        let label = label.unwrap_or("field");
        let aim = self.approach(target, label, opts).await?;
        let event = self
            .click_at(
                &aim,
                Some(label),
                opts,
                TakeClick {
                    text: Some(chunks.concat()),
                    ..TakeClick::default()
                },
            )
            .await?;
        pause(FIELD_FOCUS_MS).await;
        let select_all = if cfg!(target_os = "macos") {
            "Meta+A"
        } else {
            "Control+A"
        };
        self.input.press(&self.tab, select_all, None).await?;
        self.input.press(&self.tab, "Backspace", None).await?;
        for (i, chunk) in chunks.iter().enumerate() {
            if i > 0 {
                pause(opts.chunk_pause_ms).await;
            }
            self.input
                .type_text(&self.tab, chunk, Some(Duration::from_millis(KEY_DELAY_MS)))
                .await?;
        }
        self.sink.type_end(event, self.now());
        Ok(())
    }

    pub async fn drag(
        &self,
        from: &Target,
        to: &DragTo,
        label: Option<&str>,
        opts: &DragOpts,
    ) -> Result<()> {
        let label = label.unwrap_or("drag");
        let Aim { at: origin, rect } = self.approach(from, label, &opts.action).await?;
        let t_down_ms = self.now();
        let plan = self.drag_plan(from, to, origin, opts).await?;
        self.park(plan.end).await;
        let t_ms = self.now();
        let extra = TakeClick {
            t_down_ms: Some(t_down_ms),
            t_up_ms: Some(t_ms),
            path: Some(thin_path(&plan.points)),
            drop: Some(round_rect(plan.drop)),
            ..TakeClick::default()
        };
        self.log_event(
            &Aim { at: plan.end, rect },
            Some(label),
            &opts.action,
            extra,
            Some(t_ms),
        )
        .await;
        Ok(())
    }

    async fn mark(&self, at: Point, shape: Option<&str>) {
        let mut sample = CursorSample {
            t: self.now(),
            x: at.x.round(),
            y: at.y.round(),
            c: None,
        };
        if let Some(shape) = shape {
            let mut state = self.state.lock().await;
            if shape != state.shown {
                sample.c = Some(shape.to_owned());
                state.shown = shape.to_owned();
            }
        }
        self.sink.cursor(sample);
    }

    async fn read_shape(&self) {
        if !self.reads_shapes {
            return;
        }
        let last = self.state.lock().await.last;
        let under = self
            .tab
            .evaluate_main::<CursorUnder>(&call(CURSOR_UNDER, &json!({ "x": last.x, "y": last.y })))
            .await;
        if let Ok(under) = under {
            self.mark(last, Some(&cursor_keyword(&under))).await;
        }
    }

    async fn park(&self, at: Point) {
        self.state.lock().await.last = at;
        self.mark(at, None).await;
    }

    async fn walk(&self, from_ms: f64, span_ms: f64, at: &Curve) -> Result<()> {
        loop {
            let p01 = ((self.now() - from_ms) / span_ms).clamp(0.0, 1.0);
            let point = at(camera_ease(p01));
            self.mark(point, None).await;
            self.input
                .mouse_move(&self.tab, point.x, point.y, 1)
                .await?;
            if p01 >= 1.0 {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(SAMPLE_MS)).await;
        }
    }

    async fn glide_to(&self, to: Point, settle_ms: f64, travel_ms: Option<f64>) -> Result<()> {
        let from = {
            let mut state = self.state.lock().await;
            state.last_depart_ms = self.now();
            state.last
        };
        if from.distance(to) >= GLIDE_MIN_PX * self.design_scale {
            if self.reads_shapes {
                self.mark(from, Some(DEFAULT_CURSOR)).await;
            }
            let glide = glide_plan(from, to, self.design_scale, Some(self.viewport), travel_ms);
            self.walk(self.now(), glide.duration_ms, &glide.at).await?;
            if glide.overshooting {
                let aim = glide.aim;
                let back: Curve = Box::new(move |p| aim.lerp(to, p));
                self.walk(self.now(), OVERSHOOT_RETURN_MS, &back).await?;
            }
        }
        self.park(to).await;
        self.read_shape().await;
        if settle_ms > 0.0 {
            pause(settle_ms).await;
        }
        Ok(())
    }

    async fn box_of(&self, target: &Target) -> Result<TargetBox> {
        let selector = match target {
            Target::Point(point) => {
                return Ok(TargetBox {
                    cx: point.x,
                    cy: point.y,
                    rect: point_rect(*point, self.design_scale),
                });
            }
            Target::Element(selector) => selector,
        };
        let handle = self.tab.wait_visible(selector, VISIBLE_TIMEOUT).await?;
        let measured = async {
            let before = self.tab.bounding_box(&handle).await?;
            self.tab.scroll_into_view_if_needed(&handle).await?;
            let found = self
                .tab
                .bounding_box(&handle)
                .await?
                .ok_or_else(|| CaptureError::failed("Couldn't see the element on the page."))?;
            if let Some(before) = before
                && (Point {
                    x: before.x,
                    y: before.y,
                })
                .distance(Point {
                    x: found.x,
                    y: found.y,
                }) > SCROLL_MOVED_PX * self.design_scale
            {
                pause(SCROLL_SETTLE_MS).await;
            }
            Ok::<_, CaptureError>(found)
        }
        .await;
        self.tab.release(&handle).await;
        let rect = measured?;
        let center = rect.center();
        Ok(TargetBox {
            cx: center.x,
            cy: center.y,
            rect: pad_small_target(rect, self.design_scale),
        })
    }

    async fn drop_box(&self, target: &Target) -> Result<TargetBox> {
        let selector = match target {
            Target::Point(_) => return self.box_of(target).await,
            Target::Element(selector) => selector,
        };
        let handle = self
            .tab
            .query(selector)
            .await?
            .ok_or_else(|| CaptureError::failed(NOT_ON_SCREEN))?;
        let measured = self.tab.bounding_box(&handle).await;
        self.tab.release(&handle).await;
        let rect = measured?.ok_or_else(|| CaptureError::failed(NOT_ON_SCREEN))?;
        let Point { x: cx, y: cy } = rect.center();
        let (width, height) = (
            f64::from(self.viewport.width),
            f64::from(self.viewport.height),
        );
        if cx < 0.0 || cy < 0.0 || cx > width || cy > height {
            return Err(CaptureError::failed(NOT_ON_SCREEN));
        }
        Ok(TargetBox { cx, cy, rect })
    }

    async fn approach(&self, target: &Target, label: &str, opts: &ActionOpts) -> Result<Aim> {
        let point = self.box_of(target).await?;
        let frame = match &opts.frame {
            Some(frame) => self.box_of(frame).await?,
            None => TargetBox {
                cx: point.cx,
                cy: point.cy,
                rect: point.rect,
            },
        };
        let at = Point {
            x: point.cx,
            y: point.cy,
        };
        self.sink.aim(label, at);
        self.glide_to(at, opts.hover_ms.unwrap_or(SETTLE_MS), opts.travel_ms)
            .await?;
        Ok(Aim {
            at,
            rect: frame.rect,
        })
    }

    async fn log_event(
        &self,
        aim: &Aim,
        label: Option<&str>,
        opts: &ActionOpts,
        extra: TakeClick,
        at_ms: Option<f64>,
    ) -> usize {
        let t_depart_ms = self.state.lock().await.last_depart_ms;
        let t_ms = at_ms.unwrap_or_else(|| self.now());
        self.sink.click(TakeClick {
            label: label.map(str::to_owned),
            t_ms,
            t_depart_ms: Some(t_depart_ms),
            x: aim.at.x.round(),
            y: aim.at.y.round(),
            rect: Some(round_rect(aim.rect)),
            selector: opts.selector.clone(),
            cluster: opts.cluster.clone(),
            zoom: (!opts.zoom).then_some(false),
            zoom_scale: opts.zoom_scale,
            ..extra
        })
    }

    async fn click_at(
        &self,
        aim: &Aim,
        label: Option<&str>,
        opts: &ActionOpts,
        extra: TakeClick,
    ) -> Result<usize> {
        let t_down_ms = self.now();
        self.input.mouse_down(&self.tab).await?;
        pause(CLICK_HOLD_MS).await;
        self.input.mouse_up(&self.tab).await?;
        let event = self
            .log_event(
                aim,
                label,
                opts,
                TakeClick {
                    t_down_ms: Some(t_down_ms),
                    ..extra
                },
                None,
            )
            .await;
        self.read_shape().await;
        Ok(event)
    }

    async fn drag_plan(
        &self,
        from: &Target,
        to: &DragTo,
        origin: Point,
        opts: &DragOpts,
    ) -> Result<DragPlan> {
        let offsets = match to {
            DragTo::Target(onto) => return self.drag_onto(from, onto, origin, opts).await,
            DragTo::By(by) => vec![*by],
            DragTo::Path(path) => path.clone(),
        };
        let points = offset_path(origin, &offsets);
        self.pressed(opts, async {
            if opts.method == DragMethod::Quick {
                self.quick_along(&points).await
            } else {
                self.smooth_along(&points, origin, opts.drag_ms).await
            }
        })
        .await?;
        let end = points[points.len() - 1];
        Ok(DragPlan {
            drop: bounds_of(&points),
            end,
            points,
        })
    }

    async fn drag_onto(
        &self,
        from: &Target,
        onto: &Target,
        origin: Point,
        opts: &DragOpts,
    ) -> Result<DragPlan> {
        if opts.method == DragMethod::Quick {
            return self.quick_onto(from, onto, origin).await;
        }
        self.pressed(opts, self.smooth_onto(onto, origin, opts.drag_ms))
            .await
    }

    async fn pressed<T>(
        &self,
        opts: &DragOpts,
        moving: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        self.input.mouse_down(&self.tab).await?;
        pause(opts.hold_ms.unwrap_or(DRAG_HOLD_MS)).await;
        let moved = moving.await?;
        pause(DROP_SETTLE_MS).await;
        self.input.mouse_up(&self.tab).await?;
        Ok(moved)
    }

    async fn nudge_from(&self, origin: Point, toward: Point) -> Result<Point> {
        let nudge = nudge_toward(origin, toward, DRAG_NUDGE_PX * self.design_scale);
        self.mark(nudge, None).await;
        self.input
            .mouse_move(&self.tab, nudge.x, nudge.y, 1)
            .await?;
        Ok(nudge)
    }

    async fn quick_onto(&self, from: &Target, onto: &Target, origin: Point) -> Result<DragPlan> {
        let plan = onto_plan(origin, &self.drop_box(onto).await?);
        let end = plan.end;
        if let (Target::Element(_), Target::Element(_)) = (from, onto) {
            let source = self.box_of(from).await?;
            let target = self.box_of(onto).await?;
            self.input
                .mouse_move(&self.tab, source.cx, source.cy, 1)
                .await?;
            self.input.mouse_down(&self.tab).await?;
            self.input
                .mouse_move(&self.tab, target.cx, target.cy, QUICK_DRAG_STEPS)
                .await?;
            self.input.mouse_up(&self.tab).await?;
        } else {
            self.input.mouse_down(&self.tab).await?;
            self.input
                .mouse_move(&self.tab, end.x, end.y, QUICK_DRAG_STEPS)
                .await?;
            self.input.mouse_up(&self.tab).await?;
        }
        self.mark(end, None).await;
        Ok(plan)
    }

    async fn smooth_onto(&self, onto: &Target, origin: Point, drag_ms: f64) -> Result<DragPlan> {
        let before = self.drop_box(onto).await?;
        let nudge = self
            .nudge_from(
                origin,
                Point {
                    x: before.cx,
                    y: before.cy,
                },
            )
            .await?;
        let plan = onto_plan(origin, &self.drop_box(onto).await?);
        let curve = bulged_curve(nudge, plan.end, plan.end);
        self.walk(
            self.now(),
            drag_duration(&[nudge, plan.end], self.design_scale, drag_ms),
            &curve,
        )
        .await?;
        Ok(plan)
    }

    async fn quick_along(&self, points: &[Point]) -> Result<()> {
        for point in &points[1..] {
            self.input
                .mouse_move(&self.tab, point.x, point.y, QUICK_DRAG_STEPS)
                .await?;
        }
        self.mark(points[points.len() - 1], None).await;
        Ok(())
    }

    async fn smooth_along(&self, points: &[Point], origin: Point, drag_ms: f64) -> Result<()> {
        let nudge = self
            .nudge_from(origin, points.get(1).copied().unwrap_or(origin))
            .await?;
        let walked: Vec<Point> = std::iter::once(nudge)
            .chain(points[1..].iter().copied())
            .collect();
        let duration = drag_duration(points, self.design_scale, drag_ms);
        self.walk(self.now(), duration, &polyline_at(walked)).await
    }
}
