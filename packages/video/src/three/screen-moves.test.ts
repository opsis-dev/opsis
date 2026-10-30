import * as fc from "fast-check";
import { describe, expect, it } from "vitest";
import type { ScreenMove } from "#video/generated/spec.ts";
import { screenPoseAt } from "#video/three/screen-moves.ts";

const PRESETS: ScreenMove[] = ["push-in", "orbit", "tilt-reveal", "rise"];

describe("camera moves around a 3D screen", () => {
  it("ends every move facing the camera when asked, and holds it to the end", () => {
    for (const move of PRESETS) {
      for (const frame of [86, 95, 100]) {
        const pose = screenPoseAt(move, frame, 100, true, 0.85);
        expect(pose.rotation.every((v) => Math.abs(v) < 1e-9)).toBe(true);
        expect(pose.position.every((v) => Math.abs(v) < 1e-9)).toBe(true);
      }
    }
  });

  it("starts away from the camera and settles at an angle otherwise", () => {
    for (const move of PRESETS) {
      expect(screenPoseAt(move, 0, 100, true, 0.85).position[2]).toBeLessThan(-2);
      expect(screenPoseAt(move, 100, 100, false, 0.85).rotation[1]).toBeCloseTo(-0.42);
    }
  });

  it("follows camera keyframes and still ends facing the camera when asked", () => {
    const camera = {
      keys: [
        { frame: 0, yaw: 30, pitch: 10, distance: 2, height: 0, ease: "linear" as const },
        { frame: 90, yaw: 10, pitch: 0, distance: 1.5, height: 0, ease: "linear" as const },
      ],
    };
    const start = screenPoseAt("keyframes", 0, 100, true, 0.85, camera);
    expect(start.rotation[1]).toBeCloseTo((-30 * Math.PI) / 180);
    expect(start.position[2]).toBeCloseTo(-3);
    const end = screenPoseAt("keyframes", 95, 100, true, 0.85, camera);
    expect(end.rotation.every((v) => Math.abs(v) < 1e-9)).toBe(true);
    expect(screenPoseAt("keyframes", 95, 100, false, 0.85, camera).rotation[1]).toBeCloseTo(
      (-10 * Math.PI) / 180,
    );
  });

  it("moves smoothly from frame to frame and faces the camera at the end of any length", () => {
    fc.assert(
      fc.property(
        fc.constantFrom(...PRESETS),
        fc.integer({ min: 60, max: 900 }),
        fc.double({ min: 0, max: 1, noNaN: true }),
        (move, length, share) => {
          const frame = Math.floor(share * (length - 1));
          const now = screenPoseAt(move, frame, length, true, 0.85);
          const next = screenPoseAt(move, frame + 1, length, true, 0.85);
          function parts(p: typeof now) {
            return [...p.rotation, ...p.position];
          }
          for (const [a, b] of parts(now).map((v, i) => [v, parts(next)[i]])) {
            expect(Number.isFinite(a)).toBe(true);
            expect(Math.abs(b - a)).toBeLessThan(Math.PI / 4);
          }
          const last = screenPoseAt(move, length, length, true, 0.85);
          expect(parts(last).every((v) => Math.abs(v) < 1e-9)).toBe(true);
        },
      ),
    );
  });
});
