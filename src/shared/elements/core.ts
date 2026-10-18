import { zColor } from "@remotion/zod-types";
import { z } from "zod";
import { BACKDROPS, DEFAULT_BACKDROP } from "../backdrops";
import { MAX_VOLUME } from "../limits";
import { asset, bool, choice, more, text } from "../nodes/params";
import {
  ACCENT_COLOR,
  CALLOUT_COLOR,
  CAPTION_COLOR,
  HIGHLIGHT_COLOR,
  PANEL_COLOR,
  TEXT_COLOR,
} from "./colors";
import { defineElementMeta, type ElementMeta, type ElementMetaInput } from "./types";

const source = {
  registry: "Demoflow",
  url: "https://github.com/bismitpanda/demoflow",
  license: "Apache-2.0",
};
const num = (def: number, min: number, max: number, description: string, step = 1) =>
  z.number().min(min).max(max).default(def).meta({ description, step });
const color = (def: string, description: string) => zColor().default(def).meta({ description });
const font = () =>
  z.string().default("").meta({
    description: "Font",
    widget: "font",
    help: "Project font follows the fonts set for the whole project.",
  });
const background = (description = "Background") =>
  z.string().default("transparent").meta({ description });
const direction = <const T extends readonly [string, ...string[]]>(values: T, def: T[number]) =>
  choice(values, def, "Direction");

const ENTRANCES = ["rise", "fade", "blur", "scale", "none"] as const;
const DIRECTIONS = ["from-left", "from-right", "from-top", "from-bottom"] as const;

export const CAPTIONS_ELEMENT = "core/captions";

export const captionsSchema = z.object({
  style: choice(["word-by-word", "karaoke", "line"], "word-by-word", "Style"),
  position: choice(["bottom", "center", "top"], "bottom", "Position"),
  fontSize: num(56, 16, 140, "Font size"),
  color: color(CAPTION_COLOR, "Text colour"),
  highlightColor: color(HIGHLIGHT_COLOR, "Highlight colour"),
  fontFamily: font(),
});

type CoreComponent = Omit<ElementMetaInput, "key" | "source" | "fixedProps" | "kind"> & {
  kind?: "wrapper";
};

const components = {
  "core/title": {
    title: "Title card",
    description: "A full-screen title with an optional eyebrow and subtitle.",
    category: "text",
    schema: z.object({
      eyebrow: text("", "Eyebrow"),
      title: text("Sign up in 10 seconds", "Title"),
      subtitle: text("", "Subtitle"),
      align: choice(["center", "left"], "center", "Alignment"),
      entrance: choice(ENTRANCES, "rise", "Entrance"),
      color: color(TEXT_COLOR, "Text colour"),
      accent: color(ACCENT_COLOR, "Accent"),
      background: background("Background (colour, gradient or 'transparent')"),
      fontSize: num(112, 24, 260, "Font size"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 75,
  },
  "core/kinetic-headline": {
    title: "Animated headline",
    description: "A headline that animates in word by word.",
    category: "text",
    schema: z.object({
      text: text(
        "Ship *faster* with demos that *move*",
        "Text — wrap words in *asterisks* to emphasise",
      ),
      color: color(TEXT_COLOR, "Text colour"),
      emphasisColor: color(HIGHLIGHT_COLOR, "Emphasis colour"),
      background: background(),
      fontSize: num(104, 24, 240, "Font size"),
      wordStagger: more(num(4, 1, 20, "Time between words"), { unit: "frames" }),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 90,
  },
  "core/lower-third": {
    title: "Lower third",
    description: "A name or caption bar in the lower part of the screen.",
    category: "overlay",
    schema: z.object({
      title: text("Ada Lovelace", "Title"),
      subtitle: text("Founder, Acme", "Subtitle"),
      position: choice(
        ["bottom-left", "bottom-right", "top-left", "top-right"],
        "bottom-left",
        "Position",
      ),
      accent: color(ACCENT_COLOR, "Accent"),
      color: color(TEXT_COLOR, "Text colour"),
      panel: color(PANEL_COLOR, "Panel colour"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 120,
  },
  "core/callout": {
    title: "Callout",
    description: "A short note that points something out.",
    category: "overlay",
    schema: z.object({
      text: text("Click here", "Text"),
      x: num(0.5, 0, 1, "Target X (0–1)", 0.01),
      y: num(0.5, 0, 1, "Target Y (0–1)", 0.01),
      side: choice(["top", "bottom", "left", "right"], "top", "Label side"),
      color: color(TEXT_COLOR, "Label colour"),
      accent: color(CALLOUT_COLOR, "Arrow colour"),
      fontFamily: font(),
    }),
    defaultDurationInFrames: 75,
  },
  [CAPTIONS_ELEMENT]: {
    title: "Captions",
    description: "Captions shown over the video.",
    category: "overlay",
    schema: captionsSchema,
    defaultDurationInFrames: 150,
  },
  "core/image": {
    title: "Image",
    description: "Shows an image from your project.",
    category: "core",
    schema: z.object({
      src: asset("Image", "image/*"),
      fit: choice(["contain", "cover", "fill"], "contain", "Fit"),
      background: background(),
      kenBurns: bool(false, "Slow push-in"),
    }),
    defaultDurationInFrames: 90,
  },
  "core/mosaic": {
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
  "core/lottie": {
    title: "Animation",
    description: "Plays an animation file from your project, looped or once, at any speed.",
    category: "overlay",
    schema: z.object({
      src: asset("Animation file", ".json,.lottie"),
      loop: bool(true, "Loop"),
      speed: num(1, 0.25, 4, "Speed", 0.25),
      backwards: bool(false, "Play backwards"),
      fit: choice(["contain", "cover"], "contain", "Fit", {
        contain: "Show all",
        cover: "Fill the frame",
      }),
      background: background(),
    }),
    defaultDurationInFrames: 90,
  },
  "core/video": {
    title: "Video",
    description: "Plays a video from your project.",
    category: "core",
    schema: z.object({
      src: asset("Video", "video/*"),
      fit: choice(["contain", "cover", "fill"], "cover", "Fit"),
      loop: bool(false, "Loop"),
      muted: bool(true, "Muted"),
      volume: num(1, 0, MAX_VOLUME, "Volume", 0.05),
      playbackRate: num(1, 0.25, 4, "Speed", 0.05),
      trimBeforeMs: more(num(0, 0, 3_600_000, "Start at", 100), { unit: "ms" }),
    }),
    defaultDurationInFrames: 150,
  },
  "core/solid": {
    title: "Solid colour",
    description: "A plain colour background.",
    category: "background",
    schema: z.object({ color: color("#09090b", "Colour") }),
    defaultDurationInFrames: 90,
  },
  "core/gradient": {
    title: "Gradient",
    description: "A gradient background.",
    category: "background",
    schema: z.object({
      from: color("#1e1b4b", "From"),
      to: color("#0f172a", "To"),
      angle: num(135, 0, 360, "Angle"),
      grain: num(0.08, 0, 0.4, "Grain", 0.01),
    }),
    defaultDurationInFrames: 90,
  },
  "core/noise": {
    title: "Noise",
    description: "A soft moving noise background.",
    category: "background",
    schema: z.object({
      color1: color("#0b1020", "Base"),
      color2: color("#4f46e5", "Highlight"),
      scale: num(0.004, 0.0005, 0.02, "Scale", 0.0005),
      speed: num(0.004, 0, 0.05, "Speed", 0.001),
    }),
    defaultDurationInFrames: 150,
  },
  "core/starburst": {
    title: "Starburst",
    description: "Rays spreading from the centre.",
    category: "background",
    schema: z.object({
      color1: color("#18181b", "Ray A"),
      color2: color("#27272a", "Ray B"),
      rays: num(24, 6, 64, "Rays"),
      rotationSpeed: more(num(0.15, 0, 2, "Rotation speed", 0.05), { unit: "°/frame" }),
    }),
    defaultDurationInFrames: 150,
  },
  "core/paper": {
    title: "Paper",
    description: "A paper texture background.",
    category: "background",
    schema: z.object({
      color: color("#f5f1e8", "Paper colour"),
      grain: num(0.18, 0, 0.5, "Grain", 0.01),
    }),
    defaultDurationInFrames: 150,
  },
  "core/backdrop": {
    title: "Studio backdrop",
    description: "A ready-made studio background image.",
    category: "background",
    schema: z.object({
      name: choice(BACKDROPS, DEFAULT_BACKDROP, "Backdrop"),
    }),
    defaultDurationInFrames: 150,
  },
  "core/video-loop": {
    title: "Looping video",
    description: "A video that loops for as long as needed.",
    category: "background",
    schema: z.object({ src: asset("Video", "video/*"), dim: num(0.3, 0, 1, "Dim", 0.05) }),
    defaultDurationInFrames: 150,
  },
  "core/light-leak": {
    title: "Light leak",
    description: "A warm light flare over the video.",
    category: "overlay",
    schema: z.object({ seed: num(3, 0, 100, "Seed"), hueShift: num(0, 0, 360, "Hue shift") }),
    defaultDurationInFrames: 45,
  },
} satisfies Record<string, CoreComponent>;

export type CoreElementProps = {
  [K in keyof typeof components]: z.output<(typeof components)[K]["schema"]>;
};

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
  ...Object.entries(components).map(([key, def]): ElementMetaInput => ({
    kind: "component",
    key,
    ...def,
    source,
    fixedProps: {},
  })),
  ...presentations,
].map(defineElementMeta);
