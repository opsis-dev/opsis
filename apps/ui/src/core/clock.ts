import type { BarClock, VideoBar } from "@/generated/bindings";

type Remap = Array<[number, number]>;

const BISECT_STEPS = 24;

const lerpSegment = (
  [x0, y0]: readonly [number, number],
  [x1, y1]: readonly [number, number],
  x: number,
): number => y0 + ((x - x0) / (x1 - x0)) * (y1 - y0);

function beforeFirst(remap: Remap, r: number): number {
  const [o0, i0] = remap[0];
  const next = remap.find(([, i]) => i > i0);
  const slope = next ? (next[0] - o0) / (next[1] - i0) : 1;
  return o0 - (i0 - r) * slope;
}

function earliestOut(remap: Remap, r: number): number {
  for (let k = 1; k < remap.length; k++) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    if (ib >= r) return ib === ia ? a : lerpSegment([ia, a], [ib, b], r);
  }
  const [oN, iN] = remap[remap.length - 1];
  return oN + (r - iN);
}

function outFrame(remap: Remap, r: number): number {
  if (!remap.length) return r;
  const [o0, i0] = remap[0];
  if (r < i0) return beforeFirst(remap, r);
  return r === i0 ? o0 : earliestOut(remap, r);
}

const toAbs = (clock: BarClock, content: number): number =>
  clock.steps.reduce((v, s) => {
    const r = v - s.startFrom + s.loopOffset;
    return s.from + (s.remap ? outFrame(s.remap, r) : r);
  }, content);

export function localAt(clock: BarClock, abs: number): number {
  let lo = clock.itemFrom;
  let hi = clock.contentEnd;
  for (let i = 0; i < BISECT_STEPS; i++) {
    const mid = (lo + hi) / 2;
    if (toAbs(clock, mid) < abs) lo = mid;
    else hi = mid;
  }
  return (lo + hi) / 2 - clock.itemFrom;
}

export const barLocalAt = (clock: BarClock | undefined, bar: VideoBar, abs: number): number =>
  clock ? localAt(clock, abs) : abs - bar.from + bar.contentStart;
