import { Video } from "@remotion/media";
import { useMemo } from "react";
import { useCurrentFrame, useVideoConfig } from "remotion";
import { msToFrames } from "#video/common/time.ts";
import { lookEffects } from "#video/core/picture-look.ts";
import type { Fit, PictureLook } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

export function VideoElement({
  src,
  fit = "cover",
  loop = false,
  muted = true,
  volume = 1,
  playbackRate = 1,
  trimBeforeMs = 0,
  background,
  look,
}: {
  src: string;
  fit?: Fit;
  loop?: boolean;
  muted?: boolean;
  volume?: number;
  playbackRate?: number;
  trimBeforeMs?: number;
  background?: string;
  look?: PictureLook;
}) {
  const { fps } = useVideoConfig();
  const frame = useCurrentFrame();
  const effects = useMemo(() => lookEffects(look, frame), [look, frame]);
  if (!src) return null;
  return (
    <Backgrounded background={background}>
      <Video
        src={src}
        loop={loop}
        muted={muted}
        volume={volume}
        playbackRate={playbackRate}
        trimBefore={trimBeforeMs ? msToFrames(trimBeforeMs, fps, 0) : undefined}
        objectFit={fit}
        effects={effects}
        style={{ width: "100%", height: "100%" }}
      />
    </Backgrounded>
  );
}
