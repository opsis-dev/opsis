import { describe, expect, it } from "vitest";
import { lastRunSchema, nodeResultSchema } from "./values";

const result = (outputs: Record<string, unknown>) => ({
  hash: "h",
  outputs,
  artifacts: [],
  finishedAt: 0,
  durationMs: 0,
});

describe("saved node results", () => {
  it("reads every saved output kind", () => {
    const outputs = {
      media: { type: "media", src: "project:a.mp4", kind: "video", durationMs: 1200 },
      image: { type: "image", src: null, width: 640, height: 360, scale: 2 },
      info: { type: "file-info", src: "project:probe.json" },
      folder: { type: "folder", src: "project:video-code" },
      recording: {
        type: "recording",
        src: "project:r.mp4",
        log: null,
        durationMs: 5000,
        viewport: { width: 1920, height: 1080 },
        nodeId: "record",
        style: [{ nodeId: "zoom", type: "camera.manualZoom", params: { atMs: 500 } }],
      },
    };
    expect(nodeResultSchema.parse(result(outputs)).outputs).toEqual(outputs);
  });

  it("rejects an output whose fields don't match its kind", () => {
    expect(nodeResultSchema.safeParse(result({ media: { type: "media", src: 3 } })).success).toBe(
      false,
    );
    expect(nodeResultSchema.safeParse(result({ x: { type: "unknown" } })).success).toBe(false);
  });
});

describe("lastRunSchema", () => {
  it("keeps the readable node results when one entry is damaged", () => {
    const parsed = lastRunSchema.parse({
      updatedAt: 1,
      nodes: { good: result({}), bad: result({ info: { type: "file-info" } }) },
    });
    expect(Object.keys(parsed.nodes)).toEqual(["good"]);
  });
});
