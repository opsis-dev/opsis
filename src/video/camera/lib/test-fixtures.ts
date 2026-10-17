import type { CameraPose } from "./camera";

export const isInFrame = (
  pose: CameraPose,
  vp: { width: number; height: number },
  x: number,
  y: number,
): boolean => {
  const half = 1 / (2 * pose.scale);
  const nx = x / vp.width;
  const ny = y / vp.height;
  return Math.abs(nx - pose.cx) <= half + 1e-6 && Math.abs(ny - pose.cy) <= half + 1e-6;
};
