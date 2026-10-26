import { partitionPoint } from "@opsis/video/common/math";
import type { TraceFrame } from "@/generated/bindings";

export function frameAt(frames: TraceFrame[], time: number): TraceFrame | undefined {
  const after = partitionPoint(frames, (f) => f.time <= time);
  return frames[Math.max(0, after - 1)];
}
