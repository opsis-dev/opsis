import { fileUrl } from "../file-url";

const buffers = new Map<string, Promise<AudioBuffer | null>>();
let context: AudioContext | null = null;
let last = 0;
let current: AudioBufferSourceNode | null = null;

function load(src: string): Promise<AudioBuffer | null> {
  let buffer = buffers.get(src);
  if (!buffer) {
    context ??= new AudioContext();
    const ctx = context;
    buffer = fetch(fileUrl(src))
      .then((r) => r.arrayBuffer())
      .then((data) => ctx.decodeAudioData(data))
      .catch(() => null);
    buffers.set(src, buffer);
    if (buffers.size > 3) buffers.delete(buffers.keys().next().value!);
  }
  return buffer;
}

export function preloadScrub(src: string | null): void {
  if (src) void load(src);
}

export function scrubAt(src: string | null, seconds: number): void {
  if (!src) return;
  const now = performance.now();
  if (now - last < 45) return;
  last = now;
  void load(src).then((buffer) => {
    if (!buffer || !context) return;
    if (context.state === "suspended") void context.resume();
    current?.stop();
    const node = context.createBufferSource();
    const gain = context.createGain();
    node.buffer = buffer;
    gain.gain.setValueAtTime(0.9, context.currentTime);
    gain.gain.linearRampToValueAtTime(0, context.currentTime + 0.09);
    node.connect(gain).connect(context.destination);
    node.start(0, Math.max(0, Math.min(buffer.duration - 0.1, seconds)), 0.09);
    current = node;
  });
}
