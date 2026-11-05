import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  FFMPEG_PINS,
  FFMPEG_RELEASE,
  FFMPEG_TARGETS,
  ffmpegLicenceUrl,
  ffmpegProgramUrl,
  ffprobeProgramUrl,
  readFfmpegPins,
  sidecarFileName,
} from "./ffmpeg";
import { generatedJson } from "./pins";
import { ROOT } from "./tool-config";

describe("the pinned FFmpeg", () => {
  it("names the sidecar the way Tauri looks for it", () => {
    expect(sidecarFileName("x86_64-unknown-linux-gnu")).toBe(
      "opsis-ffmpeg-x86_64-unknown-linux-gnu",
    );
    expect(sidecarFileName("x86_64-pc-windows-msvc")).toBe(
      "opsis-ffmpeg-x86_64-pc-windows-msvc.exe",
    );
  });

  it("pins a program, its ffprobe and its licence for every target, at the addresses built today", () => {
    const pins = readFfmpegPins();
    expect(pins.release).toBe(FFMPEG_RELEASE);
    expect(pins.targets.map((pin) => [pin.target, pin.asset])).toEqual(
      FFMPEG_TARGETS.map(([target, asset]) => [target, asset]),
    );
    for (const pin of pins.targets) {
      expect(pin.program.url).toBe(ffmpegProgramUrl(pin.asset));
      expect(pin.probe.url).toBe(ffprobeProgramUrl(pin.asset));
      expect(pin.licence.url).toBe(ffmpegLicenceUrl(pin.asset));
    }
  });

  it("is committed in the form the pin script writes", () => {
    const text = readFileSync(FFMPEG_PINS, "utf8");
    expect(generatedJson(readFfmpegPins())).toBe(text);
  });

  it("is the release the source notice links", () => {
    const notices = readFileSync(path.join(ROOT, "THIRD_PARTY_NOTICES.md"), "utf8");
    expect(notices).toContain(
      `https://github.com/eugeneware/ffmpeg-static/releases/tag/${readFfmpegPins().release}`,
    );
  });
});
