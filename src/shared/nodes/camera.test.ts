import { describe, expect, it } from "vitest";
import { clipFrameOfRecording, emptyClip, groupOf, itemClip } from "../compile/clip";
import { defaultRecordingProps } from "../compile/recording";
import type { ClipValue } from "../schema/values";
import { compileClip, makeLog, must, nodeCard } from "../test-fixtures";
import type { InputTake } from "./card";
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
    expect(problem("camera.crop", { x: 0, y: 0, w: 0, h: 0.5 })).toBe(
      "Width must be at least 0.05",
    );
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

describe("Device frame", () => {
  it("frames any clip in a laptop or phone", () => {
    const red = emptyClip(60);
    for (const device of ["laptop", "phone"]) {
      const framed = compileClip("camera.deviceFrame", { device }, { clip: red });
      expect(groupOf(framed.tracks[0].items[0])?.wrapper?.component).toBe(`snapcn/${device}-frame`);
    }
  });

  it("warns that a browser window needs a recording", () => {
    const texts = (recording: boolean) =>
      nodeCard(
        "camera.deviceFrame",
        { device: "browser" },
        { inputTakes: { clip: recording ? { durationMs: 5000, cursorTrack: true } : null } },
      ).map((r) => r.text);
    expect(texts(false)).toContain("Needs a recording; try Laptop or Phone");
    expect(texts(true)).not.toContain("Needs a recording; try Laptop or Phone");
  });
});

describe("Backdrop", () => {
  it("asks for the colour, gradient or image its kind needs", () => {
    expect(problem("camera.backdrop", { kind: "solid", color: "" })).toBe("Pick a colour");
    expect(problem("camera.backdrop", { kind: "solid", color: "red" })).toBeNull();
    expect(problem("camera.backdrop", { kind: "gradient", gradient: " " })).toBe("Pick a gradient");
    expect(problem("camera.backdrop", { kind: "asset", asset: "" })).toBe("Choose an image");
    expect(problem("camera.backdrop", { kind: "image" })).toBeNull();
  });
});

describe("Zoom here, Pan and Follow cursor", () => {
  const take = { durationMs: 6000, cursorTrack: true };
  const texts = (type: string, params: Record<string, unknown>, clip: InputTake) =>
    nodeCard(type, params, { inputTakes: { clip } }).map((r) => r.text);

  it("warns when the move starts after the take ends", () => {
    expect(texts("camera.manualZoom", { atMs: 60_000 }, take)).toContain(
      "After the take ends at 6 s",
    );
    expect(texts("camera.pan", { atMs: 7000 }, take)).toContain("After the take ends at 6 s");
    expect(texts("camera.manualZoom", { atMs: 1000 }, take)).not.toContain(
      "After the take ends at 6 s",
    );
  });

  it("warns when there is no recording to move over", () => {
    for (const type of ["camera.manualZoom", "camera.pan", "camera.followCursor"])
      expect(texts(type, {}, null)).toContain("Needs a recording");
  });

  it("warns when the take has no cursor to follow", () => {
    expect(texts("camera.followCursor", {}, { ...take, cursorTrack: false })).toContain(
      "The take has no cursor to follow",
    );
  });

  it("never holds, pans or re-aims for no time at all", () => {
    expect(problem("camera.manualZoom", { holdMs: 0 })).toBe("Hold must be at least 0.1 s");
    expect(problem("camera.pan", { durationMs: 0 })).toBe("Pan for must be at least 0.1 s");
    expect(problem("camera.followCursor", { everyMs: 0 })).toBe(
      "Re-aim every must be at least 0.2 s",
    );
  });
});
