import { z } from "zod";
import { isClip, msToFrames } from "../compile/clip";
import { projectRef } from "../elements/introspect";
import type { AudioItem } from "../schema/timeline";
import { builtinSound } from "../sounds";
import type { AudioValue, MediaValue } from "../schema/values";
import { hasKeys } from "../timeline/envelope";
import { row, secs } from "./card";
import { defineNode, opt, port } from "./define";
import { asset, audioEnvelope, bool, more, ms } from "./params";

const fileName = (path: string) => builtinSound(path)?.label ?? path.split("/").pop() ?? path;
const pct = (v: number) => `${Math.round(v * 100)} %`;

const srcFrom = (input: unknown, path: string): string | null =>
  (input as MediaValue | undefined)?.src ?? (path ? projectRef(path) : null);

export const musicBedNode = defineNode({
  type: "audio.music",
  title: "Music",
  description: "Background music under the video, with fades.",
  card: ({ params: p, connected }) => [
    p.src
      ? row("music", fileName(p.src), { title: p.src })
      : connected.includes("media")
        ? row("music", "From the connected file")
        : row("music", "Choose a song", { tone: "warn" }),
    row("volume-2", pct(p.volume)),
    p.loop && row("repeat", "Loops"),
    p.lengthMs > 0 && row("clock", `Plays for ${secs(p.lengthMs)}`),
  ],
  category: "Audio",
  icon: "music",
  phase: "compile",
  inputs: { media: opt("media", "File") },
  outputs: { audio: port("audio", "Sound") },
  params: z.object({
    src: asset("Song", "audio/*"),
    volume: z.number().min(0).max(2).default(0.5).meta({ description: "Volume", step: 0.05 }),
    loop: bool(true, "Loop"),
    startMs: ms(0, "Starts at"),
    trimMs: more(ms(0, "Start the song from"), { help: "Skips the start of the song." }),
    lengthMs: more(ms(0, "Play for", 3_600_000), {
      zeroLabel: "To the end",
      advanced: true,
    }),
    fadeInMs: ms(800, "Fade in", 20_000),
    fadeOutMs: ms(1500, "Fade out", 20_000),
    envelope: audioEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const src = srcFrom(inputs.media, params.src);
    if (!src) return { audio: { type: "audio", items: [] } };
    const item: AudioItem = {
      id: nodeId,
      src,
      from: params.startMs ? msToFrames(params.startMs, settings.fps) : 0,
      ...(params.lengthMs ? { durationInFrames: msToFrames(params.lengthMs, settings.fps) } : {}),
      volume: params.volume,
      loop: params.loop,
      trimBefore: params.trimMs ? msToFrames(params.trimMs, settings.fps) : undefined,
      fadeInFrames: params.fadeInMs ? msToFrames(params.fadeInMs, settings.fps) : undefined,
      fadeOutFrames: params.fadeOutMs ? msToFrames(params.fadeOutMs, settings.fps) : undefined,
      nodeId,
      channel: nodeId,
      role: "music",
      ...(hasKeys(params.envelope) ? { envelope: params.envelope } : {}),
      handles: {
        move: { nodeId, param: "startMs", unit: "ms" },
        trimStart: {
          nodeId,
          param: "trimMs",
          unit: "ms",
          linked: [
            { param: "startMs", sign: 1 },
            { param: "lengthMs", sign: -1, ifNonZero: true },
          ],
        },
        trimEnd: { nodeId, param: "lengthMs", unit: "ms" },
        volume: { nodeId, param: "volume", unit: "number" },
      },
    };
    const media = inputs.media as MediaValue | undefined;
    return {
      audio: {
        type: "audio",
        items: [item],
        durationInFrames: media?.durationMs
          ? msToFrames(media.durationMs, settings.fps)
          : undefined,
      },
    };
  },
});

export const sfxNode = defineNode({
  type: "audio.sfx",
  title: "Sound effect",
  description: "A sound at a set time.",
  card: ({ params: p, connected }) => [
    p.src
      ? row("audio-lines", fileName(p.src), { title: p.src })
      : connected.includes("media")
        ? row("audio-lines", "From the connected file")
        : row("audio-lines", "Choose a sound", { tone: "warn" }),
    row("clock", `At ${secs(p.atMs)}`),
    p.keepMusic && row("music", "Music stays up"),
  ],
  category: "Audio",
  icon: "audio-lines",
  phase: "compile",
  inputs: { media: opt("media", "File") },
  outputs: { audio: port("audio", "Sound") },
  params: z.object({
    src: more(asset("Sound", "audio/*"), { widget: "sound" }),
    atMs: ms(0, "At"),
    volume: z.number().min(0).max(2).default(1).meta({ description: "Volume", step: 0.05 }),
    keepMusic: more(bool(false, "Keep music up"), {
      help: "Music set to lower under other sounds stays at full volume for this one.",
      advanced: true,
    }),
    envelope: audioEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const src = srcFrom(inputs.media, params.src);
    if (!src) return { audio: { type: "audio", items: [] } };
    const lengthMs =
      (inputs.media as MediaValue | undefined)?.durationMs ?? builtinSound(src)?.durationMs;
    return {
      audio: {
        type: "audio",
        items: [
          {
            id: nodeId,
            src,
            from: params.atMs ? msToFrames(params.atMs, settings.fps) : 0,
            volume: params.volume,
            nodeId,
            channel: nodeId,
            role: "effect",
            lengthFrames: lengthMs ? msToFrames(lengthMs, settings.fps) : undefined,
            keepMusic: params.keepMusic || undefined,
            ...(hasKeys(params.envelope) ? { envelope: params.envelope } : {}),
            handles: {
              move: { nodeId, param: "atMs", unit: "ms" },
              volume: { nodeId, param: "volume", unit: "number" },
            },
          },
        ],
      },
    };
  },
});

export const addAudioNode = defineNode({
  type: "audio.mix",
  title: "Add sound",
  category: "Audio",
  icon: "audio-waveform",
  phase: "compile",
  description: "Puts up to three sounds under a clip. Sound stops where the clip ends.",
  card: ({ connected }) => {
    const n = ["a1", "a2", "a3"].filter((k) => connected.includes(k)).length;
    return [row("volume-2", n === 1 ? "1 sound" : `${n} sounds`, n ? {} : { tone: "warn" })];
  },
  inputs: {
    clip: port("clip", "Clip"),
    a1: opt("audio", "Sound 1"),
    a2: opt("audio", "Sound 2"),
    a3: opt("audio", "Sound 3"),
  },
  outputs: { clip: port("clip", "Clip") },
  params: z.object({}),
  compile: ({ inputs }) => {
    const clip = isClip(inputs.clip) ? inputs.clip : undefined;
    if (!clip) return {};
    const items = (["a1", "a2", "a3"] as const).flatMap(
      (k) => (inputs[k] as AudioValue | undefined)?.items ?? [],
    );
    return {
      clip: {
        ...clip,
        audio: [
          ...clip.audio,
          ...items.map((a) => ({
            ...a,
            durationInFrames: Math.max(
              1,
              Math.min(a.durationInFrames ?? Infinity, clip.durationInFrames - a.from),
            ),
          })),
        ],
      },
    };
  },
});

export const audioNodes = [musicBedNode, sfxNode, addAudioNode];
