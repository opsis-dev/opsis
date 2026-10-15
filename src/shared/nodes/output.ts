import { z } from "zod";
import { clipToSpec, isClip } from "../compile/clip";
import type { CompileValue, MediaValue, TimelineValue } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import type { Artifact } from "../schema/values";
import { bytes, row, secs } from "./card";
import { defineNode, port } from "./define";
import { choice, more, ms, quality, text } from "./params";

const GL_LABELS = {
  angle: "Graphics card",
  swangle: "Software",
  egl: "EGL",
  vulkan: "Vulkan",
  swiftshader: "Software (legacy)",
} as const;

const gl = () =>
  choice(
    ["angle", "swangle", "egl", "vulkan", "swiftshader"],
    "angle",
    "Draw with",
    GL_LABELS,
  ).meta({
    description: "Draw with",
    help: "The graphics card is about ten times faster. Use Software if exports fail.",
    labels: GL_LABELS,
    advanced: true,
  });

const FORMAT_NAMES: Record<string, string> = {
  h264: "MP4",
  h265: "MP4 (HEVC)",
  vp8: "WebM (VP8)",
  vp9: "WebM",
  prores: "ProRes",
  gif: "GIF",
};

const STILL_NAMES: Record<string, string> = {
  wide: "Wide 4K",
  og: "Link preview",
  square: "Square",
  portrait: "Portrait",
  story: "Story",
};

const fileBytes = (artifacts?: Artifact[]) =>
  artifacts?.reduce((sum, a) => sum + (a.bytes ?? 0), 0) ?? 0;

export const VARIANTS = ["16:9", "1:1", "4:5", "9:16"] as const;
export type Variant = (typeof VARIANTS)[number];

export const STILL_PRESETS = {
  wide: { width: 3840, height: 2160 },
  og: { width: 2400, height: 1260 },
  square: { width: 2160, height: 2160 },
  portrait: { width: 1728, height: 2160 },
  story: { width: 1080, height: 1920 },
} as const;
export type StillPreset = keyof typeof STILL_PRESETS;

export function variantSize(
  variant: Variant,
  settings: Pick<WorkflowSettings, "width" | "height">,
) {
  const long = Math.max(settings.width, settings.height);
  const [w, h] = variant.split(":").map(Number);
  const even = (n: number) => Math.round(n / 2) * 2;
  return w >= h
    ? { width: even(long), height: even((long * h) / w) }
    : { width: even((long * w) / h), height: even(long) };
}

export const toTimeline = (
  input: CompileValue | undefined,
  settings: WorkflowSettings,
): TimelineValue | undefined => {
  if (input?.type === "timeline") return input;
  if (isClip(input)) return { type: "timeline", spec: clipToSpec(input, settings) };
  return undefined;
};

export const renderNode = defineNode({
  type: "output.render",
  title: "Export video",
  category: "Output",
  icon: "clapperboard",
  phase: "render",
  executes: true,
  description: "Renders the final video, one file per shape (16:9, 9:16…).",
  inputs: { timeline: port("timeline", "Clip") },
  outputs: { media: port("media", "File") },
  card: ({ params: p, settings, result }) => {
    const media = result?.outputs.media as MediaValue | undefined;
    const size = fileBytes(result?.artifacts);
    const short = Math.round(Math.min(settings.width, settings.height) * p.scale);
    return [
      row("ratio", p.variants.join(" + ")),
      row("film", `${FORMAT_NAMES[p.codec] ?? p.codec} · ${short}p · ${settings.fps} fps`),
      media?.durationMs && row("clock", secs(media.durationMs)),
      size > 0 && row("hard-drive", bytes(size)),
    ];
  },
  params: z.object({
    fileName: text("{{Project name}}", "File name"),
    codec: choice(["h264", "h265", "vp8", "vp9", "prores", "gif"], "h264", "Format", FORMAT_NAMES),
    crf: quality(18),
    variants: z
      .array(z.enum(VARIANTS))
      .min(1)
      .default(["16:9"])
      .meta({ description: "Shapes", widget: "multiselect" }),
    reframe: choice(["follow", "center", "fit"], "follow", "Other shapes", {
      follow: "Follow the action",
      center: "Crop the centre",
      fit: "Add bars",
    }),
    scale: z
      .number()
      .min(0.25)
      .max(2)
      .default(1)
      .meta({
        description: "Size",
        step: 0.25,
        unit: "×",
        labels: { "0.5": "Half", "1": "Full", "2": "Double" },
      }),
    faststart: z.boolean().default(true).meta({ description: "Web-ready", hidden: true }),
    gl: gl(),
  }),
  compile: ({ inputs, settings, result }) => ({
    ...(toTimeline(inputs.timeline, settings)
      ? { timeline: toTimeline(inputs.timeline, settings) }
      : {}),
    media: result?.outputs.media as MediaValue | undefined,
  }),
});

export const stillOutputNode = defineNode({
  type: "output.still",
  title: "Export images",
  category: "Output",
  icon: "image-down",
  phase: "render",
  executes: true,
  description:
    "Saves a frame as images at social sizes: wide, link preview, square, portrait, story.",
  inputs: { timeline: port("timeline", "Clip") },
  outputs: { image: port("image", "Image") },
  card: ({ params: p }) => [
    row("image", p.presets.map((k) => STILL_NAMES[k] ?? k).join(" · ")),
    row("clock", `At ${secs(p.atMs)} · ${p.format.toUpperCase()}`),
  ],
  params: z.object({
    atMs: ms(1000, "Frame at"),
    presets: z
      .array(z.enum(Object.keys(STILL_PRESETS) as [StillPreset, ...StillPreset[]]))
      .min(1)
      .default(["og"])
      .meta({ description: "Sizes", widget: "multiselect", labels: STILL_NAMES }),
    format: choice(["png", "jpeg"], "png", "Format", { png: "PNG", jpeg: "JPEG" }),
    gl: gl(),
  }),
  compile: ({ result, inputs, settings }) => ({
    image: result?.outputs.image,
    ...(toTimeline(inputs.timeline, settings)
      ? { timeline: toTimeline(inputs.timeline, settings) }
      : {}),
  }),
});

export const exportProjectNode = defineNode({
  type: "output.exportProject",
  title: "Export as code",
  category: "Output",
  icon: "folder-code",
  phase: "post",
  executes: true,
  description: "Saves this video as an editable code project for developers.",
  inputs: { timeline: port("timeline", "Clip") },
  outputs: { file: port("file", "Folder") },
  card: ({ params: p }) => [row("folder-output", p.folderName)],
  params: z.object({ folderName: more(text("video-code", "Folder name"), {}) }),
  compile: ({ result, inputs, settings }) => ({
    file: result?.outputs.file,
    ...(toTimeline(inputs.timeline, settings)
      ? { timeline: toTimeline(inputs.timeline, settings) }
      : {}),
  }),
});

export const outputNodes = [renderNode, stillOutputNode, exportProjectNode];

export const isOutputType = (type: string) => type.startsWith("output.");
