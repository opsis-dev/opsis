import { isProjectRef } from "../asset-ref";
import { recordingOf } from "../compile/clip";
import { recordingSpeed } from "../compile/recording";
import { numberOr } from "../math";
import type { TimelineItem } from "../schema/timeline";
import type { BarThumbnail } from "./flatten";

type MediaProps = { src: string; trimBeforeMs: number; loop: boolean; volume: number };

export function mediaProps(props: Record<string, unknown>): MediaProps | null {
  const { src, trimBeforeMs, loop, volume } = props;
  if (typeof src !== "string" || !src) return null;
  return {
    src,
    trimBeforeMs: numberOr(trimBeforeMs, 0),
    loop: !!loop,
    volume: numberOr(volume, 1),
  };
}

export function thumbnailOf(item: TimelineItem): BarThumbnail | null {
  const recording = recordingOf(item);
  if (recording) {
    const src = recording.proxySrc || recording.src;
    return src ? { kind: "filmstrip", src, speed: recordingSpeed(recording), trimMs: 0 } : null;
  }
  const media = mediaProps(item.props);
  if (!media) return null;
  switch (item.component) {
    case "core/video":
      return { kind: "filmstrip", src: media.src, speed: 1, trimMs: media.trimBeforeMs };
    case "core/image":
    case "core/still":
      return isProjectRef(media.src) ? { kind: "still", src: media.src } : null;
    default:
      return null;
  }
}
