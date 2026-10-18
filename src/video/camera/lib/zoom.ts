import { poseToCss, type CameraPose, type ZoomState } from "./camera";
import {
  buildCameraTrack,
  cursorAim,
  isNarrow,
  narrowPose,
  NO_TUNING,
  sampleTrack,
  type CameraTrack,
  type CameraTuning,
} from "./camera-track";
import type { ClickLog } from "./click-log";

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

type CachedTrack = { speed: number; tuning: CameraTuning; keys: CameraTrack };

const trackCache = new WeakMap<ClickLog, CachedTrack>();

const sameTuning = (a: CameraTuning, b: CameraTuning): boolean =>
  a.sMax === b.sMax && a.holdMinS === b.holdMinS && a.narrow === b.narrow;

function cachedTrack(log: ClickLog, speed: number, tuning: CameraTuning): CameraTrack {
  const hit = trackCache.get(log);
  if (hit && hit.speed === speed && sameTuning(hit.tuning, tuning)) return hit.keys;
  const keys = buildCameraTrack(log, speed, tuning);
  trackCache.set(log, { speed, tuning, keys });
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
