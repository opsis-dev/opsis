import { z } from "zod";
import { itemClip } from "../compile/clip";
import type { VisualEnvelope } from "../schema/timeline";
import {
  asValue,
  outputOf,
  type CompileValue,
  type MediaValue,
  type NodeResult,
} from "../schema/values";
import { fileRow, row, warnIf, type CardRow } from "./card";
import { defineNode, opt, port, type NodeDef } from "./define";
import {
  asset,
  bool,
  choice,
  keysOf,
  LOUDNESS_PRESETS,
  more,
  ms,
  quality,
  QUALITY_PRESETS,
  rate,
  text,
  visualEnvelope,
} from "./params";
import { formatDuration } from "../format";
import { msToFrames } from "../timeline/time";
import { projectRef } from "../asset-ref";

const base = { category: "Media", phase: "media", executes: true } as const;
const one = {
  inputs: { media: port("media", "File") },
  outputs: { media: port("media", "File") },
} as const;
const NEW_FILE = " Makes a new file.";

const lengthRow = (result: NodeResult | undefined, icon = "clock"): CardRow | null => {
  const m = outputOf(result, "media", "media");
  return m?.durationMs ? row(icon, formatDuration(m.durationMs)) : null;
};
const CODEC_NAMES = {
  h264: "MP4 (H.264)",
  hevc: "MP4 (HEVC)",
  vp9: "WebM",
  prores: "ProRes",
};
const PRESET_LABELS = {
  ultrafast: "Fastest, biggest",
  fast: "Fast",
  medium: "Balanced",
  slow: "Small",
  veryslow: "Smallest, slowest",
};
const loudnessName = (lufs: number): string =>
  Object.entries(LOUDNESS_PRESETS).find(([, v]) => v === lufs)?.[0] ?? `${lufs} LUFS`;
export const qualityName = (crf: number) =>
  Object.entries(QUALITY_PRESETS).reduce((best, cur) =>
    Math.abs(cur[1] - crf) <= Math.abs(best[1] - crf) ? cur : best,
  )[0];

const IMAGE_EXTENSIONS = new Set(["png", "jpg", "jpeg", "webp", "gif", "svg"]);
const AUDIO_EXTENSIONS = new Set(["mp3", "wav", "m4a", "aac", "ogg", "flac"]);

function kindFromPath(path: string): MediaValue["kind"] {
  const ext = path.slice(path.lastIndexOf(".") + 1).toLowerCase();
  if (IMAGE_EXTENSIONS.has(ext)) return "image";
  return AUDIO_EXTENSIONS.has(ext) ? "audio" : "video";
}

export function mediaInput(value: CompileValue | undefined): MediaValue | undefined {
  const media = asValue(value, "media");
  if (media) return media;
  const image = asValue(value, "image");
  if (image) {
    const { type: _image, ...rest } = image;
    return { ...rest, type: "media", kind: "image" };
  }
  const file = asValue(value, "file");
  return file && { type: "media", src: file.src, kind: kindFromPath(file.src ?? "") };
}

const passMedia = (
  { result, inputs }: { result?: NodeResult; inputs: Partial<Record<string, CompileValue>> },
  input = "media",
) => ({ media: outputOf(result, "media", "media") ?? mediaInput(inputs[input]) });

const fitField = () =>
  choice(["cover", "contain"], "contain", "Fit", {
    cover: "Fill the frame",
    contain: "Show all",
  });

type MediaClip = {
  src: string;
  kind: MediaValue["kind"];
  fit: string;
  muted: boolean;
  frames: number;
  envelope: VisualEnvelope;
};

const mediaClip = (nodeId: string, { src, kind, fit, muted, frames, envelope }: MediaClip) =>
  itemClip(nodeId, kind === "image" ? "core/image" : "core/video", { src, fit, muted }, frames, {
    envelope,
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
    p.path ? fileRow("file-video", p.path) : row("file-video", "Choose a file", { tone: "warn" }),
    lengthRow(result),
  ],
  params: z.object({
    path: asset("File"),
    fit: fitField(),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, result, params, settings }) => {
    const media: MediaValue = outputOf(result, "media", "media") ?? {
      type: "media",
      src: params.path ? projectRef(params.path) : null,
      kind: kindFromPath(params.path),
    };
    const frames = msToFrames(
      media.durationMs ?? (media.kind === "image" ? 3000 : 5000),
      settings.fps,
    );
    const clip =
      !media.src || media.kind === "audio"
        ? itemClip(
            nodeId,
            "core/placeholder",
            { title: media.src ? "Sound file" : "No file chosen", subtitle: params.path },
            frames,
            { envelope: params.envelope },
          )
        : mediaClip(nodeId, {
            src: media.src,
            kind: media.kind,
            fit: params.fit,
            muted: false,
            frames,
            envelope: params.envelope,
          });
    return { media, clip };
  },
});

const mediaToClipNode = defineNode({
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
    fit: fitField(),
    muted: bool(false, "Muted"),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const media = mediaInput(inputs.media);
    if (!media?.src) return {};
    return {
      clip: mediaClip(nodeId, {
        src: media.src,
        kind: media.kind,
        fit: params.fit,
        muted: params.muted,
        frames: msToFrames(media.durationMs ?? 5000, settings.fps),
        envelope: params.envelope,
      }),
    };
  },
});

const SCALE_ROWS = { pad: "Fit with bars", crop: "Crop to fill", stretch: "Stretch" };

const op = <S extends z.ZodObject>(def: Omit<NodeDef<S>, "category" | "phase" | "executes">) =>
  defineNode<S>({ ...base, ...def });

export const trimNode = op({
  ...one,
  type: "media.trim",
  title: "Trim file",
  icon: "scissors",
  description: "Cuts a file down to a start and end." + NEW_FILE,
  card: ({ params: p, result }) => [
    row(
      "scissors",
      p.endMs
        ? `${formatDuration(p.startMs)} → ${formatDuration(p.endMs)}`
        : `From ${formatDuration(p.startMs)} to the end`,
    ),
    lengthRow(result),
  ],
  params: z.object({
    startMs: ms(0, "Start"),
    endMs: more(ms(0, "End", 36_000_000), { zeroLabel: "To the end" }),
  }),
  compile: passMedia,
});

export const concatNode = op({
  type: "media.concat",
  title: "Join files",
  icon: "combine",
  description: "Joins files end to end. Fast when they match, otherwise converts them." + NEW_FILE,
  inputs: {
    in1: port("media", "File 1"),
    in2: port("media", "File 2"),
    in3: opt("media", "File 3"),
    in4: opt("media", "File 4"),
  },
  outputs: { media: port("media", "File") },
  card: ({ connected, result }) => {
    const n = ["in1", "in2", "in3", "in4"].filter((k) => connected.includes(k)).length;
    return [row("combine", `${n} files`, warnIf(n < 2)), lengthRow(result)];
  },
  params: z.object({ forceReencode: more(bool(false, "Always convert"), { advanced: true }) }),
  compile: (ctx) => passMedia(ctx, "in1"),
});

export const speedNode = op({
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
    rate: rate(2, "Speed", { min: 0.25 }),
    keepAudio: bool(true, "Keep the sound"),
  }),
  compile: passMedia,
});

export const reverseNode = op({
  ...one,
  type: "media.reverse",
  title: "Reverse",
  icon: "undo-2",
  description: "Plays a file backwards." + NEW_FILE,
  card: ({ result }) => [lengthRow(result)],
  params: z.object({}),
  compile: passMedia,
});

export const scalePadNode = op({
  ...one,
  type: "media.scalePad",
  title: "Resize",
  icon: "maximize",
  description: "Resizes a file, adding bars or cropping to fit." + NEW_FILE,
  card: ({ params: p }) => [
    row("ratio", `${p.width} × ${p.height}`),
    row("maximize", SCALE_ROWS[p.mode]),
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
    background: more(text("black", "Bar colour"), {
      widget: "color",
      showIf: { mode: ["pad"] },
    }),
  }),
  compile: passMedia,
});

export const convertNode = op({
  ...one,
  type: "media.convert",
  title: "Convert",
  icon: "refresh-cw",
  description: "Saves a file in another format or quality." + NEW_FILE,
  card: ({ params: p }) => [
    row("file-video", CODEC_NAMES[p.codec]),
    row("sparkles", `${qualityName(p.crf)} quality`),
  ],
  params: z.object({
    codec: choice(keysOf(CODEC_NAMES), "h264", "Format", CODEC_NAMES),
    crf: quality(18),
    preset: more(choice(keysOf(PRESET_LABELS), "medium", "Speed vs file size", PRESET_LABELS), {
      advanced: true,
    }),
  }),
  compile: passMedia,
});

export const gifNode = op({
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
});

export const extractAudioNode = op({
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
});

export const loudnormNode = op({
  ...one,
  type: "media.loudnorm",
  title: "Even out volume",
  icon: "volume-2",
  description: "Makes the sound as loud as most online videos." + NEW_FILE,
  card: ({ params: p }) => [row("volume-2", loudnessName(p.integrated))],
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
});

export const thumbnailNode = op({
  type: "media.thumbnail",
  title: "Grab frame",
  icon: "image",
  description: "Saves one frame of a video as an image." + NEW_FILE,
  inputs: { media: port("media", "File") },
  outputs: { image: port("image", "Image") },
  card: ({ params: p }) => [row("image", `At ${formatDuration(p.atMs)} · ${p.width} px`)],
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
});

export const faststartNode = op({
  ...one,
  type: "media.faststart",
  title: "Web-ready",
  icon: "zap",
  description: "Prepares a video to start playing instantly on the web." + NEW_FILE,
  params: z.object({}),
  compile: passMedia,
});

export const probeNode = op({
  type: "media.probe",
  title: "File info",
  icon: "info",
  description: "Shows a file's length, size, frame rate and format.",
  inputs: { media: port("media", "File") },
  outputs: { file: port("file", "Info"), media: port("media", "File") },
  card: ({ result }) => [
    lengthRow(result, "info") ?? row("info", "Run to read the file", { tone: "muted" }),
  ],
  params: z.object({}),
  compile: (ctx) => ({ file: ctx.result?.outputs.file, ...passMedia(ctx) }),
});

export const mediaNodes: NodeDef[] = [
  importMediaNode,
  mediaToClipNode,
  trimNode,
  concatNode,
  speedNode,
  reverseNode,
  scalePadNode,
  convertNode,
  gifNode,
  extractAudioNode,
  loudnormNode,
  thumbnailNode,
  faststartNode,
  probeNode,
];
