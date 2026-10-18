import type { Page } from "playwright-core";
import { clamp01 } from "../../shared/math";
import { pointRect, rectCentre, rectFromBounds, type Rect } from "../../shared/schema/actions";
import { cameraEase } from "../../video/camera/lib/camera";
import { designScaleFor } from "../../video/camera/lib/design-width";
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
  type Point,
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

const DEFAULT_PAUSE_MS = 700;
const SAMPLE_MS = 8;
const GLIDE_MIN_PX = 6;
const SETTLE_MS = 220;
const MOVE_SETTLE_MS = 120;
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

type Box = { cx: number; cy: number; rect: Rect };

type DragPlan = { points: Point[]; end: Point; drop: Rect };

type Aim = { at: Point; rect: Rect };

type Viewport = { width: number; height: number } | null;

const viewportScale = (vp: Viewport): number => (vp ? designScaleFor(vp.width) : 1);

const centreOf = (vp: Viewport): Point =>
  vp ? { x: vp.width / 2, y: vp.height / 2 } : { x: 0, y: 0 };

const roundRect = (r: Rect): Rect => ({
  x: Math.round(r.x),
  y: Math.round(r.y),
  w: Math.round(r.w),
  h: Math.round(r.h),
});

function padSmallTarget(rect: Rect, k: number): Rect {
  if (rect.h >= MIN_TARGET_H * k) return rect;
  const height = GROWN_TARGET_H * k;
  const grow = (height - rect.h) / 2;
  return {
    x: Math.max(0, rect.x - TARGET_PAD_X * k),
    y: Math.max(0, rect.y - grow - TARGET_LIFT * k),
    w: rect.w + 2 * TARGET_PAD_X * k,
    h: height,
  };
}

async function boxOf(page: Page, target: Target, k: number): Promise<Box> {
  if (isPoint(target)) {
    return {
      cx: target.x,
      cy: target.y,
      rect: pointRect(target, k),
    };
  }
  const loc = target.first();
  await loc.waitFor({ state: "visible", timeout: VISIBLE_TIMEOUT_MS });
  const before = await loc.boundingBox();
  await loc.scrollIntoViewIfNeeded();
  const box = await loc.boundingBox();
  if (!box) throw new Error("Couldn't see the element on the page.");
  if (before && Math.hypot(box.x - before.x, box.y - before.y) > SCROLL_MOVED_PX * k)
    await page.waitForTimeout(SCROLL_SETTLE_MS);
  const rect = rectFromBounds(box);
  const centre = rectCentre(rect);
  return { cx: centre.x, cy: centre.y, rect: padSmallTarget(rect, k) };
}

const NOT_ON_SCREEN =
  "The place to drop it isn't on screen. Scroll to it first, or use a bigger window.";

async function dropBox(page: Page, target: Target, k: number, vp: Viewport): Promise<Box> {
  if (isPoint(target)) return boxOf(page, target, k);
  const box = await target.first().boundingBox();
  if (!box) throw new Error(NOT_ON_SCREEN);
  const rect = rectFromBounds(box);
  const { x: cx, y: cy } = rectCentre(rect);
  if (vp && (cx < 0 || cy < 0 || cx > vp.width || cy > vp.height)) throw new Error(NOT_ON_SCREEN);
  return { cx, cy, rect };
}

function ontoPlan(origin: Point, drop: Box): DragPlan {
  const end = { x: drop.cx, y: drop.cy };
  return { points: [origin, end], end, drop: drop.rect };
}

export class CursorDriver {
  private readonly viewport: Viewport;
  private readonly k: number;
  private last: Point;
  private lastDepartMs = 0;

  constructor(
    private readonly page: Page,
    private readonly elapsed: () => number,
    private readonly sink: TakeSink,
  ) {
    this.viewport = page.viewportSize();
    this.k = viewportScale(this.viewport);
    this.last = centreOf(this.viewport);
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

  async moveTo(target: Target): Promise<void> {
    const { cx, cy } = await boxOf(this.page, target, this.k);
    await this.glideTo(cx, cy, MOVE_SETTLE_MS);
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

  private mark(x: number, y: number) {
    this.sink.cursor({ t: this.elapsed(), x: Math.round(x), y: Math.round(y) });
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
    const { k } = this;
    const dist = Math.hypot(x - this.last.x, y - this.last.y);
    this.lastDepartMs = this.elapsed();
    if (dist >= GLIDE_MIN_PX * k) {
      const glide = glidePlan(this.last, { x, y }, k, this.viewport, travelMs);
      await this.walk(this.elapsed(), glide.durationMs, glide.at);
      if (glide.overshooting) {
        const { aim } = glide;
        await this.walk(this.elapsed(), OVERSHOOT_RETURN_MS, (p) => ({
          x: aim.x + (x - aim.x) * p,
          y: aim.y + (y - aim.y) * p,
        }));
      }
    }
    this.park(x, y);
    if (settleMs > 0) await this.pause(settleMs);
  }

  private async approach(target: Target, opts: ActionOpts | undefined): Promise<Aim> {
    const point = await boxOf(this.page, target, this.k);
    const frame = opts?.frame ? await boxOf(this.page, opts.frame, this.k) : point;
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
      ...extra,
    };
    if (opts?.selector != null) event.selector = opts.selector;
    if (opts?.cluster != null) event.cluster = opts.cluster;
    if (opts?.zoom === false) event.zoom = false;
    if (opts?.zoomScale != null) event.zoomScale = opts.zoomScale;
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
    return this.logEvent(aim, label, opts, { tDownMs, ...extra });
  }

  private async dragPlan(
    from: Target,
    to: DragTo,
    origin: Point,
    opts?: DragOpts,
  ): Promise<DragPlan> {
    const quick = opts?.method === "quick";
    const dragMs = opts?.dragMs;
    if ("target" in to) {
      const onto = to.target;
      return quick
        ? this.quickOnto(from, onto, origin)
        : this.pressed(opts, () => this.smoothOnto(onto, origin, dragMs));
    }
    const points = offsetPath(origin, "by" in to ? [to.by] : to.path);
    await this.pressed(opts, () =>
      quick ? this.quickAlong(points) : this.smoothAlong(points, origin, dragMs),
    );
    return { points, end: lastPoint(points), drop: boundsOf(points) };
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
    const nudge = nudgeToward(origin, toward, DRAG_NUDGE_PX * this.k);
    this.mark(nudge.x, nudge.y);
    await this.page.mouse.move(nudge.x, nudge.y);
    return nudge;
  }

  private async quickOnto(from: Target, onto: Target, origin: Point): Promise<DragPlan> {
    const plan = ontoPlan(origin, await dropBox(this.page, onto, this.k, this.viewport));
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
    const before = await dropBox(this.page, onto, this.k, this.viewport);
    const nudge = await this.nudgeFrom(origin, { x: before.cx, y: before.cy });
    const plan = ontoPlan(origin, await dropBox(this.page, onto, this.k, this.viewport));
    await this.walk(
      this.elapsed(),
      dragDuration([nudge, plan.end], this.k, dragMs),
      bulgedCurve(nudge, plan.end),
    );
    return plan;
  }

  private async quickAlong(points: Point[]): Promise<void> {
    for (const point of points.slice(1))
      await this.page.mouse.move(point.x, point.y, { steps: QUICK_DRAG_STEPS });
    const end = lastPoint(points);
    this.mark(end.x, end.y);
  }

  private async smoothAlong(
    points: Point[],
    origin: Point,
    dragMs: number | undefined,
  ): Promise<void> {
    const nudge = await this.nudgeFrom(origin, points[1] ?? origin);
    await this.walk(
      this.elapsed(),
      dragDuration(points, this.k, dragMs),
      polylineAt([nudge, ...points.slice(1)]),
    );
  }
}
