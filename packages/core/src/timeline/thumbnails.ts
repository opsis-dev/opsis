import { isProjectRef } from "#core/asset-ref";
import { videoOf } from "#core/compile/clip";
import { recordingSpeed } from "#core/compile/recording";
import { isString } from "#core/guards";
import type { ElementItem, RecordingItem } from "#core/schema/timeline";
import type { BarThumbnail } from "#core/timeline/flatten";

export function thumbnailOf(item: RecordingItem | ElementItem): BarThumbnail | null {
  if (item.kind === "recording") {
    const src = item.props.proxySrc || item.props.src;
    return src ? { kind: "filmstrip", src, speed: recordingSpeed(item.props), trimMs: 0 } : null;
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
