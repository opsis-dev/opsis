import { describe, expect, it } from "vitest";
import { soundLength } from "../audio";
import { emptyClip } from "../compile/clip";
import type { AudioItem } from "../schema/timeline";
import { BUILTIN_SOUNDS } from "../sounds";
import { compileClip, compileNode, nodeCard } from "../test-fixtures";

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

describe("sound from a connected file", () => {
  const silentVideo = {
    type: "media",
    src: "project:a.mp4",
    kind: "video",
    hasAudio: false,
  } as const;
  const image = { type: "media", src: "project:a.png", kind: "image" } as const;

  it("adds no sound from a file that has none", () => {
    for (const media of [silentVideo, image])
      for (const type of ["audio.music", "audio.sfx"]) {
        const audio = compileNode(type, {}, { media }).audio;
        expect(audio?.type === "audio" ? audio.items : null).toEqual([]);
      }
  });

  it("says so on the card", () => {
    const text = (media: typeof silentVideo | typeof image) =>
      nodeCard("audio.music", {}, { connected: ["media"], inputMedia: { media } })[0].text;
    expect(text(silentVideo)).toBe("The connected file has no sound");
    expect(text(image)).toBe("The connected file has no sound");
  });
});

describe("chosen sound and a connected file", () => {
  it("says the connected file plays instead of the chosen sound", () => {
    const tone = { type: "media", src: "project:tone.wav", kind: "audio", hasAudio: true } as const;
    const [first] = nodeCard(
      "audio.sfx",
      { src: BUILTIN_SOUNDS[0].ref },
      { connected: ["media"], inputMedia: { media: tone } },
    );
    expect(first).toMatchObject({
      text: `Plays the connected file, not “${BUILTIN_SOUNDS[0].label}”`,
      tone: "warn",
    });
  });
});
