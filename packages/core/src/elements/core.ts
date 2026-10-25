import { zColor } from "@remotion/zod-types";
import { z } from "zod";
import { BACKDROPS, DEFAULT_BACKDROP } from "../backdrops";
import { MAX_VOLUME } from "../limits";
import { asset, bool, cameraPath, choice, choiceFrom, more, text } from "../nodes/params";
import {
  ACCENT_COLOR,
  CALLOUT_COLOR,
  CAPTION_COLOR,
  CAPTION_PANEL_COLOR,
  HIGHLIGHT_COLOR,
  PANEL_COLOR,
  TEXT_COLOR,
} from "./colors";
import { DEVICE_LABELS, DEVICES, SCREEN_MOVE_LABELS, SCREEN_MOVES } from "./screen-3d";
import { LIGHTING_LABELS, LIGHTINGS, MOTION_LABELS, MOTIONS } from "./three";
import { defineElementMeta, type ElementMeta, type ElementMetaInput } from "./types";

const source = {
  registry: "Opsis",
  url: "https://github.com/bismitpanda/opsis",
  license: "Apache-2.0",
};
const num = (def: number, min: number, max: number, description: string, step = 1) =>
  z.number().min(min).max(max).default(def).meta({ description, step });
const color = (def: string, description: string) => zColor().default(def).meta({ description });
const weight = (def: number) =>
  z.number().min(100).max(900).default(def).meta({ description: "Font weight", step: 10 });
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

const motion = (def: (typeof MOTIONS)[number]) => choice(MOTIONS, def, "Motion", MOTION_LABELS);
const motionSpeed = () => num(1, 0.25, 3, "Speed", 0.05);
const lighting = (def: (typeof LIGHTINGS)[number] = "studio") =>
  choice(LIGHTINGS, def, "Lighting", LIGHTING_LABELS);
const optionalColor = (description: string, zeroLabel: string) =>
  z.string().default("").meta({ description, widget: "color", zeroLabel });

const ENTRANCES = ["rise", "fade", "blur", "scale", "none"] as const;
const TITLE_MARKS = ["underline", "circle", "box", "marker", "crossed-off"] as const;
const DIRECTIONS = ["from-left", "from-right", "from-top", "from-bottom"] as const;

export const CAPTIONS_ELEMENT = "core/captions";

export const captionsSchema = z.object({
  style: choice(["word-by-word", "karaoke", "line"], "word-by-word", "Style"),
  position: choice(["bottom", "center", "top"], "bottom", "Position"),
  fontSize: num(56, 16, 140, "Font size"),
  color: color(CAPTION_COLOR, "Text colour"),
  highlightColor: color(HIGHLIGHT_COLOR, "Highlight colour"),
  panel: more(color(CAPTION_PANEL_COLOR, "Panel colour"), {
    help: "The box behind the words. Lower its opacity to let the video show through.",
  }),
  fontWeight: weight(800),
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
      mark: more(text("", "Mark a word"), {
        help: "Words from the title to draw a hand-drawn mark on, after the title comes in. Leave empty for none.",
      }),
      markLook: choice(TITLE_MARKS, "underline", "Mark", {
        underline: "Underline",
        circle: "Circle",
        box: "Box",
        marker: "Marker",
        "crossed-off": "Crossed out",
      }),
      markColor: color(HIGHLIGHT_COLOR, "Mark colour"),
      align: choice(["center", "left"], "center", "Alignment"),
      entrance: choice(ENTRANCES, "rise", "Entrance"),
      color: color(TEXT_COLOR, "Text colour"),
      accent: color(ACCENT_COLOR, "Accent"),
      background: background("Background (colour, gradient or 'transparent')"),
      fontSize: num(112, 24, 260, "Font size"),
      fontWeight: weight(760),
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
          timeKey: "frameMs",
          help: "Drag on the picture to add an area. The areas stay in place on screen; they don't follow the camera. To hide something on a recording while the camera moves, use Blur areas on the recording instead.",
        }),
      frameMs: more(num(0, 0, 3_600_000, "Frame to draw on"), { unit: "ms", hidden: true }),
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
  "core/logo-3d": {
    title: "3D logo",
    description: "Your logo in 3D, spinning, tilting in or floating, with studio lighting.",
    category: "scene",
    schema: z.object({
      src: more(asset("Logo file", ".svg,.png,.jpg,.jpeg,.webp"), {
        zeroLabel: "Sample logo",
        help: "A drawn logo is given depth; a picture of a logo goes on a card.",
      }),
      depth: num(0.25, 0.05, 1, "Depth", 0.05),
      color: optionalColor("Colour", "The logo's own colours"),
      metalness: num(0.3, 0, 1, "Metal", 0.05),
      motion: motion("spin"),
      camera: cameraPath({ motion: ["keyframes"] }),
      speed: motionSpeed(),
      lighting: lighting(),
      background: background(),
    }),
    defaultDurationInFrames: 150,
  },
  "core/card-3d": {
    title: "3D card",
    description: "An image on a card with rounded corners and a soft shadow that tilts into place.",
    category: "scene",
    schema: z.object({
      src: more(asset("Image", "image/*"), { zeroLabel: "Sample screen" }),
      cornerRadius: num(0.04, 0, 0.2, "Rounded corners", 0.01),
      depth: num(0.04, 0.01, 0.3, "Thickness", 0.01),
      tilt: more(num(40, 0, 80, "Tilt", 1), {
        unit: "°",
        help: "How far the card is turned before it settles.",
      }),
      motion: motion("tilt-in"),
      camera: cameraPath({ motion: ["keyframes"] }),
      speed: motionSpeed(),
      lighting: lighting("soft"),
      background: background(),
    }),
    defaultDurationInFrames: 120,
  },
  "core/text-3d": {
    title: "3D text",
    description: "Words in solid 3D letters with rounded edges, in any project font.",
    category: "text",
    schema: z.object({
      text: text("Ship faster", "Text", "textarea"),
      fontFamily: more(font(), { help: "Empty uses the project's heading font." }),
      fontWeight: weight(800),
      italic: bool(false, "Italic"),
      depth: num(0.35, 0.05, 1, "Depth", 0.05),
      bevel: num(0.4, 0, 1, "Rounded edges", 0.05),
      color: color(TEXT_COLOR, "Colour"),
      metalness: num(0.2, 0, 1, "Metal", 0.05),
      align: choice(["center", "left", "right"], "center", "Alignment"),
      lineWidth: more(num(16, 4, 60, "Line length"), {
        unit: "letters",
        help: "Long text wraps onto a new line after about this many letters.",
      }),
      motion: motion("tilt-in"),
      camera: cameraPath({ motion: ["keyframes"] }),
      speed: motionSpeed(),
      lighting: lighting(),
      background: background(),
    }),
    defaultDurationInFrames: 120,
  },
  "core/model-3d": {
    title: "3D model",
    description:
      "A 3D model from your project on a turntable, with studio lighting and its own animation.",
    category: "scene",
    schema: z.object({
      src: more(asset("Model", ".glb,.gltf"), {
        help: "A 3D model file. A model saved as several files keeps them next to it.",
      }),
      motion: choiceFrom(
        { ...MOTION_LABELS, spin: "Turntable" },
        ["spin", "orbit", "swing", "float", "still", "keyframes"],
        "spin",
        "Motion",
      ),
      camera: cameraPath({ motion: ["keyframes"] }),
      speed: motionSpeed(),
      animation: more(text("", "Animation"), {
        help: "The name of the model's own animation to play. Leave empty for the first one.",
      }),
      animationSpeed: num(1, 0.1, 4, "Animation speed", 0.05),
      loop: bool(true, "Loop the animation"),
      zoom: num(1, 0.3, 3, "Zoom", 0.05),
      lighting: lighting(),
      shadow: bool(true, "Floor shadow"),
      background: background(),
    }),
    defaultDurationInFrames: 150,
  },
  "core/screen-3d": {
    title: "3D screen",
    description:
      "Shows a clip on a laptop, monitor, phone or floating screen in 3D, with a camera move.",
    category: "device",
    kind: "wrapper",
    schema: z.object({
      device: choice(DEVICES, "laptop", "Device", DEVICE_LABELS),
      deviceColor: color("#c9ced6", "Device colour"),
      move: choice(SCREEN_MOVES, "push-in", "Camera move", SCREEN_MOVE_LABELS),
      camera: cameraPath({ move: ["keyframes"] }),
      endFaceOn: more(bool(true, "End facing the camera"), {
        help: "The move ends with the screen filling the frame, so it cuts cleanly to the same clip without the device.",
      }),
      lighting: lighting(),
      background: color("#0b0b10", "Background"),
    }),
    defaultDurationInFrames: 150,
  },
  "core/shapes-3d": {
    title: "Floating shapes",
    description: "Soft 3D spheres, rings and rounded boxes drifting slowly.",
    category: "background",
    schema: z.object({
      count: num(14, 3, 40, "Shapes"),
      palette: z
        .array(zColor())
        .min(1)
        .max(8)
        .default(["#6366f1", "#a78bfa", "#f472b6", "#38bdf8"])
        .meta({ description: "Colours" }),
      seed: num(1, 1, 100, "Arrangement"),
      speed: num(1, 0, 3, "Speed", 0.05),
      camera: cameraPath(),
      lighting: lighting(),
      background: color("#0b1020", "Background"),
    }),
    defaultDurationInFrames: 150,
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
