import { describe, expect, it } from "vitest";
import { frameLevels, parseWav } from "./wav";

function wav(format: number, bits: number, channels: number, samples: Buffer): Buffer {
  const header = Buffer.alloc(44);
  header.write("RIFF", 0, "ascii");
  header.writeUInt32LE(36 + samples.length, 4);
  header.write("WAVE", 8, "ascii");
  header.write("fmt ", 12, "ascii");
  header.writeUInt32LE(16, 16);
  header.writeUInt16LE(format, 20);
  header.writeUInt16LE(channels, 22);
  header.writeUInt32LE(8000, 24);
  header.writeUInt32LE((8000 * channels * bits) / 8, 28);
  header.writeUInt16LE((channels * bits) / 8, 32);
  header.writeUInt16LE(bits, 34);
  header.write("data", 36, "ascii");
  header.writeUInt32LE(samples.length, 40);
  return Buffer.concat([header, samples]);
}

describe("parseWav", () => {
  it("reads 16-bit stereo as one channel per array", () => {
    const samples = Buffer.alloc(8);
    samples.writeInt16LE(16384, 0);
    samples.writeInt16LE(-32768, 2);
    samples.writeInt16LE(0, 4);
    samples.writeInt16LE(8192, 6);
    const pcm = parseWav(wav(1, 16, 2, samples));
    expect(pcm.sampleRate).toBe(8000);
    expect(pcm.frames).toBe(2);
    expect([...pcm.data[0]]).toEqual([0.5, 0]);
    expect([...pcm.data[1]]).toEqual([-1, 0.25]);
  });

  it("reads 24-bit and float samples", () => {
    const pcm24 = Buffer.alloc(3);
    pcm24.writeIntLE(-4194304, 0, 3);
    expect([...parseWav(wav(1, 24, 1, pcm24)).data[0]]).toEqual([-0.5]);
    const float = Buffer.alloc(4);
    float.writeFloatLE(0.75, 0);
    expect([...parseWav(wav(3, 32, 1, float)).data[0]]).toEqual([0.75]);
  });

  it("refuses files that aren't WAV", () => {
    expect(() => parseWav(Buffer.from("not a wav file at all"))).toThrow("Not a WAV file");
  });
});

describe("frameLevels", () => {
  it("measures peak and loudness per video frame", () => {
    const data = [Float32Array.from([0.5, -1, 0, 0])];
    const levels = frameLevels({ sampleRate: 4, channels: 1, frames: 4, data }, 2, 2);
    expect(levels.peak).toEqual([0, -120]);
    expect(levels.rms).toEqual([-2, -120]);
  });
});
