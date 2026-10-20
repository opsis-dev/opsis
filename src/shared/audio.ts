import type { ChannelKind } from "./mix/types";
import type { AudioItem } from "./schema/timeline";

export const channelOf = (a: Pick<AudioItem, "channel" | "nodeId" | "id">): string =>
  a.channel ?? a.nodeId ?? a.id;

export function soundLength(a: AudioItem, total: number): number {
  const rest = Math.max(0, total - a.from);
  if (a.role === "effect") {
    const natural = a.lengthFrames === undefined ? rest : a.lengthFrames / (a.rate ?? 1);
    return Math.min(natural, a.durationInFrames ?? natural);
  }
  return a.durationInFrames ?? rest;
}

export const audioKind = (a: AudioItem): ChannelKind => a.role ?? "effect";
