import { EPSILON, round1 } from "../../shared/math";
import { SILENCE_DB } from "../../shared/mix/types";

const WAVE_FORMAT_EXTENSIBLE = 0xfffe;
const UNKNOWN_DATA_SIZE = 0xffffffff;

type Pcm = { sampleRate: number; channels: number; frames: number; data: Float32Array[] };

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
      if (format === WAVE_FORMAT_EXTENSIBLE && size >= 26) format = buf.readUInt16LE(body + 24);
    } else if (id === "data") {
      dataStart = body;
      if (size === UNKNOWN_DATA_SIZE || body + size > buf.length) size = buf.length - body;
      dataLength = size;
      break;
    }
    offset = body + size + (size % 2);
  }
  if (dataStart < 0 || !channels) throw new Error("WAV file has no audio data");
  const bytes = bits / 8;
  const frames = Math.floor(dataLength / (bytes * channels));
  const data = Array.from({ length: channels }, () => new Float32Array(frames));
  const read = sampleReader(buf, format, bits);
  for (let i = 0; i < frames; i++) {
    for (let c = 0; c < channels; c++) data[c][i] = read(dataStart + (i * channels + c) * bytes);
  }
  return { sampleRate, channels, frames, data };
}

const FLOAT_FORMAT = 3;

function sampleReader(buf: Buffer, format: number, bits: number): (at: number) => number {
  if (format === FLOAT_FORMAT)
    return bits === 64 ? (at) => buf.readDoubleLE(at) : (at) => buf.readFloatLE(at);
  if (bits === 16) return (at) => buf.readInt16LE(at) / 32768;
  if (bits === 24) return (at) => buf.readIntLE(at, 3) / 8388608;
  return (at) => buf.readInt32LE(at) / 2147483648;
}

type FrameLevels = { fps: number; peak: number[]; rms: number[] };

const toDb = (v: number) => (v <= EPSILON ? SILENCE_DB : round1(20 * Math.log10(v)));

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
        p = Math.max(p, Math.abs(v));
        sum += v * v;
        n++;
      }
    }
    peak.push(toDb(p));
    rms.push(toDb(n ? Math.sqrt(sum / n) : 0));
  }
  return { fps, peak, rms };
}
