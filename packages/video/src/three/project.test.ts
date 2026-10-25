import { describe, expect, it } from "vitest";
import type { Pose } from "#video/three/motion.ts";
import {
  facesCamera,
  quadMatrix,
  screenDistance,
  screenQuad,
  type Point2,
  type ScreenView,
} from "#video/three/project.ts";

const W = 1920;
const H = 1080;
const FOV = 30;
const screen = { width: 2, height: 2 / (W / H), z: 0 };
const faceOn: Pose = { rotation: [0, 0, 0], position: [0, 0, 0], scale: 1 };
const view = (pose: Pose): ScreenView => ({
  fovDeg: FOV,
  distance: screenDistance(screen.height, FOV),
  width: W,
  height: H,
  pose,
  screen,
});

function mapPoint(matrix: string, x: number, y: number): Point2 {
  const m = matrix.slice("matrix3d(".length, -1).split(",").map(Number);
  const w = m[3] * x + m[7] * y + m[15];
  return { x: (m[0] * x + m[4] * y + m[12]) / w, y: (m[1] * x + m[5] * y + m[13]) / w };
}

const close = (a: Point2, b: Point2) => {
  expect(a.x).toBeCloseTo(b.x, 3);
  expect(a.y).toBeCloseTo(b.y, 3);
};

describe("placing the clip on a 3D screen", () => {
  it("fills the whole frame when the screen faces the camera", () => {
    const quad = screenQuad(view(faceOn));
    expect(quad).not.toBeNull();
    if (!quad) return;
    const frame = [
      { x: 0, y: 0 },
      { x: W, y: 0 },
      { x: W, y: H },
      { x: 0, y: H },
    ];
    quad.forEach((corner, i) => close(corner, frame[i]));
    expect(facesCamera(quad)).toBe(true);
  });

  it("maps the clip's corners onto the screen's corners for a turned screen", () => {
    const quad = screenQuad(
      view({ rotation: [0.2, -0.5, 0.05], position: [0.3, -0.1, -2], scale: 0.9 }),
    );
    expect(quad).not.toBeNull();
    if (!quad) return;
    const matrix = quadMatrix(W, H, quad);
    close(mapPoint(matrix, 0, 0), quad[0]);
    close(mapPoint(matrix, W, 0), quad[1]);
    close(mapPoint(matrix, W, H), quad[2]);
    close(mapPoint(matrix, 0, H), quad[3]);
    expect(quad[1].x - quad[0].x).toBeLessThan(W);
  });

  it("hides the clip when the screen turns away or goes behind the camera", () => {
    const back = screenQuad(
      view({ rotation: [0, Math.PI * 0.9, 0], position: [0, 0, -2], scale: 1 }),
    );
    expect(back && facesCamera(back)).toBe(false);
    expect(screenQuad(view({ rotation: [0, 0, 0], position: [0, 0, 10], scale: 1 }))).toBeNull();
  });
});
