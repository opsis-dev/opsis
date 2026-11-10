import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { FootageFrame, type FootageUrls } from "#video/core/footage.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

export function VideoElement({
  src,
  fit,
  playbackRate,
  background,
  dim,
  dimColor,
  footageUrls,
}: ElementSettings["video"] & { footageUrls?: FootageUrls }) {
  const { fps } = useFilm();
  const frame = useFrame();
  const footageUrl = footageUrls?.src;
  if (!src || !footageUrl) return null;
  return (
    <Backgrounded background={background}>
      <FootageFrame url={footageUrl} timeS={(frame / fps) * playbackRate} fit={fit} />
      {dim > 0 ? <AbsoluteFill style={{ background: dimColor, opacity: dim }} /> : null}
    </Backgrounded>
  );
}
