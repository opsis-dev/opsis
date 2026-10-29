import { random, spring } from "remotion";
import { cameraAt, type CameraView } from "#video/common/camera-path.ts";
import type { Motion, CameraPath } from "#video/generated/spec.ts";

export type Vec3 = [number, number, number];
export type Pose = { rotation: Vec3; position: Vec3; scale: number };

export type MotionTime = {
  frame: number;
  fps: number;
  speed: number;
  seed: number;
  tiltDeg: number;
  endFacing: boolean;
  amount: number;
  reverse: boolean;
  growIn: boolean;
};

const TAU = Math.PI * 2;
const SPIN_PERIOD_S = 6;
const ORBIT_PERIOD_S = 8;
const SWING_PERIOD_S = 3.2;
const ENTER_FROM = 0.9;
const ENTER_SPRING = { damping: 200 };
const TILT_SPRING = { damping: 14, mass: 0.9 };
const TILT_REST: Vec3 = [0.08, -0.18, 0];
const FACING: Vec3 = [0, 0, 0];
const TILT_DEPTH = -2;
const FLOAT = { tiltX: 0.08, turnY: 0.25, rollZ: 0.04, lift: 0.08 };
const ORBIT = { pitch: 0.15, turn: 0.7 };
const SWING_TURN = 0.45;
const DEG = Math.PI / 180;
const DISTANCE_UNITS = 3;
const SPIN_PITCH = 0.12;

const still = (scale: number): Pose => ({ rotation: [0, 0, 0], position: [0, 0, 0], scale });

const lerp3 = (a: Vec3, b: Vec3, t: number): Vec3 => [
  a[0] + (b[0] - a[0]) * t,
  a[1] + (b[1] - a[1]) * t,
  a[2] + (b[2] - a[2]) * t,
];

const tiltFrom = (deg: number): Vec3 => {
  const rad = (deg * Math.PI) / 180;
  return [-rad, rad, 0];
};

export function poseAt(
  motion: Motion,
  { frame, fps, speed, seed, tiltDeg, endFacing, amount, reverse, growIn }: MotionTime,
): Pose {
  const t = (frame / fps) * speed;
  const enter = spring({ frame, fps, config: ENTER_SPRING });
  const scale = growIn ? ENTER_FROM + (1 - ENTER_FROM) * enter : 1;
  const turn = reverse ? -1 : 1;
  switch (motion) {
    case "spin":
      return { ...still(scale), rotation: [SPIN_PITCH, (turn * TAU * t) / SPIN_PERIOD_S, 0] };
    case "tilt-in": {
      const s = spring({ frame: frame * speed, fps, config: TILT_SPRING });
      return {
        rotation: lerp3(tiltFrom(tiltDeg), endFacing ? FACING : TILT_REST, s),
        position: [0, 0, (1 - s) * TILT_DEPTH],
        scale,
      };
    }
    case "float": {
      const phase = random(seed) * TAU;
      return {
        rotation: [
          Math.sin(t * 0.9 + phase) * FLOAT.tiltX * amount,
          Math.sin(t * 0.6 + phase * 2) * FLOAT.turnY * amount * turn,
          Math.sin(t * 0.7 + phase * 3) * FLOAT.rollZ * amount,
        ],
        position: [0, Math.sin(t * 1.1 + phase) * FLOAT.lift * amount, 0],
        scale,
      };
    }
    case "orbit":
      return {
        ...still(scale),
        rotation: [
          ORBIT.pitch,
          Math.sin((TAU * t) / ORBIT_PERIOD_S) * ORBIT.turn * amount * turn,
          0,
        ],
      };
    case "swing":
      return {
        ...still(scale),
        rotation: [0, Math.sin((TAU * t) / SWING_PERIOD_S) * SWING_TURN * amount * turn, 0],
      };
    case "still":
    case "keyframes":
      return still(scale);
  }
}

export const viewPose = (view: CameraView): Pose => ({
  rotation: [view.pitch * DEG, -view.yaw * DEG, 0],
  position: [0, -view.height, -(view.distance - 1) * DISTANCE_UNITS],
  scale: 1,
});

export const elementPose = (
  motion: Motion,
  camera: CameraPath | undefined,
  time: MotionTime,
): Pose => (motion === "keyframes" ? viewPose(cameraAt(camera, time.frame)) : poseAt(motion, time));

export function fitDistance(radius: number, fovDeg: number, aspect: number): number {
  const vertical = (fovDeg * Math.PI) / 360;
  const horizontal = Math.atan(Math.tan(vertical) * aspect);
  return radius / Math.sin(Math.min(vertical, horizontal));
}

const BOX_MARGIN = 1.15;

export function boxDistance(
  box: { width: number; height: number },
  fovDeg: number,
  aspect: number,
): number {
  const tanVertical = Math.tan((fovDeg * Math.PI) / 360);
  const tanHorizontal = tanVertical * aspect;
  return Math.max(box.width / 2 / tanHorizontal, box.height / 2 / tanVertical) * BOX_MARGIN;
}
