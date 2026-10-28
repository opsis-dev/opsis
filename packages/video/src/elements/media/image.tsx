import React, { useMemo } from "react";
import { Img, useCurrentFrame, useVideoConfig } from "remotion";
import { zoomOver } from "#video/core/anim.ts";
import { lookEffects } from "#video/core/picture-look.ts";
import type { Fit, PictureLook } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

const KEN_BURNS_SCALE = 1.06;

export const ImageElement: React.FC<{
  src: string;
  fit?: Fit;
  background?: string;
  kenBurns?: boolean;
  look?: PictureLook;
}> = ({ src, fit = "contain", background, kenBurns, look }) => {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const effects = useMemo(() => lookEffects(look, frame), [look, frame]);
  if (!src) return null;
  const scale = kenBurns ? zoomOver(frame, durationInFrames, KEN_BURNS_SCALE) : 1;
  return (
    <Backgrounded background={background}>
      <Img
        src={src}
        effects={effects}
        style={{ width: "100%", height: "100%", objectFit: fit, transform: `scale(${scale})` }}
      />
    </Backgrounded>
  );
};
