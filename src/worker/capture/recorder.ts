import type { Page } from "playwright-core";
import { clamp } from "../../shared/math";
import { rectFromBounds, type Rect } from "../../shared/schema/actions";
import { cameraEase } from "../../video/camera/lib/camera";
import { DESIGN_WIDTH, type CursorSample, type KeyEvent } from "../../video/camera/lib/click-log";
import {
  boundsOf,
  bulgedCurve,
  dragDuration,
  glidePlan,
  nudgeToward,
  offsetPath,
  polylineAt,
  thinPath,
  type Point,
} from "./drag-path";
import {
  isPoint,
  type ActionOpts,
  type ClickEvent,
  type DragOpts,
  type DragTo,
  type Target,
} from "./flow-types";

const START = { x: 960, y: 540 };
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
const POINT_BOX = { w: 120, h: 48 };
const MIN_TARGET_H = 48;
const GROWN_TARGET_H = 72;
const TARGET_PAD_X = 24;
const TARGET_LIFT = 8;

type Box = { cx: number; cy: number; rect: Rect };

type DragPlan = { points: Point[]; drop: Rect };

const captureScaleOf = (page: Page): number => {
  const vp = page.viewportSize();
  return vp && vp.width > 0 ? vp.width / DESIGN_WIDTH : 1;
};

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

async function boxOf(page: Page, target: Target): Promise<Box> {
  const k = captureScaleOf(page);
  if (isPoint(target)) {
    return {
      cx: target.x,
      cy: target.y,
      rect: {
        x: target.x - (POINT_BOX.w / 2) * k,
        y: target.y - (POINT_BOX.h / 2) * k,
        w: POINT_BOX.w * k,
        h: POINT_BOX.h * k,
      },
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
  return {
    cx: box.x + box.width / 2,
    cy: box.y + box.height / 2,
    rect: padSmallTarget(rectFromBounds(box), k),
  };
}

async function dropBox(page: Page, target: Target): Promise<Box> {
  if (isPoint(target)) return boxOf(page, target);
  const box = await target.first().boundingBox();
  const vp = page.viewportSize();
  const cx = box ? box.x + box.width / 2 : 0;
  const cy = box ? box.y + box.height / 2 : 0;
  const onScreen = !!box && (!vp || (cx >= 0 && cy >= 0 && cx <= vp.width && cy <= vp.height));
  if (!box || !onScreen)
    throw new Error(
      "The place to drop it isn't on screen. Scroll to it first, or use a bigger window.",
    );
  return { cx, cy, rect: rectFromBounds(box) };
}

export class CursorDriver {
  private last: Point = { ...START };
  private lastDepartMs = 0;

  constructor(
    private readonly page: Page,
    private readonly clicks: ClickEvent[],
    private readonly elapsed: () => number,
    private readonly cursor: CursorSample[],
    private readonly keys: KeyEvent[],
  ) {
    this.mark(START.x, START.y);
  }

  pause(ms = DEFAULT_PAUSE_MS): Promise<void> {
    return this.page.waitForTimeout(ms);
  }

  async pressKey(chord: string, label?: string): Promise<void> {
    const tMs = this.elapsed();
    await this.page.keyboard.press(chord);
    this.keys.push(label ? { tMs, chord, label } : { tMs, chord });
  }

  async moveTo(target: Target): Promise<void> {
    const { cx, cy } = await boxOf(this.page, target);
    await this.glideTo(cx, cy, MOVE_SETTLE_MS);
  }

  async focus(target: Target, label?: string, opts?: ActionOpts): Promise<void> {
    const { point, frame } = await this.approach(target, opts);
    this.logEvent(point.cx, point.cy, frame.rect, label ?? "focus", opts);
  }

  async moveAndClick(target: Target, label?: string, opts?: ActionOpts): Promise<void> {
    const { point, frame } = await this.approach(target, opts);
    await this.clickAt(point.cx, point.cy, frame.rect, label, opts);
  }

  async typeInto(
    target: Target,
    chunks: readonly string[],
    label?: string,
    opts?: ActionOpts,
  ): Promise<void> {
    const { point, frame } = await this.approach(target, opts);
    const event = await this.clickAt(point.cx, point.cy, frame.rect, label ?? "field", opts);
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
    const k = captureScaleOf(this.page);
    const { point: start, frame } = await this.approach(from, opts);
    const origin = { x: start.cx, y: start.cy };
    const quick = opts?.method === "quick";
    const tDownMs = this.elapsed();
    let plan: DragPlan;
    if ("target" in to && quick) plan = await this.quickDragOnto(from, to.target, origin);
    else {
      await this.page.mouse.down();
      await this.pause(opts?.holdMs ?? DRAG_HOLD_MS);
      plan =
        "target" in to
          ? await this.dragOnto(to.target, origin, k, opts?.dragMs)
          : await this.dragAlong(
              offsetPath(origin, "by" in to ? [to.by] : to.path),
              origin,
              k,
              quick,
              opts?.dragMs,
            );
      await this.pause(DROP_SETTLE_MS);
      await this.page.mouse.up();
    }

    const end = plan.points.at(-1)!;
    this.last = { x: end.x, y: end.y };
    this.mark(end.x, end.y);
    const event = this.logEvent(end.x, end.y, frame.rect, label ?? "drag", opts, tDownMs);
    event.tUpMs = event.tMs;
    event.path = thinPath(plan.points);
    event.drop = roundRect(plan.drop);
  }

  private mark(x: number, y: number) {
    this.cursor.push({ t: this.elapsed(), x: Math.round(x), y: Math.round(y) });
  }

  private async walk(fromMs: number, spanMs: number, at: (p: number) => Point) {
    for (;;) {
      const p01 = clamp((this.elapsed() - fromMs) / spanMs, 0, 1);
      const { x: px, y: py } = at(cameraEase(p01));
      this.mark(px, py);
      await this.page.mouse.move(px, py);
      if (p01 >= 1) return;
      await this.page.waitForTimeout(SAMPLE_MS);
    }
  }

  private async glideTo(x: number, y: number, settleMs = SETTLE_MS, travelMs?: number) {
    const k = captureScaleOf(this.page);
    const dist = Math.hypot(x - this.last.x, y - this.last.y);
    this.lastDepartMs = this.elapsed();
    if (dist >= GLIDE_MIN_PX * k) {
      const glide = glidePlan(this.last, { x, y }, k, this.page.viewportSize(), travelMs);
      await this.walk(this.elapsed(), glide.durationMs, glide.at);
      if (glide.overshooting) {
        const { aim } = glide;
        await this.walk(this.elapsed(), OVERSHOOT_RETURN_MS, (p) => ({
          x: aim.x + (x - aim.x) * p,
          y: aim.y + (y - aim.y) * p,
        }));
      }
    }
    this.last = { x, y };
    this.mark(x, y);
    if (settleMs > 0) await this.pause(settleMs);
  }

  private async approach(target: Target, opts: ActionOpts | undefined) {
    const point = await boxOf(this.page, target);
    const frame = opts?.frame ? await boxOf(this.page, opts.frame) : point;
    await this.glideTo(point.cx, point.cy, opts?.hoverMs ?? SETTLE_MS, opts?.travelMs);
    return { point, frame };
  }

  private logEvent(
    x: number,
    y: number,
    rect: Rect,
    label?: string,
    opts?: ActionOpts,
    tDownMs?: number,
  ): ClickEvent {
    const event: ClickEvent = {
      label,
      tMs: this.elapsed(),
      tDepartMs: this.lastDepartMs,
      x: Math.round(x),
      y: Math.round(y),
      rect: roundRect(rect),
    };
    if (tDownMs != null) event.tDownMs = tDownMs;
    if (opts?.cluster != null) event.cluster = opts.cluster;
    if (opts?.zoom === false) event.zoom = false;
    if (opts?.zoomScale != null) event.zoomScale = opts.zoomScale;
    this.clicks.push(event);
    return event;
  }

  private async clickAt(x: number, y: number, rect: Rect, label?: string, opts?: ActionOpts) {
    const tDownMs = this.elapsed();
    await this.page.mouse.down();
    await this.pause(CLICK_HOLD_MS);
    await this.page.mouse.up();
    return this.logEvent(x, y, rect, label, opts, tDownMs);
  }

  private async quickDragOnto(from: Target, onto: Target, origin: Point): Promise<DragPlan> {
    const end = await dropBox(this.page, onto);
    if (!isPoint(from) && !isPoint(onto))
      await from.first().dragTo(onto.first(), { steps: QUICK_DRAG_STEPS });
    else {
      await this.page.mouse.down();
      await this.page.mouse.move(end.cx, end.cy, { steps: QUICK_DRAG_STEPS });
      await this.page.mouse.up();
    }
    this.mark(end.cx, end.cy);
    return { points: [origin, { x: end.cx, y: end.cy }], drop: end.rect };
  }

  private async dragOnto(
    onto: Target,
    origin: Point,
    k: number,
    dragMs: number | undefined,
  ): Promise<DragPlan> {
    const before = await dropBox(this.page, onto);
    const nudge = nudgeToward(origin, { x: before.cx, y: before.cy }, DRAG_NUDGE_PX * k);
    this.mark(nudge.x, nudge.y);
    await this.page.mouse.move(nudge.x, nudge.y);
    const end = await dropBox(this.page, onto);
    const target = { x: end.cx, y: end.cy };
    await this.walk(
      this.elapsed(),
      dragDuration([nudge, target], k, dragMs),
      bulgedCurve(nudge, target),
    );
    return { points: [origin, target], drop: end.rect };
  }

  private async dragAlong(
    points: Point[],
    origin: Point,
    k: number,
    quick: boolean,
    dragMs: number | undefined,
  ): Promise<DragPlan> {
    const drop = boundsOf(points);
    if (quick) {
      for (const point of points.slice(1))
        await this.page.mouse.move(point.x, point.y, { steps: QUICK_DRAG_STEPS });
      const end = points.at(-1)!;
      this.mark(end.x, end.y);
    } else {
      const nudge = nudgeToward(origin, points[1] ?? origin, DRAG_NUDGE_PX * k);
      this.mark(nudge.x, nudge.y);
      await this.page.mouse.move(nudge.x, nudge.y);
      await this.walk(
        this.elapsed(),
        dragDuration(points, k, dragMs),
        polylineAt([nudge, ...points.slice(1)]),
      );
    }
    return { points, drop };
  }
}
