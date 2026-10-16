import type { Page, Locator } from "playwright-core";
import { cameraEase } from "../../video/camera/lib/camera";
import { DESIGN_WIDTH, type CursorSample, type KeyEvent } from "../../video/camera/lib/click-log";
import type { ActionOpts, ClickEvent, FlowContext, Target } from "./flow-types";
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
import { findByName, type TargetOverrides } from "./selectors";

const ARROW_HOTX = 2;
const ARROW_HOTY = 2;

export const useBakedCursor = (): boolean => process.env.DEMO_BAKED_CURSOR === "1";

const GLIDE_MIN_PX = 6;
const DRAG_NUDGE_PX = 6;
const DROP_SETTLE_MS = 120;
const QUICK_DRAG_STEPS = 20;

export const CURSOR_INIT_SCRIPT = `
(() => {
  var ARROW =
    '<svg width="48" height="64" viewBox="0 0 24 32" xmlns="http://www.w3.org/2000/svg">' +
    '<path d="M1 1 L1 23 L6.5 17.8 L10.2 26.6 L13.7 25 L9.95 16.5 L17 16.3 Z" ' +
    'fill="#111418" stroke="#ffffff" stroke-width="1.85" stroke-linejoin="round" ' +
    'paint-order="stroke"/></svg>';
  var HOTX = ${ARROW_HOTX}, HOTY = ${ARROW_HOTY};
  const mount = () => {
    if (document.getElementById('pw-cursor')) return;
    if (!document.body) return;
    const style = document.createElement('style');
    style.textContent =
      '* { cursor: none !important; } ' +
      '#pw-cursor{position:fixed;top:0;left:0;pointer-events:none;z-index:2147483647;' +
      'transform:translate(-300px,-300px);transform-origin:' + HOTX + 'px ' + HOTY + 'px;' +
      'filter:drop-shadow(0 0 0.5px #fff) drop-shadow(0 1px 0 #fff) drop-shadow(0 0 2px #fff) drop-shadow(0 5px 12px rgba(0,0,0,.6));} ' +
      '#pw-cursor svg{display:block;} ' +
      '.pw-ripple{position:fixed;width:22px;height:22px;margin:-11px 0 0 -11px;border-radius:50%;' +
      'border:2px solid rgba(15,23,42,.55);background:rgba(15,23,42,.1);' +
      'pointer-events:none;z-index:2147483646;animation:pw-ripple .35s ease-out forwards;} ' +
      '@keyframes pw-ripple{from{transform:scale(.3);opacity:.85;}to{transform:scale(1.35);opacity:0;}}';
    document.documentElement.appendChild(style);
    const c = document.createElement('div');
    c.id = 'pw-cursor';
    c.innerHTML = ARROW;
    document.body.appendChild(c);
    let x = -300, y = -300;
    const place = (s) => { c.style.transform = 'translate(' + (x - HOTX) + 'px,' + (y - HOTY) + 'px) scale(' + s + ')'; };
    let down = false;
    const follow = (e) => { x = e.clientX; y = e.clientY; place(down ? 0.9 : 1); };
    const release = () => { down = false; place(1); };
    document.addEventListener('mousemove', follow, true);
    document.addEventListener('dragover', follow, true);
    document.addEventListener('mousedown', () => {
      down = true;
      place(0.9);
      const r = document.createElement('div');
      r.className = 'pw-ripple';
      r.style.left = x + 'px';
      r.style.top = y + 'px';
      document.body.appendChild(r);
      setTimeout(() => r.remove(), 380);
    }, true);
    document.addEventListener('mouseup', release, true);
    document.addEventListener('drop', release, true);
    document.addEventListener('dragend', release, true);
  };
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', mount);
  } else {
    mount();
  }
})();
`;

type Rect = { x: number; y: number; w: number; h: number };

type ResolvedTarget = Exclude<Target, string>;

function isPoint(t: Target): t is { x: number; y: number } {
  return (
    typeof t === "object" && t !== null && "x" in t && typeof (t as { x: unknown }).x === "number"
  );
}

function isCss(t: Target): t is { css: string } {
  return (
    typeof t === "object" &&
    t !== null &&
    "css" in t &&
    typeof (t as { css: unknown }).css === "string"
  );
}

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

const captureScaleOf = (page: Page): number => {
  const vp = page.viewportSize();
  return vp && vp.width > 0 ? vp.width / DESIGN_WIDTH : 1;
};

async function boxOf(
  page: Page,
  target: ResolvedTarget,
): Promise<{ cx: number; cy: number; rect: Rect }> {
  const k = captureScaleOf(page);
  if (isPoint(target)) {
    return {
      cx: target.x,
      cy: target.y,
      rect: {
        x: target.x - 60 * k,
        y: target.y - 24 * k,
        w: 120 * k,
        h: 48 * k,
      },
    };
  }
  const loc: Locator = (isCss(target) ? page.locator(target.css) : target).first() as Locator;
  await loc.waitFor({ state: "visible", timeout: 20000 });
  const before = await loc.boundingBox();
  await loc.scrollIntoViewIfNeeded();
  const box = await loc.boundingBox();
  if (!box)
    throw new Error(`Couldn't see ${isCss(target) ? target.css : "the element"} on the page.`);
  if (before && Math.hypot(box.x - before.x, box.y - before.y) > 8 * k) {
    await page.waitForTimeout(450);
  }
  let rect: Rect = { x: box.x, y: box.y, w: box.width, h: box.height };
  if (rect.h < 48 * k) {
    const targetH = 72 * k;
    const grow = (targetH - rect.h) / 2;
    rect = {
      x: Math.max(0, rect.x - 24 * k),
      y: Math.max(0, rect.y - grow - 8 * k),
      w: rect.w + 48 * k,
      h: targetH,
    };
  }
  return {
    cx: box.x + box.width / 2,
    cy: box.y + box.height / 2,
    rect,
  };
}

const roundRect = (r: Rect): Rect => ({
  x: Math.round(r.x),
  y: Math.round(r.y),
  w: Math.round(r.w),
  h: Math.round(r.h),
});

export function buildContext(
  page: Page,
  baseURL: string,
  clicks: ClickEvent[],
  getElapsedMs: () => number,
  cursor: CursorSample[] = [],
  targets?: TargetOverrides,
  keys: KeyEvent[] = [],
): FlowContext {
  const pause = (ms = 700) => page.waitForTimeout(ms);

  const pressKey: FlowContext["pressKey"] = async (chord, label) => {
    const tMs = getElapsedMs();
    await page.keyboard.press(chord);
    keys.push(label ? { tMs, chord, label } : { tMs, chord });
  };

  const find: FlowContext["find"] = (name) => findByName(page, name, targets);

  const resolve = async (target: Target): Promise<ResolvedTarget> =>
    typeof target === "string" ? await find(target) : target;

  let last = { x: 960, y: 540 };
  let lastDepartMs = 0;

  const mark = (x: number, y: number) => {
    cursor.push({ t: getElapsedMs(), x: Math.round(x), y: Math.round(y) });
  };
  mark(last.x, last.y);

  const walk = async (fromMs: number, spanMs: number, at: (p: number) => Point) => {
    for (;;) {
      const p01 = clamp((getElapsedMs() - fromMs) / spanMs, 0, 1);
      const { x: px, y: py } = at(cameraEase(p01));
      mark(px, py);
      await page.mouse.move(px, py);
      if (p01 >= 1) return;
      await page.waitForTimeout(8);
    }
  };

  const glideTo = async (x: number, y: number, settleMs = 220, travelMs?: number) => {
    const gk = captureScaleOf(page);
    const dist = Math.hypot(x - last.x, y - last.y);
    lastDepartMs = getElapsedMs();
    if (dist < GLIDE_MIN_PX * gk) {
      last = { x, y };
      mark(x, y);
      if (settleMs > 0) await pause(settleMs);
      return;
    }
    const glide = glidePlan(last, { x, y }, gk, page.viewportSize(), travelMs);
    await walk(getElapsedMs(), glide.durationMs, glide.at);

    if (glide.overshooting) {
      const { aim } = glide;
      await walk(getElapsedMs(), 130, (p) => ({
        x: aim.x + (x - aim.x) * p,
        y: aim.y + (y - aim.y) * p,
      }));
    }

    last = { x, y };
    mark(x, y);
    if (settleMs > 0) await pause(settleMs);
  };

  const logEvent = (
    x: number,
    y: number,
    rect: Rect,
    label?: string,
    opts?: ActionOpts,
    tDownMs?: number,
  ) => {
    const event: ClickEvent = {
      label,
      tMs: getElapsedMs(),
      tDepartMs: lastDepartMs,
      x: Math.round(x),
      y: Math.round(y),
      rect: roundRect(rect),
    };
    if (tDownMs != null) event.tDownMs = tDownMs;
    if (opts?.cluster != null) event.cluster = opts.cluster;
    if (opts?.zoom === false) event.zoom = false;
    if (opts?.zoomScale != null) event.zoomScale = opts.zoomScale;
    clicks.push(event);
    return event;
  };

  const clickAt = async (x: number, y: number, rect: Rect, label?: string, opts?: ActionOpts) => {
    const tDownMs = getElapsedMs();
    await page.mouse.down();
    await pause(80);
    await page.mouse.up();
    return logEvent(x, y, rect, label, opts, tDownMs);
  };

  const moveTo: FlowContext["moveTo"] = async (target) => {
    const { cx, cy } = await boxOf(page, await resolve(target));
    await glideTo(cx, cy, 120);
  };

  const focus: FlowContext["focus"] = async (target, label, opts) => {
    const point = await boxOf(page, await resolve(target));
    const frameBox = opts?.frame ? await boxOf(page, await resolve(opts.frame)) : point;
    await glideTo(point.cx, point.cy, opts?.hoverMs ?? 220, opts?.travelMs);
    logEvent(
      point.cx,
      point.cy,
      frameBox.rect,
      label ?? (typeof target === "string" ? target : "focus"),
      opts,
    );
  };

  const moveAndClick: FlowContext["moveAndClick"] = async (target, label, opts) => {
    const point = await boxOf(page, await resolve(target));
    const frameBox = opts?.frame ? await boxOf(page, await resolve(opts.frame)) : point;
    await glideTo(point.cx, point.cy, opts?.hoverMs ?? 220, opts?.travelMs);
    await clickAt(
      point.cx,
      point.cy,
      frameBox.rect,
      label ?? (typeof target === "string" ? target : undefined),
      opts,
    );
  };

  const dropBox = async (target: Target): Promise<{ cx: number; cy: number; rect: Rect }> => {
    const resolved = await resolve(target);
    if (isPoint(resolved)) return boxOf(page, resolved);
    const loc = (isCss(resolved) ? page.locator(resolved.css) : resolved).first() as Locator;
    const box = await loc.boundingBox();
    const vp = page.viewportSize();
    const onScreen =
      !!box &&
      (!vp ||
        (box.x + box.width / 2 >= 0 &&
          box.y + box.height / 2 >= 0 &&
          box.x + box.width / 2 <= vp.width &&
          box.y + box.height / 2 <= vp.height));
    if (!box || !onScreen)
      throw new Error(
        "The place to drop it isn't on screen. Scroll to it first, or use a bigger window.",
      );
    return {
      cx: box.x + box.width / 2,
      cy: box.y + box.height / 2,
      rect: { x: box.x, y: box.y, w: box.width, h: box.height },
    };
  };

  const locatorOf = async (target: Target): Promise<Locator | null> => {
    const resolved = await resolve(target);
    if (isPoint(resolved)) return null;
    return (isCss(resolved) ? page.locator(resolved.css) : resolved).first() as Locator;
  };

  const drag: FlowContext["drag"] = async (from, to, label, opts) => {
    const gk = captureScaleOf(page);
    const start = await boxOf(page, await resolve(from));
    const frameBox = opts?.frame ? await boxOf(page, await resolve(opts.frame)) : start;
    await glideTo(start.cx, start.cy, opts?.hoverMs ?? 220, opts?.travelMs);
    const origin = { x: start.cx, y: start.cy };
    const quick = opts?.method === "quick";
    const tDownMs = getElapsedMs();
    let points: Point[];
    let drop: Rect;

    if ("target" in to && quick) {
      const source = await locatorOf(from);
      const target = await locatorOf(to.target);
      const end = await dropBox(to.target);
      points = [origin, { x: end.cx, y: end.cy }];
      drop = end.rect;
      if (source && target) await source.dragTo(target, { steps: QUICK_DRAG_STEPS });
      else {
        await page.mouse.down();
        await page.mouse.move(end.cx, end.cy, { steps: QUICK_DRAG_STEPS });
        await page.mouse.up();
      }
      mark(end.cx, end.cy);
    } else {
      await page.mouse.down();
      await pause(opts?.holdMs ?? 150);
      if ("target" in to) {
        const before = await dropBox(to.target);
        const nudge = nudgeToward(origin, { x: before.cx, y: before.cy }, DRAG_NUDGE_PX * gk);
        mark(nudge.x, nudge.y);
        await page.mouse.move(nudge.x, nudge.y);
        const end = await dropBox(to.target);
        const target = { x: end.cx, y: end.cy };
        points = [origin, target];
        drop = end.rect;
        const curve = bulgedCurve(nudge, target);
        await walk(getElapsedMs(), dragDuration([nudge, target], gk, opts?.dragMs), curve);
      } else {
        points = offsetPath(origin, "by" in to ? [to.by] : to.path);
        drop = boundsOf(points);
        if (quick) {
          for (const point of points.slice(1))
            await page.mouse.move(point.x, point.y, { steps: QUICK_DRAG_STEPS });
          mark(points.at(-1)!.x, points.at(-1)!.y);
        } else {
          const nudge = nudgeToward(origin, points[1] ?? origin, DRAG_NUDGE_PX * gk);
          mark(nudge.x, nudge.y);
          await page.mouse.move(nudge.x, nudge.y);
          await walk(
            getElapsedMs(),
            dragDuration(points, gk, opts?.dragMs),
            polylineAt([nudge, ...points.slice(1)]),
          );
        }
      }
      await pause(DROP_SETTLE_MS);
      await page.mouse.up();
    }

    const end = points.at(-1)!;
    last = { x: end.x, y: end.y };
    mark(end.x, end.y);
    const event = logEvent(
      end.x,
      end.y,
      frameBox.rect,
      label ?? (typeof from === "string" ? from : "drag"),
      opts,
      tDownMs,
    );
    event.tUpMs = event.tMs;
    event.path = thinPath(points);
    event.drop = roundRect(drop);
  };

  const typeInto: FlowContext["typeInto"] = async (selector, text, label, opts) => {
    const { cx, cy, rect } = await boxOf(page, await resolve(selector));
    const frameRect = opts?.frame ? (await boxOf(page, await resolve(opts.frame))).rect : rect;
    await glideTo(cx, cy);
    const event = await clickAt(
      cx,
      cy,
      frameRect,
      label ?? (typeof selector === "string" ? selector : "field"),
      opts,
    );
    await pause(150);
    const selectAll = process.platform === "darwin" ? "Meta+A" : "Control+A";
    await page.keyboard.press(selectAll);
    await page.keyboard.press("Backspace");
    const chunks = typeof text === "string" ? [text] : text;
    for (let i = 0; i < chunks.length; i++) {
      if (i > 0) await pause(opts?.chunkPauseMs ?? 0);
      await page.keyboard.type(chunks[i], { delay: 30 });
    }
    event.typeEndMs = getElapsedMs();
  };

  return {
    page,
    baseURL,
    moveAndClick,
    typeInto,
    moveTo,
    drag,
    focus,
    find,
    pause,
    pressKey,
    page_waitForURL: page.waitForURL.bind(page),
  };
}
