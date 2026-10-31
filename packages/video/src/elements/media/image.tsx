import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { Img } from "@opsis/runtime/parts.tsx";
import { zoomOver } from "#video/core/anim.ts";
import { FootageFrame, type FootageUrls } from "#video/core/footage.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

export function ImageElement({
  src,
  fit,
  background,
  kenBurns,
  kenBurnsZoom,
  footageUrls,
}: ElementSettings["image"] & { footageUrls?: FootageUrls }) {
  const frame = useFrame();
  const { durationInFrames } = useFilm();
  const footageUrl = footageUrls?.src;
  if (!src) return null;
  const scale = kenBurns ? zoomOver(frame, durationInFrames, kenBurnsZoom) : 1;
  const style = { transform: `scale(${scale})` };
  return (
    <Backgrounded background={background}>
      {footageUrl ? (
        <FootageFrame url={footageUrl} timeS={0} fit={fit} style={style} />
      ) : (
        <Img src={src} style={{ width: "100%", height: "100%", objectFit: fit, ...style }} />
      )}
    </Backgrounded>
  );
}
