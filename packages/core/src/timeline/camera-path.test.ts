import { describe, expect, it } from "vitest";
import type { CameraPath } from "#core/schema/timeline";
import { cameraAt, cameraKeyAt, moveCameraKey } from "#core/timeline/camera-path";

const path: CameraPath = {
  keys: [
    { frame: 60, yaw: 40, pitch: 10, distance: 2, height: 0.5, ease: "linear" },
    { frame: 0, yaw: -20, pitch: 0, distance: 1, height: 0, ease: "linear" },
  ],
};

describe("camera keyframes", () => {
  it("lands exactly on each key and blends between them", () => {
    expect(cameraAt(path, 0)).toEqual({ yaw: -20, pitch: 0, distance: 1, height: 0 });
    expect(cameraAt(path, 60)).toEqual({ yaw: 40, pitch: 10, distance: 2, height: 0.5 });
    expect(cameraAt(path, 30)).toEqual({ yaw: 10, pitch: 5, distance: 1.5, height: 0.25 });
  });

  it("holds the first key before it and the last key after it", () => {
    expect(cameraAt({ keys: [{ ...path.keys[0] }] }, 10).yaw).toBe(40);
    expect(cameraAt(path, 500).yaw).toBe(40);
    expect(cameraAt({ keys: [] }, 10)).toEqual({ yaw: 0, pitch: 0, distance: 1, height: 0 });
  });

  it("ends facing the camera when asked", () => {
    expect(cameraAt(path, 60, { endFaceOn: true })).toEqual({
      yaw: 0,
      pitch: 0,
      distance: 1,
      height: 0,
    });
    expect(cameraAt(path, 30, { endFaceOn: true }).yaw).toBeCloseTo(-10);
  });

  it("gives the same view for a frame whichever way the playhead moves", () => {
    const forward = [0, 15, 30, 45, 60].map((f) => cameraAt(path, f));
    const backward = [60, 45, 30, 15, 0].map((f) => cameraAt(path, f)).toReversed();
    expect(backward).toEqual(forward);
  });

  it("moves a key in time and keeps the keys in order", () => {
    const moved = moveCameraKey(path, 0, 90);
    expect(moved.keys.map((k) => [k.frame, k.yaw])).toEqual([
      [60, 40],
      [90, -20],
    ]);
  });

  it("adds a key at the playhead with the view the camera has there", () => {
    const added = cameraKeyAt(path, 30.4);
    expect(added.keys.map((k) => k.frame)).toEqual([0, 30, 60]);
    expect(added.keys[1]).toMatchObject({ yaw: 10, pitch: 5, distance: 1.5 });
  });
});
