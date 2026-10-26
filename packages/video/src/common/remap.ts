import { isNonEmpty } from "#video/common/lists.ts";
import { lerp, sampleKeyed } from "#video/common/math.ts";

export type Remap = Array<[number, number]>;

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
