import { describe, expect, it } from "vitest";
import type { MediaValue } from "../schema/values";
import { compileClip, nodeCard } from "../test-fixtures";
import { parseNodeParams } from "./define";
import { importMediaNode, mediaTrimNode } from "./media";
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

describe("Media file settings", () => {
  it("never shows an image for no time at all", () => {
    expect(parseParams(importMediaNode.params, { path: "a.png", stillMs: 0 })).toEqual({
      ok: false,
      problem: "Length is too small",
    });
  });
});

describe("Convert card", () => {
  it("shows quality only for formats that use it", () => {
    const texts = (codec: string) => nodeCard("media.convert", { codec }).map((r) => r.text);
    expect(texts("h264")).toEqual(["MP4", "High quality"]);
    expect(texts("prores")).toEqual(["ProRes"]);
  });
});

describe("Join files card", () => {
  it("counts the files and asks for what's missing", () => {
    const text = (connected: string[]) => nodeCard("media.concat", {}, { connected })[0].text;
    expect(text([])).toBe("Connect two files");
    expect(text(["in1"])).toBe("Connect one more file");
    expect(text(["in1", "in2", "in3"])).toBe("3 files");
  });
});

describe("media cards", () => {
  it("sums up what each media node does", () => {
    const cards = Object.fromEntries(
      [
        ["media.import", { path: "assets/a.mp4" }],
        ["media.trim", { startMs: 1000, endMs: 4000 }],
        ["media.speed", { rate: 2, keepAudio: false }],
        ["media.scalePad", { width: 1080, height: 1920, mode: "crop" }],
        ["media.gif", { width: 640, fps: 12, loop: false }],
        ["media.extractAudio", { format: "wav" }],
        ["media.thumbnail", { atMs: 2500, width: 640 }],
      ].map(([type, params]) => [type, nodeCard(String(type), Object(params)).map((r) => r.text)]),
    );
    expect(cards).toEqual({
      "media.import": ["a.mp4"],
      "media.trim": ["1 s → 4 s"],
      "media.speed": ["2×", "No sound"],
      "media.scalePad": ["1080 × 1920", expect.any(String)],
      "media.gif": ["640 px · 12 fps"],
      "media.extractAudio": ["WAV"],
      "media.thumbnail": ["At 2.5 s · 640 px"],
    });
  });
});

describe("media paths with variables", () => {
  it("fills a variable into a file path before reading the file", () => {
    const parsed = parseNodeParams(
      importMediaNode,
      { params: { path: "{{Folder}}/intro.mp4" } },
      { Folder: "assets/clips" },
    );
    expect(parsed.path).toBe("assets/clips/intro.mp4");
  });
});
