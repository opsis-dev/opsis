import { describe, expect, it } from "vitest";
import { editedFile, ffmpegSeconds, hasSoundTrack } from "./codecs";

describe("codecs", () => {
  it("knows which formats carry sound", () => {
    expect(hasSoundTrack("gif")).toBe(false);
    expect(hasSoundTrack("h264")).toBe(true);
  });

  it("writes times as seconds with millisecond precision", () => {
    expect(ffmpegSeconds(1500)).toBe("1.500");
    expect(ffmpegSeconds(33.4)).toBe("0.033");
  });
});

describe("editedFile", () => {
  it("writes moving pictures as MP4, with sound only when there is some", () => {
    expect(editedFile(".webm", { video: true, audio: true })?.ext).toBe(".mp4");
    expect(editedFile(".gif", { video: true, audio: false })?.codecs).toContain("-an");
  });

  it("keeps a sound file's own type when it can, as real audio of that type", () => {
    expect(editedFile(".wav", { video: false, audio: true })).toEqual({
      ext: ".wav",
      codecs: ["-vn", "-c:a", "pcm_s16le"],
    });
    expect(editedFile(".mp3", { video: false, audio: true })?.codecs).toContain("libmp3lame");
    expect(editedFile(".ogg", { video: false, audio: true })?.ext).toBe(".m4a");
  });

  it("has nothing to make from a still image", () => {
    expect(editedFile(".png", { video: false, audio: false })).toBeNull();
  });
});
