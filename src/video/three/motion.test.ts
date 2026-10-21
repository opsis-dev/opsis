import { describe, expect, it } from "vitest";
import { MOTIONS } from "../../shared/elements/three";
import { boxDistance, fitDistance, poseAt, type MotionTime } from "./motion";

const at = (frame: number, extra: Partial<MotionTime> = {}): MotionTime => ({
  frame,
  fps: 30,
  speed: 1,
  seed: 1,
  ...extra,
});

describe("3D motion presets", () => {
  it("spins once every six seconds", () => {
    expect(poseAt("spin", at(0)).rotation[1]).toBe(0);
    expect(poseAt("spin", at(90)).rotation[1]).toBeCloseTo(Math.PI);
    expect(poseAt("spin", at(90, { speed: 2 })).rotation[1]).toBeCloseTo(Math.PI * 2);
  });

  it("tilts in from an angle and settles facing the camera", () => {
    const start = poseAt("tilt-in", at(0));
    const settled = poseAt("tilt-in", at(120));
    expect(start.rotation[0]).toBeCloseTo((-52 * Math.PI) / 180);
    expect(poseAt("tilt-in", at(0, { tiltDeg: 20 })).rotation[1]).toBeCloseTo((20 * Math.PI) / 180);
    expect(start.position[2]).toBeCloseTo(-2);
    expect(settled.rotation[0]).toBeCloseTo(0.08, 2);
    expect(settled.rotation[1]).toBeCloseTo(-0.18, 2);
    expect(settled.position[2]).toBeCloseTo(0, 2);
  });

  it("grows in a little and holds its size", () => {
    expect(poseAt("still", at(0)).scale).toBeCloseTo(0.9);
    expect(poseAt("still", at(60)).scale).toBeCloseTo(1, 2);
  });

  it("gives the same pose for a frame whichever way the playhead moves", () => {
    for (const motion of MOTIONS) {
      const forward = [0, 10, 20, 30, 40].map((f) => poseAt(motion, at(f)));
      const backward = [40, 30, 20, 10, 0].map((f) => poseAt(motion, at(f))).toReversed();
      expect(backward).toEqual(forward);
    }
  });

  it("floats differently for another seed", () => {
    expect(poseAt("float", at(30, { seed: 1 }))).not.toEqual(poseAt("float", at(30, { seed: 2 })));
  });
});

describe("fitting the camera to the frame", () => {
  it("moves the camera back for a narrow frame so the subject stays whole", () => {
    const wide = fitDistance(1, 30, 16 / 9);
    const tall = fitDistance(1, 30, 9 / 16);
    expect(wide).toBeCloseTo(1 / Math.sin(Math.PI / 12));
    expect(tall).toBeGreaterThan(wide * 1.5);
    expect(fitDistance(1, 30, 1)).toBeCloseTo(wide);
  });
});

describe("fitting the camera to a wide subject", () => {
  it("keeps a wide box whole in a tall frame and close in a wide one", () => {
    const box = { width: 2.6, height: 0.8 };
    const tanHalf = Math.tan(Math.PI / 12);
    const wide = boxDistance(box, 30, 16 / 9);
    const tall = boxDistance(box, 30, 9 / 16);
    expect((wide * tanHalf * (16 / 9)) / 1.15).toBeGreaterThanOrEqual(1.3 - 1e-9);
    expect((tall * tanHalf * (9 / 16)) / 1.15).toBeCloseTo(1.3);
    expect(tall).toBeGreaterThan(wide);
  });
});
