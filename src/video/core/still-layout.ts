export type Size = { width: number; height: number };
export type Box = { x: number; y: number; width: number; height: number };
export type StillLook = "window" | "cutout" | "bleed";
export type Anchor = "top" | "center";

export type Shot = Size & { scale: number; body?: Box };

export const CUTOUT_FIT = 0.8;
export const TALL_SHOT = 1.5;
export const MAX_UPSCALE = 1.5;
export const MAX_CAPTURE_SCALE = 8;
export const SURFACE_BUDGET_PX = 1920 * 1080 * 4 * 4;

export function windowBox(aspect: number, canvas: Size, fit: number): Size {
  if (!(aspect > 0)) throw new Error("aspect must be > 0");
  if (!(fit > 0)) throw new Error("fit must be > 0");
  let width = canvas.width * fit;
  let height = width * aspect;
  const maxHeight = canvas.height * fit;
  if (height > maxHeight) {
    height = maxHeight;
    width = height / aspect;
  }
  return { width: Math.round(width), height: Math.round(height) };
}

export function anchorBox(size: Size, canvas: Size, anchor: Anchor): { left: number; top: number } {
  return {
    left: (canvas.width - size.width) / 2,
    top: anchor === "top" ? 0 : (canvas.height - size.height) / 2,
  };
}

const aspectOf = (s: Size) => s.height / s.width;

export function windowLayout(shot: Shot, canvas: Size, fit: number): Box {
  const tall = aspectOf(shot) > aspectOf(canvas) * TALL_SHOT;
  const size = windowBox(tall ? aspectOf(canvas) : aspectOf(shot), canvas, fit);
  const at = anchorBox(size, canvas, "center");
  return { x: Math.round(at.left), y: Math.round(at.top), ...size };
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
  const at = anchorBox(placed, canvas, "center");
  const left = at.left - body.x * s;
  const top = at.top - body.y * s;
  return {
    img: { x: left, y: top, width: shot.width * s, height: shot.height * s },
    body: { x: at.left, y: at.top, ...placed },
    stretched: s > shot.scale,
  };
}

export function bleedLayout(shot: Shot, canvas: Size): Box {
  const zoom = Math.max(canvas.width / shot.width, canvas.height / shot.height);
  const size = { width: shot.width * zoom, height: shot.height * zoom };
  const at = anchorBox(size, canvas, "top");
  return { x: at.left, y: at.top, ...size };
}

export function maxScaleFor(region: Size): number {
  const byArea = Math.sqrt(SURFACE_BUDGET_PX / (region.width * region.height));
  return Math.max(1, Math.min(MAX_CAPTURE_SCALE, Math.floor(byArea * 10) / 10));
}

export function captureScale(
  look: StillLook,
  region: Size,
  canvas: Size,
  opts: { body?: Box; fit: number; least: number },
): number {
  const drawn =
    look === "bleed"
      ? bleedLayout({ ...region, scale: 1 }, canvas).width / region.width
      : look === "cutout"
        ? windowBox(aspectOf(opts.body ?? region), canvas, CUTOUT_FIT).width /
          (opts.body ?? region).width
        : windowLayout({ ...region, scale: 1 }, canvas, opts.fit).width / region.width;
  const wanted = Math.ceil(drawn * 2) / 2;
  return Math.min(maxScaleFor(region), Math.max(opts.least, wanted));
}
