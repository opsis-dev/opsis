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

function load(src: string): Promise<AudioBuffer | null> {
  let buffer = buffers.get(src);
  if (!buffer) {
    context ??= new AudioContext();
    buffer = decode(context, src);
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
  void load(src).then((buffer) => {
    if (!buffer || !context) return;
    if (context.state === "suspended") void context.resume();
    current?.stop();
    const node = context.createBufferSource();
    const gain = context.createGain();
    node.buffer = buffer;
    gain.gain.setValueAtTime(BLIP_GAIN, context.currentTime);
    gain.gain.linearRampToValueAtTime(0, context.currentTime + BLIP_S);
    node.connect(gain).connect(context.destination);
    node.start(0, clamp(seconds, 0, Math.max(0, buffer.duration - END_MARGIN_S)), BLIP_S);
    current = node;
  });
}
