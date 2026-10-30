import { playRange } from "@opsis/video/common/time";
import type { FilmTarget } from "@/generated/bindings";

export type Loop = { from: number; to: number };

export function activeAt(spans: FilmTarget["spans"], frame: number): string[] {
  return Object.entries(spans)
    .filter(([, s]) => frame >= s.from && frame < s.to)
    .map(([id]) => id)
    .sort();
}

export function loopOf(
  inPoint: number | null,
  outPoint: number | null,
  durationInFrames: number,
): Loop {
  const range = playRange(inPoint, outPoint, durationInFrames);
  return { from: range.inFrame ?? 0, to: (range.outFrame ?? durationInFrames - 1) + 1 };
}

export function wrapInLoop(position: number, { from, to }: Loop): number {
  if (position >= from && position < to) return position;
  const length = to - from;
  return from + ((((position - from) % length) + length) % length);
}
