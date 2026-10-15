import { describe, expect, it } from "vitest";
import { frameEvents, jpegSize } from "./inject";

describe("recording frames in traces", () => {
  it("reads the size of a JPEG", () => {
    const sof = [
      0xff, 0xd8, 0xff, 0xe0, 0x00, 0x04, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x02, 0x1c,
      0x03, 0xc0,
    ];
    expect(jpegSize(new Uint8Array(sof))).toEqual({ width: 960, height: 540 });
    expect(jpegSize(new Uint8Array([1, 2, 3]))).toBeNull();
  });

  it("places frames on the trace clock", () => {
    const trace = [
      JSON.stringify({ type: "context-options", wallTime: 1_000_000, monotonicTime: 500 }),
      JSON.stringify({ type: "before", callId: "c@1", startTime: 900 }),
    ].join("\n");
    const lines = frameEvents(
      trace,
      [
        { file: "screencast/demoflow-00000.jpeg", width: 960, height: 540, atMs: 0 },
        { file: "screencast/demoflow-00001.jpeg", width: 960, height: 540, atMs: 250 },
      ],
      1_002_000,
    ).map((l) => JSON.parse(l) as { timestamp: number; type: string; file: string });
    expect(lines.map((l) => [l.type, l.timestamp])).toEqual([
      ["screencast-frame", 2500],
      ["screencast-frame", 2750],
    ]);
  });

  it("adds nothing when the trace has no clock", () => {
    expect(frameEvents("", [{ file: "x", width: 1, height: 1, atMs: 0 }], 0)).toEqual([]);
  });
});
