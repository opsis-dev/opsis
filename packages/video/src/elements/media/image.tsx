import { useMemo } from "react";
import { Img, useCurrentFrame, useVideoConfig } from "remotion";
import { zoomOver } from "#video/core/anim.ts";
import { lookEffects } from "#video/core/picture-look.ts";
import type { Fit, PictureLook } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

export function ImageElement({
  src,
  fit = "contain",
  background,
  kenBurns,
  kenBurnsZoom,
  look,
}: {
  src: string;
  fit?: Fit;
  background?: string;
  kenBurns?: boolean;
  kenBurnsZoom: number;
  look?: PictureLook;
}) {
  const frame = useCurrentFrame();
  const { durationInFrames } = useVideoConfig();
  const effects = useMemo(() => lookEffects(look, frame), [look, frame]);
  if (!src) return null;
  const scale = kenBurns ? zoomOver(frame, durationInFrames, kenBurnsZoom) : 1;
  return (
    <Backgrounded background={background}>
      <Img
        src={src}
        effects={effects}
        style={{ width: "100%", height: "100%", objectFit: fit, transform: `scale(${scale})` }}
      />
    </Backgrounded>
  );
}
