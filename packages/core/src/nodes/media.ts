import { z } from "zod";
import { projectRef } from "#core/asset-ref";
import { itemClip } from "#core/compile/clip";
import { formatDuration, plural } from "#core/format";
import { kindFromPath } from "#core/mime";
import {
  connectedPorts,
  fileRow,
  missing,
  rangeRow,
  row,
  truncate,
  type CardRow,
} from "#core/nodes/card";
import {
  defineNode,
  GRADE_PORT,
  nodeBase,
  opt,
  port,
  type CompileInputs,
  type NodeDef,
} from "#core/nodes/define";
import { gradeChangesNothing, gradeNeedsVideo, gradeStepTitle } from "#core/nodes/grade";
import type { NodeIconName } from "#core/nodes/icon-name";
import {
  asset,
  bool,
  choice,
  choiceFrom,
  choiceOf,
  colorSetting,
  FORMAT_NAMES,
  int,
  loudnessName,
  more,
  ms,
  msAtLeast,
  num,
  quality,
  qualityName,
  rate,
  visualEnvelope,
} from "#core/nodes/params";
import type { EditHandles, VisualEnvelope } from "#core/schema/timeline";
import {
  asValue,
  outputOf,
  type ClipValue,
  type MediaValue,
  type NodeResult,
} from "#core/schema/values";
import { DEFAULT_STILL_MS, DEFAULT_VIEWPORT } from "#core/schema/workflow";
import { msToFrames } from "#core/timeline/time";

const fileOp = { ...nodeBase("Media", "media"), executes: true };
const one = {
  inputs: { media: port("media", "File") },
  outputs: { media: port("media", "File") },
} as const;
const NEW_FILE = " Makes a new file.";
const CONVERT_FORMATS = ["h264", "h265", "vp9", "prores"] as const;
const QUALITY_FORMATS: ReadonlyArray<(typeof CONVERT_FORMATS)[number]> = ["h264", "h265", "vp9"];
const PRESET_FORMATS: ReadonlyArray<(typeof CONVERT_FORMATS)[number]> = ["h264", "h265"];
const PRESET_LABELS = {
  ultrafast: "Fastest, biggest",
  fast: "Fast",
  medium: "Balanced",
  slow: "Small",
  veryslow: "Smallest, slowest",
};
const FITS = { cover: "Fill the frame", contain: "Show all" };
type Fit = keyof typeof FITS;
const MEDIA_MS = 5000;

const formatRow = (p: { codec: (typeof CONVERT_FORMATS)[number]; crf: number }): CardRow =>
  row(
    "file-video",
    [
      FORMAT_NAMES[p.codec],
      QUALITY_FORMATS.includes(p.codec) && `${qualityName(p.crf)} quality`,
      p.codec === "prores" && "no Preview",
      p.codec === "h265" && "may not preview",
    ]
      .filter(Boolean)
      .join(" · "),
  );

const lengthRow = (
  result: NodeResult | undefined,
  icon: NodeIconName = "clock",
): CardRow | null => {
  const m = outputOf(result, "media", "media");
  return m?.durationMs ? row(icon, formatDuration(m.durationMs)) : null;
};

const passMedia = (
  { result, inputs }: { result?: NodeResult; inputs: CompileInputs },
  fallbackPort = "media",
) => ({ media: outputOf(result, "media", "media") ?? asValue(inputs[fallbackPort], "media") });

const fitField = () => choiceOf(FITS, "contain", "Fit");

type ClipLook = {
  fit: Fit;
  muted: boolean;
  envelope: VisualEnvelope;
  fps: number;
  unlinked: { title: string; subtitle: string };
  soundLabel: string;
  still?: { ms: number; handles: EditHandles };
};

function clipMs(media: MediaValue | undefined, still: ClipLook["still"]): number {
  if (media?.kind === "image") return still?.ms ?? media.durationMs ?? DEFAULT_STILL_MS;
  return media?.durationMs ?? MEDIA_MS;
}

function mediaToClip(
  nodeId: string,
  media: MediaValue | undefined,
  { fit, muted, envelope, fps, unlinked, soundLabel, still }: ClipLook,
): ClipValue {
  const frames = msToFrames(clipMs(media, still), fps);
  if (!media?.src) return itemClip(nodeId, "core/placeholder", unlinked, frames, { envelope });
  if (media.kind === "audio")
    return itemClip(
      nodeId,
      "core/placeholder",
      { title: "Sound file", subtitle: soundLabel },
      frames,
      {
        envelope,
      },
    );
  if (media.kind === "image")
    return itemClip(nodeId, "core/image", { src: media.src, fit, muted }, frames, {
      envelope,
      ...(still ? { handles: still.handles } : {}),
    });
  return itemClip(
    nodeId,
    "core/video",
    { src: media.src, fit, muted: muted || media.hasAudio === false },
    frames,
    { envelope },
  );
}

const op = <S extends z.ZodObject>(def: Omit<NodeDef<S>, "category" | "phase" | "executes">) =>
  defineNode<S>({ ...fileOp, compile: passMedia, ...def });

export const importMediaNode = op({
  type: "media.import",
  title: "Media file",
  description: "A video, image or sound from your project. Drop files on the canvas to add one.",
  inputs: {},
  outputs: { media: port("media", "File"), clip: port("clip", "Clip") },
  card: ({ params: p, result }) => [
    p.path ? fileRow("file-video", p.path) : missing("file-video", "Choose a file"),
    p.path && kindFromPath(p.path) === "image"
      ? row("clock", formatDuration(p.stillMs))
      : lengthRow(result),
  ],
  params: z.object({
    path: asset("File"),
    fit: fitField(),
    stillMs: more(msAtLeast(DEFAULT_STILL_MS, "Length", 100, 3_600_000), {
      help: "How long the image stays on screen.",
      showIfFile: { path: ["image"] },
    }),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, result, params, settings }) => {
    const media: MediaValue = outputOf(result, "media", "media") ?? {
      type: "media",
      src: params.path ? projectRef(params.path) : null,
      kind: kindFromPath(params.path),
    };
    const clip = mediaToClip(nodeId, media, {
      fit: params.fit,
      muted: false,
      envelope: params.envelope,
      fps: settings.fps,
      unlinked: { title: "No file chosen", subtitle: params.path },
      soundLabel: params.path,
      still: {
        ms: params.stillMs,
        handles: { trimEnd: { nodeId, param: "stillMs", unit: "ms" } },
      },
    });
    return { media, clip };
  },
});

const mediaToClipNode = defineNode({
  ...nodeBase("Media", "compile"),
  type: "media.toClip",
  title: "Use as clip",
  description: "Turns a file into a clip you can place in the video.",
  inputs: { media: port("media", "File") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => [row("maximize", FITS[p.fit]), p.muted && row("volume-x", "Muted")],
  params: z.object({
    fit: fitField(),
    muted: bool(false, "Muted"),
    envelope: visualEnvelope(),
  }),
  compile: ({ nodeId, inputs, params, settings }) => ({
    clip: mediaToClip(nodeId, asValue(inputs.media, "media"), {
      fit: params.fit,
      muted: params.muted,
      envelope: params.envelope,
      fps: settings.fps,
      unlinked: { title: "No file connected", subtitle: "Connect a file to use it as a clip" },
      soundLabel: "Sound files can't be shown as a clip",
    }),
  }),
});

const SCALE_MODES = {
  pad: { label: "Add bars", row: "Fit with bars" },
  crop: { label: "Crop to fill", row: "Crop to fill" },
  stretch: { label: "Stretch", row: "Stretch" },
};

const MAX_FILE_MS = 36_000_000;

export const mediaTrimNode = op({
  ...one,
  type: "media.trim",
  title: "Trim file",
  description: "Cuts a file down to a start and end." + NEW_FILE,
  card: ({ params: p, result }) => [rangeRow(p.startMs, p.endMs || null), lengthRow(result)],
  params: z
    .object({
      startMs: ms(0, "Start", MAX_FILE_MS),
      endMs: more(ms(0, "End", MAX_FILE_MS), { zeroLabel: "To the end" }),
    })
    .superRefine((p, ctx) => {
      if (p.endMs && p.endMs <= p.startMs)
        ctx.addIssue({ code: "custom", message: "End must come after Start", path: ["endMs"] });
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
  usesProjectSize: true,
  description: "Joins files end to end. Fast when they match, otherwise converts them." + NEW_FILE,
  inputs: CONCAT_INPUTS,
  outputs: { media: port("media", "File") },
  card: ({ connected, result }) => {
    const n = connectedPorts(Object.keys(CONCAT_INPUTS), connected).length;
    return [
      n < 2
        ? missing("combine", n ? "Connect one more file" : "Connect two files")
        : row("combine", plural(n, "file")),
      lengthRow(result),
    ];
  },
  params: z.object({ forceReencode: more(bool(false, "Always convert"), { advanced: true }) }),
  compile: (ctx) => passMedia(ctx, "in1"),
});

export const mediaSpeedNode = op({
  ...one,
  type: "media.speed",
  title: "Change speed",
  usesProjectSize: true,
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
  usesProjectSize: true,
  title: "Reverse",
  description: "Plays a file backwards." + NEW_FILE,
  card: ({ result }) => [row("refresh-cw", "Plays backwards"), lengthRow(result)],
  params: z.object({}),
});

export const scalePadNode = op({
  ...one,
  type: "media.scalePad",
  title: "Resize",
  description: "Resizes a file, adding bars or cropping to fit." + NEW_FILE,
  card: ({ params: p }) => [
    row("ratio", `${p.width} × ${p.height}`),
    row("maximize", SCALE_MODES[p.mode].row),
  ],
  params: z.object({
    width: int(DEFAULT_VIEWPORT.width, "Width", {
      min: 16,
      max: 7680,
      step: 2,
      unit: "px",
      help: "A video takes even sizes; an odd one loses a pixel.",
    }),
    height: int(DEFAULT_VIEWPORT.height, "Height", {
      min: 16,
      max: 7680,
      step: 2,
      unit: "px",
      help: "A video takes even sizes; an odd one loses a pixel.",
    }),
    mode: choiceOf(SCALE_MODES, "pad", "Fit"),
    background: more(colorSetting("black", "Bar colour"), { showIf: { mode: ["pad"] } }),
  }),
});

export const convertNode = op({
  ...one,
  type: "media.convert",
  title: "Convert",
  description: "Saves a file in another format or quality." + NEW_FILE,
  card: ({ params: p }) => [formatRow(p)],
  params: z.object({
    codec: choiceFrom(FORMAT_NAMES, CONVERT_FORMATS, "h264", "Format"),
    crf: more(quality(18), { showIf: { codec: [...QUALITY_FORMATS] } }),
    preset: more(choiceOf(PRESET_LABELS, "medium", "Speed vs file size"), {
      advanced: true,
      showIf: { codec: [...PRESET_FORMATS] },
    }),
  }),
});

export const applyGradeNode = op({
  type: "media.applyGrade",
  title: "Apply grade",
  description:
    "Applies a colour grade to a whole video or image file: everything in the picture, window and backdrop included." +
    NEW_FILE,
  inputs: { media: port("media", "File"), [GRADE_PORT]: port("grade", "Grade") },
  outputs: { media: port("media", "File") },
  card: ({ params: p, inputGrade, inputMedia, result }) => {
    const entries = inputGrade?.[GRADE_PORT];
    const image = inputMedia?.media?.kind === "image";
    const titles = [...new Set((entries ?? []).map((e) => gradeStepTitle(e.type)))];
    return [
      entries?.length
        ? row("palette", truncate(`${plural(entries.length, "step")}: ${titles.join(", ")}`))
        : missing("palette", "Connect a grade"),
      entries?.length &&
        gradeChangesNothing(entries) &&
        missing("triangle-alert", "The grade changes nothing"),
      image ? row("image", "PNG") : formatRow(p),
      image && entries && gradeNeedsVideo(entries)
        ? missing("triangle-alert", "Trails and film grain need a video")
        : lengthRow(result),
    ];
  },
  params: z.object({
    codec: choiceFrom(FORMAT_NAMES, CONVERT_FORMATS, "h264", "Format"),
    crf: more(quality(18), { showIf: { codec: [...QUALITY_FORMATS] } }),
    preset: more(choiceOf(PRESET_LABELS, "medium", "Speed vs file size"), {
      advanced: true,
      showIf: { codec: [...PRESET_FORMATS] },
    }),
  }),
});

export const gifNode = op({
  ...one,
  type: "media.gif",
  title: "Make GIF",
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
  description: "Prepares a video to start playing instantly on the web." + NEW_FILE,
  card: ({ result }) => [row("globe", "Starts playing before it fully loads"), lengthRow(result)],
  params: z.object({}),
});

export const probeNode = op({
  type: "media.probe",
  title: "File info",
  description: "Shows a file's length, size, frame rate and format.",
  inputs: { media: port("media", "File") },
  outputs: { info: port("file-info", "Info"), media: port("media", "File") },
  card: ({ result }) => [
    lengthRow(result, "info") ?? row("info", "Run to read the file", { tone: "muted" }),
  ],
  params: z.object({}),
  compile: (ctx) => ({ info: outputOf(ctx.result, "info", "file-info"), ...passMedia(ctx) }),
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
  applyGradeNode,
  gifNode,
  extractAudioNode,
  loudnormNode,
  thumbnailNode,
  faststartNode,
  probeNode,
];
