import { describe, expect, it } from "vitest";
import { clipFrameOfRecording, itemClip } from "../compile/clip";
import { defaultRecordingProps } from "../compile/recording";
import type { ClipValue } from "../schema/values";
import { compileClip, makeLog, must } from "../test-fixtures";
import { parseParams } from "./params";
import { getNodeDef } from "./registry";

const problem = (type: string, params: Record<string, unknown>) => {
  const parsed = parseParams(must(getNodeDef(type), type).params, params);
  return parsed.ok ? null : parsed.problem;
};

describe("Crop", () => {
  it("keeps the area inside the recording and big enough to see", () => {
    expect(problem("camera.crop", { x: 0, y: 0, w: 0.5, h: 0.25 })).toBeNull();
    expect(problem("camera.crop", { x: 0.8, y: 0, w: 0.5, h: 0.5 })).toBe(
      "The area runs past the right edge",
    );
    expect(problem("camera.crop", { x: 0, y: 0.6, w: 0.5, h: 0.5 })).toBe(
      "The area runs past the bottom edge",
    );
    expect(problem("camera.crop", { x: 0, y: 0, w: 0, h: 0.5 })).toBe("Width is too small");
  });
});

describe("Speed ramp", () => {
  const take = (): ClipValue => {
    const log = makeLog([], 10_000);
    const props = { ...defaultRecordingProps(), speed: 1, src: "rec.mp4", log, name: "Take" };
    return itemClip("rec", "core/recording", props, 300, {
      recording: {
        type: "recording",
        src: "rec.mp4",
        log,
        durationMs: 10_000,
        nodeId: "rec",
        viewport: log.viewport,
      },
    });
  };
  const ramp = (clip: ClipValue, fromMs: number, toMs: number) =>
    compileClip("camera.speedRamp", { segments: [{ fromMs, toMs, rate: 2 }] }, { clip });
  const frameOf = (clip: ClipValue, ms: number) =>
    clipFrameOfRecording(clip.tracks, () => (ms / 1000) * 30);

  it("places a second ramp at its own recording time, after the first one sped up", () => {
    const both = ramp(ramp(take(), 1000, 2000), 3000, 4000);
    expect(both.durationInFrames).toBe(270);
    expect(frameOf(both, 3000)).toBeCloseTo(75);
    expect(frameOf(both, 4000)).toBeCloseTo(90);
    expect(frameOf(both, 6000)).toBeCloseTo(150);
  });
});
