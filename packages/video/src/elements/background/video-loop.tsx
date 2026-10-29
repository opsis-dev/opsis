import { AbsoluteFill } from "remotion";
import { VideoElement } from "#video/elements/media/video.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

export function VideoLoop({
  src,
  dim,
  dimColor,
  fit,
  playbackRate,
  trimBeforeMs,
}: ElementSettings["video-loop"]) {
  return src ? (
    <AbsoluteFill>
      <VideoElement
        src={src}
        fit={fit}
        playbackRate={playbackRate}
        trimBeforeMs={trimBeforeMs}
        loop
      />
      <AbsoluteFill style={{ background: dimColor, opacity: dim }} />
    </AbsoluteFill>
  ) : null;
}
