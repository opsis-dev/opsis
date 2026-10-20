import type { FieldWidget } from "../src/shared/elements/introspect";
import type { ElementCategory } from "../src/shared/elements/types";
import { pascalCase } from "../src/shared/format";
import type { StepRange } from "../src/shared/math";

type RegistryCategory = Exclude<ElementCategory, "core">;

export const CONTROL_TYPES = [
  "text",
  "text-content",
  "code",
  "font",
  "image",
  "asset",
  "number",
  "color",
  "boolean",
  "enum",
  "select",
  "strings",
  "numbers",
  "rows",
  "object",
  "colors",
  "gradient",
  "theme",
] as const;

export type ControlType = (typeof CONTROL_TYPES)[number];

export type ControlSpec = {
  type: ControlType;
  default?: unknown;
  min?: number;
  max?: number;
  step?: number;
  description?: string;
  help?: string;
  unit?: string;
  advanced?: boolean;
  accept?: string;
  options?: string[];
  widget?: FieldWidget;
  hidden?: boolean;
  columns?: Record<string, ControlSpec>;
  emptyLabel?: string;
  optional?: boolean;
  showIf?: Record<string, string[]>;
  labels?: Record<string, string>;
  framesAt?: number;
};

export const OMIT_KINDS = ["timing", "function", "react-node", "internal", "sound"] as const;

type OmitKind = (typeof OMIT_KINDS)[number];

export type OmitReason = OmitKind | `${OmitKind}: ${string}`;

export const HIDDEN_PROPS: Record<string, OmitReason> = {
  children: "react-node",
  ref: "internal",
  className: "internal",
  style: "internal",
  frame: "internal: the timeline sets the frame",
  delay: "timing",
  delayInFrames: "timing",
  startFrame: "timing",
  startFrom: "timing",
  holdSeconds: "timing",
  exitAtInFrames: "timing",
  exitInFrames: "timing",
  exitStaggerInFrames: "timing",
  valueFormatter: "function",
  mediaFilter: "internal",
};

export type ThemeFamily = "prop" | "context";

export type ThemeSource = { family: ThemeFamily; presets: string; mode: "light" | "dark" };

export type RowLists = Record<string, Record<string, string>>;

export type LogoMarks = { from: string; to: string };

export type CuratedElement = {
  registry: "remocn" | "snapcn" | "remotionui";
  name: string;
  category: RegistryCategory;
  title?: string;
  exportName?: string;
  controls?: Record<string, ControlSpec>;
  omit?: Partial<Record<string, OmitReason>>;
  nest?: Record<string, string>;
  lists?: RowLists;
  marks?: LogoMarks;
  theme?: ThemeSource;
  ticks?: string;
  tree?: string;
  props?: "unchecked";
  defaults?: Record<string, unknown>;
  fixedProps?: Record<string, unknown>;
  durationInFrames?: number;
  design?: { width: number; height: number };
  kind?: "component" | "wrapper" | "presentation" | "sequencer";
  sceneItem?: string;
  center?: boolean;
  screen?: boolean;
  place?: "spot" | "points";
  paint?: string;
  atEnd?: boolean;
  installOnly?: boolean;
};

type RegistryInfo = {
  label: string;
  keyPrefix: string;
  license: string;
  docs: string;
  rawBase: string;
  configDirs: string[];
  propsUrl?: (name: string) => string;
  durationsUrl?: string;
  itemUrl: (name: string) => string;
  installDir: (item: CuratedElement) => string;
};

const REMOCN_TEMPLATE_NAMES = new Set(["release-teaser", "launch-anything"]);

const REMOTION_UI_RAW =
  "https://raw.githubusercontent.com/riaz37/remotion-ui/b7e0e6becc3d22b8b03dfef72c064f72ff9fad1f/apps";

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
      item.category === "scene" && REMOCN_TEMPLATE_NAMES.has(item.name)
        ? "remocn/templates"
        : "remocn",
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
  remotionui: {
    label: "RemotionUI",
    keyPrefix: "remotion-ui",
    license: "MIT",
    docs: "https://remotionui.com/docs/components",
    rawBase: `${REMOTION_UI_RAW}/web/registry/bases/default`,
    configDirs: [],
    propsUrl: (name) => `https://remotionui.com/ai/components/${name}.json`,
    durationsUrl: `${REMOTION_UI_RAW}/web/lib/preview-config.ts`,
    itemUrl: (name) => `https://remotionui.com/r/${name}.json`,
    installDir: () => "remotion-ui",
  },
};

const fromRegistry =
  (registry: CuratedElement["registry"], base: Partial<CuratedElement> = {}) =>
  (
    category: RegistryCategory,
    names: string[],
    extra: Partial<CuratedElement> = {},
  ): CuratedElement[] =>
    names.map((name) => ({ registry, name, category, ...base, ...extra }));

const FULL_HD = { width: 1920, height: 1080 };

const remocn = fromRegistry("remocn");
const snapcn = fromRegistry("snapcn");

const remotionui = fromRegistry("remotionui");

export const TITLE_ACRONYMS: ReadonlySet<string> = new Set(["AI", "CRT", "RGB", "TV", "UI", "VHS"]);

export const SECONDS: Partial<Record<string, Record<string, string | true>>> = {
  "matrix-decode": { revealDuration: "Reveal time" },
  "staggered-fade-up": { staggerDelay: "Stagger" },
  "rgb-glitch-text": { glitchAt: "Glitch at", glitchDuration: "Glitch length" },
  "gradient-scale-cut-text": { cutFrame: "Hard cut at" },
  "ink-underline": { step: "Time per pose" },
  "scribble-circle": { step: "Time per pose" },
  "ink-arrow": { step: "Time per pose", drawDur: "Draw time", headDur: "Arrowhead time" },
  polaroid: { step: "Time per pose" },
  keystroke: { linger: "Linger", "steps.at": "At", "steps.hold": "Hold" },
  "terminal-simulator": { "lines.delay": "Wait before" },
  "glass-code-block": { staggerFrames: "Line stagger" },
  "code-morph": { morphDuration: "Morph time", "steps.at": "Show at" },
  "logo-enter": { stagger: "Stagger" },
  "check-list": {
    step: "Time per pose",
    itemGap: "Time between items",
    closeGap: "Time between ticks",
  },
  "search-reveal": { framesPerCharacter: "Time per letter" },
  "animated-bar-chart": { staggerFrames: "Stagger" },
  confetti: { lifetime: true },
  "status-cycle": {
    introFrames: "Intro length",
    statusHold: "Status hold",
    chipStagger: "Chip stagger",
    startAt: "Start at",
  },
  "text-highlight": {
    startAt: "Start at",
    drawDuration: "Draw time",
    holdDuration: "Lockup hold",
    rushDuration: "Rush time",
  },
  "punch-lines": {
    holds: "Card lengths",
    lead: "Entrance lead",
    lineStagger: "Line stagger",
    punchFrames: "Punch",
    wordStagger: "Word stagger",
    rushLead: "Rush lead",
    rushFrames: "Rush",
  },
};

export const DESCRIPTIONS: Partial<Record<string, string>> = {
  "spring-settle":
    "Plays clips as scenes: each scene shrinks away, the stage holds empty for a beat, and the next one lands. Connect Layers to make a scene whose layers land one after another.",
  typewriter: "Types the text out letter by letter with a blinking cursor.",
  "shimmer-sweep": "A band of light sweeps across the text.",
  keystroke: "Shows a keyboard shortcut as keycaps being pressed.",
  confetti: "A burst of confetti to celebrate a moment.",
  "shader-mesh-gradient": "Soft colour blobs that drift and blend into each other.",
  "glass-code-block": "A frosted-glass code editor that reveals the code line by line.",
  "signup-flow": "A sign-up form that fills itself in, step by step.",
  "command-menu": "A command palette that opens, filters and picks an item.",
  "agent-run": "An AI assistant working through a plan, ticking off each step.",
  "grid-pixelate-wipe": "The next scene appears one grid cell at a time.",
  "frosted-glass-wipe":
    "A pane of frosted glass crosses the frame and leaves the next scene behind it.",
  "chromatic-aberration-wipe":
    "A fast slide that pulls the colours apart mid-move and brings them back together as it lands.",
  "transition-blinds": "Slats open one after another to reveal the next scene.",
  "transition-morph-shape":
    "The next scene shows through a shape that grows and changes form until it fills the frame.",
  "stat-card":
    "One big number counts up while a ring fills, then its label and the change since last period appear.",
  "metric-ticker":
    "Key numbers on cards that count up, each with its change and a small trend line.",
  "funnel-chart": "A bar for each step of a funnel, narrowing to show how many people drop off.",
  "donut-chart": "A ring chart that fills segment by segment, with a legend and a total.",
  "comparison-bars": "Pairs of bars comparing before and after, with the change called out.",
  "bar-chart-race": "Ranked bars that grow and swap places as the values change over time.",
  "gauge-dial": "A dial whose needle sweeps up to a value.",
  "data-story":
    "A whole data story: title, bar chart, key numbers, timeline, takeaway and a closing call to action.",
  "logo-reveal": "Draws your logo outline, then brings in the name and a tagline.",
  "pricing-card": "A pricing plan card: the price rolls in, then each feature is ticked off.",
  "comparison-table": "A table comparing plans feature by feature, with one plan highlighted.",
  "changelog-entry": "A release note: version and date, then each change appears with its own tag.",
  "logo-wall": "A grid of customer logos that light up from grey to colour.",
  "end-card": "A closing card with a title, a button, your web address and social handles.",
  "code-reveal": "A code editor that types out a file, then highlights the lines that matter.",
  "code-diff-wipe":
    "A code editor that applies a change line by line: removed lines collapse and new lines type in.",
  "kanban-move": "A task board where cards move from one column to the next.",
  "file-tree-reveal": "A folder tree that opens one item at a time and highlights a file.",
  "drag-drop-flow": "A pointer drags a file into an upload area, which catches it and uploads it.",
  "commit-graph": "A version history graph that draws its branches and merges commit by commit.",
  "notification-stack": "Notifications slide in, stack up and dismiss themselves.",
  "device-mockup-zoom":
    "Shows the clip on a laptop, phone or browser screen while the camera slowly pulls back.",
  "multi-device-lineup": "Shows the clip on a phone, a tablet and a laptop side by side.",
  "handwriting-text": "Writes the text out by hand, stroke by stroke.",
  "neon-flicker-text": "Text that flickers on like a neon sign, then hums gently.",
  "text-mask-video": "Big letters with a video, image or gradient showing through them.",
  "aurora-bg": "Soft ribbons of light drifting across a dark sky.",
  "particle-field": "Glowing particles drifting at different depths.",
  "laptop-frame":
    "A laptop that opens, shows a notification in its notch, then moves the camera into the screen until your clip fills the frame.",
  "phone-frame":
    "A phone frame with a notch and a screen slot for your clip, flat or tilted, with rise, rotate-in or float entrances.",
};

const METRICS = [
  { label: "Sign-ups", value: 12400, delta: "+18%", trend: [62, 71, 68, 84, 96, 112, 124] },
  {
    label: "Avg. session",
    value: 48,
    suffix: "min",
    delta: "+6%",
    trend: [39, 41, 40, 43, 45, 44, 48],
  },
  { label: "Support tickets", value: 320, delta: "-4%", trend: [36, 35, 37, 34, 33, 33.5, 32] },
];

const REMOTION_UI: CuratedElement[] = [
  ...remotionui("transition", ["grid-pixelate-wipe"], {
    title: "Pixel grid wipe",
    exportName: "gridPixelateWipe",
  }),
  ...remotionui("transition", ["frosted-glass-wipe"], {
    title: "Frosted glass wipe",
    exportName: "frostedGlassWipe",
  }),
  ...remotionui("transition", ["chromatic-aberration-wipe"], {
    title: "Colour split slide",
    exportName: "chromaticAberrationWipe",
  }),
  ...remotionui("transition", ["transition-blinds"], { title: "Blinds", exportName: "blinds" }),
  ...remotionui("transition", ["transition-morph-shape"], {
    title: "Shape morph",
    exportName: "morphShape",
  }),
  ...remotionui("data", ["stat-card"], {
    title: "Stat card",
    defaults: { prefix: "", caption: "Customer survey, last 30 days", delta: 4 },
  }),
  ...remotionui("data", ["metric-ticker"], {
    title: "Metric cards",
    defaults: { eyebrow: "Quarter recap", title: "Growth this quarter", metrics: METRICS },
  }),
  ...remotionui("data", ["funnel-chart"], {
    title: "Funnel chart",
    design: FULL_HD,
    center: true,
    defaults: {
      width: 1100,
      height: 560,
      stages: [
        { label: "Visited the site", value: 42800 },
        { label: "Signed up", value: 18600 },
        { label: "Created a project", value: 9400 },
        { label: "Invited a teammate", value: 4100 },
        { label: "Upgraded", value: 1750 },
      ],
    },
  }),
  ...remotionui("data", ["donut-chart"], {
    title: "Donut chart",
    design: FULL_HD,
    center: true,
    defaults: {
      size: 520,
      totalLabel: "Sessions",
      segments: [
        { label: "Direct", value: 4820 },
        { label: "Search", value: 3140 },
        { label: "Social", value: 1960 },
        { label: "Referral", value: 880 },
      ],
    },
  }),
  ...remotionui("data", ["comparison-bars"], {
    title: "Before and after bars",
    design: FULL_HD,
    center: true,
    defaults: {
      width: 1100,
      rowHeight: 84,
      labelWidth: 280,
      seriesLabels: ["Before", "After"],
      rows: [
        { label: "Setup time (min)", before: 240, after: 96 },
        { label: "Tickets closed / week", before: 120, after: 310 },
        { label: "Active teams", before: 34, after: 88 },
        { label: "Manual steps", before: 62, after: 18 },
      ],
    },
  }),
  ...remotionui("data", ["bar-chart-race"], {
    title: "Bar chart race",
    design: FULL_HD,
    center: true,
    defaults: {
      width: 1200,
      rowHeight: 76,
      labelWidth: 240,
      framesPerStep: 18,
      steps: ["Q1", "Q2", "Q3", "Q4", "Q5", "Q6"],
      series: [
        { label: "Mobile", values: [42, 58, 66, 72, 78, 84] },
        { label: "Web", values: [18, 34, 57, 76, 92, 108] },
        { label: "Desktop", values: [30, 40, 47, 52, 57, 61] },
        { label: "API", values: [12, 22, 36, 50, 63, 79] },
        { label: "Email", values: [24, 28, 31, 35, 38, 42] },
        { label: "Partners", values: [8, 12, 17, 21, 26, 30] },
      ],
    },
  }),
  ...remotionui("data", ["gauge-dial"], {
    title: "Gauge",
    center: true,
    defaults: { value: 78, label: "Weekly goal", unit: "%", size: 480 },
  }),
  ...remotionui("scene", ["data-story"], {
    title: "Data story",
    defaults: {
      ctaLabel: "Get started",
      barData: [
        { label: "Mobile", value: 124000, delta: "+32%" },
        { label: "Web", value: 82000, delta: "+11%" },
        { label: "Desktop", value: 64000, delta: "+4%" },
        { label: "API", value: 48000, delta: "-6%" },
      ],
      metrics: METRICS,
      steps: [
        { title: "Hook", description: "Lead with the sharpest insight." },
        { title: "Proof", description: "Show the chart and the numbers." },
        { title: "Next step", description: "Close with one clear call to action." },
      ],
    },
  }),
  ...remotionui("scene", ["logo-reveal"], {
    title: "Logo reveal",
    defaults: {
      pathD:
        "M 100 20 L 170 60 L 170 140 L 100 180 L 30 140 L 30 60 Z M 100 64 L 138 86 L 138 130 L 100 152 L 62 130 L 62 86 Z",
      wordmark: "Acme",
      tagline: "Ship your next release faster",
    },
  }),
  ...remotionui("scene", ["pricing-card"], { title: "Pricing card" }),
  ...remotionui("scene", ["comparison-table"], { title: "Plan comparison table" }),
  ...remotionui("scene", ["changelog-entry"], { title: "Changelog" }),
  ...remotionui("scene", ["logo-wall"], {
    title: "Logo wall",
    defaults: { title: "Teams that ship with us" },
  }),
  ...remotionui("scene", ["end-card"], {
    title: "End card",
    defaults: {
      eyebrow: "",
      title: "Ready to try it?",
      subtitle: "Start free. No credit card needed.",
      cta: "Start free trial",
      url: "example.com/signup",
      handles: ["@acme", "acme.example/blog"],
    },
  }),
  ...remotionui("ui", ["code-reveal"], {
    title: "Code reveal",
    defaults: {
      title: "checkout.ts",
      code: [
        "export async function checkout(cart: Cart) {",
        "  const total = cart.items.reduce((sum, item) => sum + item.price, 0);",
        "  const order = await api.orders.create({ items: cart.items, total });",
        "",
        "  return order.id;",
        "}",
      ].join("\n"),
      highlightedLines: [3],
    },
  }),
  ...remotionui("ui", ["code-diff-wipe"], { title: "Code diff" }),
  ...remotionui("ui", ["kanban-move"], {
    title: "Kanban board",
    defaults: {
      cards: [
        { title: "Onboarding checklist", meta: "APP-218 · Ana", column: 0, tint: "#8FB8F0" },
        {
          title: "Retry failed payments",
          meta: "APP-204 · Piotr",
          column: 0,
          tint: "#E8B86D",
          moveTo: 1,
          moveAtSeconds: 0.95,
        },
        { title: "Dark mode", meta: "APP-231 · Dai", column: 0, tint: "#C99BE8" },
        {
          title: "Search filters",
          meta: "APP-190 · Ana",
          column: 1,
          tint: "#7DD3E8",
          moveTo: 2,
          moveAtSeconds: 1.72,
        },
        { title: "Team invites", meta: "APP-177 · Kit", column: 1, tint: "#9BD4A0" },
        { title: "CSV export", meta: "APP-166 · Dai", column: 2, tint: "#9BD4A0" },
      ],
    },
  }),
  ...remotionui("ui", ["file-tree-reveal"], {
    title: "File tree",
    defaults: {
      title: "acme-app",
      selectedPath: "src/components/checkout.tsx",
      nodes: [
        {
          name: "src",
          children: [
            {
              name: "components",
              children: [{ name: "cart.tsx" }, { name: "checkout.tsx" }],
            },
            { name: "lib", children: [{ name: "api.ts" }] },
            { name: "index.ts" },
          ],
        },
        { name: "public", children: [{ name: "logo.svg" }] },
        { name: "README.md" },
        { name: "package.json" },
      ],
    },
  }),
  ...remotionui("ui", ["drag-drop-flow"], { title: "File upload" }),
  ...remotionui("ui", ["commit-graph"], { title: "Commit graph" }),
  ...remotionui("overlay", ["notification-stack"], { title: "Notification stack" }),
  ...remotionui("device", ["device-mockup-zoom"], {
    title: "Device mockup",
    kind: "wrapper",
    screen: true,
    defaults: { eyebrow: "", title: "", subtitle: "" },
  }),
  ...remotionui("device", ["multi-device-lineup"], {
    title: "Device lineup",
    design: FULL_HD,
    kind: "wrapper",
    screen: true,
    center: true,
    defaults: { scale: 1.6 },
  }),
  ...remotionui("text", ["handwriting-text"], {
    title: "Handwriting",
    center: true,
    omit: { durationInFrames: "timing: the clip length sets it" },
    defaults: { text: "Signed by hand", penColor: "#e8b86d" },
  }),
  ...remotionui("text", ["neon-flicker-text"], {
    title: "Neon sign",
    center: true,
    omit: { durationInFrames: "timing: the clip length sets it" },
    defaults: { text: "Open all night" },
  }),
  ...remotionui("text", ["text-mask-video"], {
    title: "Text mask",
    center: true,
    defaults: { text: "LAUNCH\nDAY", media: "gradient" },
    controls: {
      src: {
        type: "asset",
        default: "",
        description: "Media",
        help: "A video or image to show through the letters.",
        accept: "video/*,image/*",
      },
      gradient: {
        type: "gradient",
        default: "linear-gradient(115deg, #e8b86d 0%, #f472b6 42%, #2dd4bf 100%)",
        description: "Gradient fill",
        showIf: { media: ["gradient"] },
      },
      fontSize: {
        type: "number",
        optional: true,
        min: 12,
        max: 600,
        description: "Font size",
        emptyLabel: "Fit",
        unit: "px",
        help: "Leave empty to fit the text to the frame.",
        advanced: true,
      },
    },
    omit: { width: "internal: sized from the text", height: "internal: sized from the text" },
  }),
  ...remotionui("background", ["aurora-bg"], { title: "Aurora" }),
  ...remotionui("background", ["particle-field"], { title: "Particle field" }),
];

export const numberControl = (
  description: string,
  defaultValue: number,
  { min, max, step }: StepRange,
  extra: Pick<ControlSpec, "unit" | "help" | "advanced" | "widget" | "hidden"> = {},
): ControlSpec => ({
  type: "number",
  default: defaultValue,
  min,
  max,
  step,
  description,
  ...extra,
});

const FRACTION: StepRange = { min: 0, max: 1, step: 0.01 };

const FRAME_COUNT: StepRange = { min: 0, max: 30, step: 1 };

function spotControls(
  label: string,
  x: number,
  y: number,
  prefix = "",
): Record<string, ControlSpec> {
  const xKey = prefix ? `${prefix}X` : "x";
  const yKey = prefix ? `${prefix}Y` : "y";
  return {
    [xKey]: numberControl(label, x, FRACTION, {
      widget: "spot",
      help: "Click the frame to choose the spot.",
    }),
    [yKey]: numberControl(`${label} vertical`, y, FRACTION, { hidden: true }),
  };
}

const NEUTRAL_DEFAULTS: Partial<Record<string, Record<string, unknown>>> = {
  "staggered-fade-up": { text: "Ship your demo in minutes" },
  "soft-blur-in": { text: "Made for every take." },
  "line-by-line-slide": { text: "Record once.\nShare everywhere." },
  "inline-highlight": { before: "Ship faster with ", highlight: "Acme", after: "." },
  "terminal-simulator": { title: "~/projects/acme" },
  "search-reveal": { text: "acme" },
  "laptop-frame": {
    screenSrc: "app:placeholders/screen-wide.svg",
    notchLabel: "Recording started",
  },
  "phone-frame": { screenSrc: "app:placeholders/screen-tall.svg" },
  "screen-recording": { src: "app:placeholders/screen-wide.svg" },
  "announce-title": {
    title: "Acme 2.0",
    tagline: "Everything your team needs to ship, in one place.",
  },
  "punch-lines": { script: "Record it. / Edit it. | One take. | Ship." },
  "status-cycle": {
    prefix: "Acme is",
    statuses: "recording, editing, exporting, live",
    chips: "Record, Auto zoom, Captions, Export, Share",
  },
  "logo-assemble": {
    brandName: "Acme",
    middleText: "Product demos, made simple",
    logoSrc: "app:placeholders/logo-mark.svg",
  },
  "hero-launch": {
    image1: "app:placeholders/screen-wide.svg",
    image2: "app:backdrops/aurora.jpg",
    heading: "Launch day, done right",
  },
};

const NEUTRAL_FIXED_PROPS: Partial<Record<string, Record<string, unknown>>> = {};

const PAINTED: Partial<Record<string, string>> = {
  typewriter: "white",
  "shimmer-sweep": "white",
  "matrix-decode": "white",
  "slot-machine-roll": "white",
  "soft-blur-in": "white",
  "kinetic-center-build": "white",
  "mask-reveal-up": "white",
  "word-push": "white",
  "line-by-line-slide": "white",
  "staggered-fade-up": "white",
  "tracking-in": "white",
  "rgb-glitch-text": "#fafafa",
  "strikethrough-replace": "white",
  "marker-highlight": "white",
  "inline-highlight": "white",
};

export const FONT_WEIGHTS = ["400", "500", "600", "700"];

const fontWeight = (value: number): ControlSpec => ({
  type: "enum",
  options: FONT_WEIGHTS,
  default: String(value),
  description: "Font weight",
});

const colorsControl = (min: number, max: number, help?: string): ControlSpec => ({
  type: "colors",
  min,
  max,
  description: "Colours",
  ...(help ? { help } : {}),
});

export const SPEED_RANGE: StepRange = { min: 0.25, max: 4, step: 0.25 };

export const ANIMATION_FRAMES: StepRange = { min: 1, max: 300, step: 1 };

export const ANIMATION_SECONDS: StepRange = { min: 0.05, max: 10, step: 0.01 };

const advancedNumber = (description: string, value: number, range: StepRange): ControlSpec =>
  numberControl(description, value, range, { advanced: true });

const themeBackground: ControlSpec = {
  type: "color",
  default: "",
  emptyLabel: "Theme",
  description: "Background",
  help: "Leave empty to follow the theme.",
};

const optionalColor = (description: string, emptyLabel = "Default"): ControlSpec => ({
  type: "color",
  default: "",
  emptyLabel,
  description,
});

const optionalNumber = (
  description: string,
  range: StepRange,
  extra: Pick<ControlSpec, "unit" | "help" | "advanced" | "emptyLabel"> = {},
): ControlSpec => ({
  type: "number",
  optional: true,
  emptyLabel: "Auto",
  description,
  ...range,
  ...extra,
});

const text = (
  description: string,
  value: string,
  extra: Partial<ControlSpec> = {},
): ControlSpec => ({
  type: "text",
  default: value,
  description,
  ...extra,
});

const image = (description: string, value = "", extra: Partial<ControlSpec> = {}): ControlSpec => ({
  type: "asset",
  default: value,
  accept: "image/*",
  description,
  ...extra,
});

const theme = (family: ThemeFamily, mode: "light" | "dark"): Partial<CuratedElement> => ({
  theme: {
    family,
    mode,
    presets: family === "prop" ? "snap-cn-ui/theme.ts" : "remocn-ui/theme.ts",
  },
  controls: { theme: { type: "theme", description: "Theme" } },
  omit: { mode: "internal: the theme sets it" },
});

const snapTheme = (mode: "light" | "dark", extra: Partial<CuratedElement> = {}) => {
  const base = theme("prop", mode);
  return {
    ...extra,
    ...base,
    controls: { ...base.controls, ...extra.controls },
    omit: { ...base.omit, ...extra.omit },
  };
};

const remocnTheme = (extra: Partial<CuratedElement> = {}) => {
  const base = theme("context", "light");
  return {
    ...extra,
    ...base,
    controls: { ...base.controls, ...extra.controls },
    omit: { ...base.omit, ...extra.omit },
  };
};

const SHADER_SPEED: Record<string, ControlSpec> = {
  speed: advancedNumber("Speed", 1, SPEED_RANGE),
};

const transitionColors = (
  [min, max]: [number, number],
  extra: Record<string, ControlSpec> = {},
) => ({
  controls: { colors: colorsControl(min, max), ...SHADER_SPEED, ...extra },
});

const LOGO_MARK = "app:placeholders/logo-mark.svg";

const MUSIC: Record<string, ControlSpec> = {
  music: {
    type: "text",
    default: "",
    widget: "sound",
    emptyLabel: "None",
    description: "Music",
    help: "Plays through the Mixer like any other music.",
  },
  musicVolume: numberControl("Music volume", 1, { min: 0, max: 1, step: 0.01 }),
};

const LAUNCH_COPY: Array<[key: string, scene: string, label: string, text: string]> = [
  ["ready", "Opening", "after the subject", "deserve a stage"],
  ["next", "Action", "lead-in", "Let’s make"],
  ["action", "Action", "button text", "them real"],
  ["proof", "Proof", "headline", "Your work.\nA clearer view."],
  ["industry", "Industry", "headline", "Built around you"],
  ["promise", "Industry", "promise", "Made to be yours."],
  ["launch", "Closing", "call to action", "Begin"],
  ["shoppers", "Shop screen", "headline", "Small things. Well made."],
  ["shopName", "Shop screen", "shop name", "Object Study"],
  ["shopHeadline", "Shop screen", "shop headline", "Everyday, considered."],
  ["shopDetail", "Shop screen", "shop detail", "Useful objects. Quiet details."],
  ["shopAction", "Shop screen", "shop button", "Explore the objects"],
  ["builder", "Builder screen", "headline", "Turn a brief into a workspace"],
  ["builderPrompt", "Builder screen", "prompt", "Create a workspace for our next product release"],
  ["trading", "Review screen", "headline", "Review once.\nMove forward."],
  ["loyalty", "Ideas screen", "headline", "A place for every idea"],
  ["loyaltyPayoff", "Ideas screen", "payoff", "Room for the next one"],
  ["integration", "Tools screen", "headline", "Bring your work together"],
  ["connections", "Tools screen", "closing line", "Connect your tools.\nKeep work moving."],
];

const LAUNCH_SCREENS = [
  "dashboard",
  "shoppers",
  "exchange",
  "builder",
  "trading",
  "loyalty",
  "integrations",
];

const LAUNCH_MEDIA = ["desk", "horizon", "rocket", "earth"];

const launchAnything: Partial<CuratedElement> = {
  controls: {
    opening: text("Opening: first word", "Good"),
    subject: text("Opening: second word", "ideas"),
    brandUrl: text("Closing: web address", "yourproduct.example"),
    accentColor: { type: "color", default: "#245744", description: "Accent colour" },
    ...Object.fromEntries(
      LAUNCH_COPY.map(([key, scene, label, value]) => [
        `copy${pascalCase(key)}`,
        text(`${scene}: ${label}`, value, { advanced: true }),
      ]),
    ),
    copyIntegrations: {
      type: "strings",
      default: ["Files", "Messages", "Calendar"],
      description: "Tools screen: tools",
      advanced: true,
    },
    ...Object.fromEntries(
      LAUNCH_SCREENS.map((id) => [
        `screen${pascalCase(id)}`,
        image(`Screen: ${id}`, "", {
          emptyLabel: "Built-in screen",
          help: "Shown on the laptop for this part. Leave empty for the built-in screen.",
          advanced: true,
        }),
      ]),
    ),
    ...Object.fromEntries(
      LAUNCH_MEDIA.map((id) => [
        `photo${pascalCase(id)}`,
        image(`Photo: ${id}`, "", { emptyLabel: "Built-in photo", advanced: true }),
      ]),
    ),
    logoImages: {
      type: "rows",
      default: [],
      description: "Logos",
      help: "Marks for the floating grid in the proof scene.",
      columns: { image: image("Image", LOGO_MARK) },
    },
    logoSrc: image("Logo", "", { emptyLabel: "Built-in mark" }),
    ...MUSIC,
  },
  nest: {
    ...Object.fromEntries(LAUNCH_COPY.map(([key]) => [`copy${pascalCase(key)}`, `content.${key}`])),
    copyIntegrations: "content.integrations",
    ...Object.fromEntries(
      LAUNCH_SCREENS.map((id) => [`screen${pascalCase(id)}`, `screenImages.${id}`]),
    ),
    ...Object.fromEntries(LAUNCH_MEDIA.map((id) => [`photo${pascalCase(id)}`, `media.${id}`])),
  },
  lists: { logoImages: { image: "logos" } },
  omit: {
    audioSrc: "sound: the Music setting plays it through the mix",
    volume: "sound: the Music setting plays it through the mix",
  },
};

const NEUTRAL_AGENT_RUN = {
  title: "Checkout conversion drop",
  prompt: "Why did checkout conversion drop last week?",
  plan: [
    "Compare the checkout funnel with the week before",
    "Break the drop down by device and step",
    "Check what shipped and what the error logs say",
  ],
  steps: [
    {
      icon: "chart",
      name: "Query analytics",
      detail: "checkout funnel, last 7 days vs prior 7",
      duration: 1.4,
    },
    {
      icon: "database",
      name: "Run a query",
      detail: "conversion by device and step",
      duration: 2.3,
    },
    {
      icon: "search",
      name: "Search logs",
      detail: "card payment errors on tablets",
      duration: 1.1,
    },
    {
      icon: "book",
      name: "Read 12 sources",
      duration: 3.2,
      sources: [
        "Checkout funnel",
        "Device report",
        "Release 4.12",
        "Change #2186",
        "Payments guide",
        "Error log",
        "Incident notes",
        "Runbook",
        "Support tickets",
        "Status page",
        "Team notes",
        "Design spec",
      ],
    },
  ],
  answer: [
    "## Card payments broke on tablets in release 4.12",
    "",
    "Checkout conversion fell from 3.4% to 2.7% last week, a 21% drop [1]. All of it comes from the payment step on tablets, 41% of checkouts [2].",
    "",
    "- Tablet conversion halved, from 3.5% to 1.8%",
    "- Phones and desktops held steady at 3.3%",
    "- 4.12 moved the card form into a frame without the payment permission [3]",
    "",
    "```tsx",
    "<iframe",
    "  src={cardFormUrl}",
    '  allow="payment"',
    "/>",
    "```",
    "",
    "**Fix:** add the permission and ship 4.12.1 today.",
  ].join("\n"),
};

const DRIFT_TILES = [
  ["Rc", "Record", "#1f2430", "#ffffff", 163, 232, -54, 12, 109, 0.28],
  ["Ed", "Edit", "#f2f4f7", "#16181d", 128, -209, 81, 32, 122, 0.48],
  ["Zm", "Zoom", "#10131a", "#7dd3fc", -384, 344, -27, -68, 200, 0.95],
  ["Cc", "Captions", "#e7e8ea", "#16181d", 519, 45, -31, 64, 144, 1.51],
  ["Mx", "Mix", "#f1ece2", "#8a5a2b", -692, -125, 41, -71, 214, 1.55],
] as const;

const SETTINGS: Partial<Record<string, Partial<CuratedElement>>> = {
  "rgb-glitch-text": {
    controls: {
      glitchColors: {
        ...colorsControl(3, 3, "The three copies that split off while the text glitches."),
        default: ["#ff0040", "#00ff80", "#0080ff"],
        description: "Glitch colours",
      },
      blend: {
        type: "enum",
        options: ["multiply", "screen", "normal"],
        default: "multiply",
        description: "Glitch blend",
        labels: {
          multiply: "On a light background",
          screen: "On a dark background",
          normal: "On any background",
        },
        help: "How the split copies mix with what is behind them.",
      },
    },
  },
  "matrix-decode": {
    controls: {
      charset: {
        type: "text",
        default: "!@#$%^&*()_+-=<>?/\\|",
        description: "Scramble characters",
        emptyLabel: "Built-in",
        help: "Characters the text flickers through before it settles.",
      },
    },
  },
  "word-push": { controls: { fontWeight: fontWeight(400) }, design: FULL_HD },
  "kinetic-center-build": { design: FULL_HD },
  "tracking-in": { design: FULL_HD },
  "strikethrough-replace": { design: FULL_HD },
  typewriter: {
    controls: {
      text: {
        type: "text",
        default: "console.log('hello, world')",
        description: "Text",
        widget: "textarea",
      },
    },
  },
  "gradient-scale-cut-text": { controls: { fontWeight: fontWeight(700) }, design: FULL_HD },
  "scribble-circle": { omit: { release: "internal: how the stroke lifts off at the end" } },
  keystroke: {
    controls: {
      steps: {
        type: "rows",
        description: "Keys",
        help: "Join keys pressed together with +, like mod+k. Separate keys pressed one after another with a space.",
        columns: {
          keys: text("Keys", "mod+k"),
          at: numberControl("At", 0, { min: 0, max: 3600, step: 1 }, { unit: "frames" }),
          hold: { ...optionalNumber("Hold", { min: 1, max: 600, step: 1 }), unit: "frames" },
        },
      },
    },
  },
  confetti: {
    controls: {
      colors: colorsControl(1, 12),
      ...spotControls("Burst from", 0.5, 0.5, "origin"),
      lifetime: numberControl(
        "How long pieces last",
        90,
        { min: 10, max: 600, step: 1 },
        { unit: "frames" },
      ),
    },
  },
  "lens-zoom": {
    omit: Object.fromEntries(
      [
        "inScaleFrom",
        "inScaleTo",
        "inFovFrom",
        "inFovTo",
        "inBlurFrom",
        "inBlurTo",
        "inRotationFrom",
        "inRotationTo",
        "outScaleFrom",
        "outScaleTo",
        "outFovFrom",
        "outFovTo",
        "outBlurFrom",
        "outBlurTo",
        "outRotationFrom",
        "outRotationTo",
        "anticipateScale",
        "anticipatePortion",
        "blurScaleGain",
        "blurFade",
        "shakeFrequency",
        "shakeCyclesPerUnit",
        "shakeRotateDeg",
        "redScale",
        "greenScale",
        "blueScale",
        "letterboxAspect",
        "distortionCenterX",
        "distortionCenterY",
        "width",
        "height",
      ].map((k): [string, OmitReason] => [k, "internal: animation tuning"]),
    ),
  },
  "ripple-zoom": transitionColors([2, 7], {
    zoom: advancedNumber("Zoom", 4, { min: 1, max: 8, step: 0.1 }),
    noise: advancedNumber("Grain", 0.5, { min: 0, max: 1, step: 0.01 }),
  }),
  "wave-wipe": transitionColors([2, 7], {
    zoom: advancedNumber("Zoom", 1.16, { min: 0.5, max: 4, step: 0.01 }),
  }),
  "smoke-dissolve": transitionColors([1, 10]),
  "grain-dissolve": transitionColors([2, 7]),
  "swirl-dissolve": transitionColors([2, 10]),
  "dither-dissolve": { controls: { speed: advancedNumber("Speed", 1.5, SPEED_RANGE) } },
  "page-turn": { omit: { origin: "internal: the corner the page turns on, tuned with the angle" } },
  "shader-mesh-gradient": { controls: { colors: colorsControl(2, 10) } },
  "shader-grain-gradient": { controls: { colors: colorsControl(2, 7) } },
  "shader-warp": { controls: { colors: colorsControl(2, 10) } },
  "shader-swirl": { controls: { colors: colorsControl(2, 10) } },
  "shader-god-rays": {
    controls: {
      colors: colorsControl(1, 5),
      colorBloom: { type: "color", default: "#3a3a5c", description: "Glow colour" },
    },
  },
  "shader-dithering": {
    controls: {
      shape: {
        type: "enum",
        options: ["simplex", "warp", "dots", "wave", "ripple", "swirl", "sphere"],
        default: "wave",
        description: "Pattern",
        advanced: true,
      },
      type: {
        type: "enum",
        options: ["random", "2x2", "4x4", "8x8"],
        default: "4x4",
        description: "Dot grid",
        labels: { random: "Random", "2x2": "2 × 2", "4x4": "4 × 4", "8x8": "8 × 8" },
        advanced: true,
      },
    },
  },
  polaroid: {
    controls: {
      frameColor: { type: "color", default: "#fdfcf8", description: "Frame colour" },
      captionColor: { type: "color", default: "#26242c", description: "Caption colour" },
    },
  },
  "speed-lines": {
    omit: {
      from: "internal: where the lines come from, set by the mode",
      to: "internal: the spot sets where the lines aim",
    },
  },
  "spring-settle": {
    controls: {
      loop: {
        type: "boolean",
        default: false,
        description: "Loop",
        help: "Start again from the first scene after the last.",
      },
    },
  },
  "terminal-simulator": {
    controls: {
      chromeColor: { type: "color", default: "#1a1a1a", description: "Title bar colour" },
      lines: {
        type: "rows",
        description: "Lines",
        help: "Lines ending in ... pause before the next one.",
        default: [
          { text: "deploy --production", type: "command", delay: 0 },
          { text: "Running tests...", type: "log", delay: 6 },
          { text: "128 tests passed", type: "success", delay: 10 },
          { text: "Building the image...", type: "log", delay: 8 },
          { text: "Image ready in 41s", type: "success", delay: 10 },
          { text: "Rolling out to everyone...", type: "log", delay: 8 },
          { text: "Live in production", type: "success", delay: 12 },
        ],
        columns: {
          text: text("Text", "Done"),
          type: {
            type: "enum",
            options: ["command", "log", "success", "error"],
            default: "log",
            description: "Kind",
          },
          delay: numberControl("Wait before", 8, { min: 0, max: 600, step: 1 }, { unit: "frames" }),
        },
      },
    },
  },
  "glass-code-block": {
    controls: {
      code: {
        type: "code",
        default: [
          "export async function publish(draft: Draft) {",
          "  const page = await render(draft);",
          "  await upload(page, { public: true });",
          "  return page.url;",
          "}",
        ].join("\n"),
        description: "Code",
      },
      aura: { type: "boolean", default: false, description: "Glow behind the window" },
    },
  },
  "code-morph": {
    controls: {
      steps: {
        type: "rows",
        description: "Versions",
        help: "Each version of the code and when it starts to show.",
        default: [
          {
            at: 0,
            code: [
              'import { loadInvoices } from "./api";',
              "",
              "export async function showInvoices(list: HTMLElement) {",
              '  list.textContent = "Loading…";',
              "  const invoices = await loadInvoices();",
              '  list.textContent = "";',
              "  for (const invoice of invoices) {",
              '    const row = document.createElement("li");',
              "    row.textContent = invoice.title;",
              "    list.append(row);",
              "  }",
              "}",
            ].join("\n"),
          },
          {
            at: 48,
            code: [
              'import { loadInvoices } from "./api";',
              'import { renderRows } from "./rows";',
              "",
              "export async function showInvoices(list: HTMLElement) {",
              '  list.textContent = "Loading…";',
              "  renderRows(list, await loadInvoices());",
              "}",
            ].join("\n"),
          },
        ],
        columns: {
          code: { type: "code", default: "", description: "Code" },
          at: numberControl("Show at", 0, { min: 0, max: 3600, step: 1 }, { unit: "frames" }),
        },
      },
    },
  },
  "logo-enter": {
    controls: {
      logos: {
        type: "rows",
        description: "Logos",
        min: 1,
        default: ["#2563eb", "#0f766e", "#b45309", "#7c3aed"].map((background) => ({
          image: LOGO_MARK,
          background,
        })),
        columns: {
          image: image("Image", LOGO_MARK),
          background: { type: "color", default: "#2563eb", description: "Background" },
        },
      },
    },
    marks: { from: "logos", to: "logos" },
  },
  "check-list": {
    center: true,
    controls: {
      items: {
        type: "rows",
        description: "Items",
        default: [
          { text: "Record the walkthrough", checked: true },
          { text: "Add captions and zooms", checked: true },
          { text: "Export for the launch", checked: false },
        ],
        columns: {
          text: text("Text", "New item"),
          checked: { type: "boolean", default: true, description: "Ticked" },
        },
      },
      width: numberControl("Width", 720, { min: 200, max: 1800, step: 10 }, { unit: "px" }),
      boxColor: { type: "color", default: "#26242c", description: "Box colour" },
      weight: fontWeight(600),
      rowGap: optionalNumber(
        "Row gap",
        { min: 0, max: 200, step: 1 },
        { unit: "px", advanced: true },
      ),
      seed: text("Variation", "checklist", {
        advanced: true,
        help: "Change it to draw the handwriting a little differently.",
      }),
    },
  },
  "agent-run": {
    controls: {
      runTitle: text("Title", NEUTRAL_AGENT_RUN.title),
      runPrompt: text("Prompt", NEUTRAL_AGENT_RUN.prompt),
      runPlan: { type: "strings", default: NEUTRAL_AGENT_RUN.plan, description: "Plan" },
      runSteps: {
        type: "rows",
        default: NEUTRAL_AGENT_RUN.steps,
        description: "Steps",
        columns: {
          name: text("Name", "Search logs"),
          detail: { type: "text", optional: true, description: "Detail" },
          duration: numberControl("Length", 1.5, { min: 0.2, max: 20, step: 0.1 }, { unit: "s" }),
          icon: {
            type: "enum",
            options: ["chart", "database", "search", "book", "globe", "terminal", "file"],
            optional: true,
            description: "Icon",
          },
          sources: { type: "strings", optional: true, description: "Sources" },
        },
      },
      runAnswer: {
        type: "text",
        default: NEUTRAL_AGENT_RUN.answer,
        description: "Answer",
        help: "Use ## for a heading, - for a list, **bold**, `code` and [1] to cite a source.",
      },
    },
    nest: {
      runTitle: "run.title",
      runPrompt: "run.prompt",
      runPlan: "run.plan",
      runSteps: "run.steps",
      runAnswer: "run.answer",
    },
  },
  "animated-bar-chart": {
    controls: {
      bars: {
        type: "rows",
        description: "Bars",
        help: "Leave every label empty to draw the bars without labels.",
        default: [35, 60, 45, 80, 55, 70, 90, 65].map((value) => ({ label: "", value })),
        columns: {
          label: text("Label", ""),
          value: numberControl("Value", 50, { min: 0, max: 1_000_000, step: 1 }),
        },
      },
      width: advancedNumber("Width", 1000, { min: 200, max: 3840, step: 10 }),
      height: advancedNumber("Height", 500, { min: 100, max: 2160, step: 10 }),
    },
    lists: { bars: { label: "labels", value: "data" } },
  },
  "animated-line-chart": {
    controls: {
      data: {
        type: "numbers",
        description: "Values",
        help: "Paste a column from a spreadsheet to fill the list.",
      },
      width: advancedNumber("Width", 1000, { min: 200, max: 3840, step: 10 }),
      height: advancedNumber("Height", 500, { min: 100, max: 2160, step: 10 }),
    },
  },
  "signup-flow": remocnTheme({
    controls: {
      title: text("Title", "Create an account"),
      description: text("Description", "Enter your information below to create your account"),
      fullName: text("Name typed in", "John Doe"),
      email: text("Email typed in", "m@example.com"),
      password: text("Password shown", "••••••••"),
      createLabel: text("Button label", "Create account"),
      toastTitle: text("Notice", "Account created"),
    },
    omit: {
      googleLabel: "internal: declared but never drawn",
      signinText: "internal: declared but never drawn",
    },
  }),
  "ai-prompt-flow": remocnTheme({
    controls: {
      prompt: text("Prompt", "Summarize this thread"),
      buttonLabel: text("Button label", "Generate"),
      answerLines: {
        type: "strings",
        description: "Answer",
        help: "One line of the answer per item.",
      },
      toastTitle: text("Notice", "Response ready"),
    },
  }),
  "command-menu": remocnTheme({
    controls: {
      items: {
        type: "rows",
        description: "Items",
        columns: {
          icon: {
            type: "enum",
            options: ["search", "settings", "user", "file"],
            optional: true,
            description: "Icon",
          },
          label: text("Label", "New item"),
          shortcut: { type: "text", optional: true, description: "Shortcut" },
        },
      },
      pressedIndex: numberControl(
        "Press item",
        -1,
        { min: -1, max: 20, step: 1 },
        { help: "The item that gets pressed, counting from 0. -1 presses none." },
      ),
    },
    omit: { itemStyles: "internal: per-item animation styles" },
  }),
  "release-teaser": {
    controls: {
      statements: {
        type: "strings",
        default: [
          "Start with an idea",
          "Make room\nfor better work",
          "Bring every detail to life",
          "Move together.\nGo further.",
          "A fresh chapter.\nNo starting over.",
        ],
        min: 5,
        max: 5,
        description: "Statements",
        help: "The five lines, in order.",
      },
      logoSrc: image("Logo", "", { emptyLabel: "Name only" }),
      themeBackground: optionalColor("Background"),
      themeSurface: optionalColor("Panel colour"),
      themeForeground: optionalColor("Text colour"),
      themeMuted: optionalColor("Soft text colour"),
      lightIntensity: advancedNumber("Light", 1, { min: 0, max: 2, step: 0.05 }),
      ...MUSIC,
    },
    nest: {
      themeBackground: "theme.background",
      themeSurface: "theme.surface",
      themeForeground: "theme.foreground",
      themeMuted: "theme.muted",
    },
    omit: {
      audioSrc: "sound: the Music setting plays it through the mix",
      volume: "sound: the Music setting plays it through the mix",
    },
  },
  "launch-anything": launchAnything,
  "laptop-frame": snapTheme("dark", {
    controls: {
      indicatorColor: { type: "color", default: "#30D158", description: "Recording light" },
    },
  }),
  "phone-frame": snapTheme("dark"),
  "screen-recording": snapTheme("dark", {
    omit: {
      crop: "internal: the crop settings set it",
      sourceAspect: "internal: the recording sets it",
      camera: "internal: the recording sets it",
    },
  }),
  "punch-lines": snapTheme("light", { controls: { fontWeight: fontWeight(600) } }),
  "text-highlight": snapTheme("dark", {
    controls: {
      fontWeight: fontWeight(600),
      logoSrc: image("Logo", LOGO_MARK, { help: "The mark the logo wipe uses." }),
    },
    omit: {
      springConfig: "internal: animation tuning",
      logo: "react-node: the logo setting sets the mark",
    },
  }),
  "status-cycle": snapTheme("dark", {
    controls: {
      fontWeight: fontWeight(400),
      chipFontWeight: { ...fontWeight(400), description: "Chip font weight" },
      pillRadius: optionalNumber(
        "Status corner radius",
        { min: 0, max: 200, step: 1 },
        { unit: "px", advanced: true },
      ),
      chipRadius: optionalNumber(
        "Chip corner radius",
        { min: 0, max: 200, step: 1 },
        { unit: "px", advanced: true },
      ),
      chipBorderWidth: optionalNumber(
        "Chip border width",
        { min: 0, max: 20, step: 0.5 },
        { unit: "px", advanced: true },
      ),
    },
    omit: { motion: "internal: animation tuning" },
  }),
  "logo-assemble": snapTheme("dark", {
    controls: {
      images: {
        type: "rows",
        description: "Images",
        min: 1,
        default: ["aurora", "bloom", "canyon", "chalk", "cobalt", "dusk", "ember", "flare"].map(
          (name) => ({ image: `app:backdrops/${name}.jpg` }),
        ),
        columns: { image: image("Image", "app:backdrops/aurora.jpg") },
      },
    },
    lists: { images: { image: "images" } },
  }),
  "logo-drift": snapTheme("light", {
    controls: {
      fontWeight: fontWeight(500),
      glowColor: optionalColor("Glow colour", "Accent"),
      tiles: {
        type: "rows",
        description: "Tiles",
        default: DRIFT_TILES.map(([glyph, label, background, color, x, y, vx, vy, size, at]) => ({
          glyph,
          label,
          background,
          color,
          x,
          y,
          vx,
          vy,
          size,
          at,
        })),
        columns: {
          glyph: text("Mark", "Ab", { help: "One to three characters." }),
          label: { type: "text", optional: true, description: "Label" },
          background: { type: "color", default: "#1f2430", description: "Background" },
          color: optionalColor("Text colour"),
          x: advancedNumber("Start across", 0, { min: -1200, max: 1200, step: 1 }),
          y: advancedNumber("Start down", 0, { min: -800, max: 800, step: 1 }),
          vx: advancedNumber("Drift across", 0, { min: -200, max: 200, step: 1 }),
          vy: advancedNumber("Drift down", 0, { min: -200, max: 200, step: 1 }),
          size: advancedNumber("Size", 140, { min: 20, max: 400, step: 1 }),
          at: advancedNumber("Appears at", 1, { min: 0, max: 10, step: 0.01 }),
        },
      },
    },
    omit: {
      wordAt: "internal: animation tuning",
      exitScale: "internal: animation tuning",
      exitBlur: "internal: animation tuning",
    },
  }),
  "hero-launch": snapTheme("dark"),
  "stat-card": {
    controls: {
      backgroundColor: themeBackground,
      max: optionalNumber(
        "Ring full at",
        { min: 0, max: 1_000_000_000, step: 1 },
        {
          emptyLabel: "Full ring",
          help: "The ring fills to the value out of this. Leave empty for a full ring.",
        },
      ),
    },
  },
  "funnel-chart": {
    controls: {
      labelWidth: optionalNumber(
        "Label width",
        { min: 0, max: 2000, step: 10 },
        { unit: "px", advanced: true },
      ),
    },
  },
  "logo-reveal": {
    controls: {
      viewBox: {
        type: "text",
        default: "",
        emptyLabel: "Fit",
        description: "Outline area",
        advanced: true,
        help: "Leave empty to fit the outline.",
      },
      size: optionalNumber("Logo size", { min: 20, max: 2000, step: 10 }, { unit: "px" }),
      strokeWidth: optionalNumber(
        "Line width",
        { min: 1, max: 40, step: 0.5 },
        { unit: "px", advanced: true },
      ),
      fill: optionalColor("Fill colour", "None"),
    },
  },
  "pricing-card": {
    controls: {
      backgroundColor: themeBackground,
      wasPrice: optionalNumber(
        "Old price",
        { min: 0, max: 1_000_000, step: 1 },
        {
          emptyLabel: "None",
          help: "Shown struck out next to the price.",
        },
      ),
    },
  },
  "comparison-table": {
    controls: {
      backgroundColor: themeBackground,
      rows: {
        type: "rows",
        description: "Rows",
        help: "Type yes or no in a cell for a tick or a dash.",
        columns: {
          label: text("Label", "New feature"),
          cells: { type: "strings", default: ["yes", "yes", "yes"], description: "Cells" },
        },
        default: [
          { label: "Render minutes", cells: ["60", "2,000", "Unlimited"] },
          { label: "1080p exports", cells: ["yes", "yes", "yes"] },
          { label: "4K exports", cells: ["no", "yes", "yes"] },
          { label: "Team seats", cells: ["1", "9", "Unlimited"] },
          { label: "Render API", cells: ["no", "yes", "yes"] },
          { label: "Priority queue", cells: ["no", "no", "yes"] },
        ],
      },
    },
    ticks: "rows",
  },
  "changelog-entry": {
    controls: {
      backgroundColor: themeBackground,
      kindAdded: optionalColor("Added colour"),
      kindFixed: optionalColor("Fixed colour"),
      kindChanged: optionalColor("Changed colour"),
      kindRemoved: optionalColor("Removed colour"),
    },
    nest: {
      kindAdded: "kindColors.added",
      kindFixed: "kindColors.fixed",
      kindChanged: "kindColors.changed",
      kindRemoved: "kindColors.removed",
    },
  },
  "logo-wall": { controls: { backgroundColor: themeBackground } },
  "end-card": {
    controls: {
      backgroundColor: themeBackground,
      logoSize: optionalNumber("Logo size", { min: 10, max: 600, step: 1 }, { unit: "px" }),
    },
  },
  "code-reveal": { controls: { backgroundColor: themeBackground } },
  "code-diff-wipe": { controls: { backgroundColor: themeBackground } },
  "kanban-move": { controls: { backgroundColor: themeBackground } },
  "file-tree-reveal": {
    controls: {
      backgroundColor: themeBackground,
      nodes: {
        type: "strings",
        description: "Files",
        help: "One path per item, like src/app.ts. End a folder with / to show it empty.",
        default: [
          "src/components/cart.tsx",
          "src/components/checkout.tsx",
          "src/lib/api.ts",
          "src/index.ts",
          "public/logo.svg",
          "README.md",
          "package.json",
        ],
      },
    },
    tree: "nodes",
  },
  "drag-drop-flow": { controls: { backgroundColor: themeBackground } },
  "commit-graph": { controls: { backgroundColor: themeBackground } },
  "multi-device-lineup": {
    controls: {
      phoneLabel: text("Phone label", "Phone"),
      tabletLabel: text("Tablet label", "Tablet"),
      laptopLabel: text("Laptop label", "Laptop"),
    },
    nest: { phoneLabel: "phone.label", tabletLabel: "tablet.label", laptopLabel: "laptop.label" },
  },
};

const LIST: CuratedElement[] = [
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
  ...remocn("overlay", ["marker-highlight", "inline-highlight"]),
  ...remocn("overlay", ["ink-underline"], {
    place: "spot",
    controls: spotControls("Where", 0.5, 0.62),
  }),
  ...remocn("overlay", ["scribble-circle"], {
    place: "spot",
    controls: {
      ...spotControls("Around", 0.5, 0.5),
      width: numberControl("Width", 420, { min: 20, max: 3840, step: 10 }, { unit: "px" }),
      height: numberControl("Height", 220, { min: 20, max: 2160, step: 10 }, { unit: "px" }),
    },
  }),
  ...remocn("overlay", ["ink-arrow"], {
    place: "points",
    controls: {
      ...spotControls("From", 0.3, 0.72, "from"),
      ...spotControls("To", 0.48, 0.5, "to"),
    },
  }),
  ...remocn("overlay", ["keystroke", "confetti", "radial-burst"]),
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
  ...remocn("effect", ["crt-screen", "vhs-filter", "halftone-print", "camera-lens"]),
  ...remocn("effect", ["polaroid"], { center: true }),
  ...remocn("effect", ["tv-power-off"], { atEnd: true }),
  ...remocn("effect", ["speed-lines"]),
  ...remocn("scene", ["spring-settle"], {
    kind: "sequencer",
    sceneItem: "SpringSettleItem",
    controls: {
      enterStagger: numberControl(
        "Time between items",
        3,
        { min: 0, max: 10, step: 0.5 },
        { unit: "frames", help: "How long each item in a scene waits after the one before it." },
      ),
      exitFrames: numberControl("Exit time", 6, { min: 2, max: 24, step: 1 }, { unit: "frames" }),
      enterFadeFrames: numberControl("Fade-in time", 5, FRAME_COUNT, {
        unit: "frames",
        advanced: true,
      }),
      springDamping: numberControl(
        "Spring damping",
        30,
        { min: 1, max: 100, step: 1 },
        { help: "Lower values bounce more as items land.", advanced: true },
      ),
      springStiffness: numberControl(
        "Spring stiffness",
        320,
        { min: 10, max: 1000, step: 10 },
        { help: "Higher values land faster.", advanced: true },
      ),
      bgFadeFrames: numberControl("Background fade", 4, FRAME_COUNT, {
        unit: "frames",
        advanced: true,
      }),
    },
  }),
  ...remocn("ui", ["terminal-simulator"]),
  ...remocn("ui", [
    "glass-code-block",
    "code-morph",
    "logo-enter",
    "check-list",
    "search-reveal",
    "agent-run",
  ]),
  ...remocn("data", ["animated-bar-chart", "animated-line-chart", "rolling-number"]),
  ...remocn("scene", ["signup-flow", "ai-prompt-flow", "command-menu"], {
    durationInFrames: 240,
    design: FULL_HD,
  }),
  ...remocn("scene", ["release-teaser"], { durationInFrames: 300 }),
  ...remocn("scene", ["launch-anything"], { durationInFrames: 600 }),
  ...snapcn("device", ["laptop-frame", "phone-frame"], { durationInFrames: 240, design: FULL_HD }),
  ...snapcn("device", ["screen-recording"], { durationInFrames: 240 }),
  ...snapcn("text", ["announce-title"]),
  ...snapcn("text", ["punch-lines", "text-highlight", "status-cycle"]),
  ...snapcn("scene", ["logo-assemble", "logo-drift", "hero-launch"], { durationInFrames: 180 }),
  ...remocn("effect", ["pixelate-region"], { installOnly: true }),
  ...REMOTION_UI,
];

const OVERRIDES = { DESCRIPTIONS, NEUTRAL_DEFAULTS, NEUTRAL_FIXED_PROPS, PAINTED, SETTINGS };

export function unknownOverrideNames(): string[] {
  const names = new Set(LIST.map((item) => item.name));
  return Object.entries(OVERRIDES).flatMap(([table, entries]) =>
    Object.keys(entries)
      .filter((name) => !names.has(name))
      .map((name) => `${table}: "${name}" names no element in the list`),
  );
}

const paintControl = (paint: string): ControlSpec => ({
  type: "color",
  default: paint,
  description: "Background",
  help: "Set it to transparent to lay the text over other layers.",
});

function withSettings(item: CuratedElement): CuratedElement {
  const extra = SETTINGS[item.name];
  if (!extra) return item;
  return {
    ...item,
    ...extra,
    controls: { ...item.controls, ...extra.controls },
    omit: { ...item.omit, ...extra.omit },
  };
}

function withOverrides(base: CuratedElement): CuratedElement {
  const item = withSettings(base);
  const defaults = NEUTRAL_DEFAULTS[item.name];
  const fixedProps = NEUTRAL_FIXED_PROPS[item.name];
  const paint = PAINTED[item.name];
  return {
    ...item,
    ...(defaults ? { defaults: { ...item.defaults, ...defaults } } : {}),
    ...(fixedProps ? { fixedProps: { ...item.fixedProps, ...fixedProps } } : {}),
    ...(paint ? { paint, controls: { ...item.controls, background: paintControl(paint) } } : {}),
  };
}

export const CURATED: CuratedElement[] = LIST.map(withOverrides);
