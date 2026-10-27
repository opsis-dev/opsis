import { ease } from "#video/common/envelope.ts";
import { lerp, sampleKeyed } from "#video/common/math.ts";
import type { CameraKey, CameraPath } from "#video/generated/spec.ts";

export type CameraView = Pick<CameraKey, "yaw" | "pitch" | "distance" | "height">;

const REST_VIEW: CameraView = { yaw: 0, pitch: 0, distance: 1, height: 0 };

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

const sortedCameraKeys = (path: CameraPath | undefined): CameraKey[] =>
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
