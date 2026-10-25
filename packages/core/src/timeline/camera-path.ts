import { lerp, sampleKeyed } from "#core/math";
import type { CameraKey, CameraPath } from "#core/schema/timeline";
import { ease } from "#core/timeline/envelope";

export type CameraView = Pick<CameraKey, "yaw" | "pitch" | "distance" | "height">;

export const REST_VIEW: CameraView = { yaw: 0, pitch: 0, distance: 1, height: 0 };

const mixViews = (a: CameraKey, b: CameraKey, t: number): CameraView => {
  const e = ease(a.ease, t);
  return {
    yaw: lerp(a.yaw, b.yaw, e),
    pitch: lerp(a.pitch, b.pitch, e),
    distance: lerp(a.distance, b.distance, e),
    height: lerp(a.height, b.height, e),
  };
};

const viewOf = ({ yaw, pitch, distance, height }: CameraKey): CameraView => ({
  yaw,
  pitch,
  distance,
  height,
});

export const sortedCameraKeys = (path: CameraPath | undefined): CameraKey[] =>
  (path?.keys ?? []).toSorted((a, b) => a.frame - b.frame);

export function cameraAt(
  path: CameraPath | undefined,
  frame: number,
  { endFaceOn = false }: { endFaceOn?: boolean } = {},
): CameraView {
  const keys = sortedCameraKeys(path);
  const last = keys.at(-1);
  const ended = endFaceOn && last ? [...keys.slice(0, -1), { ...last, ...REST_VIEW }] : keys;
  return sampleKeyed(ended, frame, (k) => k.frame, viewOf, mixViews) ?? REST_VIEW;
}

export function moveCameraKey(path: CameraPath, index: number, frame: number): CameraPath {
  const keys = sortedCameraKeys(path);
  if (!keys[index]) return path;
  return {
    keys: keys
      .map((k, i) => (i === index ? { ...k, frame: Math.max(0, Math.round(frame)) } : k))
      .toSorted((a, b) => a.frame - b.frame),
  };
}

export function cameraKeyAt(path: CameraPath, frame: number): CameraPath {
  const at = Math.max(0, Math.round(frame));
  const keys = sortedCameraKeys(path).filter((k) => k.frame !== at);
  return {
    keys: [...keys, { frame: at, ...cameraAt(path, at), ease: "in-out" as const }].toSorted(
      (a, b) => a.frame - b.frame,
    ),
  };
}
