import { EPSILON } from "../math";
import type { AudioBar, FrameRange, VideoBar } from "./flatten";

export const overlaps = (a: FrameRange, b: FrameRange) =>
  a.from < b.to - EPSILON && b.from < a.to - EPSILON;

export function packLanes(bars: VideoBar[]): number {
  let lanes = 0;
  for (let i = 0; i < bars.length; i++) {
    const taken = new Set<number>();
    for (let j = 0; j < i; j++) if (overlaps(bars[j], bars[i])) taken.add(bars[j].lane);
    let lane = 0;
    while (taken.has(lane)) lane++;
    bars[i].lane = lane;
    lanes = Math.max(lanes, lane + 1);
  }
  return lanes;
}

export function packAudio(audio: AudioBar[]): number {
  const ends: number[] = [];
  let lastRole: AudioBar["role"] | null = null;
  let base = 0;
  for (const a of audio) {
    if (a.role !== lastRole) {
      base = ends.length;
      lastRole = a.role;
    }
    let lane = base;
    while (lane < ends.length && ends[lane] > a.from + EPSILON) lane++;
    a.lane = lane;
    ends[lane] = a.to;
  }
  return ends.length;
}
