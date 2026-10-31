import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AUTHORED_FPS } from "#video/common/spec.ts";

export function useAuthoredFrame(): number {
  const frame = useFrame();
  const { fps } = useFilm();
  return (frame * AUTHORED_FPS) / fps;
}

export function useAuthoredConfig(): ReturnType<typeof useFilm> {
  const config = useFilm();
  return {
    ...config,
    fps: AUTHORED_FPS,
    durationInFrames: (config.durationInFrames * AUTHORED_FPS) / config.fps,
  };
}

export function useRealFrames(): (authored: number) => number {
  const { fps } = useFilm();
  return (authored) => (authored * fps) / AUTHORED_FPS;
}
