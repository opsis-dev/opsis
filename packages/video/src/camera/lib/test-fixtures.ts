import type { Viewport } from "@opsis/core/schema/actions";
import type { CameraPose } from "#video/camera/lib/camera.ts";

export const isInFrame = (pose: CameraPose, vp: Viewport, x: number, y: number): boolean => {
  const half = 1 / (2 * pose.scale);
  const nx = x / vp.width;
  const ny = y / vp.height;
  return Math.abs(nx - pose.cx) <= half + 1e-6 && Math.abs(ny - pose.cy) <= half + 1e-6;
};
