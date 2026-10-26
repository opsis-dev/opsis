import { clamp } from "#video/common/math.ts";

export type Size = { width: number; height: number };
export type Box = { x: number; y: number; width: number; height: number };
export type StillLook = "window" | "cutout" | "bleed";

type ShotArea = Size & { body?: Box };

export type Shot = ShotArea & { scale: number };

const CUTOUT_FIT = 0.8;
const CUTOUT_MAX_UPSCALE = 1.5;
const TALL_SHOT = 1.5;
const TALLEST_CUTOUT = 4;

const CAPTURE = {
  maxExportScale: 2,
  maxScale: 8,
  scaleSteps: 2,
  hdPixels: 1920 * 1080,
  maxHdSurfaceScale: 4,
  surfaceScaleSteps: 10,
} as const;

const SURFACE_BUDGET_PX = CAPTURE.hdPixels * CAPTURE.maxHdSurfaceScale ** 2;

export function containBox(outer: Size, aspect: number): Size {
  return outer.width / outer.height > aspect
    ? { width: outer.height * aspect, height: outer.height }
    : { width: outer.width, height: outer.width / aspect };
}

function windowBox(aspect: number, canvas: Size, fit: number): Size {
  const box = containBox({ width: canvas.width * fit, height: canvas.height * fit }, aspect);
  return { width: Math.round(box.width), height: Math.round(box.height) };
}

const centeredLeft = (size: Size, canvas: Size) => (canvas.width - size.width) / 2;
const centeredTop = (size: Size, canvas: Size) => (canvas.height - size.height) / 2;

const aspectOf = (s: Size) => s.width / s.height;

function windowLayout(shot: ShotArea, canvas: Size, fit: number): Box {
  const tall = aspectOf(shot) * TALL_SHOT < aspectOf(canvas);
  const size = windowBox(tall ? aspectOf(canvas) : aspectOf(shot), canvas, fit);
  return { x: centeredLeft(size, canvas), y: centeredTop(size, canvas), ...size };
}

function cutoutLayout(shot: ShotArea, canvas: Size, maxUpscale: number): Box {
  const body = shot.body ?? { x: 0, y: 0, width: shot.width, height: shot.height };
  const want = windowBox(aspectOf(body), canvas, CUTOUT_FIT);
  const s = Math.min(want.width / body.width, maxUpscale);
  const placed = { width: body.width * s, height: body.height * s };
  return {
    x: centeredLeft(placed, canvas) - body.x * s,
    y: centeredTop(placed, canvas) - body.y * s,
    width: shot.width * s,
    height: shot.height * s,
  };
}

function bleedLayout(shot: ShotArea, canvas: Size): Box {
  const zoom = Math.max(canvas.width / shot.width, canvas.height / shot.height);
  const size = { width: shot.width * zoom, height: shot.height * zoom };
  return { x: centeredLeft(size, canvas), y: 0, ...size };
}

function layoutOf(
  look: StillLook,
  shot: ShotArea,
  canvas: Size,
  windowFit: number,
  maxUpscale: number,
): Box {
  if (look === "bleed") return bleedLayout(shot, canvas);
  if (look === "cutout") return cutoutLayout(shot, canvas, maxUpscale);
  return windowLayout(shot, canvas, windowFit);
}

export function stillLayout(look: StillLook, shot: Shot, canvas: Size, windowFit: number): Box {
  const box = layoutOf(look, shot, canvas, windowFit, shot.scale * CUTOUT_MAX_UPSCALE);
  return { ...box, x: Math.round(box.x), y: Math.round(box.y) };
}

function maxScaleFor(region: Size): number {
  const byArea = Math.sqrt(SURFACE_BUDGET_PX / (region.width * region.height));
  return clamp(
    Math.floor(byArea * CAPTURE.surfaceScaleSteps) / CAPTURE.surfaceScaleSteps,
    1,
    CAPTURE.maxScale,
  );
}

export const tallestShot = (look: StillLook, width: number, canvas: Size): number =>
  Math.ceil(look === "cutout" ? width * TALLEST_CUTOUT : (width * TALL_SHOT) / aspectOf(canvas));

export function paddedRegion(
  box: Box,
  scroll: { x: number; y: number },
  padding: number,
): { region: Box; body: Box } {
  const left = box.x + scroll.x;
  const top = box.y + scroll.y;
  const x = Math.max(0, left - padding);
  const y = Math.max(0, top - padding);
  return {
    region: {
      x,
      y,
      width: left + box.width + padding - x,
      height: top + box.height + padding - y,
    },
    body: { x: left - x, y: top - y, width: box.width, height: box.height },
  };
}

export function captureScale(
  look: StillLook,
  region: Size,
  canvas: Size,
  opts: { body?: Box; fit: number; least: number },
): number {
  const area = { ...region, body: opts.body };
  const drawn =
    layoutOf(look, area, canvas, opts.fit, Number.POSITIVE_INFINITY).width / region.width;
  const wanted = Math.ceil(drawn * CAPTURE.scaleSteps) / CAPTURE.scaleSteps;
  return Math.min(maxScaleFor(region), Math.max(opts.least, wanted));
}
