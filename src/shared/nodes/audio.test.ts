import { describe, expect, it } from "vitest";
import { soundLength } from "../audio";
import { emptyClip } from "../compile/clip";
import type { AudioItem } from "../schema/timeline";
import { BUILTIN_SOUNDS } from "../sounds";
import { compileClip, compileNode } from "../test-fixtures";

const sfxItem = (params: Record<string, unknown>): AudioItem => {
  const audio = compileNode("audio.sfx", params).audio;
  if (audio?.type !== "audio") throw new Error("no sound");
  return audio.items[0];
};

describe("Sound effect length", () => {
  it("uses the length the Inspector measured for your own file", () => {
    expect(sfxItem({ src: "assets/tone.wav", srcMs: 3000 }).lengthFrames).toBe(90);
  });

  it("plays a file of unknown length to the end of the film instead of cutting it", () => {
    const item = sfxItem({ src: "assets/tone.wav" });
    expect(item.lengthFrames).toBeUndefined();
    expect(soundLength(item, 150)).toBe(150);
  });

  it("knows the length of built-in sounds", () => {
    expect(sfxItem({ src: BUILTIN_SOUNDS[0].ref }).lengthFrames).toBeGreaterThan(0);
  });
});

describe("Add sound", () => {
  it("leaves out a sound that starts at or after the clip's end", () => {
    const sounds = (atMs: number) => {
      const audio = compileNode("audio.sfx", { src: BUILTIN_SOUNDS[0].ref, atMs }).audio;
      const clip = compileClip("audio.mix", {}, { clip: emptyClip(90), sounds: audio });
      return clip.audio.length;
    };
    expect(sounds(1000)).toBe(1);
    expect(sounds(3000)).toBe(0);
    expect(sounds(5000)).toBe(0);
  });
});
