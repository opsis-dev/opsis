import { describe, expect, it } from "vitest";
import { canStreamCopy, parseProbe } from "./ffmpeg";

const recording = {
  format: { duration: "12.480000", format_name: "mov,mp4,m4a,3gp,3g2,mj2" },
  streams: [
    {
      codec_type: "video",
      codec_name: "h264",
      width: 1920,
      height: 1080,
      r_frame_rate: "30/1",
      avg_frame_rate: "30000/1001",
      pix_fmt: "yuv420p",
      duration: "12.466667",
    },
    {
      codec_type: "audio",
      codec_name: "aac",
      sample_rate: "48000",
      channels: 2,
      duration: "12.480000",
    },
  ],
};

describe("parseProbe", () => {
  it("reads length, picture and sound from the probe output", () => {
    const probe = parseProbe(recording);
    expect(probe.durationMs).toBe(12480);
    expect(probe.format).toBe("mov,mp4,m4a,3gp,3g2,mj2");
    expect(probe.video).toEqual({
      codec: "h264",
      width: 1920,
      height: 1080,
      fps: 30000 / 1001,
      pixFmt: "yuv420p",
    });
    expect(probe.audio).toEqual({ codec: "aac", sampleRate: 48000, channels: 2 });
  });

  it("falls back to the stream length and the base frame rate", () => {
    const probe = parseProbe({
      streams: [{ codec_type: "video", r_frame_rate: "25", avg_frame_rate: "0/0", duration: "2" }],
    });
    expect(probe.durationMs).toBe(2000);
    expect(probe.video?.fps).toBe(25);
    expect(probe.audio).toBeUndefined();
  });

  it("reads a still image as no length", () => {
    const probe = parseProbe({
      format: { format_name: "png_pipe" },
      streams: [{ codec_type: "video", codec_name: "png", width: 10, height: 10 }],
    });
    expect(probe.durationMs).toBe(0);
    expect(probe.video?.fps).toBe(0);
  });
});

describe("canStreamCopy", () => {
  const base = parseProbe(recording);

  it("joins files that match without converting", () => {
    expect(canStreamCopy(base, parseProbe(recording))).toBe(true);
  });

  it("converts when the picture or sound differs", () => {
    const resized = parseProbe({
      ...recording,
      streams: [{ ...recording.streams[0], width: 1280 }, recording.streams[1]],
    });
    const silent = parseProbe({ ...recording, streams: [recording.streams[0]] });
    expect(canStreamCopy(base, resized)).toBe(false);
    expect(canStreamCopy(base, silent)).toBe(false);
    expect(canStreamCopy(parseProbe({ streams: [] }), base)).toBe(false);
  });
});
