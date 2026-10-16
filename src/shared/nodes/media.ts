import { z } from "zod";
import { itemClip, msToFrames } from "../compile/clip";
import { projectRef } from "../elements/introspect";
import type { CompileValue, MediaValue } from "../schema/values";
import { row, secs, type CardRow } from "./card";
import { defineNode, opt, port, type NodeDef } from "./define";
import { asset, bool, choice, MAX_SPEED, more, ms, quality, text, visualEnvelope } from "./params";

const base = { category: "Media", phase: "media", executes: true } as const;
const one = {
  inputs: { media: port("media", "File") },
  outputs: { media: port("media", "File") },
} as const;
const NEW_FILE = " Makes a new file.";

const lengthRow = (result?: { outputs: Record<string, CompileValue> }): CardRow | null => {
  const m = result?.outputs.media as MediaValue | undefined;
  return m?.durationMs ? row("clock", secs(m.durationMs)) : null;
};
const QUALITY_NAMES: Array<[number, string]> = [
  [12, "Max"],
  [18, "High"],
  [23, "Medium"],
  [28, "Low"],
];
const CODEC_NAMES: Record<string, string> = {
  h264: "MP4 (H.264)",
  hevc: "MP4 (HEVC)",
  vp9: "WebM",
  prores: "ProRes",
};
const LOUDNESS_NAMES: Record<number, string> = {
  [-14]: "Online video",
  [-16]: "Podcast",
  [-23]: "Broadcast",
};
export const qualityName = (crf: number) =>
  QUALITY_NAMES.reduce((best, cur) =>
    Math.abs(cur[0] - crf) < Math.abs(best[0] - crf) ? cur : best,
  )[1];

const passMedia = ({
  result,
  inputs,
}: {
  result?: { outputs: Record<string, CompileValue> };
  inputs: Partial<Record<string, CompileValue>>;
}) => ({
  media:
    (result?.outputs.media as MediaValue | undefined) ?? (inputs.media as MediaValue | undefined),
});

export const importMediaNode = defineNode({
  ...base,
  type: "media.import",
  title: "Media file",
  icon: "file-play",
  description: "A video, image or sound from your project. Drop files on the canvas to add one.",
  inputs: {},
  outputs: { media: port("media", "File"), clip: port("clip", "Clip") },
  card: ({ params: p, result }) => [
    p.path
      ? row("file-video", p.path.split("/").pop() ?? p.path, { title: p.path })
      : row("file-video", "Choose a file", { tone: "warn" }),
    lengthRow(result),
  ],
  params: z.object({
    path: asset("File"),
    fit: choice(["cover", "contain"], "contain", "Fit", {
      cover: "Fill the frame",
      contain: "Show all",
    }),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, result, params, settings }) => {
    const probed = result?.outputs.media as MediaValue | undefined;
    const ext = params.path.split(".").pop()?.toLowerCase() ?? "";
    const kind: MediaValue["kind"] =
      probed?.kind ??
      (["png", "jpg", "jpeg", "webp", "gif", "svg"].includes(ext)
        ? "image"
        : ["mp3", "wav", "m4a", "aac", "ogg", "flac"].includes(ext)
          ? "audio"
          : "video");
    const media: MediaValue = probed ?? {
      type: "media",
      src: params.path ? projectRef(params.path) : null,
      kind,
    };
    const frames = msToFrames(media.durationMs ?? (kind === "image" ? 3000 : 5000), settings.fps);
    const clip =
      !media.src || kind === "audio"
        ? itemClip(
            nodeId,
            "core/placeholder",
            { title: media.src ? "Sound file" : "No file chosen", subtitle: params.path },
            frames,
            { envelope: params.envelope },
          )
        : itemClip(
            nodeId,
            kind === "image" ? "core/image" : "core/video",
            { src: media.src, fit: params.fit, muted: false },
            frames,
            { envelope: params.envelope },
          );
    return { media, clip };
  },
});

export const mediaToClipNode = defineNode({
  type: "media.toClip",
  title: "Use as clip",
  category: "Media",
  icon: "film",
  phase: "compile",
  description: "Turns a file into a clip you can place in the video.",
  inputs: { media: port("media", "File") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => [
    row("maximize", p.fit === "cover" ? "Fill the frame" : "Show all"),
    p.muted && row("volume-x", "Muted"),
  ],
  params: z.object({
    fit: choice(["cover", "contain"], "contain", "Fit", {
      cover: "Fill the frame",
      contain: "Show all",
    }),
    muted: bool(false, "Muted"),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const media = inputs.media as MediaValue | undefined;
    if (!media?.src) return {};
    const frames = msToFrames(media.durationMs ?? 5000, settings.fps);
    return {
      clip: itemClip(
        nodeId,
        media.kind === "image" ? "core/image" : "core/video",
        { src: media.src, fit: params.fit, muted: params.muted },
        frames,
        { envelope: params.envelope },
      ),
    };
  },
});

// oxlint-disable-next-line typescript/no-explicit-any
const op = <S extends z.ZodObject<any>>(def: Omit<NodeDef<S>, "category" | "phase" | "executes">) =>
  defineNode<S>({ ...base, ...def }) as unknown as NodeDef;

export const mediaNodes: NodeDef[] = [
  importMediaNode,
  mediaToClipNode,
  op({
    ...one,
    type: "media.trim",
    title: "Trim file",
    icon: "scissors",
    description: "Cuts a file down to a start and end." + NEW_FILE,
    card: ({ params: p, result }) => [
      row(
        "scissors",
        p.endMs ? `${secs(p.startMs)} → ${secs(p.endMs)}` : `From ${secs(p.startMs)} to the end`,
      ),
      lengthRow(result),
    ],
    params: z.object({
      startMs: ms(0, "Start"),
      endMs: more(ms(0, "End", 36_000_000), { zeroLabel: "To the end" }),
    }),
    compile: passMedia,
  }),
  op({
    type: "media.concat",
    title: "Join files",
    icon: "combine",
    description:
      "Joins files end to end. Fast when they match, otherwise converts them." + NEW_FILE,
    inputs: {
      in1: port("media", "File 1"),
      in2: port("media", "File 2"),
      in3: opt("media", "File 3"),
      in4: opt("media", "File 4"),
    },
    outputs: { media: port("media", "File") },
    card: ({ connected, result }) => {
      const n = ["in1", "in2", "in3", "in4"].filter((k) => connected.includes(k)).length;
      return [row("combine", `${n} files`, n >= 2 ? {} : { tone: "warn" }), lengthRow(result)];
    },
    params: z.object({ forceReencode: more(bool(false, "Always convert"), { advanced: true }) }),
    compile: ({ result, inputs }) => ({
      media:
        (result?.outputs.media as MediaValue | undefined) ?? (inputs.in1 as MediaValue | undefined),
    }),
  }),
  op({
    ...one,
    type: "media.speed",
    title: "Change speed",
    icon: "fast-forward",
    description: "Speeds a file up or slows it down." + NEW_FILE,
    card: ({ params: p }) => [
      row("gauge", `${p.rate}×`),
      row(p.keepAudio ? "volume-2" : "volume-x", p.keepAudio ? "Keeps sound" : "No sound"),
    ],
    params: z.object({
      rate: z
        .number()
        .min(0.25)
        .max(MAX_SPEED)
        .default(2)
        .meta({ description: "Speed", step: 0.05, unit: "×" }),
      keepAudio: bool(true, "Keep the sound"),
    }),
    compile: passMedia,
  }),
  op({
    ...one,
    type: "media.reverse",
    title: "Reverse",
    icon: "undo-2",
    description: "Plays a file backwards." + NEW_FILE,
    card: ({ result }) => [lengthRow(result)],
    params: z.object({}),
    compile: passMedia,
  }),
  op({
    ...one,
    type: "media.scalePad",
    title: "Resize",
    icon: "maximize",
    description: "Resizes a file, adding bars or cropping to fit." + NEW_FILE,
    card: ({ params: p }) => [
      row("ratio", `${p.width} × ${p.height}`),
      row(
        "maximize",
        p.mode === "pad" ? "Fit with bars" : p.mode === "crop" ? "Crop to fill" : "Stretch",
      ),
    ],
    params: z.object({
      width: z
        .number()
        .int()
        .min(16)
        .max(7680)
        .default(1920)
        .meta({ description: "Width", unit: "px" }),
      height: z
        .number()
        .int()
        .min(16)
        .max(7680)
        .default(1080)
        .meta({ description: "Height", unit: "px" }),
      mode: choice(["pad", "crop", "stretch"], "pad", "Fit", {
        pad: "Add bars",
        crop: "Crop to fill",
        stretch: "Stretch",
      }),
      background: text("black", "Bar colour").meta({
        description: "Bar colour",
        widget: "color",
        showIf: { mode: ["pad"] },
      }),
    }),
    compile: passMedia,
  }),
  op({
    ...one,
    type: "media.convert",
    title: "Convert",
    icon: "refresh-cw",
    description: "Saves a file in another format or quality." + NEW_FILE,
    card: ({ params: p }) => [
      row("file-video", CODEC_NAMES[p.codec] ?? p.codec),
      row("sparkles", `${qualityName(p.crf)} quality`),
    ],
    params: z.object({
      codec: choice(["h264", "hevc", "vp9", "prores"], "h264", "Format", CODEC_NAMES),
      crf: quality(18),
      preset: choice(
        ["ultrafast", "fast", "medium", "slow", "veryslow"],
        "medium",
        "Speed vs file size",
        {
          ultrafast: "Fastest, biggest",
          fast: "Fast",
          medium: "Balanced",
          slow: "Small",
          veryslow: "Smallest, slowest",
        },
      ).meta({
        description: "Speed vs file size",
        labels: {
          ultrafast: "Fastest, biggest",
          fast: "Fast",
          medium: "Balanced",
          slow: "Small",
          veryslow: "Smallest, slowest",
        },
        advanced: true,
      }),
    }),
    compile: passMedia,
  }),
  op({
    ...one,
    type: "media.gif",
    title: "Make GIF",
    icon: "image-play",
    description: "Turns a video into a GIF with clean colours." + NEW_FILE,
    card: ({ params: p }) => [
      row("image-play", `${p.width} px · ${p.fps} fps`),
      p.loop && row("repeat", "Loops"),
    ],
    params: z.object({
      fps: z
        .number()
        .int()
        .min(5)
        .max(50)
        .default(15)
        .meta({ description: "Frame rate", unit: "fps" }),
      width: z
        .number()
        .int()
        .min(64)
        .max(3840)
        .default(960)
        .meta({ description: "Width", unit: "px" }),
      loop: bool(true, "Loop forever"),
      dither: choice(
        ["sierra2_4a", "bayer", "floyd_steinberg", "none"],
        "sierra2_4a",
        "Smooth colours",
        {
          sierra2_4a: "Smooth",
          bayer: "Patterned",
          floyd_steinberg: "Fine",
          none: "Off",
        },
      ),
    }),
    compile: passMedia,
  }),
  op({
    ...one,
    type: "media.extractAudio",
    title: "Extract sound",
    icon: "audio-lines",
    description: "Saves the sound of a video as its own file." + NEW_FILE,
    card: ({ params: p }) => [row("audio-lines", p.format.toUpperCase())],
    params: z.object({
      format: choice(["m4a", "mp3", "wav"], "m4a", "Format", {
        m4a: "M4A",
        mp3: "MP3",
        wav: "WAV",
      }),
    }),
    compile: passMedia,
  }),
  op({
    ...one,
    type: "media.loudnorm",
    title: "Even out volume",
    icon: "volume-2",
    description: "Makes the sound as loud as most online videos." + NEW_FILE,
    card: ({ params: p }) => [
      row("volume-2", LOUDNESS_NAMES[p.integrated] ?? `${p.integrated} LUFS`),
    ],
    params: z.object({
      integrated: z.number().min(-40).max(-5).default(-16).meta({
        description: "Loudness",
        widget: "loudness",
        unit: "LUFS",
      }),
      truePeak: z.number().min(-9).max(0).default(-1.5).meta({
        description: "Peak limit",
        step: 0.1,
        unit: "dB",
        advanced: true,
      }),
      lra: z
        .number()
        .min(1)
        .max(20)
        .default(11)
        .meta({ description: "Dynamic range", advanced: true }),
    }),
    compile: passMedia,
  }),
  op({
    type: "media.thumbnail",
    title: "Grab frame",
    icon: "image",
    description: "Saves one frame of a video as an image." + NEW_FILE,
    inputs: { media: port("media", "File") },
    outputs: { image: port("image", "Image") },
    card: ({ params: p }) => [row("image", `At ${secs(p.atMs)} · ${p.width} px`)],
    params: z.object({
      atMs: ms(1000, "At"),
      width: z
        .number()
        .int()
        .min(64)
        .max(3840)
        .default(1280)
        .meta({ description: "Width", unit: "px" }),
    }),
    compile: ({ result }) => ({ image: result?.outputs.image }),
  }),
  op({
    ...one,
    type: "media.faststart",
    title: "Web-ready",
    icon: "zap",
    description: "Prepares a video to start playing instantly on the web." + NEW_FILE,
    params: z.object({}),
    compile: passMedia,
  }),
  op({
    type: "media.probe",
    title: "File info",
    icon: "info",
    description: "Shows a file's length, size, frame rate and format.",
    inputs: { media: port("media", "File") },
    outputs: { file: port("file", "Info"), media: port("media", "File") },
    card: ({ result }) => {
      const m = result?.outputs.media as MediaValue | undefined;
      return [
        m?.durationMs
          ? row("info", secs(m.durationMs))
          : row("info", "Run to read the file", { tone: "muted" }),
      ];
    },
    params: z.object({}),
    compile: ({ result, inputs }) => ({
      file: result?.outputs.file,
      media:
        (result?.outputs.media as MediaValue | undefined) ??
        (inputs.media as MediaValue | undefined),
    }),
  }),
];
