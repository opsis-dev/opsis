import { describe, expect, it } from "vitest";
import { NODE_MOTION_SPECS, motionTimeline } from "./node-motions";

describe("node motions", () => {
  it("spreads frames without an offset evenly between the ones that have one", () => {
    const { times, values, ease } = motionTimeline([
      { opacity: 1 },
      { opacity: 0.5 },
      { opacity: 0, offset: 0.6 },
      { opacity: 0.2 },
      { opacity: 1 },
    ]);
    expect(times.map((t) => Number(t.toFixed(3)))).toEqual([0, 0.3, 0.6, 0.8, 1]);
    expect(values).toEqual({ opacity: [1, 0.5, 0, 0.2, 1] });
    expect(ease).toHaveLength(4);
  });

  it("gives every frame of a motion the same properties, in rising order", () => {
    const broken = Object.entries(NODE_MOTION_SPECS).filter(([, spec]) => {
      const props = (f: object) =>
        Object.keys(f)
          .filter((k) => k !== "offset" && k !== "ease")
          .sort()
          .join();
      const { times } = motionTimeline(spec.frames);
      const sameProps = spec.frames.every((f) => props(f) === props(spec.frames[0]));
      const rising = times.every((t, i) => i === 0 || t >= times[i - 1]);
      return !sameProps || !rising || times[0] !== 0 || times.at(-1) !== 1;
    });
    expect(broken.map(([name]) => name)).toEqual([]);
  });

  it("gives opacity as numbers, since motion leaves string opacities where they are", () => {
    const stringly = Object.entries(NODE_MOTION_SPECS).filter(([, spec]) =>
      spec.frames.some((f) => f.opacity !== undefined && typeof f.opacity !== "number"),
    );
    expect(stringly.map(([name]) => name)).toEqual([]);
  });
});
