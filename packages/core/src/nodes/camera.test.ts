import { describe, expect, it } from "vitest";
import {
  clipFrameOfRecording,
  emptyClip,
  groupOf,
  recordingProps,
  sequenceClips,
} from "#core/compile/clip";
import type { InputTake } from "#core/nodes/card";
import { parseParams } from "#core/nodes/params";
import { getNodeDef } from "#core/nodes/registry";
import type { ClipValue } from "#core/schema/values";
import {
  compileClip,
  compileNode,
  makeLog,
  must,
  nodeCard,
  styleValue,
  takeRecording,
  zoomed,
  type StyleStepSpec,
} from "#core/test-fixtures";

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
  const ramp = (fromMs: number, toMs: number): StyleStepSpec => [
    "camera.speedRamp",
    { segments: [{ fromMs, toMs, rate: 2 }] },
  ];
  const frameOf = (clip: ClipValue, ms: number) =>
    clipFrameOfRecording(clip.tracks, () => (ms / 1000) * 30);

  it("places a second ramp at its own recording time, after the first one sped up", () => {
    const both = zoomed({ own: [ramp(1000, 2000), ramp(3000, 4000)], log: makeLog([], 10_000) });
    expect(both.durationInFrames).toBe(270);
    expect(frameOf(both, 3000)).toBeCloseTo(75);
    expect(frameOf(both, 4000)).toBeCloseTo(90);
    expect(frameOf(both, 6000)).toBeCloseTo(150);
  });

  it("adds its part to the take it's connected to", () => {
    const out = compileNode("camera.speedRamp", {}, { recording: takeRecording() }).recording;
    expect(out).toMatchObject({ type: "recording", style: [{ type: "camera.speedRamp" }] });
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
});

describe("Window frame", () => {
  it("draws the title bar it's set to", () => {
    const [props] = recordingProps(
      zoomed({ style: [["camera.windowFrame", { windowStyle: "windows", addressBar: false }]] }),
    );
    expect(props).toMatchObject({ chrome: true, windowStyle: "windows", addressBar: false });
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
  const texts = (type: string, params: Record<string, unknown>, recording: InputTake) =>
    nodeCard(type, params, { inputTakes: { recording } }).map((r) => r.text);

  it("warns when the move starts after the take ends", () => {
    expect(texts("camera.manualZoom", { atMs: 60_000 }, take)).toContain(
      "After the take ends at 6 s",
    );
    expect(texts("camera.pan", { atMs: 7000 }, take)).toContain("After the take ends at 6 s");
    expect(texts("camera.manualZoom", { atMs: 1000 }, take)).not.toContain(
      "After the take ends at 6 s",
    );
  });

  it("warns on Auto zoom when its style follows a take with no cursor", () => {
    const card = (cursorTrack: boolean, style: string[]) =>
      nodeCard(
        "camera.autoZoom",
        {},
        {
          inputTakes: { recording: { ...take, cursorTrack } },
          inputStyle: { style },
        },
      ).map((r) => r.text);
    const warning = "The take has no cursor to follow";
    expect(card(false, ["camera.followCursor"])).toContain(warning);
    expect(card(true, ["camera.followCursor"])).not.toContain(warning);
    expect(card(false, ["camera.backdrop"])).not.toContain(warning);
    const filmed = "The take films its own cursor";
    expect(card(false, ["cursor.style"])).toContain(filmed);
    expect(card(true, ["cursor.style"])).not.toContain(filmed);
  });

  it("never holds, pans or re-aims for no time at all", () => {
    expect(problem("camera.manualZoom", { holdMs: 0 })).toBe("Hold must be at least 0.1 s");
    expect(problem("camera.pan", { durationMs: 0 })).toBe("Pan for must be at least 0.1 s");
    expect(problem("camera.followCursor", { everyMs: 0 })).toBe(
      "Re-aim every must be at least 0.2 s",
    );
  });
});

describe("a recording's style", () => {
  it("lets the style beat Auto zoom's own settings, and the take's own nodes come last", () => {
    const [props] = recordingProps(
      zoomed(
        {
          style: [
            ["camera.backdrop", { kind: "solid", color: "#123456" }],
            ["camera.followCursor"],
          ],
          own: [["camera.manualZoom", { atMs: 500 }]],
        },
        { speed: 1, backdrop: "aurora" },
      ),
    );
    expect(props.backdrop).toBe("#123456");
    expect(props.backdropChosen).toBe(true);
    expect(props.extraBeats.map((b) => b.nodeId)).toEqual(["s1", "t0", "t0"]);
  });

  it("keeps Auto zoom's backdrop without a style", () => {
    const [props] = recordingProps(zoomed({}, { speed: 1, backdrop: "aurora" }));
    expect(props.backdrop).toBe("aurora");
  });

  it("styles every take in a sequence through Apply style", () => {
    const sequence = sequenceClips("seq", [zoomed(), zoomed()]);
    const styled = compileClip(
      "camera.applyStyle",
      {},
      { clip: sequence, style: styleValue([["cursor.style", { size: 2 }]]) },
    );
    expect(recordingProps(styled).map((p) => p.cursorScale)).toEqual([2, 2]);
  });

  it("warns on Apply style when the clip has no recording", () => {
    const texts = (take: InputTake) =>
      nodeCard("camera.applyStyle", {}, { inputTakes: { clip: take } }).map((r) => r.text);
    expect(texts(null)).toContain("No recording in this clip");
    expect(texts({ durationMs: 1000, cursorTrack: true })).not.toContain(
      "No recording in this clip",
    );
  });
});
