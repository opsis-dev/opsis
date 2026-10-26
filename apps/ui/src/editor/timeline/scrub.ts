import { clamp } from "@opsis/video/common/math";
import { lruCache } from "@opsis/video/common/memo";
import { errorMessage } from "@/core/errors";
import { currentFileUrl } from "../../store/project-files";

const CACHED_SOURCES = 3;
const MIN_INTERVAL_MS = 45;
const BLIP_S = 0.09;
const BLIP_GAIN = 0.9;
const END_MARGIN_S = 0.1;

const buffers = lruCache<Promise<AudioBuffer | null>>(CACHED_SOURCES);
let context: AudioContext | null = null;
let lastScrubAt = 0;
let current: AudioBufferSourceNode | null = null;

function decode(ctx: AudioContext, url: string): Promise<AudioBuffer | null> {
  return fetch(url)
    .then((r) => r.arrayBuffer())
    .then((data) => ctx.decodeAudioData(data))
    .catch((error: unknown) => {
      buffers.delete(url);
      console.warn(`Scrub sound unavailable for ${url}: ${errorMessage(error)}`);
      return null;
    });
}

function audioContext(): AudioContext {
  context ??= new AudioContext();
  return context;
}

function load(src: string): Promise<AudioBuffer | null> {
  const url = currentFileUrl(src);
  return buffers.use(url, () => decode(audioContext(), url));
}

export function preloadScrub(src: string | null): void {
  if (src) void load(src);
}

export function scrubAt(src: string | null, seconds: number): void {
  if (!src) return;
  const now = performance.now();
  if (now - lastScrubAt < MIN_INTERVAL_MS) return;
  lastScrubAt = now;
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
