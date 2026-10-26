import { clamp } from "@opsis/video/common/math";
import type { LocalFrames, VideoBar } from "@/generated/bindings";

export function barLocalAt(frames: LocalFrames | undefined, bar: VideoBar, abs: number): number {
  const values = frames?.values ?? [];
  if (!frames || values.length === 0) return abs - bar.from + bar.contentStart;
  const x = clamp(abs - frames.first, 0, values.length - 1);
  const i = Math.floor(x);
  const j = Math.min(i + 1, values.length - 1);
  const t = x - i;
  if (values[j] < values[i]) return t < 0.5 ? values[i] : values[j];
  return values[i] + (values[j] - values[i]) * t;
}
