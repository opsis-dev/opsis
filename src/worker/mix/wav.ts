import { readFile } from "node:fs/promises";

export type Pcm = { sampleRate: number; channels: number; frames: number; data: Float32Array[] };

export function parseWav(buf: Buffer): Pcm {
  if (buf.toString("ascii", 0, 4) !== "RIFF" || buf.toString("ascii", 8, 12) !== "WAVE")
    throw new Error("Not a WAV file");
  let offset = 12;
  let format = 0;
  let channels = 0;
  let sampleRate = 0;
  let bits = 0;
  let dataStart = -1;
  let dataLength = 0;
  while (offset + 8 <= buf.length) {
    const id = buf.toString("ascii", offset, offset + 4);
    let size = buf.readUInt32LE(offset + 4);
    const body = offset + 8;
    if (id === "fmt ") {
      format = buf.readUInt16LE(body);
      channels = buf.readUInt16LE(body + 2);
      sampleRate = buf.readUInt32LE(body + 4);
      bits = buf.readUInt16LE(body + 14);
      if (format === 0xfffe && size >= 26) format = buf.readUInt16LE(body + 24);
    } else if (id === "data") {
      dataStart = body;
      if (size === 0xffffffff || body + size > buf.length) size = buf.length - body;
      dataLength = size;
      break;
    }
    offset = body + size + (size % 2);
  }
  if (dataStart < 0 || !channels) throw new Error("WAV file has no audio data");
  const bytes = bits / 8;
  const frames = Math.floor(dataLength / (bytes * channels));
  const data = Array.from({ length: channels }, () => new Float32Array(frames));
  for (let i = 0; i < frames; i++) {
    for (let c = 0; c < channels; c++) {
      const at = dataStart + (i * channels + c) * bytes;
      data[c][i] =
        format === 3
          ? bits === 64
            ? buf.readDoubleLE(at)
            : buf.readFloatLE(at)
          : bits === 16
            ? buf.readInt16LE(at) / 32768
            : bits === 24
              ? (buf.readIntLE(at, 3) as number) / 8388608
              : buf.readInt32LE(at) / 2147483648;
    }
  }
  return { sampleRate, channels, frames, data };
}

export const readWav = async (file: string) => parseWav(await readFile(file));

export type FrameLevels = { fps: number; peak: number[]; rms: number[] };

const toDb = (v: number) => (v <= 1e-6 ? -120 : Math.round(20 * Math.log10(v) * 10) / 10);

export function frameLevels(pcm: Pcm, fps: number, frames: number): FrameLevels {
  const per = pcm.sampleRate / fps;
  const peak: number[] = [];
  const rms: number[] = [];
  for (let f = 0; f < frames; f++) {
    const a = Math.floor(f * per);
    const b = Math.min(pcm.frames, Math.floor((f + 1) * per));
    let p = 0;
    let sum = 0;
    let n = 0;
    for (let i = a; i < b; i++) {
      for (let c = 0; c < pcm.channels; c++) {
        const v = pcm.data[c][i];
        const abs = v < 0 ? -v : v;
        if (abs > p) p = abs;
        sum += v * v;
        n++;
      }
    }
    peak.push(toDb(p));
    rms.push(toDb(n ? Math.sqrt(sum / n) : 0));
  }
  return { fps, peak, rms };
}
