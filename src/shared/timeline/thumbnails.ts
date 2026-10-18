import { isProjectRef } from "../asset-ref";
import { recordingOf, videoOf } from "../compile/clip";
import { recordingSpeed } from "../compile/recording";
import { isString } from "../guards";
import type { TimelineItem } from "../schema/timeline";
import type { BarThumbnail } from "./flatten";

export function thumbnailOf(item: TimelineItem): BarThumbnail | null {
  const recording = recordingOf(item);
  if (recording) {
    const src = recording.proxySrc || recording.src;
    return src ? { kind: "filmstrip", src, speed: recordingSpeed(recording), trimMs: 0 } : null;
  }
  const video = videoOf(item);
  if (video)
    return video.src
      ? { kind: "filmstrip", src: video.src, speed: 1, trimMs: video.trimBeforeMs }
      : null;
  const { src } = item.props;
  const still = item.component === "core/image" || item.component === "core/still";
  return still && isString(src) && isProjectRef(src) ? { kind: "still", src } : null;
}
