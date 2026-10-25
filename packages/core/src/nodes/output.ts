import { z } from "zod";
import { formatDuration, formatBytes } from "../format";
import { parseOrDefault } from "../guards";
import { evenPx } from "../math";
import { safeName } from "../safe-name";
import { REFRAMES, type ReframeMode } from "../schema/timeline";
import { outputOf, type Artifact } from "../schema/values";
import type { WorkflowNode, WorkflowSettings } from "../schema/workflow";
import { framesToMs } from "../timeline/time";
import { PROJECT_NAME } from "../vars";
import { row } from "./card";
import { CLIP_PORT, defineNode, port } from "./define";
import {
  bool,
  choice,
  choiceOf,
  FORMAT_NAMES,
  keysOf,
  more,
  ms,
  num,
  quality,
  text,
} from "./params";

const GL_LABELS = {
  angle: "Graphics card",
  swangle: "Software",
  egl: "Graphics card, direct",
  vulkan: "Graphics card, newer drivers",
  swiftshader: "Software (legacy)",
} as const;

const gl = () =>
  more(choiceOf(GL_LABELS, "angle", "Draw with"), {
    help: "The graphics card is about ten times faster. Use Software if exports fail.",
    advanced: true,
  });

const FORMATS = keysOf(FORMAT_NAMES);

export const EXPORT_FOLDER_FALLBACK = "output";

const VIDEO_CODECS = FORMATS.filter((f) => f !== "gif");

export const CODEC_EXTENSIONS: Record<(typeof FORMATS)[number], string> = {
  h264: "mp4",
  h265: "mp4",
  vp8: "webm",
  vp9: "webm",
  prores: "mov",
  gif: "gif",
};

const eachOnce = <T>(list: readonly T[]) => new Set(list).size === list.length;

export const videoFileNames = (
  base: string,
  codec: (typeof FORMATS)[number],
  variants: readonly string[],
): string[] =>
  variants.map(
    (v, i) => `${base}${i === 0 ? "" : `-${v.replace(":", "x")}`}.${CODEC_EXTENSIONS[codec]}`,
  );

export const stillFileNames = (
  base: string,
  presets: readonly string[],
  format: "png" | "jpeg",
): string[] => presets.map((preset) => `${base}-${preset}.${format === "jpeg" ? "jpg" : "png"}`);

export const exportBaseName = (fileName: string): string =>
  safeName(fileName, { fallback: EXPORT_FOLDER_FALLBACK });

const fileBytes = (artifacts?: Artifact[]) =>
  (artifacts ?? []).reduce((sum, a) => sum + (a.bytes ?? 0), 0);

export const VARIANTS = ["16:9", "1:1", "4:5", "9:16"] as const;
export type Variant = (typeof VARIANTS)[number];

export const STILL_PRESETS = {
  wide: { width: 3840, height: 2160 },
  og: { width: 2400, height: 1260 },
  square: { width: 2160, height: 2160 },
  portrait: { width: 1728, height: 2160 },
  story: { width: 1080, height: 1920 },
} as const;
type StillPreset = keyof typeof STILL_PRESETS;

const STILL_NAMES: Record<StillPreset, string> = {
  wide: "Wide 4K",
  og: "Link preview",
  square: "Square",
  portrait: "Portrait",
  story: "Story",
};

const VARIANT_RATIOS: Record<Variant, [width: number, height: number]> = {
  "16:9": [16, 9],
  "1:1": [1, 1],
  "4:5": [4, 5],
  "9:16": [9, 16],
};

function sizeFor(variant: Variant, edge: number, anchor: "long" | "short") {
  const [w, h] = VARIANT_RATIOS[variant];
  const landscape = w >= h;
  const edgeIsWidth = anchor === "long" ? landscape : !landscape;
  return edgeIsWidth
    ? { width: evenPx(edge), height: evenPx((edge * h) / w) }
    : { width: evenPx((edge * w) / h), height: evenPx(edge) };
}

export const variantSize = (
  variant: Variant,
  settings: Pick<WorkflowSettings, "width" | "height">,
) => sizeFor(variant, Math.max(settings.width, settings.height), "long");

export const variantCanvas = (variant: Variant, short: number) => sizeFor(variant, short, "short");

const exportedShortSide = (variant: Variant, settings: WorkflowSettings, scale: number) => {
  const size = variantSize(variant, settings);
  return Math.round(Math.min(size.width, size.height) * scale);
};

const fileNameField = () => text(PROJECT_NAME, "File name");

const base = {
  category: "Output",
  executes: true,
  inputs: { [CLIP_PORT]: port("clip", "Clip") },
} as const;

export const renderNode = defineNode({
  ...base,
  type: "output.render",
  title: "Export video",
  phase: "render",
  description: "Renders the final video, one file per shape (16:9, 9:16…).",
  outputs: { media: port("media", "File") },
  card: ({ params: p, settings, result }) => {
    const media = outputOf(result, "media", "media");
    const bytes = fileBytes(result?.artifacts);
    const sides = new Set(p.variants.map((v) => `${exportedShortSide(v, settings, p.scale)}p`));
    return [
      row("ratio", p.variants.join(" + ")),
      row("film", `${FORMAT_NAMES[p.codec]} · ${[...sides].join(" / ")} · ${settings.fps} fps`),
      media?.durationMs && row("clock", formatDuration(media.durationMs)),
      bytes > 0 && row("hard-drive", formatBytes(bytes)),
    ];
  },
  params: z.object({
    fileName: fileNameField(),
    codec: choice(FORMATS, "h264", "Format", FORMAT_NAMES),
    crf: quality(18),
    variants: z
      .array(z.enum(VARIANTS))
      .min(1)
      .refine(eachOnce, "Pick each shape once")
      .default(["16:9"])
      .meta({ description: "Shapes", widget: "multiselect" }),
    reframe: choice(REFRAMES, "follow", "Other shapes", {
      follow: "Follow the action",
      center: "Crop the centre",
      fit: "Add bars",
    }),
    scale: num(1, "Size", {
      min: 0.25,
      max: 2,
      step: 0.25,
      unit: "×",
      labels: { "0.5": "Half", "1": "Full", "2": "Double" },
    }),
    chapters: more(bool(true, "Chapters in the file"), {
      help: "Players list the film's chapters, taken from its Chapter actions.",
      showIf: { codec: VIDEO_CODECS },
    }),
    chapterList: more(bool(false, "YouTube chapter list"), {
      help: "Also saves the chapter times as text, to paste into a video's description. It needs at least three chapters of ten seconds or more.",
      showIf: { codec: VIDEO_CODECS },
    }),
    gl: gl(),
  }),
  compile: ({ result }) => ({ media: outputOf(result, "media", "media") }),
});

export function exportReframe(nodes: WorkflowNode[]): ReframeMode {
  const render = nodes.find((n) => n.type === renderNode.type && !n.disabled);
  return parseOrDefault(renderNode.params.shape.reframe, render?.params.reframe);
}

export const stillOutputNode = defineNode({
  ...base,
  type: "output.still",
  title: "Export images",
  phase: "render",
  description:
    "Saves a frame as images at social sizes: wide, link preview, square, portrait, story.",
  outputs: { image: port("image", "Image") },
  card: ({ params: p, inputFrames, settings }) => {
    const filmMs = inputFrames?.clip && framesToMs(inputFrames.clip, settings.fps);
    return [
      row("image", p.presets.map((k) => STILL_NAMES[k]).join(" · ")),
      row("clock", `At ${formatDuration(p.atMs)} · ${p.format.toUpperCase()}`),
      filmMs &&
        p.atMs >= filmMs &&
        row(
          "triangle-alert",
          `The film ends at ${formatDuration(filmMs)}, so it uses the last frame`,
          {
            tone: "warn",
          },
        ),
    ];
  },
  params: z.object({
    fileName: fileNameField(),
    atMs: ms(1000, "Frame at"),
    presets: z
      .array(z.enum(keysOf(STILL_PRESETS)))
      .min(1)
      .refine(eachOnce, "Pick each size once")
      .default(["og"])
      .meta({ description: "Sizes", widget: "multiselect", labels: STILL_NAMES }),
    format: choice(["png", "jpeg"], "png", "Format", { png: "PNG", jpeg: "JPEG" }),
    gl: gl(),
  }),
  compile: ({ result }) => ({ image: outputOf(result, "image", "image") }),
});

export const exportProjectNode = defineNode({
  ...base,
  type: "output.exportProject",
  title: "Export as code",
  phase: "post",
  description: "Saves this video as an editable code project for developers.",
  outputs: { folder: port("folder", "Folder") },
  card: ({ params: p }) => [
    row("folder-output", safeName(p.folderName, { fallback: EXPORT_FOLDER_FALLBACK })),
  ],
  params: z.object({ folderName: text("video-code", "Folder name") }),
  compile: ({ result }) => ({ folder: outputOf(result, "folder", "folder") }),
});

export const outputNodes = [renderNode, stillOutputNode, exportProjectNode];
