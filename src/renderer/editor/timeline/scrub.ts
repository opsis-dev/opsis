import { errorMessage } from "@/shared/errors";
import { clamp } from "@/shared/math";
import { fileUrl } from "../file-url";

const CACHED_SOURCES = 3;
const MIN_INTERVAL_MS = 45;
const BLIP_S = 0.09;
const BLIP_GAIN = 0.9;
const END_MARGIN_S = 0.1;

const buffers = new Map<string, Promise<AudioBuffer | null>>();
let context: AudioContext | null = null;
let last = 0;
let current: AudioBufferSourceNode | null = null;

function decode(ctx: AudioContext, src: string): Promise<AudioBuffer | null> {
  return fetch(fileUrl(src))
    .then((r) => r.arrayBuffer())
    .then((data) => ctx.decodeAudioData(data))
    .catch((error: unknown) => {
      buffers.delete(src);
      console.warn(`Scrub sound unavailable for ${src}: ${errorMessage(error)}`);
      return null;
    });
}

function audioContext(): AudioContext {
  context ??= new AudioContext();
  return context;
}

function load(src: string): Promise<AudioBuffer | null> {
  let buffer = buffers.get(src);
  if (!buffer) {
    buffer = decode(audioContext(), src);
    buffers.set(src, buffer);
    const [oldest] = buffers.keys();
    if (buffers.size > CACHED_SOURCES && oldest !== undefined) buffers.delete(oldest);
  }
  return buffer;
}

export function preloadScrub(src: string | null): void {
  if (src) void load(src);
}

export function scrubAt(src: string | null, seconds: number): void {
  if (!src) return;
  const now = performance.now();
  if (now - last < MIN_INTERVAL_MS) return;
  last = now;
  const ctx = audioContext();
  void load(src).then((buffer) => {
    if (!buffer) return;
    if (ctx.state === "suspended") void ctx.resume();
    current?.stop();
    const node = ctx.createBufferSource();
    const gain = ctx.createGain();
    node.buffer = buffer;
    gain.gain.setValueAtTime(BLIP_GAIN, ctx.currentTime);
    gain.gain.linearRampToValueAtTime(0, ctx.currentTime + BLIP_S);
    node.connect(gain).connect(ctx.destination);
    node.start(0, clamp(seconds, 0, Math.max(0, buffer.duration - END_MARGIN_S)), BLIP_S);
    current = node;
  });
}
