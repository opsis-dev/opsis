import { stableJson } from "@opsis/core/hash";
import {
  buildCameraTrack,
  cursorAim,
  isNarrow,
  narrowPose,
  NO_TUNING,
  sampleTrack,
  type CameraTrack,
  type CameraTuning,
} from "#video/camera/lib/camera-track.ts";
import { poseToCss, type CameraPose, type ZoomState } from "#video/camera/lib/camera.ts";
import type { ClickLog } from "#video/camera/lib/click-log.ts";

type TrackOptions = {
  speed?: number;
  tuning?: CameraTuning;
};

type ZoomOptions = TrackOptions & {
  chromeFrac?: number;
  fit?: number;
};

export const logTimeS = (frame: number, fps: number, speed: number): number =>
  (frame / fps) * speed;

type CachedTrack = { options: string; keys: CameraTrack };

const trackCache = new WeakMap<ClickLog, CachedTrack>();

function cachedTrack(log: ClickLog, speed: number, tuning: CameraTuning): CameraTrack {
  const options = stableJson({ speed, tuning });
  const hit = trackCache.get(log);
  if (hit?.options === options) return hit.keys;
  const keys = buildCameraTrack(log, speed, tuning);
  trackCache.set(log, { options, keys });
  return keys;
}

export function poseAt(
  frame: number,
  fps: number,
  log: ClickLog,
  { speed = 1, tuning = NO_TUNING }: TrackOptions = {},
): CameraPose {
  return sampleTrack(cachedTrack(log, speed, tuning), logTimeS(frame, fps, speed));
}

export function zoomAt(
  frame: number,
  fps: number,
  log: ClickLog,
  { speed = 1, tuning = NO_TUNING, chromeFrac = 0, fit = 1 }: ZoomOptions = {},
): ZoomState {
  const aim = isNarrow(tuning) ? cursorAim(log, logTimeS(frame, fps, speed)) : null;
  const pose = narrowPose(poseAt(frame, fps, log, { speed, tuning }), tuning, aim);
  return poseToCss(pose, chromeFrac, fit);
}
