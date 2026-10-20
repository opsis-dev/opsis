import { describe, expect, it } from "vitest";
import type { MediaValue } from "../schema/values";
import { compileClip } from "../test-fixtures";
import { mediaTrimNode } from "./media";
import { parseParams } from "./params";

const media = (kind: MediaValue["kind"], src: string): MediaValue => ({ type: "media", src, kind });

const itemOf = (type: string, params: Record<string, unknown>, inputs = {}) => {
  const clip = compileClip(type, params, inputs);
  return { item: clip.tracks[0].items[0], frames: clip.durationInFrames };
};

describe("media clips", () => {
  it("shows a sound file as a placeholder in Use as clip, like Media file", () => {
    const sound = { media: media("audio", "project:assets/voice.mp3") };
    expect(itemOf("media.toClip", {}, sound).item.component).toBe("core/placeholder");
    expect(itemOf("media.import", { path: "assets/voice.mp3" }).item.component).toBe(
      "core/placeholder",
    );
  });

  it("gives an image the same default length in both nodes", () => {
    const image = { media: media("image", "project:assets/shot.png") };
    expect(itemOf("media.toClip", {}, image).frames).toBe(
      itemOf("media.import", { path: "assets/shot.png" }).frames,
    );
  });
});

describe("Trim file settings", () => {
  const problem = (params: Record<string, unknown>) => {
    const parsed = parseParams(mediaTrimNode.params, params);
    return parsed.ok ? null : parsed.problem;
  };

  it("needs the end after the start, unless it trims to the end", () => {
    expect(problem({ startMs: 2000, endMs: 1000 })).toBe("End must come after Start");
    expect(problem({ startMs: 2000, endMs: 2000 })).toBe("End must come after Start");
    expect(problem({ startMs: 2000, endMs: 0 })).toBeNull();
  });

  it("lets the start go as far into a file as the end can", () => {
    expect(problem({ startMs: 700_000, endMs: 800_000 })).toBeNull();
  });
});
