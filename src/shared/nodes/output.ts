import { z } from "zod";
import { clipToSpec, isClip } from "../compile/clip";
import { outputOf, type Artifact, type CompileValue, type TimelineValue } from "../schema/values";
import type { WorkflowNode, WorkflowSettings } from "../schema/workflow";
import { row } from "./card";
import { defineNode, port, type CompileOutputs } from "./define";
import { bool, choice, keysOf, more, ms, quality, text } from "./params";
import { formatDuration, formatBytes } from "../format";
import { evenPx } from "../math";

const GL_LABELS = {
  angle: "Graphics card",
  swangle: "Software",
  egl: "EGL",
  vulkan: "Vulkan",
  swiftshader: "Software (legacy)",
} as const;

const gl = () =>
  more(choice(keysOf(GL_LABELS), "angle", "Draw with", GL_LABELS), {
    help: "The graphics card is about ten times faster. Use Software if exports fail.",
    advanced: true,
  });

const FORMAT_NAMES = {
  h264: "MP4",
  h265: "MP4 (HEVC)",
  vp8: "WebM (VP8)",
  vp9: "WebM",
  prores: "ProRes",
  gif: "GIF",
};

const FORMATS = keysOf(FORMAT_NAMES);

const VIDEO_CODECS = FORMATS.filter((f) => f !== "gif");

const STILL_NAMES: Record<StillPreset, string> = {
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
type StillPreset = keyof typeof STILL_PRESETS;

export function variantSize(
  variant: Variant,
  settings: Pick<WorkflowSettings, "width" | "height">,
) {
  const long = Math.max(settings.width, settings.height);
  const [w, h] = variant.split(":").map(Number);
  return w >= h
    ? { width: evenPx(long), height: evenPx((long * h) / w) }
    : { width: evenPx((long * w) / h), height: evenPx(long) };
}

const toTimeline = (
  input: CompileValue | undefined,
  settings: WorkflowSettings,
): TimelineValue | undefined => {
  if (input?.type === "timeline") return input;
  if (isClip(input)) return { type: "timeline", spec: clipToSpec(input, settings) };
  return undefined;
};

const passTimeline = (
  inputs: Partial<Record<string, CompileValue>>,
  settings: WorkflowSettings,
): CompileOutputs => {
  const timeline = toTimeline(inputs.timeline, settings);
  return timeline ? { timeline } : {};
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
    const media = outputOf(result, "media", "media");
    const size = fileBytes(result?.artifacts);
    const short = Math.round(Math.min(settings.width, settings.height) * p.scale);
    return [
      row("ratio", p.variants.join(" + ")),
      row("film", `${FORMAT_NAMES[p.codec]} · ${short}p · ${settings.fps} fps`),
      media?.durationMs && row("clock", formatDuration(media.durationMs)),
      size > 0 && row("hard-drive", formatBytes(size)),
    ];
  },
  params: z.object({
    fileName: text("{{Project name}}", "File name"),
    codec: choice(FORMATS, "h264", "Format", FORMAT_NAMES),
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
    chapters: more(bool(true, "Chapters in the file"), {
      help: "Players list the film's chapters, taken from its Chapter actions.",
      showIf: { codec: VIDEO_CODECS },
    }),
    chapterList: more(bool(false, "YouTube chapter list"), {
      help: "Also saves the chapter times as text, to paste into a video's description. It needs at least three chapters of ten seconds or more.",
      showIf: { codec: VIDEO_CODECS },
    }),
    faststart: z.boolean().default(true).meta({ description: "Web-ready", hidden: true }),
    gl: gl(),
  }),
  compile: ({ inputs, settings, result }) => ({
    ...passTimeline(inputs, settings),
    media: outputOf(result, "media", "media"),
  }),
});

export function exportReframe(nodes: WorkflowNode[]): z.infer<typeof renderNode.params>["reframe"] {
  const render = nodes.find((n) => n.type === renderNode.type && !n.disabled);
  const parsed = renderNode.params.shape.reframe.safeParse(render?.params.reframe);
  return parsed.success ? parsed.data : "follow";
}

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
    row("image", p.presets.map((k) => STILL_NAMES[k]).join(" · ")),
    row("clock", `At ${formatDuration(p.atMs)} · ${p.format.toUpperCase()}`),
  ],
  params: z.object({
    atMs: ms(1000, "Frame at"),
    presets: z
      .array(z.enum(keysOf(STILL_PRESETS)))
      .min(1)
      .default(["og"])
      .meta({ description: "Sizes", widget: "multiselect", labels: STILL_NAMES }),
    format: choice(["png", "jpeg"], "png", "Format", { png: "PNG", jpeg: "JPEG" }),
    gl: gl(),
  }),
  compile: ({ result, inputs, settings }) => ({
    image: result?.outputs.image,
    ...passTimeline(inputs, settings),
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
  params: z.object({ folderName: text("video-code", "Folder name") }),
  compile: ({ result, inputs, settings }) => ({
    file: result?.outputs.file,
    ...passTimeline(inputs, settings),
  }),
});

export const outputNodes = [renderNode, stillOutputNode, exportProjectNode];

export const isOutputType = (type: string) => type.startsWith("output.");
