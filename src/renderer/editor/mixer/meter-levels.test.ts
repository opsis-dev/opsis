import { describe, expect, it } from "vitest";
import type { MixResult } from "@/shared/mix/result";
import { levelAt, reductionAt, sampleAt, share } from "./meter-levels";

const withMaster = (rms: number[]): MixResult => ({
  key: "k",
  master: null,
  levels: { channels: {}, buses: {}, master: { fps: 30, peak: rms, rms } },
  reduction: { channels: {}, master: null },
  loudness: null,
});

describe("meter levels", () => {
  it("samples the nearest frame inside the list", () => {
    expect(sampleAt([1, 2, 3], 1.4)).toBe(2);
    expect(sampleAt([1, 2, 3], 9)).toBe(3);
    expect(sampleAt([], 0)).toBeNull();
    expect(reductionAt(null, 0)).toBeNull();
    expect(reductionAt([], 0)).toBe(0);
  });

  it("gives a channel's share of the master's power", () => {
    const result = withMaster([-10, -10]);
    const channel = { peak: [-4, -14], rms: [-10, -20] };
    expect(share(result, channel, 0)).toBeCloseTo(1);
    expect(share(result, channel, 1)).toBeCloseTo(0.1);
    expect(share(null, channel, 0)).toBe(1);
    expect(share(result, undefined, 0)).toBe(1);
    expect(share(withMaster([-150]), channel, 0)).toBe(1);
  });

  it("reads peak, average and the held peak together", () => {
    const levels = { peak: [-3, -1, -6], rms: [-9, -7, -12] };
    expect(levelAt(levels, 2, 1)).toEqual({ peak: -6, rms: -12, hold: -1 });
    expect(levelAt({ peak: [], rms: [] }, 0, 1)).toBeNull();
  });
});
