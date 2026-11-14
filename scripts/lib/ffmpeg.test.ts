import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { ffmpegReleaseUrl, readFfmpegPins, sidecarFileName } from "./ffmpeg";
import { ROOT } from "./tool-config";

describe("the pinned FFmpeg", () => {
  it("names the staged program after its target", () => {
    expect(sidecarFileName("x86_64-unknown-linux-gnu")).toBe(
      "opsis-ffmpeg-x86_64-unknown-linux-gnu",
    );
    expect(sidecarFileName("x86_64-pc-windows-msvc")).toBe(
      "opsis-ffmpeg-x86_64-pc-windows-msvc.exe",
    );
  });

  it("pins a program, its ffprobe and its licence for every target from the same release", () => {
    const pins = readFfmpegPins();
    for (const pin of pins.targets) {
      for (const file of [pin.program, pin.probe, pin.licence])
        expect(file.url).toContain(`/releases/download/${pins.release}/`);
      expect(pin.program.url).toContain(`ffmpeg-${pin.asset}.gz`);
      expect(pin.probe.url).toContain(`ffprobe-${pin.asset}.gz`);
      expect(pin.licence.url).toContain(`${pin.asset}.LICENSE`);
    }
  });

  it("is the release the source notice links", () => {
    const notices = readFileSync(path.join(ROOT, "THIRD_PARTY_NOTICES.md"), "utf8");
    expect(notices).toContain(ffmpegReleaseUrl(readFfmpegPins().release));
  });
});
