import { lerp, round3, sampleKeyed } from "../math";
import { MIN_SPEED } from "../nodes/params";

export type Remap = Array<[number, number]>;

export const lerpSegment = (
  [x0, y0]: readonly [number, number],
  [x1, y1]: readonly [number, number],
  x: number,
): number => y0 + ((x - x0) / (x1 - x0)) * (y1 - y0);

export const speedRemap = (innerDuration: number, outDuration: number): Remap => [
  [0, 0],
  [outDuration, innerDuration],
];

export function remapDuration(remap: Remap): number {
  return remap.length ? remap[remap.length - 1][0] : 0;
}

export function remapFrame(remap: Remap, frame: number): number {
  const last = remap.at(-1);
  if (!last) return frame;
  if (frame > last[0]) return last[1] + (frame - last[0]);
  return (
    sampleKeyed(
      remap,
      frame,
      ([out]) => out,
      ([, inner]) => inner,
      (a, b, t) => lerp(a[1], b[1], t),
    ) ?? frame
  );
}

export function remapFromSegments(
  innerDuration: number,
  segments: Array<[number, number, number]>,
): Remap {
  const out: Remap = [[0, 0]];
  let outPos = 0;
  let inPos = 0;
  const push = (inTo: number, rate: number) => {
    if (inTo <= inPos) return;
    outPos += (inTo - inPos) / rate;
    inPos = inTo;
    out.push([round3(outPos), inPos]);
  };
  for (const [a, b, rate] of segments) {
    push(Math.min(a, innerDuration), 1);
    push(Math.min(b, innerDuration), Math.max(MIN_SPEED, rate));
  }
  push(innerDuration, 1);
  return out;
}
