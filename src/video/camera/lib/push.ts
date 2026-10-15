import { PUSH_BEZIER, clamp } from "./camera";

export type PushAxis = "x" | "y" | "scale" | "none";

export type PushMove = {
  axis: PushAxis;
  dist: number;
  frames: number;
};

export type PushSpec = { in?: PushMove; out?: PushMove };

export type PushTransform = { x: number; y: number; scale: number };

export const PUSH_REST: PushTransform = { x: 0, y: 0, scale: 1 };

export const PUSH_IN_FRAMES = 15;
export const PUSH_OUT_FRAMES = 13;
export const PUSH_IN_DIST = 114;
export const PUSH_OUT_DIST = 72;

export const HOLD_AFTER_TEXT_S = 2.07;

const clamp01 = (x: number): number => clamp(x, 0, 1);

export const settle = (u: number): number => 1 - PUSH_BEZIER(clamp01(u));

export const pushIn = (move: PushMove, f: number): number =>
  move.frames > 0 ? move.dist * settle(f / move.frames) : 0;

export const pushOut = (move: PushMove, g: number): number =>
  move.frames > 0 ? move.dist * settle(1 - g / move.frames) : 0;

export function pushEnvelope(
  frame: number,
  durationInFrames: number,
  spec: PushSpec | undefined,
): PushTransform {
  if (!spec || (!spec.in && !spec.out)) return PUSH_REST;

  let x = 0;
  let y = 0;
  let scale = 1;

  const apply = (axis: PushAxis, offset: number): void => {
    if (axis === "x") x += offset;
    else if (axis === "y") y += offset;
    else if (axis === "scale") scale += offset;
  };

  const inMove = spec.in;
  if (inMove && inMove.axis !== "none" && frame < inMove.frames)
    apply(inMove.axis, pushIn(inMove, frame));

  const outMove = spec.out;
  if (outMove && outMove.axis !== "none") {
    const start = durationInFrames - outMove.frames;
    if (frame > start) apply(outMove.axis, pushOut(outMove, frame - start));
  }

  return { x, y, scale };
}

export function pushToCss(p: PushTransform, designScale = 1): string {
  const x = p.x * designScale;
  const y = p.y * designScale;
  return `translate3d(${x}px, ${y}px, 0) scale(${p.scale})`;
}

export function cutsMidMove(durationInFrames: number, spec: PushSpec | undefined): boolean {
  const out = spec?.out;
  if (!out || out.axis === "none" || out.frames <= 0) return false;
  return durationInFrames - out.frames > 0 && out.frames > 1;
}

export function exitVelocity(durationInFrames: number, spec: PushSpec | undefined): number {
  const out = spec?.out;
  if (!out || out.axis === "none" || out.frames <= 0) return 0;
  const last = durationInFrames - 1;
  const start = durationInFrames - out.frames;
  if (last <= start) return 0;
  return pushOut(out, last - start) - pushOut(out, last - start - 1);
}
