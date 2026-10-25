import { EPSILON } from "../math";
import type { AudioBar, FrameRange, VideoBar } from "./flatten";

export const overlaps = (a: FrameRange, b: FrameRange) =>
  a.from < b.to - EPSILON && b.from < a.to - EPSILON;

export function packLanes(bars: VideoBar[]): { bars: VideoBar[]; lanes: number } {
  const placed: VideoBar[] = [];
  let lanes = 0;
  for (const bar of bars) {
    const taken = new Set(placed.filter((p) => overlaps(p, bar)).map((p) => p.lane));
    let lane = 0;
    while (taken.has(lane)) lane++;
    placed.push({ ...bar, lane });
    lanes = Math.max(lanes, lane + 1);
  }
  return { bars: placed, lanes };
}

export function packAudio(audio: AudioBar[]): { audio: AudioBar[]; lanes: number } {
  const ends: number[] = [];
  let lastRole: AudioBar["role"] | null = null;
  let base = 0;
  const placed: AudioBar[] = [];
  for (const a of audio) {
    if (a.role !== lastRole) {
      base = ends.length;
      lastRole = a.role;
    }
    let lane = base;
    while (lane < ends.length && ends[lane] > a.from + EPSILON) lane++;
    ends[lane] = a.to;
    placed.push({ ...a, lane });
  }
  return { audio: placed, lanes: ends.length };
}
