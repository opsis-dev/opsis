import { AUTHORED_FPS } from "@opsis/core/elements/lengths";
import { useCurrentFrame, useVideoConfig } from "remotion";

export function useAuthoredFrame(): number {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (frame * AUTHORED_FPS) / fps;
}

export function useAuthoredConfig(): ReturnType<typeof useVideoConfig> {
  const config = useVideoConfig();
  return {
    ...config,
    fps: AUTHORED_FPS,
    durationInFrames: (config.durationInFrames * AUTHORED_FPS) / config.fps,
  };
}

export function useRealFrames(): (authored: number) => number {
  const { fps } = useVideoConfig();
  return (authored) => (authored * fps) / AUTHORED_FPS;
}
