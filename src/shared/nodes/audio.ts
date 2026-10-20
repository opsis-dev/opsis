import { z } from "zod";
import { baseName, projectRef } from "../asset-ref";
import { clipToSpec } from "../compile/clip";
import { ANY_VALUE } from "../elements/introspect";
import { formatDuration, percentText, plural, signOf } from "../format";
import type { AudioEnvelope, AudioItem, EditHandles } from "../schema/timeline";
import { asValue, type ClipValue, type CompileValue, type MediaValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { builtinSound } from "../sounds";
import { hasKeys } from "../timeline/envelope";
import { flattenSpec } from "../timeline/flatten";
import { msToFrames, msToSignedFrames } from "../timeline/time";
import { fileRow, missing, row, warnIf, type CardRow } from "./card";
import { defineNode, many, nodeBase, opt, port, withClip, type CompileOutputs } from "./define";
import type { NodeIconName } from "./icon-name";
import { mediaInput } from "./media";
import { asset, audioEnvelope, bool, int, more, ms, volume } from "./params";

const base = nodeBase("Audio", "compile");

const soundSource = {
  ...base,
  inputs: { media: opt("media", "File") },
  outputs: { audio: port("audio", "Sound") },
} as const;

const soundFile = (description: string) => asset(description, "audio/*");

const withSource = (
  input: CompileValue | undefined,
  path: string,
  build: (src: string, media: MediaValue | undefined) => CompileOutputs,
): CompileOutputs => {
  const media = mediaInput(input);
  const src = media?.src ?? (path ? projectRef(path) : null);
  return src ? build(src, media) : { audio: { type: "audio", items: [] } };
};

type SoundParams = { volume: number; envelope: AudioEnvelope };

type SoundTiming = Omit<AudioItem, "id" | "src" | "role" | "handles">;

function soundItem(sound: {
  nodeId: string;
  src: string;
  role: NonNullable<AudioItem["role"]>;
  params: SoundParams;
  timing: SoundTiming;
  handles: EditHandles;
}): AudioItem {
  const { nodeId, params } = sound;
  return {
    id: nodeId,
    src: sound.src,
    ...sound.timing,
    volume: params.volume,
    nodeId,
    channel: nodeId,
    role: sound.role,
    envelope: hasKeys(params.envelope) ? params.envelope : undefined,
    handles: { ...sound.handles, volume: { nodeId, param: "volume", unit: "number" } },
  };
}

const framesIfSet = (durationMs: number, fps: number): number | undefined =>
  durationMs ? msToFrames(durationMs, fps) : undefined;

const offsetLabel = (offsetMs: number): string =>
  offsetMs ? ` ${signOf(offsetMs, true)}${formatDuration(Math.abs(offsetMs))}` : "";

function sourceRow(icon: NodeIconName, src: string, connected: string[], empty: string): CardRow {
  if (src) return fileRow(icon, src, builtinSound(src)?.label ?? baseName(src));
  return connected.includes("media") ? row(icon, "From the connected file") : missing(icon, empty);
}

type MusicTimingParams = {
  startMs: number;
  lengthMs: number;
  loop: boolean;
  trimMs: number;
  fadeInMs: number;
  fadeOutMs: number;
};

const musicTiming = (p: MusicTimingParams, fps: number): SoundTiming => ({
  from: msToFrames(p.startMs, fps, 0),
  durationInFrames: framesIfSet(p.lengthMs, fps),
  loop: p.loop,
  trimBefore: framesIfSet(p.trimMs, fps),
  fadeInFrames: framesIfSet(p.fadeInMs, fps),
  fadeOutFrames: framesIfSet(p.fadeOutMs, fps),
});

const musicHandles = (nodeId: string): EditHandles => ({
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
});

const musicNode = defineNode({
  ...soundSource,
  type: "audio.music",
  title: "Music",
  description: "Background music under the video, with fades.",
  card: ({ params: p, connected }) => [
    sourceRow("music", p.src, connected, "Choose a song"),
    row("volume-2", percentText(p.volume)),
    p.loop && row("repeat", "Loops"),
    p.lengthMs > 0 && row("clock", `Plays for ${formatDuration(p.lengthMs)}`),
  ],
  params: z.object({
    src: soundFile("Song"),
    volume: volume(0.5),
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
  compile: ({ nodeId, inputs, params, settings }) =>
    withSource(inputs.media, params.src, (src, media) => {
      const item = soundItem({
        nodeId,
        src,
        role: "music",
        params,
        timing: musicTiming(params, settings.fps),
        handles: musicHandles(nodeId),
      });
      return {
        audio: {
          type: "audio",
          items: [item],
          durationInFrames: framesIfSet(media?.durationMs ?? 0, settings.fps),
        },
      };
    }),
});

type SfxTimingParams = { atAction: string; atMs: number; offsetMs: number; keepMusic: boolean };

const sfxTiming = (
  p: SfxTimingParams,
  lengthFrames: number | undefined,
  fps: number,
): SoundTiming => ({
  from: p.atAction ? msToSignedFrames(p.offsetMs, fps) : msToFrames(p.atMs, fps, 0),
  anchor: p.atAction ? { nodeId: p.atAction } : undefined,
  lengthFrames,
  keepMusic: p.keepMusic || undefined,
});

const sfxNode = defineNode({
  ...soundSource,
  type: "audio.sfx",
  title: "Sound effect",
  description: "A sound at a set time.",
  card: ({ params: p, connected }) => [
    sourceRow("audio-lines", p.src, connected, "Choose a sound"),
    p.atAction
      ? row("mouse-pointer-click", `On an action${offsetLabel(p.offsetMs)}`)
      : row("clock", `At ${formatDuration(p.atMs)}`),
    p.keepMusic && row("music", "Music stays up"),
  ],
  params: z.object({
    src: more(soundFile("Sound"), { widget: "sound", lengthKey: "srcMs" }),
    srcMs: z.number().positive().optional().meta({ description: "Sound length", hidden: true }),
    atAction: z.string().default("").meta({
      description: "On an action",
      widget: "action",
      help: "Plays when this browser action happens in the clip the sound is added to, even after you record again or edit the clip.",
    }),
    atMs: more(ms(0, "At"), { showIf: { atAction: [""] } }),
    offsetMs: int(0, "From the action", {
      min: -10_000,
      max: 10_000,
      unit: "ms",
      step: 50,
      help: "Negative plays it before the action, for sounds that build up to it.",
      showIf: { atAction: [ANY_VALUE] },
    }),
    volume: volume(1),
    keepMusic: more(bool(false, "Keep music up"), {
      help: "Music set to lower under other sounds stays at full volume for this one.",
      advanced: true,
    }),
    envelope: audioEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    withSource(inputs.media, params.src, (src, media) => {
      const lengthMs =
        media?.durationMs ?? builtinSound(src)?.durationMs ?? (media ? 0 : (params.srcMs ?? 0));
      const item = soundItem({
        nodeId,
        src,
        role: "effect",
        params,
        timing: sfxTiming(params, framesIfSet(lengthMs, settings.fps), settings.fps),
        handles: { move: { nodeId, param: params.atAction ? "offsetMs" : "atMs", unit: "ms" } },
      });
      return { audio: { type: "audio", items: [item] } };
    }),
});

function anchorSounds(
  items: AudioItem[],
  clip: ClipValue,
  settings: WorkflowSettings,
): AudioItem[] {
  if (!items.some((a) => a.anchor)) return items;
  const marks = flattenSpec(clipToSpec(clip, settings)).marks;
  return items.flatMap((a) => {
    if (!a.anchor) return [a];
    const { nodeId } = a.anchor;
    const mark = marks.find((m) => m.nodeId === nodeId && m.kind !== "chapter");
    return mark ? [{ ...a, from: Math.max(0, Math.round(mark.frame) + a.from) }] : [];
  });
}

const mixNode = defineNode({
  ...base,
  type: "audio.mix",
  title: "Add sound",
  description:
    "Puts sounds under a clip. Connect as many as you like. Sound stops where the clip ends.",
  card: ({ connected }) => {
    const n = connected.filter((p) => p === "sounds").length;
    return [row("volume-2", plural(n, "sound"), warnIf(!n))];
  },
  inputs: {
    clip: port("clip", "Clip"),
    sounds: many("audio", "Sounds"),
  },
  outputs: { clip: port("clip", "Clip") },
  params: z.object({}),
  compile: ({ inputs, settings }) =>
    withClip(inputs, (clip) => {
      const items = asValue(inputs.sounds, "audio")?.items ?? [];
      const placed = anchorSounds(items, clip, settings).flatMap((a) => {
        const remaining = clip.durationInFrames - a.from;
        if (remaining < 1) return [];
        const length =
          a.durationInFrames === undefined ? remaining : Math.min(a.durationInFrames, remaining);
        return [{ ...a, durationInFrames: Math.max(1, length) }];
      });
      return { ...clip, audio: [...clip.audio, ...placed] };
    }),
});

export const audioNodes = [musicNode, sfxNode, mixNode];
