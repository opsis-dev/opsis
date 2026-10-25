import { Euler, Matrix4, Quaternion, Vector3 } from "three";
import type { Pose } from "#video/three/motion.ts";

export type Point2 = { x: number; y: number };

export type ScreenView = {
  fovDeg: number;
  distance: number;
  width: number;
  height: number;
  pose: Pose;
  screen: { width: number; height: number; z: number };
};

const NEAR = 0.05;

export function poseMatrix(pose: Pose): Matrix4 {
  const quaternion = new Quaternion().setFromEuler(new Euler(...pose.rotation));
  return new Matrix4().compose(
    new Vector3(...pose.position),
    quaternion,
    new Vector3(pose.scale, pose.scale, pose.scale),
  );
}

export function screenQuad(view: ScreenView): Point2[] | null {
  const { screen, distance, width, height } = view;
  const tanHalf = Math.tan((view.fovDeg * Math.PI) / 360);
  const aspect = width / height;
  const matrix = poseMatrix(view.pose);
  const halfW = screen.width / 2;
  const halfH = screen.height / 2;
  const corners = [
    [-halfW, halfH],
    [halfW, halfH],
    [halfW, -halfH],
    [-halfW, -halfH],
  ];
  const quad: Point2[] = [];
  for (const [x, y] of corners) {
    const p = new Vector3(x, y, screen.z).applyMatrix4(matrix);
    const depth = distance - p.z;
    if (depth <= NEAR) return null;
    const ndcX = p.x / depth / (tanHalf * aspect);
    const ndcY = p.y / depth / tanHalf;
    quad.push({ x: ((ndcX + 1) / 2) * width, y: ((1 - ndcY) / 2) * height });
  }
  return quad;
}

export function facesCamera(quad: Point2[]): boolean {
  let area = 0;
  for (let i = 0; i < quad.length; i++) {
    const a = quad[i];
    const b = quad[(i + 1) % quad.length];
    area += a.x * b.y - b.x * a.y;
  }
  return area > 0;
}

export function quadMatrix(width: number, height: number, quad: Point2[]): string {
  const [q0, q1, q2, q3] = quad;
  const dx1 = q1.x - q2.x;
  const dx2 = q3.x - q2.x;
  const dx3 = q0.x - q1.x + q2.x - q3.x;
  const dy1 = q1.y - q2.y;
  const dy2 = q3.y - q2.y;
  const dy3 = q0.y - q1.y + q2.y - q3.y;
  const den = dx1 * dy2 - dx2 * dy1;
  const g = den ? (dx3 * dy2 - dx2 * dy3) / den : 0;
  const h = den ? (dx1 * dy3 - dx3 * dy1) / den : 0;
  const a = q1.x - q0.x + g * q1.x;
  const b = q3.x - q0.x + h * q3.x;
  const d = q1.y - q0.y + g * q1.y;
  const e = q3.y - q0.y + h * q3.y;
  const m = [
    a / width,
    d / width,
    0,
    g / width,
    b / height,
    e / height,
    0,
    h / height,
    0,
    0,
    1,
    0,
    q0.x,
    q0.y,
    0,
    1,
  ];
  return `matrix3d(${m.map((v) => Number(v.toFixed(9))).join(",")})`;
}

export function screenDistance(screenHeight: number, fovDeg: number): number {
  return screenHeight / 2 / Math.tan((fovDeg * Math.PI) / 360);
}
