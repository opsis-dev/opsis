import type { AudioItem } from "./schema/timeline";

export const channelOf = (a: Pick<AudioItem, "channel" | "nodeId" | "id">): string =>
  a.channel ?? a.nodeId ?? a.id;

export function soundLength(a: AudioItem, fps: number, total: number): number {
  if (a.role === "effect") {
    const natural = a.lengthFrames ?? fps;
    return Math.min(natural, a.durationInFrames ?? natural);
  }
  return a.durationInFrames ?? Math.max(0, total - a.from);
}
