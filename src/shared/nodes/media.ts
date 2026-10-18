import { z } from "zod";
import { projectRef } from "../asset-ref";
import { itemClip, placeholderClip } from "../compile/clip";
import { formatDuration } from "../format";
import { kindFromPath } from "../mime";
import type { VisualEnvelope } from "../schema/timeline";
import {
  asValue,
  outputOf,
  type CompileValue,
  type MediaValue,
  type NodeResult,
} from "../schema/values";
import { DEFAULT_VIEWPORT } from "../schema/workflow";
import { msToFrames } from "../timeline/time";
import { connectedPorts, fileRow, missing, row, warnIf, type CardRow } from "./card";
import { defineNode, nodeBase, opt, port, type CompileInputs, type NodeDef } from "./define";
import type { NodeIconName } from "./icon-name";
import {
  asset,
  bool,
  choice,
  choiceFrom,
  choiceOf,
  FORMAT_NAMES,
  int,
  loudnessName,
  more,
  ms,
  num,
  quality,
  qualityName,
  rate,
  text,
  visualEnvelope,
} from "./params";

const fileOp = { ...nodeBase("Media", "media"), executes: true };
const one = {
  inputs: { media: port("media", "File") },
  outputs: { media: port("media", "File") },
} as const;
const NEW_FILE = " Makes a new file.";

const lengthRow = (
  result: NodeResult | undefined,
  icon: NodeIconName = "clock",
): CardRow | null => {
  const m = outputOf(result, "media", "media");
  return m?.durationMs ? row(icon, formatDuration(m.durationMs)) : null;
};
const CONVERT_FORMATS = ["h264", "h265", "vp9", "prores"] as const;
const PRESET_LABELS = {
  ultrafast: "Fastest, biggest",
  fast: "Fast",
  medium: "Balanced",
  slow: "Small",
  veryslow: "Smallest, slowest",
};

export function mediaInput(value: CompileValue | undefined): MediaValue | undefined {
  const media = asValue(value, "media");
  if (media) return media;
  const image = asValue(value, "image");
  if (image) {
    return {
      type: "media",
      kind: "image",
      src: image.src,
      width: image.width,
      height: image.height,
    };
  }
  const file = asValue(value, "file");
  if (!file?.src) return undefined;
  return { type: "media", src: file.src, kind: kindFromPath(file.src) };
}

const passMedia = (
  { result, inputs }: { result?: NodeResult; inputs: CompileInputs },
  fallbackPort = "media",
) => ({ media: outputOf(result, "media", "media") ?? mediaInput(inputs[fallbackPort]) });

const FITS = { cover: "Fill the frame", contain: "Show all" };
type Fit = keyof typeof FITS;

const fitField = () => choiceOf(FITS, "contain", "Fit");

type MediaClip = {
  src: string;
  kind: MediaValue["kind"];
  fit: Fit;
  muted: boolean;
  frames: number;
  envelope: VisualEnvelope;
};

const mediaClip = (nodeId: string, { src, kind, fit, muted, frames, envelope }: MediaClip) =>
  itemClip(nodeId, kind === "image" ? "core/image" : "core/video", { src, fit, muted }, frames, {
    envelope,
  });

const IMAGE_MS = 3000;
const MEDIA_MS = 5000;

const op = <S extends z.ZodObject>(def: Omit<NodeDef<S>, "category" | "phase" | "executes">) =>
  defineNode<S>({ ...fileOp, compile: passMedia, ...def });

export const importMediaNode = op({
  type: "media.import",
  title: "Media file",
  icon: "file-play",
  description: "A video, image or sound from your project. Drop files on the canvas to add one.",
  inputs: {},
  outputs: { media: port("media", "File"), clip: port("clip", "Clip") },
  card: ({ params: p, result }) => [
    p.path ? fileRow("file-video", p.path) : missing("file-video", "Choose a file"),
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
      media.durationMs ?? (media.kind === "image" ? IMAGE_MS : MEDIA_MS),
      settings.fps,
    );
    const clip =
      !media.src || media.kind === "audio"
        ? placeholderClip(
            nodeId,
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
  ...nodeBase("Media", "compile"),
  type: "media.toClip",
  title: "Use as clip",
  icon: "film",
  description: "Turns a file into a clip you can place in the video.",
  inputs: { media: port("media", "File") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => [row("maximize", FITS[p.fit]), p.muted && row("volume-x", "Muted")],
  params: z.object({
    fit: fitField(),
    muted: bool(false, "Muted"),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const media = mediaInput(inputs.media);
    const frames = msToFrames(media?.durationMs ?? MEDIA_MS, settings.fps);
    if (!media?.src)
      return {
        clip: placeholderClip(
          nodeId,
          { title: "No file connected", subtitle: "Connect a file to use it as a clip" },
          frames,
          { envelope: params.envelope },
        ),
      };
    return {
      clip: mediaClip(nodeId, {
        src: media.src,
        kind: media.kind,
        fit: params.fit,
        muted: params.muted,
        frames,
        envelope: params.envelope,
      }),
    };
  },
});

const SCALE_MODES = {
  pad: { label: "Add bars", row: "Fit with bars" },
  crop: { label: "Crop to fill", row: "Crop to fill" },
  stretch: { label: "Stretch", row: "Stretch" },
};

export const mediaTrimNode = op({
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
});

const CONCAT_INPUTS = {
  in1: port("media", "File 1"),
  in2: port("media", "File 2"),
  in3: opt("media", "File 3"),
  in4: opt("media", "File 4"),
};

export const concatNode = op({
  type: "media.concat",
  title: "Join files",
  icon: "combine",
  description: "Joins files end to end. Fast when they match, otherwise converts them." + NEW_FILE,
  inputs: CONCAT_INPUTS,
  outputs: { media: port("media", "File") },
  card: ({ connected, result }) => {
    const n = connectedPorts(Object.keys(CONCAT_INPUTS), connected).length;
    return [row("combine", `${n} files`, warnIf(n < 2)), lengthRow(result)];
  },
  params: z.object({ forceReencode: more(bool(false, "Always convert"), { advanced: true }) }),
  compile: (ctx) => passMedia(ctx, "in1"),
});

export const mediaSpeedNode = op({
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
});

export const reverseNode = op({
  ...one,
  type: "media.reverse",
  title: "Reverse",
  icon: "undo-2",
  description: "Plays a file backwards." + NEW_FILE,
  card: ({ result }) => [lengthRow(result)],
  params: z.object({}),
});

export const scalePadNode = op({
  ...one,
  type: "media.scalePad",
  title: "Resize",
  icon: "maximize",
  description: "Resizes a file, adding bars or cropping to fit." + NEW_FILE,
  card: ({ params: p }) => [
    row("ratio", `${p.width} × ${p.height}`),
    row("maximize", SCALE_MODES[p.mode].row),
  ],
  params: z.object({
    width: int(DEFAULT_VIEWPORT.width, "Width", { min: 16, max: 7680, unit: "px" }),
    height: int(DEFAULT_VIEWPORT.height, "Height", { min: 16, max: 7680, unit: "px" }),
    mode: choiceOf(SCALE_MODES, "pad", "Fit"),
    background: more(text("black", "Bar colour"), {
      widget: "color",
      showIf: { mode: ["pad"] },
    }),
  }),
});

export const convertNode = op({
  ...one,
  type: "media.convert",
  title: "Convert",
  icon: "refresh-cw",
  description: "Saves a file in another format or quality." + NEW_FILE,
  card: ({ params: p }) => [
    row("file-video", FORMAT_NAMES[p.codec]),
    row("sparkles", `${qualityName(p.crf)} quality`),
  ],
  params: z.object({
    codec: choiceFrom(FORMAT_NAMES, CONVERT_FORMATS, "h264", "Format"),
    crf: quality(18),
    preset: more(choiceOf(PRESET_LABELS, "medium", "Speed vs file size"), {
      advanced: true,
    }),
  }),
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
    fps: int(15, "Frame rate", { min: 5, max: 50, unit: "fps" }),
    width: int(960, "Width", { min: 64, max: 3840, unit: "px" }),
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
});

export const loudnormNode = op({
  ...one,
  type: "media.loudnorm",
  title: "Even out volume",
  icon: "volume-2",
  description: "Makes the sound as loud as most online videos." + NEW_FILE,
  card: ({ params: p }) => [row("volume-2", loudnessName(p.integrated))],
  params: z.object({
    integrated: num(-16, "Loudness", { min: -40, max: -5, widget: "loudness", unit: "LUFS" }),
    truePeak: num(-1.5, "Peak limit", { min: -9, max: 0, step: 0.1, unit: "dB", advanced: true }),
    lra: num(11, "Dynamic range", { min: 1, max: 20, advanced: true }),
  }),
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
    width: int(1280, "Width", { min: 64, max: 3840, unit: "px" }),
  }),
  compile: ({ result }) => ({ image: outputOf(result, "image", "image") }),
});

export const faststartNode = op({
  ...one,
  type: "media.faststart",
  title: "Web-ready",
  icon: "zap",
  description: "Prepares a video to start playing instantly on the web." + NEW_FILE,
  params: z.object({}),
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
  compile: (ctx) => ({ file: outputOf(ctx.result, "file", "file"), ...passMedia(ctx) }),
});

export const mediaNodes: NodeDef[] = [
  importMediaNode,
  mediaToClipNode,
  mediaTrimNode,
  concatNode,
  mediaSpeedNode,
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
