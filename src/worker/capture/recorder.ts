import { clamp01 } from "@opsis/core/math";
import {
  distance,
  lerpPoint,
  pointRect,
  rectCenter,
  rectFromBounds,
  type Point,
  type Rect,
  type Viewport,
} from "@opsis/core/schema/actions";
import { cameraEase, designScaleFor, type CursorSample } from "@opsis/video/capture";
import type { Page } from "playwright-core";
import { cursorKeyword, DEFAULT_CURSOR } from "./cursor-keyword";
import {
  boundsOf,
  bulgedCurve,
  dragDuration,
  glidePlan,
  lastPoint,
  nudgeToward,
  offsetPath,
  polylineAt,
  thinPath,
} from "./drag-path";
import {
  isPoint,
  type ActionOpts,
  type DragOpts,
  type DragTo,
  type Target,
  type TakeClick,
  type TakeSink,
} from "./flow-types";
import { cursorUnder } from "./page-scripts";

const DEFAULT_PAUSE_MS = 700;
const SAMPLE_MS = 8;
const GLIDE_MIN_PX = 6;
const SETTLE_MS = 220;
const OVERSHOOT_RETURN_MS = 130;
const CLICK_HOLD_MS = 80;
const FIELD_FOCUS_MS = 150;
const KEY_DELAY_MS = 30;
const DRAG_HOLD_MS = 150;
const DRAG_NUDGE_PX = 6;
const DROP_SETTLE_MS = 120;
const QUICK_DRAG_STEPS = 20;
const VISIBLE_TIMEOUT_MS = 20_000;
const SCROLL_MOVED_PX = 8;
const SCROLL_SETTLE_MS = 450;
const MIN_TARGET_H = 48;
const GROWN_TARGET_H = 72;
const TARGET_PAD_X = 24;
const TARGET_LIFT = 8;

type TargetBox = { cx: number; cy: number; rect: Rect };

type Path = [Point, ...Point[]];

type DragPlan = { points: Point[]; end: Point; drop: Rect };

type Aim = { at: Point; rect: Rect };

const viewportScale = (vp: Viewport | null): number => (vp ? designScaleFor(vp.width) : 1);

const centerOf = (vp: Viewport | null): Point =>
  vp ? { x: vp.width / 2, y: vp.height / 2 } : { x: 0, y: 0 };

const roundRect = (r: Rect): Rect => ({
  x: Math.round(r.x),
  y: Math.round(r.y),
  w: Math.round(r.w),
  h: Math.round(r.h),
});

function padSmallTarget(rect: Rect, designScale: number): Rect {
  if (rect.h >= MIN_TARGET_H * designScale) return rect;
  const height = GROWN_TARGET_H * designScale;
  const grow = (height - rect.h) / 2;
  return {
    x: Math.max(0, rect.x - TARGET_PAD_X * designScale),
    y: Math.max(0, rect.y - grow - TARGET_LIFT * designScale),
    w: rect.w + 2 * TARGET_PAD_X * designScale,
    h: height,
  };
}

async function boxOf(page: Page, target: Target, designScale: number): Promise<TargetBox> {
  if (isPoint(target)) {
    return {
      cx: target.x,
      cy: target.y,
      rect: pointRect(target, designScale),
    };
  }
  const loc = target.first();
  await loc.waitFor({ state: "visible", timeout: VISIBLE_TIMEOUT_MS });
  const before = await loc.boundingBox();
  await loc.scrollIntoViewIfNeeded();
  const box = await loc.boundingBox();
  if (!box) throw new Error("Couldn't see the element on the page.");
  if (before && distance(before, box) > SCROLL_MOVED_PX * designScale)
    await page.waitForTimeout(SCROLL_SETTLE_MS);
  const rect = rectFromBounds(box);
  const center = rectCenter(rect);
  return { cx: center.x, cy: center.y, rect: padSmallTarget(rect, designScale) };
}

const NOT_ON_SCREEN =
  "The place to drop it isn't on screen. Scroll to it first, or use a bigger window.";

async function dropBox(
  page: Page,
  target: Target,
  designScale: number,
  vp: Viewport | null,
): Promise<TargetBox> {
  if (isPoint(target)) return boxOf(page, target, designScale);
  const box = await target.first().boundingBox();
  if (!box) throw new Error(NOT_ON_SCREEN);
  const rect = rectFromBounds(box);
  const { x: cx, y: cy } = rectCenter(rect);
  if (vp && (cx < 0 || cy < 0 || cx > vp.width || cy > vp.height)) throw new Error(NOT_ON_SCREEN);
  return { cx, cy, rect };
}

function ontoPlan(origin: Point, drop: TargetBox): DragPlan {
  const end = { x: drop.cx, y: drop.cy };
  return { points: [origin, end], end, drop: drop.rect };
}

export class CursorDriver {
  private readonly viewport: Viewport | null;
  private readonly designScale: number;
  private last: Point;
  private lastDepartMs = 0;
  private shown = DEFAULT_CURSOR;

  constructor(
    private readonly page: Page,
    private readonly elapsed: () => number,
    private readonly sink: TakeSink,
    private readonly readsShapes = false,
  ) {
    this.viewport = page.viewportSize();
    this.designScale = viewportScale(this.viewport);
    this.last = centerOf(this.viewport);
    this.mark(this.last.x, this.last.y);
  }

  pause(ms = DEFAULT_PAUSE_MS): Promise<void> {
    return this.page.waitForTimeout(ms);
  }

  async pressKey(chord: string, label?: string): Promise<void> {
    const tMs = this.elapsed();
    await this.page.keyboard.press(chord);
    this.sink.key(label ? { tMs, chord, label } : { tMs, chord });
  }

  async focus(target: Target, label?: string, opts?: ActionOpts): Promise<void> {
    const aim = await this.approach(target, opts);
    this.logEvent(aim, label ?? "focus", opts);
  }

  async moveAndClick(target: Target, label?: string, opts?: ActionOpts): Promise<void> {
    const aim = await this.approach(target, opts);
    await this.clickAt(aim, label, opts);
  }

  async typeInto(
    target: Target,
    chunks: readonly string[],
    label?: string,
    opts?: ActionOpts,
  ): Promise<void> {
    const aim = await this.approach(target, opts);
    const event = await this.clickAt(aim, label ?? "field", opts, { text: chunks.join("") });
    await this.pause(FIELD_FOCUS_MS);
    const selectAll = process.platform === "darwin" ? "Meta+A" : "Control+A";
    await this.page.keyboard.press(selectAll);
    await this.page.keyboard.press("Backspace");
    for (const [i, chunk] of chunks.entries()) {
      if (i > 0) await this.pause(opts?.chunkPauseMs ?? 0);
      await this.page.keyboard.type(chunk, { delay: KEY_DELAY_MS });
    }
    event.typeEndMs = this.elapsed();
  }

  async drag(from: Target, to: DragTo, label?: string, opts?: DragOpts): Promise<void> {
    const { at: origin, rect } = await this.approach(from, opts);
    const tDownMs = this.elapsed();
    const plan = await this.dragPlan(from, to, origin, opts);
    this.park(plan.end.x, plan.end.y);
    const tMs = this.elapsed();
    this.logEvent({ at: plan.end, rect }, label ?? "drag", opts, {
      tMs,
      tDownMs,
      tUpMs: tMs,
      path: thinPath(plan.points),
      drop: roundRect(plan.drop),
    });
  }

  private mark(x: number, y: number, shape?: string) {
    const sample: CursorSample = { t: this.elapsed(), x: Math.round(x), y: Math.round(y) };
    if (shape !== undefined && shape !== this.shown) {
      sample.c = shape;
      this.shown = shape;
    }
    this.sink.cursor(sample);
  }

  private async readShape() {
    if (!this.readsShapes) return;
    const under = await this.page.evaluate(cursorUnder, this.last).catch(() => null);
    if (under) this.mark(this.last.x, this.last.y, cursorKeyword(under));
  }

  private park(x: number, y: number) {
    this.last = { x, y };
    this.mark(x, y);
  }

  private async walk(fromMs: number, spanMs: number, at: (p: number) => Point) {
    for (;;) {
      const p01 = clamp01((this.elapsed() - fromMs) / spanMs);
      const { x: px, y: py } = at(cameraEase(p01));
      this.mark(px, py);
      await this.page.mouse.move(px, py);
      if (p01 >= 1) return;
      await this.page.waitForTimeout(SAMPLE_MS);
    }
  }

  private async glideTo(x: number, y: number, settleMs = SETTLE_MS, travelMs?: number) {
    const { designScale } = this;
    const dist = distance(this.last, { x, y });
    this.lastDepartMs = this.elapsed();
    if (dist >= GLIDE_MIN_PX * designScale) {
      if (this.readsShapes) this.mark(this.last.x, this.last.y, DEFAULT_CURSOR);
      const glide = glidePlan(this.last, { x, y }, designScale, this.viewport, travelMs);
      await this.walk(this.elapsed(), glide.durationMs, glide.at);
      if (glide.overshooting) {
        const { aim } = glide;
        await this.walk(this.elapsed(), OVERSHOOT_RETURN_MS, (p) => lerpPoint(aim, { x, y }, p));
      }
    }
    this.park(x, y);
    await this.readShape();
    if (settleMs > 0) await this.pause(settleMs);
  }

  private async approach(target: Target, opts: ActionOpts | undefined): Promise<Aim> {
    const point = await boxOf(this.page, target, this.designScale);
    const frame = opts?.frame ? await boxOf(this.page, opts.frame, this.designScale) : point;
    await this.glideTo(point.cx, point.cy, opts?.hoverMs ?? SETTLE_MS, opts?.travelMs);
    return { at: { x: point.cx, y: point.cy }, rect: frame.rect };
  }

  private logEvent(
    { at, rect }: Aim,
    label: string | undefined,
    opts: ActionOpts | undefined,
    extra: Partial<TakeClick> = {},
  ): TakeClick {
    const event: TakeClick = {
      label,
      tMs: this.elapsed(),
      tDepartMs: this.lastDepartMs,
      x: Math.round(at.x),
      y: Math.round(at.y),
      rect: roundRect(rect),
      selector: opts?.selector,
      cluster: opts?.cluster,
      zoom: opts?.zoom === false ? false : undefined,
      zoomScale: opts?.zoomScale,
      ...extra,
    };
    this.sink.click(event);
    return event;
  }

  private async clickAt(
    aim: Aim,
    label: string | undefined,
    opts: ActionOpts | undefined,
    extra: Partial<TakeClick> = {},
  ) {
    const tDownMs = this.elapsed();
    await this.page.mouse.down();
    await this.pause(CLICK_HOLD_MS);
    await this.page.mouse.up();
    const event = this.logEvent(aim, label, opts, { tDownMs, ...extra });
    await this.readShape();
    return event;
  }

  private async dragPlan(
    from: Target,
    to: DragTo,
    origin: Point,
    opts?: DragOpts,
  ): Promise<DragPlan> {
    if ("target" in to) return this.dragOnto(from, to.target, origin, opts);
    const points = offsetPath(origin, "by" in to ? [to.by] : to.path);
    await this.dragAlong(points, origin, opts);
    return { points, end: lastPoint(points), drop: boundsOf(points) };
  }

  private dragOnto(from: Target, onto: Target, origin: Point, opts?: DragOpts) {
    if (opts?.method === "quick") return this.quickOnto(from, onto, origin);
    return this.pressed(opts, () => this.smoothOnto(onto, origin, opts?.dragMs));
  }

  private dragAlong(points: Path, origin: Point, opts?: DragOpts) {
    return this.pressed(opts, () =>
      opts?.method === "quick"
        ? this.quickAlong(points)
        : this.smoothAlong(points, origin, opts?.dragMs),
    );
  }

  private async pressed<T>(opts: DragOpts | undefined, move: () => Promise<T>): Promise<T> {
    await this.page.mouse.down();
    await this.pause(opts?.holdMs ?? DRAG_HOLD_MS);
    const moved = await move();
    await this.pause(DROP_SETTLE_MS);
    await this.page.mouse.up();
    return moved;
  }

  private async nudgeFrom(origin: Point, toward: Point): Promise<Point> {
    const nudge = nudgeToward(origin, toward, DRAG_NUDGE_PX * this.designScale);
    this.mark(nudge.x, nudge.y);
    await this.page.mouse.move(nudge.x, nudge.y);
    return nudge;
  }

  private async quickOnto(from: Target, onto: Target, origin: Point): Promise<DragPlan> {
    const plan = ontoPlan(origin, await dropBox(this.page, onto, this.designScale, this.viewport));
    const { end } = plan;
    if (!isPoint(from) && !isPoint(onto))
      await from.first().dragTo(onto.first(), { steps: QUICK_DRAG_STEPS });
    else {
      await this.page.mouse.down();
      await this.page.mouse.move(end.x, end.y, { steps: QUICK_DRAG_STEPS });
      await this.page.mouse.up();
    }
    this.mark(end.x, end.y);
    return plan;
  }

  private async smoothOnto(
    onto: Target,
    origin: Point,
    dragMs: number | undefined,
  ): Promise<DragPlan> {
    const before = await dropBox(this.page, onto, this.designScale, this.viewport);
    const nudge = await this.nudgeFrom(origin, { x: before.cx, y: before.cy });
    const plan = ontoPlan(origin, await dropBox(this.page, onto, this.designScale, this.viewport));
    await this.walk(
      this.elapsed(),
      dragDuration([nudge, plan.end], this.designScale, dragMs),
      bulgedCurve(nudge, plan.end),
    );
    return plan;
  }

  private async quickAlong(points: Path): Promise<void> {
    for (const point of points.slice(1))
      await this.page.mouse.move(point.x, point.y, { steps: QUICK_DRAG_STEPS });
    const end = lastPoint(points);
    this.mark(end.x, end.y);
  }

  private async smoothAlong(
    points: Path,
    origin: Point,
    dragMs: number | undefined,
  ): Promise<void> {
    const nudge = await this.nudgeFrom(origin, points[1] ?? origin);
    await this.walk(
      this.elapsed(),
      dragDuration(points, this.designScale, dragMs),
      polylineAt([nudge, ...points.slice(1)]),
    );
  }
}
