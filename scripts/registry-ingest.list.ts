export type ElementCategory =
  | "text"
  | "overlay"
  | "transition"
  | "background"
  | "effect"
  | "ui"
  | "data"
  | "scene"
  | "device";

export type CuratedElement = {
  registry: "remocn" | "snapcn";
  name: string;
  category: ElementCategory;
  title?: string;
  description?: string;
  exportName?: string;
  noConfig?: boolean;
  controls?: Record<
    string,
    {
      type: string;
      default?: unknown;
      min?: number;
      max?: number;
      step?: number;
      description?: string;
      options?: string[];
    }
  >;
  omit?: string[];
  fixedProps?: Record<string, unknown>;
  durationInFrames?: number;
};

type RegistryInfo = {
  label: string;
  keyPrefix: string;
  license: string;
  docs: string;
  rawBase: string;
  configDirs: string[];
  itemUrl: (name: string) => string;
  installDir: (item: CuratedElement) => string;
};

export const REGISTRIES: Record<CuratedElement["registry"], RegistryInfo> = {
  remocn: {
    label: "Remocn",
    keyPrefix: "remocn",
    license: "MIT",
    docs: "https://remocn.dev/docs/components",
    rawBase: "https://raw.githubusercontent.com/kapishdima/remocn/main/registry",
    configDirs: ["remocn", "remocn-ui"],
    itemUrl: (name) => `https://remocn.dev/r/${name}.json`,
    installDir: (item) =>
      item.category === "scene" && TEMPLATES.has(item.name) ? "remocn/templates" : "remocn",
  },
  snapcn: {
    label: "snapcn",
    keyPrefix: "snapcn",
    license: "MIT",
    docs: "https://snapcn.dev/docs",
    rawBase: "https://raw.githubusercontent.com/snapcndev/snapcn/main/registry",
    configDirs: ["snap-cn"],
    itemUrl: (name) => `https://snapcn.dev/r/${name}.json`,
    installDir: () => "snap-cn",
  },
};

const TEMPLATES = new Set(["release-teaser", "launch-anything"]);

const remocn = (
  category: ElementCategory,
  names: string[],
  extra: Partial<CuratedElement> = {},
): CuratedElement[] => names.map((name) => ({ registry: "remocn", name, category, ...extra }));
const snapcn = (
  category: ElementCategory,
  names: string[],
  extra: Partial<CuratedElement> = {},
): CuratedElement[] =>
  names.map((name) => ({
    registry: "snapcn",
    name,
    category,
    fixedProps: { mode: "dark" },
    ...extra,
  }));

export const DESCRIPTIONS: Record<string, string> = {
  typewriter: "Types the text out letter by letter with a blinking cursor.",
  "shimmer-sweep": "A band of light sweeps across the text.",
  keystroke: "Shows a keyboard shortcut as keycaps being pressed.",
  confetti: "A burst of confetti to celebrate a moment.",
  "shader-mesh-gradient": "Soft colour blobs that drift and blend into each other.",
  "glass-code-block": "A frosted-glass code editor that reveals the code line by line.",
  "signup-flow": "A sign-up form that fills itself in, step by step.",
  "command-menu": "A command palette that opens, filters and picks an item.",
  "agent-run": "An AI assistant working through a plan, ticking off each step.",
};

export const CURATED: CuratedElement[] = [
  ...remocn("text", [
    "soft-blur-in",
    "typewriter",
    "shimmer-sweep",
    "matrix-decode",
    "slot-machine-roll",
    "staggered-fade-up",
    "kinetic-center-build",
    "mask-reveal-up",
    "tracking-in",
    "word-push",
    "rgb-glitch-text",
    "gradient-scale-cut-text",
    "strikethrough-replace",
    "line-by-line-slide",
  ]),
  ...remocn("overlay", [
    "marker-highlight",
    "inline-highlight",
    "ink-underline",
    "scribble-circle",
    "ink-arrow",
    "keystroke",
    "confetti",
    "radial-burst",
  ]),
  ...remocn("transition", [
    "whip-pan",
    "push-through",
    "zoom-blur",
    "focus-pull",
    "lens-zoom",
    "ripple-zoom",
    "wave-wipe",
    "smoke-dissolve",
    "glitch-cut",
    "page-turn",
    "grain-dissolve",
    "dither-dissolve",
    "swirl-dissolve",
    "particle-dissolve",
  ]),
  ...remocn("background", [
    "shader-mesh-gradient",
    "shader-grain-gradient",
    "shader-warp",
    "shader-swirl",
    "shader-god-rays",
    "shader-perlin-noise",
    "shader-dithering",
  ]),
  ...remocn("effect", [
    "crt-screen",
    "vhs-filter",
    "halftone-print",
    "camera-lens",
    "polaroid",
    "tv-power-off",
    "speed-lines",
    "spring-settle",
  ]),
  ...remocn("ui", [
    "terminal-simulator",
    "glass-code-block",
    "code-morph",
    "logo-enter",
    "check-list",
    "search-reveal",
    "agent-run",
  ]),
  ...remocn("data", ["animated-bar-chart", "animated-line-chart", "rolling-number"]),
  ...remocn("scene", ["signup-flow", "ai-prompt-flow", "command-menu"], { durationInFrames: 240 }),
  { registry: "remocn", name: "release-teaser", category: "scene", durationInFrames: 300 },
  { registry: "remocn", name: "launch-anything", category: "scene", durationInFrames: 600 },
  ...snapcn("device", ["laptop-frame", "phone-frame", "screen-recording"], {
    durationInFrames: 240,
  }),
  { registry: "snapcn", name: "announce-title", category: "text" },
  ...snapcn("text", ["punch-lines", "text-highlight", "status-cycle"]),
  ...snapcn("scene", ["logo-assemble", "logo-drift", "hero-launch"], { durationInFrames: 180 }),
];
