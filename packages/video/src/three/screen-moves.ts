import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { cameraAt } from "#video/common/camera-path.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ScreenMove, CameraPath } from "#video/generated/spec.ts";
import { viewPose, type Pose, type Vec3 } from "#video/three/motion.ts";

type Keyed = { rotation: Vec3; position: Vec3 };

const FACE_ON: Keyed = { rotation: [0, 0, 0], position: [0, 0, 0] };
const ANGLED: Keyed = { rotation: [0.1, -0.42, 0], position: [0.15, -0.05, -2.4] };

const STARTS: Record<Exclude<ScreenMove, "keyframes">, Keyed> = {
  "push-in": { rotation: [0.08, -0.25, 0], position: [0, -0.1, -6] },
  orbit: { rotation: [0.15, -1.1, 0], position: [0, 0, -3.2] },
  "tilt-reveal": { rotation: [-0.9, 0, 0], position: [0, -0.6, -3.2] },
  rise: { rotation: [0.35, 0, 0], position: [0, -1.3, -3.6] },
};

function mix(a: Vec3, b: Vec3, t: number): Vec3 {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}

export function screenPoseAt(
  move: ScreenMove,
  frame: number,
  durationInFrames: number,
  endFaceOn: boolean,
  moveEndsAt: number,
  camera?: CameraPath,
): Pose {
  if (move === "keyframes") return viewPose(cameraAt(camera, frame, { endFaceOn }));
  const t = interpolate(frame, [0, Math.max(1, durationInFrames * moveEndsAt)], [0, 1], {
    ...CLAMPED,
    easing: Easing.inOut(Easing.cubic),
  });
  const start = STARTS[move];
  const rest = endFaceOn ? FACE_ON : ANGLED;
  return {
    rotation: mix(start.rotation, rest.rotation, t),
    position: mix(start.position, rest.position, t),
    scale: 1,
  };
}
