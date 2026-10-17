import { clamp } from "../../shared/math";
export type Size = { width: number; height: number };
export type Box = { x: number; y: number; width: number; height: number };
export type StillLook = "window" | "cutout" | "bleed";

export type Shot = Size & { scale: number; body?: Box };

export const CUTOUT_FIT = 0.8;
const TALL_SHOT = 1.5;
const MAX_UPSCALE = 1.5;
const MAX_CAPTURE_SCALE = 8;
const HD_PIXELS = 1920 * 1080;
const MAX_SURFACE_SCALE = 4;
const SURFACE_BUDGET_PX = HD_PIXELS * MAX_SURFACE_SCALE * MAX_SURFACE_SCALE;
const SURFACE_SCALE_STEPS = 10;
const CAPTURE_SCALE_STEPS = 2;

export function containBox(outer: Size, aspect: number): Size {
  return outer.width / outer.height > aspect
    ? { width: outer.height * aspect, height: outer.height }
    : { width: outer.width, height: outer.width / aspect };
}

export function windowBox(aspect: number, canvas: Size, fit: number): Size {
  const box = containBox({ width: canvas.width * fit, height: canvas.height * fit }, 1 / aspect);
  return { width: Math.round(box.width), height: Math.round(box.height) };
}

const centredLeft = (size: Size, canvas: Size) => (canvas.width - size.width) / 2;
const centredTop = (size: Size, canvas: Size) => (canvas.height - size.height) / 2;

const aspectOf = (s: Size) => s.height / s.width;

export function windowLayout(shot: Shot, canvas: Size, fit: number): Box {
  const tall = aspectOf(shot) > aspectOf(canvas) * TALL_SHOT;
  const size = windowBox(tall ? aspectOf(canvas) : aspectOf(shot), canvas, fit);
  return {
    x: Math.round(centredLeft(size, canvas)),
    y: Math.round(centredTop(size, canvas)),
    ...size,
  };
}

export function cutoutLayout(
  shot: Shot,
  canvas: Size,
  fit = CUTOUT_FIT,
): { img: Box; body: Box; stretched: boolean } {
  const body = shot.body ?? { x: 0, y: 0, width: shot.width, height: shot.height };
  const want = windowBox(aspectOf(body), canvas, fit);
  const s = Math.min(want.width / body.width, shot.scale * MAX_UPSCALE);
  const placed = { width: body.width * s, height: body.height * s };
  const left = centredLeft(placed, canvas);
  const top = centredTop(placed, canvas);
  return {
    img: {
      x: left - body.x * s,
      y: top - body.y * s,
      width: shot.width * s,
      height: shot.height * s,
    },
    body: { x: left, y: top, ...placed },
    stretched: s > shot.scale,
  };
}

export function bleedLayout(shot: Shot, canvas: Size): Box {
  const zoom = Math.max(canvas.width / shot.width, canvas.height / shot.height);
  const size = { width: shot.width * zoom, height: shot.height * zoom };
  return { x: centredLeft(size, canvas), y: 0, ...size };
}

export function stillLayout(look: StillLook, shot: Shot, canvas: Size, windowFit: number): Box {
  if (look === "bleed") return bleedLayout(shot, canvas);
  if (look === "cutout") return cutoutLayout(shot, canvas).img;
  return windowLayout(shot, canvas, windowFit);
}

export function maxScaleFor(region: Size): number {
  const byArea = Math.sqrt(SURFACE_BUDGET_PX / (region.width * region.height));
  return clamp(
    Math.floor(byArea * SURFACE_SCALE_STEPS) / SURFACE_SCALE_STEPS,
    1,
    MAX_CAPTURE_SCALE,
  );
}

export function captureScale(
  look: StillLook,
  region: Size,
  canvas: Size,
  opts: { body?: Box; fit: number; least: number },
): number {
  const unlimited = { ...region, body: opts.body, scale: Number.POSITIVE_INFINITY };
  const drawn = stillLayout(look, unlimited, canvas, opts.fit).width / region.width;
  const wanted = Math.ceil(drawn * CAPTURE_SCALE_STEPS) / CAPTURE_SCALE_STEPS;
  return Math.min(maxScaleFor(region), Math.max(opts.least, wanted));
}
