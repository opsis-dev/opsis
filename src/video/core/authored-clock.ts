import { useCurrentFrame, useVideoConfig } from "remotion";
import { AUTHORED_FPS } from "../../shared/elements/lengths";

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
