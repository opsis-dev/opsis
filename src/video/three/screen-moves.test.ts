import { describe, expect, it } from "vitest";
import { SCREEN_MOVES } from "../../shared/elements/screen-3d";
import { screenPoseAt } from "./screen-moves";

const PRESETS = SCREEN_MOVES.filter((m) => m !== "keyframes");

describe("camera moves around a 3D screen", () => {
  it("ends every move facing the camera when asked, and holds it to the end", () => {
    for (const move of PRESETS) {
      for (const frame of [86, 95, 100]) {
        const pose = screenPoseAt(move, frame, 100, true);
        expect(pose.rotation.every((v) => Math.abs(v) < 1e-9)).toBe(true);
        expect(pose.position.every((v) => Math.abs(v) < 1e-9)).toBe(true);
      }
    }
  });

  it("starts away from the camera and settles at an angle otherwise", () => {
    for (const move of PRESETS) {
      expect(screenPoseAt(move, 0, 100, true).position[2]).toBeLessThan(-2);
      expect(screenPoseAt(move, 100, 100, false).rotation[1]).toBeCloseTo(-0.42);
    }
  });

  it("follows camera keyframes and still ends facing the camera when asked", () => {
    const camera = {
      keys: [
        { frame: 0, yaw: 30, pitch: 10, distance: 2, height: 0, ease: "linear" as const },
        { frame: 90, yaw: 10, pitch: 0, distance: 1.5, height: 0, ease: "linear" as const },
      ],
    };
    const start = screenPoseAt("keyframes", 0, 100, true, camera);
    expect(start.rotation[1]).toBeCloseTo((-30 * Math.PI) / 180);
    expect(start.position[2]).toBeCloseTo(-3);
    const end = screenPoseAt("keyframes", 95, 100, true, camera);
    expect(end.rotation.every((v) => Math.abs(v) < 1e-9)).toBe(true);
    expect(screenPoseAt("keyframes", 95, 100, false, camera).rotation[1]).toBeCloseTo(
      (-10 * Math.PI) / 180,
    );
  });

  it("gives the same pose for a frame whichever way the playhead moves", () => {
    const forward = [0, 20, 40, 60].map((f) => screenPoseAt("orbit", f, 100, true));
    const backward = [60, 40, 20, 0].map((f) => screenPoseAt("orbit", f, 100, true)).toReversed();
    expect(backward).toEqual(forward);
  });
});
