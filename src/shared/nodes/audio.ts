import { z } from "zod";
import { clipToSpec } from "../compile/clip";
import type { AudioEnvelope, AudioItem, EditHandles } from "../schema/timeline";
import { builtinSound } from "../sounds";
import { asValue, type ClipValue, type CompileValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { flattenSpec } from "../timeline/flatten";
import { hasKeys } from "../timeline/envelope";
import { countEdgesInto, fileRow, row, warnIf, type CardRow } from "./card";
import type { NodeIconName } from "./icon-name";
import { defineNode, many, opt, port, withClip, type CompileOutputs } from "./define";
import { mediaInput } from "./media";
import { asset, audioEnvelope, bool, int, more, ms, volume } from "./params";
import { formatDuration, percentText, plural } from "../format";
import { msToFrames, msToSignedFrames } from "../timeline/time";
import { baseName, projectRef } from "../asset-ref";
import { ANY_VALUE } from "../elements/introspect";

const base = { category: "Audio", phase: "compile" } as const;

const soundSource = {
  ...base,
  inputs: { media: opt("media", "File") },
  outputs: { audio: port("audio", "Sound") },
} as const;

const soundFile = (description: string) => asset(description, "audio/*");

const withSource = (
  input: CompileValue | undefined,
  path: string,
  build: (src: string) => CompileOutputs,
): CompileOutputs => {
  const src = mediaInput(input)?.src ?? (path ? projectRef(path) : null);
  return src ? build(src) : { audio: { type: "audio", items: [] } };
};

type SoundParams = { volume: number; envelope: AudioEnvelope };

function soundItem(
  nodeId: string,
  src: string,
  role: NonNullable<AudioItem["role"]>,
  params: SoundParams,
  item: Omit<AudioItem, "id" | "src" | "role" | "handles">,
  handles: EditHandles,
): AudioItem {
  return {
    id: nodeId,
    src,
    ...item,
    volume: params.volume,
    nodeId,
    channel: nodeId,
    role,
    ...(hasKeys(params.envelope) ? { envelope: params.envelope } : {}),
    handles: { ...handles, volume: { nodeId, param: "volume", unit: "number" } },
  };
}

const offsetLabel = (offsetMs: number): string =>
  offsetMs ? ` ${offsetMs > 0 ? "+" : "−"}${formatDuration(Math.abs(offsetMs))}` : "";

function sourceRow(icon: NodeIconName, src: string, connected: string[], empty: string): CardRow {
  if (src) return fileRow(icon, src, builtinSound(src)?.label ?? baseName(src));
  return connected.includes("media")
    ? row(icon, "From the connected file")
    : row(icon, empty, { tone: "warn" });
}

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
  icon: "music",
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
    withSource(inputs.media, params.src, (src) => {
      const item = soundItem(
        nodeId,
        src,
        "music",
        params,
        {
          from: msToFrames(params.startMs, settings.fps, 0),
          ...(params.lengthMs
            ? { durationInFrames: msToFrames(params.lengthMs, settings.fps) }
            : {}),
          loop: params.loop,
          trimBefore: params.trimMs ? msToFrames(params.trimMs, settings.fps) : undefined,
          fadeInFrames: params.fadeInMs ? msToFrames(params.fadeInMs, settings.fps) : undefined,
          fadeOutFrames: params.fadeOutMs ? msToFrames(params.fadeOutMs, settings.fps) : undefined,
        },
        {
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
        },
      );
      const media = mediaInput(inputs.media);
      return {
        audio: {
          type: "audio",
          items: [item],
          durationInFrames: media?.durationMs
            ? msToFrames(media.durationMs, settings.fps)
            : undefined,
        },
      };
    }),
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
  icon: "audio-lines",
  params: z.object({
    src: more(soundFile("Sound"), { widget: "sound" }),
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
    withSource(inputs.media, params.src, (src) => {
      const lengthMs = mediaInput(inputs.media)?.durationMs ?? builtinSound(src)?.durationMs;
      const startFrame = params.atAction
        ? msToSignedFrames(params.offsetMs, settings.fps)
        : msToFrames(params.atMs, settings.fps, 0);
      const item = soundItem(
        nodeId,
        src,
        "effect",
        params,
        {
          from: startFrame,
          ...(params.atAction ? { anchor: { nodeId: params.atAction } } : {}),
          lengthFrames: lengthMs ? msToFrames(lengthMs, settings.fps) : undefined,
          keepMusic: params.keepMusic || undefined,
        },
        { move: { nodeId, param: params.atAction ? "offsetMs" : "atMs", unit: "ms" } },
      );
      return { audio: { type: "audio", items: [item] } };
    }),
});

const mixNode = defineNode({
  ...base,
  type: "audio.mix",
  title: "Add sound",
  icon: "audio-waveform",
  description:
    "Puts sounds under a clip. Connect as many as you like. Sound stops where the clip ends.",
  card: ({ connected }) => {
    const n = countEdgesInto("sounds", connected);
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
      const placed = anchorSounds(items, clip, settings).map((a) => {
        const remaining = clip.durationInFrames - a.from;
        const length =
          a.durationInFrames === undefined ? remaining : Math.min(a.durationInFrames, remaining);
        return { ...a, durationInFrames: Math.max(1, length) };
      });
      return { ...clip, audio: [...clip.audio, ...placed] };
    }),
});

function anchorSounds(
  items: AudioItem[],
  clip: ClipValue,
  settings: WorkflowSettings,
): AudioItem[] {
  if (!items.some((a) => a.anchor)) return items;
  const marks = flattenSpec(clipToSpec(clip, settings)).marks;
  return items.map((a) => {
    if (!a.anchor) return a;
    const mark = marks.find((m) => m.nodeId === a.anchor?.nodeId && m.kind !== "chapter");
    return mark
      ? { ...a, from: Math.max(0, Math.round(mark.frame) + a.from) }
      : { ...a, from: Math.max(0, a.from) };
  });
}

export const audioNodes = [musicNode, sfxNode, mixNode];
