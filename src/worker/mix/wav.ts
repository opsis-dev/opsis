import { EPSILON, forEachWindow, round1 } from "../../shared/math";
import { SILENCE_DB } from "../../shared/mix/types";

const WAVE_FORMAT_EXTENSIBLE = 0xfffe;
const FLOAT_FORMAT = 3;
const UNKNOWN_DATA_SIZE = 0xffffffff;

type Pcm = { sampleRate: number; channels: number; frames: number; data: Float32Array[] };

type WavFormat = { format: number; channels: number; sampleRate: number; bits: number };

type WavLayout = WavFormat & { dataStart: number; dataLength: number };

function readFormat(buf: Buffer, body: number, size: number): WavFormat {
  const format = buf.readUInt16LE(body);
  return {
    format: format === WAVE_FORMAT_EXTENSIBLE && size >= 26 ? buf.readUInt16LE(body + 24) : format,
    channels: buf.readUInt16LE(body + 2),
    sampleRate: buf.readUInt32LE(body + 4),
    bits: buf.readUInt16LE(body + 14),
  };
}

function readChunks(buf: Buffer): WavLayout {
  if (buf.toString("ascii", 0, 4) !== "RIFF" || buf.toString("ascii", 8, 12) !== "WAVE")
    throw new Error("Not a WAV file");
  let fmt: WavFormat | null = null;
  for (let offset = 12; offset + 8 <= buf.length;) {
    const id = buf.toString("ascii", offset, offset + 4);
    const size = buf.readUInt32LE(offset + 4);
    const body = offset + 8;
    if (id === "data") {
      if (!fmt?.channels) break;
      const whole = size !== UNKNOWN_DATA_SIZE && body + size <= buf.length;
      return { ...fmt, dataStart: body, dataLength: whole ? size : buf.length - body };
    }
    if (id === "fmt ") fmt = readFormat(buf, body, size);
    offset = body + size + (size % 2);
  }
  throw new Error("WAV file has no audio data");
}

function sampleReader(buf: Buffer, format: number, bits: number): (at: number) => number {
  if (format === FLOAT_FORMAT)
    return bits === 64 ? (at) => buf.readDoubleLE(at) : (at) => buf.readFloatLE(at);
  if (bits === 16) return (at) => buf.readInt16LE(at) / 32768;
  if (bits === 24) return (at) => buf.readIntLE(at, 3) / 8388608;
  return (at) => buf.readInt32LE(at) / 2147483648;
}

function decode(buf: Buffer, layout: WavLayout): Pcm {
  const { format, channels, sampleRate, bits, dataStart, dataLength } = layout;
  const bytes = bits / 8;
  const frames = Math.floor(dataLength / (bytes * channels));
  const data = Array.from({ length: channels }, () => new Float32Array(frames));
  const read = sampleReader(buf, format, bits);
  for (let i = 0; i < frames; i++) {
    for (let c = 0; c < channels; c++) data[c][i] = read(dataStart + (i * channels + c) * bytes);
  }
  return { sampleRate, channels, frames, data };
}

export const parseWav = (buf: Buffer): Pcm => decode(buf, readChunks(buf));

type FrameLevels = { fps: number; peak: number[]; rms: number[] };

const toDb = (v: number) => (v <= EPSILON ? SILENCE_DB : round1(20 * Math.log10(v)));

export function frameLevels(pcm: Pcm, fps: number, frames: number): FrameLevels {
  const per = pcm.sampleRate / fps;
  const peak: number[] = [];
  const rms: number[] = [];
  forEachWindow(frames, per, pcm.frames, (a, b) => {
    let p = 0;
    let sum = 0;
    let n = 0;
    for (let i = a; i < b; i++) {
      for (let c = 0; c < pcm.channels; c++) {
        const v = pcm.data[c][i];
        p = Math.max(p, Math.abs(v));
        sum += v * v;
        n++;
      }
    }
    peak.push(toDb(p));
    rms.push(toDb(n ? Math.sqrt(sum / n) : 0));
  });
  return { fps, peak, rms };
}
