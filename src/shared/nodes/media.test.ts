import { describe, expect, it } from "vitest";
import type { MediaValue } from "../schema/values";
import { compileNode } from "../test-fixtures";

const media = (kind: MediaValue["kind"], src: string): MediaValue => ({ type: "media", src, kind });

const itemOf = (type: string, params: Record<string, unknown>, inputs = {}) => {
  const clip = compileNode(type, params, inputs).clip;
  if (clip?.type !== "clip") throw new Error(`no clip from ${type}`);
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
