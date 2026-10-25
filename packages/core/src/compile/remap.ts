import { MIN_SPEED } from "../limits";
import { isNonEmpty } from "../lists";
import { lerp, round3, sampleKeyed } from "../math";
import type { Remap } from "../schema/timeline";

const lerpSegment = (
  [x0, y0]: readonly [number, number],
  [x1, y1]: readonly [number, number],
  x: number,
): number => y0 + ((x - x0) / (x1 - x0)) * (y1 - y0);

export const speedRemap = (innerDuration: number, outDuration: number): Remap => [
  [0, 0],
  [outDuration, innerDuration],
];

export function remapDuration(remap: Remap): number {
  return remap.at(-1)?.[0] ?? 0;
}

export function remapFrame(remap: Remap, frame: number): number {
  if (!isNonEmpty(remap)) return frame;
  const last = remap[remap.length - 1];
  if (frame > last[0]) return last[1] + (frame - last[0]);
  return sampleKeyed(
    remap,
    frame,
    ([out]) => out,
    ([, inner]) => inner,
    (a, b, t) => lerp(a[1], b[1], t),
  );
}

type RemapSegment = { from: number; to: number; rate: number };

export function remapFromSegments(innerDuration: number, segments: RemapSegment[]): Remap {
  const out: Remap = [[0, 0]];
  let outPos = 0;
  let inPos = 0;
  const push = (inTo: number, rate: number) => {
    if (inTo <= inPos) return;
    outPos += (inTo - inPos) / rate;
    inPos = inTo;
    out.push([round3(outPos), inPos]);
  };
  for (const { from, to, rate } of segments) {
    push(Math.min(from, innerDuration), 1);
    push(Math.min(to, innerDuration), Math.max(MIN_SPEED, rate));
  }
  push(innerDuration, 1);
  return out;
}

function beforeFirst(remap: Remap, r: number): number {
  const [o0, i0] = remap[0];
  const next = remap.find(([, i]) => i > i0);
  const slope = next ? (next[0] - o0) / (next[1] - i0) : 1;
  return o0 - (i0 - r) * slope;
}

const afterLast = ([oN, iN]: Remap[number], r: number): number => oN + (r - iN);

function earliestOut(remap: Remap, last: Remap[number], r: number): number {
  for (let k = 1; k < remap.length; k++) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    const holdsOneFrame = ib === ia;
    if (ib >= r) return holdsOneFrame ? a : lerpSegment([ia, a], [ib, b], r);
  }
  return afterLast(last, r);
}

function latestOut(remap: Remap, r: number): number {
  for (let k = remap.length - 1; k > 0; k--) {
    const [a, ia] = remap[k - 1];
    const [b, ib] = remap[k];
    const holdsOneFrame = ib === ia;
    const endsBeforeR = ib <= r;
    if (ia <= r) return endsBeforeR || holdsOneFrame ? b : lerpSegment([ia, a], [ib, b], r);
  }
  return remap[0][0];
}

export type RemapSide = "start" | "end";

export function inverseRemap(remap: Remap, r: number, side: RemapSide): number {
  const last = remap.at(-1);
  if (!last) return r;
  const [o0, i0] = remap[0];
  if (r < i0) return beforeFirst(remap, r);
  if (side === "start") return r === i0 ? o0 : earliestOut(remap, last, r);
  return r >= last[1] ? afterLast(last, r) : latestOut(remap, r);
}
