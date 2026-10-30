import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { FootageFrame, type FootageUrls } from "#video/core/footage.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

export function VideoLoop({
  src,
  dim,
  dimColor,
  fit,
  playbackRate,
  footageUrls,
}: ElementSettings["video-loop"] & { footageUrls?: FootageUrls }) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const footage = footageUrls?.src;
  return src && footage ? (
    <AbsoluteFill>
      <FootageFrame url={footage} timeS={(frame / fps) * playbackRate} fit={fit} />
      <AbsoluteFill style={{ background: dimColor, opacity: dim }} />
    </AbsoluteFill>
  ) : null;
}
