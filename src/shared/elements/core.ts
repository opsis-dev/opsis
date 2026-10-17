import { zColor } from "@remotion/zod-types";
import { z } from "zod";
import { BACKDROPS } from "../backdrops";
import { defineElementMeta, type ElementMeta, type ElementMetaInput } from "./types";

const source = {
  registry: "Demoflow",
  url: "https://github.com/bismitpanda/demoflow",
  license: "Apache-2.0",
};
const text = (def: string, description: string) => z.string().default(def).meta({ description });
const num = (def: number, min: number, max: number, description: string, step = 1) =>
  z.number().min(min).max(max).default(def).meta({ description, step });
const color = (def: string, description: string) => zColor().default(def).meta({ description });
const font = () =>
  z.string().default("").meta({
    description: "Font",
    widget: "font",
    help: "Project font follows the fonts set for the whole project.",
  });
const asset = (description: string, accept: string) =>
  z.string().default("").meta({ description, widget: "asset", accept });
const background = (description = "Background") =>
  z.string().default("transparent").meta({ description });
const direction = <const T extends readonly [string, ...string[]]>(values: T, def: T[number]) =>
  z.enum(values).default(def).meta({ description: "Direction" });

const ENTRANCES = ["rise", "fade", "blur", "scale", "none"] as const;
const DIRECTIONS = ["from-left", "from-right", "from-top", "from-bottom"] as const;

export const CAPTIONS_ELEMENT = "core/captions";

export const captionsSchema = z.object({
  style: z
    .enum(["word-by-word", "karaoke", "line"])
    .default("word-by-word")
    .meta({ description: "Style" }),
  position: z.enum(["bottom", "center", "top"]).default("bottom").meta({ description: "Position" }),
  fontSize: num(56, 16, 140, "Font size"),
  color: color("#ffffff", "Text color"),
  highlightColor: color("#facc15", "Highlight color"),
  fontFamily: font(),
});

const components: Array<Omit<ElementMetaInput, "source" | "fixedProps">> = [
  {
    key: "core/title",
    title: "Title card",
    description: "A full-screen title with an optional eyebrow and subtitle.",
    category: "text",
    kind: "component",
    schema: z.object({
      eyebrow: text("", "Eyebrow"),
      title: text("Sign up in 10 seconds", "Title"),
      subtitle: text("", "Subtitle"),
      align: z.enum(["center", "left"]).default("center").meta({ description: "Alignment" }),
      entrance: z.enum(ENTRANCES).default("rise").meta({ description: "Entrance" }),
      color: color("#fafafa", "Text color"),
      accent: color("#a78bfa", "Accent"),
      background: background("Background (color, gradient or 'transparent')"),
      fontSize: num(112, 24, 260, "Font size"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 75,
  },
  {
    key: "core/kinetic-headline",
    title: "Animated headline",
    description: "A headline that animates in word by word.",
    category: "text",
    kind: "component",
    schema: z.object({
      text: text(
        "Ship *faster* with demos that *move*",
        "Text — wrap words in *asterisks* to emphasise",
      ),
      color: color("#fafafa", "Text color"),
      emphasisColor: color("#facc15", "Emphasis color"),
      background: background(),
      fontSize: num(104, 24, 240, "Font size"),
      wordStagger: num(4, 1, 20, "Frames between words"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 90,
  },
  {
    key: "core/lower-third",
    title: "Lower third",
    description: "A name or caption bar in the lower part of the screen.",
    category: "overlay",
    kind: "component",
    schema: z.object({
      title: text("Ada Lovelace", "Title"),
      subtitle: text("Founder, Acme", "Subtitle"),
      position: z
        .enum(["bottom-left", "bottom-right", "top-left", "top-right"])
        .default("bottom-left")
        .meta({ description: "Position" }),
      accent: color("#a78bfa", "Accent"),
      color: color("#fafafa", "Text color"),
      panel: color("rgba(10,10,12,0.72)", "Panel color"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 120,
  },
  {
    key: "core/callout",
    title: "Callout",
    description: "A short note that points something out.",
    category: "overlay",
    kind: "component",
    schema: z.object({
      text: text("Click here", "Text"),
      x: num(0.5, 0, 1, "Target X (0–1)", 0.01),
      y: num(0.5, 0, 1, "Target Y (0–1)", 0.01),
      side: z
        .enum(["top", "bottom", "left", "right"])
        .default("top")
        .meta({ description: "Label side" }),
      color: color("#fafafa", "Label color"),
      accent: color("#f43f5e", "Arrow color"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 75,
  },
  {
    key: CAPTIONS_ELEMENT,
    title: "Captions",
    description: "Captions shown over the video.",
    category: "overlay",
    kind: "component",
    schema: captionsSchema,
    defaultDurationInFrames: 150,
  },
  {
    key: "core/image",
    title: "Image",
    description: "Shows an image from your project.",
    category: "core",
    kind: "component",
    schema: z.object({
      src: asset("Image", "image/*"),
      fit: z.enum(["contain", "cover", "fill"]).default("contain").meta({ description: "Fit" }),
      background: background(),
      kenBurns: z.boolean().default(false).meta({ description: "Slow push-in" }),
    }),
    defaultDurationInFrames: 90,
  },
  {
    key: "core/mosaic",
    title: "Mosaic",
    description:
      "Covers parts of the screen with coarse blocks, to hide keys, emails and personal details.",
    category: "effect",
    kind: "wrapper",
    schema: z.object({
      boxes: z
        .array(
          z.object({
            x: z.number().min(0).max(1),
            y: z.number().min(0).max(1),
            width: z.number().min(0).max(1),
            height: z.number().min(0).max(1),
          }),
        )
        .max(8)
        .default([{ x: 0.62, y: 0.06, width: 0.3, height: 0.08 }])
        .meta({
          description: "Areas to cover",
          widget: "boxes",
          help: "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera.",
        }),
      blockSize: num(24, 6, 72, "Block size", 2),
    }),
    defaultDurationInFrames: 90,
  },
  {
    key: "core/lottie",
    title: "Lottie animation",
    description:
      "Plays an animation made in After Effects or downloaded from LottieFiles (.json or .lottie).",
    category: "overlay",
    kind: "component",
    schema: z.object({
      src: asset("Animation file", ".json,.lottie"),
      loop: z.boolean().default(true).meta({ description: "Loop" }),
      speed: num(1, 0.25, 4, "Speed", 0.25),
      backwards: z.boolean().default(false).meta({ description: "Play backwards" }),
      fit: z
        .enum(["contain", "cover"])
        .default("contain")
        .meta({ description: "Fit", labels: { contain: "Show all", cover: "Fill the frame" } }),
      background: background(),
    }),
    defaultDurationInFrames: 90,
  },
  {
    key: "core/video",
    title: "Video",
    description: "Plays a video from your project.",
    category: "core",
    kind: "component",
    schema: z.object({
      src: asset("Video", "video/*"),
      fit: z.enum(["contain", "cover", "fill"]).default("cover").meta({ description: "Fit" }),
      loop: z.boolean().default(false).meta({ description: "Loop" }),
      muted: z.boolean().default(true).meta({ description: "Muted" }),
      volume: num(1, 0, 2, "Volume", 0.05),
      playbackRate: num(1, 0.25, 4, "Speed", 0.05),
      trimBeforeMs: num(0, 0, 3_600_000, "Start at (ms)", 100),
    }),
    defaultDurationInFrames: 150,
  },
  {
    key: "core/solid",
    title: "Solid colour",
    description: "A plain colour background.",
    category: "background",
    kind: "component",
    schema: z.object({ color: color("#09090b", "Color") }),
    defaultDurationInFrames: 90,
  },
  {
    key: "core/gradient",
    title: "Gradient",
    description: "A gradient background.",
    category: "background",
    kind: "component",
    schema: z.object({
      from: color("#1e1b4b", "From"),
      to: color("#0f172a", "To"),
      angle: num(135, 0, 360, "Angle"),
      grain: num(0.08, 0, 0.4, "Grain", 0.01),
    }),
    defaultDurationInFrames: 90,
  },
  {
    key: "core/noise",
    title: "Noise",
    description: "A soft moving noise background.",
    category: "background",
    kind: "component",
    schema: z.object({
      color1: color("#0b1020", "Base"),
      color2: color("#4f46e5", "Highlight"),
      scale: num(0.004, 0.0005, 0.02, "Scale", 0.0005),
      speed: num(0.004, 0, 0.05, "Speed", 0.001),
    }),
    defaultDurationInFrames: 150,
  },
  {
    key: "core/starburst",
    title: "Starburst",
    description: "Rays spreading from the centre.",
    category: "background",
    kind: "component",
    schema: z.object({
      color1: color("#18181b", "Ray A"),
      color2: color("#27272a", "Ray B"),
      rays: num(24, 6, 64, "Rays"),
      rotationSpeed: num(0.15, 0, 2, "Rotation (deg/frame)", 0.05),
    }),
    defaultDurationInFrames: 150,
  },
  {
    key: "core/paper",
    title: "Paper",
    description: "A paper texture background.",
    category: "background",
    kind: "component",
    schema: z.object({
      color: color("#f5f1e8", "Paper color"),
      grain: num(0.18, 0, 0.5, "Grain", 0.01),
    }),
    defaultDurationInFrames: 150,
  },
  {
    key: "core/backdrop",
    title: "Studio backdrop",
    description: "A ready-made studio background image.",
    category: "background",
    kind: "component",
    schema: z.object({
      name: z.enum(BACKDROPS).default("glaze").meta({ description: "Backdrop" }),
    }),
    defaultDurationInFrames: 150,
  },
  {
    key: "core/video-loop",
    title: "Looping video",
    description: "A video that loops for as long as needed.",
    category: "background",
    kind: "component",
    schema: z.object({ src: asset("Video", "video/*"), dim: num(0.3, 0, 1, "Dim", 0.05) }),
    defaultDurationInFrames: 150,
  },
  {
    key: "core/light-leak",
    title: "Light leak",
    description: "A warm light flare over the video.",
    category: "overlay",
    kind: "component",
    schema: z.object({ seed: num(3, 0, 100, "Seed"), hueShift: num(0, 0, 360, "Hue shift") }),
    defaultDurationInFrames: 45,
  },
];

const presentation = (
  slug: string,
  title: string,
  defaultDurationInFrames: number,
  schema = z.object({}),
): ElementMetaInput => ({
  key: `core/${slug}`,
  title,
  category: "transition",
  kind: "presentation",
  source: {
    registry: "@remotion/transitions",
    url: `https://www.remotion.dev/docs/transitions/presentations/${slug}`,
    license: "Remotion",
  },
  schema,
  fixedProps: {},
  defaultDurationInFrames,
});

const presentations: ElementMetaInput[] = [
  presentation("fade", "Fade", 15),
  presentation("slide", "Slide", 20, z.object({ direction: direction(DIRECTIONS, "from-right") })),
  presentation(
    "wipe",
    "Wipe",
    20,
    z.object({
      direction: direction(
        [...DIRECTIONS, "from-top-left", "from-top-right", "from-bottom-left", "from-bottom-right"],
        "from-left",
      ),
    }),
  ),
  presentation("flip", "Flip", 24, z.object({ direction: direction(DIRECTIONS, "from-right") })),
  presentation("clock-wipe", "Clock wipe", 24),
  presentation("iris", "Iris", 24),
];

export const coreElementMeta: ElementMeta[] = [
  ...components.map((def) => ({ ...def, source, fixedProps: {} })),
  ...presentations,
].map(defineElementMeta);
