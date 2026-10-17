import { describe, expect, it } from "vitest";
import type { TimelineSpec } from "../../shared/schema/timeline";
import { hasSoundTrack } from "./codecs";
import { finishArgs, muxArgs, pictureKey } from "./picture";

const spec: TimelineSpec = {
  width: 1920,
  height: 1080,
  fps: 30,
  durationInFrames: 300,
  tracks: [
    {
      id: "t",
      items: [
        {
          id: "i",
          from: 0,
          durationInFrames: 300,
          component: "core/title",
          props: { title: "Hi" },
        },
      ],
    },
  ],
  audio: [{ id: "a", src: "app:sfx/pop.wav", from: 10 }],
};
const opts = { codec: "h264", crf: 18, scale: 1, gl: "angle" };

describe("pictures rendered without sound", () => {
  it("are reused when only the sound or the asset server changes", () => {
    const key = pictureKey(spec, opts);
    expect(pictureKey({ ...spec, audio: [] }, opts)).toBe(key);
    expect(pictureKey({ ...spec, mix: { src: "project:m.wav", hash: "x" } }, opts)).toBe(key);
    expect(pictureKey({ ...spec, assets: { projectBase: "http://127.0.0.1:5/" } }, opts)).toBe(key);
  });

  it("render again when the picture, shape or quality changes", () => {
    const key = pictureKey(spec, opts);
    expect(pictureKey({ ...spec, width: 1080, height: 1920 }, opts)).not.toBe(key);
    expect(pictureKey({ ...spec, reframe: "center" }, opts)).not.toBe(key);
    expect(pictureKey(spec, { ...opts, crf: 23 })).not.toBe(key);
  });

  it("get the sound added with the right audio format", () => {
    expect(muxArgs("h264", "p.mp4", "m.wav", "o.mp4")).toEqual(
      expect.arrayContaining(["-c:v", "copy", "-c:a", "aac", "-shortest", "o.mp4"]),
    );
    expect(muxArgs("prores", "p.mov", "m.wav", "o.mov")).toContain("pcm_s16le");
    expect(muxArgs("vp9", "p.webm", "m.wav", "o.webm")).toContain("libopus");
    expect(hasSoundTrack("gif")).toBe(false);
  });
});

describe("finishArgs", () => {
  it("adds chapters and moves the index to the front in one copy", () => {
    expect(finishArgs("a.mp4", "b.mp4", { faststart: true, chapters: "c.txt" }).join(" ")).toBe(
      "-y -i a.mp4 -i c.txt -map_metadata 1 -map_chapters 1 -map 0 -c copy -movflags +faststart b.mp4",
    );
  });

  it("copies without chapters", () => {
    expect(finishArgs("a.webm", "b.webm", { faststart: false, chapters: null }).join(" ")).toBe(
      "-y -i a.webm -map 0 -c copy b.webm",
    );
  });
});
