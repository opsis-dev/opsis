import { Video } from "@remotion/media";
import React, { useMemo } from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { msToFrames } from "#video/common/time.ts";
import { lookEffects } from "#video/core/picture-look.ts";
import type { Fit, PictureLook } from "#video/generated/spec.ts";

export const VideoElement: React.FC<{
  src: string;
  fit?: Fit;
  loop?: boolean;
  muted?: boolean;
  volume?: number;
  playbackRate?: number;
  trimBeforeMs?: number;
  look?: PictureLook;
}> = ({
  src,
  fit = "cover",
  loop = false,
  muted = true,
  volume = 1,
  playbackRate = 1,
  trimBeforeMs = 0,
  look,
}) => {
  const { fps } = useVideoConfig();
  const frame = useCurrentFrame();
  const effects = useMemo(() => lookEffects(look, frame), [look, frame]);
  if (!src) return null;
  return (
    <AbsoluteFill>
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
    </AbsoluteFill>
  );
};
