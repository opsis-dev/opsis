import { describe, expect, it } from "vitest";
import { canConnect, convertInput, portLabel } from "#core/nodes/ports";
import type { CompileValue } from "#core/schema/values";

const recording: CompileValue = {
  type: "recording",
  src: "project:take.mp4",
  log: null,
  durationMs: 4000,
  viewport: { width: 1280, height: 720 },
  nodeId: "rec",
  style: [],
};

describe("canConnect", () => {
  it("takes the same resource", () => {
    expect(canConnect("browser-session", "browser-session")).toBe(true);
    expect(canConnect("recording-style", "recording-style")).toBe(true);
  });

  it("takes an image or a recording where a file goes", () => {
    expect(canConnect("image", "media")).toBe(true);
    expect(canConnect("recording", "media")).toBe(true);
  });

  it("keeps a file out of sounds, and info and folders out of files", () => {
    expect(canConnect("media", "audio")).toBe(false);
    expect(canConnect("file-info", "media")).toBe(false);
    expect(canConnect("folder", "media")).toBe(false);
    expect(canConnect("recording", "clip")).toBe(false);
  });
});

describe("convertInput", () => {
  it("hands a recording to a file input as its video", () => {
    expect(convertInput(recording, { type: "media" })).toEqual({
      type: "media",
      kind: "video",
      src: "project:take.mp4",
      durationMs: 4000,
      width: 1280,
      height: 720,
      hasAudio: false,
    });
  });

  it("hands an image to a file input as an image file", () => {
    expect(
      convertInput({ type: "image", src: "project:s.png", width: 3, height: 2 }, { type: "media" }),
    ).toEqual({ type: "media", kind: "image", src: "project:s.png", width: 3, height: 2 });
  });

  it("passes a value of the port's own type through and drops one it can't take", () => {
    expect(convertInput(recording, { type: "recording" })).toBe(recording);
    expect(convertInput(recording, { type: "clip" })).toBeUndefined();
  });
});

describe("portLabel", () => {
  it("names ports after their resource", () => {
    expect(portLabel({ type: "audio" })).toBe("Sound");
    expect(portLabel({ type: "recording-style" })).toBe("Style");
  });

  it("prefers the port's own label", () => {
    expect(portLabel({ type: "clip", label: "Intro" })).toBe("Intro");
  });
});
