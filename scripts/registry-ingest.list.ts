type ElementCategory =
  | "text"
  | "overlay"
  | "transition"
  | "background"
  | "effect"
  | "ui"
  | "data"
  | "scene"
  | "device";

export type ControlSpec = {
  type: string;
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
  widget?: string;
  hidden?: boolean;
};

export type CuratedElement = {
  registry: "remocn" | "snapcn" | "remotionui";
  name: string;
  category: ElementCategory;
  title?: string;
  description?: string;
  exportName?: string;
  noConfig?: boolean;
  controls?: Record<string, ControlSpec>;
  omit?: string[];
  defaults?: Record<string, unknown>;
  fixedProps?: Record<string, unknown>;
  durationInFrames?: number;
  kind?: "component" | "wrapper" | "presentation";
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
  remotionui: {
    label: "RemotionUI",
    keyPrefix: "remotion-ui",
    license: "MIT",
    docs: "https://remotionui.com/docs/components",
    rawBase:
      "https://raw.githubusercontent.com/riaz37/remotion-ui/b7e0e6becc3d22b8b03dfef72c064f72ff9fad1f/apps/web/registry/bases/default",
    configDirs: [],
    propsUrl: (name) => `https://remotionui.com/ai/components/${name}.json`,
    durationsUrl:
      "https://raw.githubusercontent.com/riaz37/remotion-ui/b7e0e6becc3d22b8b03dfef72c064f72ff9fad1f/apps/web/lib/preview-config.ts",
    itemUrl: (name) => `https://remotionui.com/r/${name}.json`,
    installDir: () => "remotion-ui",
  },
};

const TEMPLATES = new Set(["release-teaser", "launch-anything"]);

const fromRegistry =
  (registry: CuratedElement["registry"], base: Partial<CuratedElement> = {}) =>
  (
    category: ElementCategory,
    names: string[],
    extra: Partial<CuratedElement> = {},
  ): CuratedElement[] =>
    names.map((name) => ({ registry, name, category, ...base, ...extra }));

const remocn = fromRegistry("remocn");
const snapcn = fromRegistry("snapcn", { fixedProps: { mode: "dark" } });

const remotionui = (
  name: string,
  category: ElementCategory,
  extra: Partial<CuratedElement> = {},
): CuratedElement => ({ registry: "remotionui", name, category, ...extra });

export const DESCRIPTIONS: Partial<Record<string, string>> = {
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
  remotionui("grid-pixelate-wipe", "transition", {
    title: "Pixel grid wipe",
    exportName: "gridPixelateWipe",
  }),
  remotionui("frosted-glass-wipe", "transition", {
    title: "Frosted glass wipe",
    exportName: "frostedGlassWipe",
  }),
  remotionui("chromatic-aberration-wipe", "transition", {
    title: "Colour split slide",
    exportName: "chromaticAberrationWipe",
  }),
  remotionui("transition-blinds", "transition", { title: "Blinds", exportName: "blinds" }),
  remotionui("transition-morph-shape", "transition", {
    title: "Shape morph",
    exportName: "morphShape",
  }),
  remotionui("stat-card", "data", {
    title: "Stat card",
    defaults: { prefix: "", caption: "Customer survey, last 30 days", delta: 4 },
  }),
  remotionui("metric-ticker", "data", {
    title: "Metric cards",
    defaults: { eyebrow: "Quarter recap", title: "Growth this quarter", metrics: METRICS },
  }),
  remotionui("funnel-chart", "data", {
    title: "Funnel chart",
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
  remotionui("donut-chart", "data", {
    title: "Donut chart",
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
  remotionui("comparison-bars", "data", {
    title: "Before and after bars",
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
  remotionui("bar-chart-race", "data", {
    title: "Bar chart race",
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
  remotionui("gauge-dial", "data", {
    title: "Gauge",
    center: true,
    defaults: { value: 78, label: "Weekly goal", unit: "%", size: 480 },
  }),
  remotionui("data-story", "scene", {
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
  remotionui("logo-reveal", "scene", {
    title: "Logo reveal",
    defaults: {
      pathD:
        "M 100 20 L 170 60 L 170 140 L 100 180 L 30 140 L 30 60 Z M 100 64 L 138 86 L 138 130 L 100 152 L 62 130 L 62 86 Z",
      wordmark: "Acme",
      tagline: "Ship your next release faster",
    },
  }),
  remotionui("pricing-card", "scene", { title: "Pricing card" }),
  remotionui("comparison-table", "scene", { title: "Plan comparison table" }),
  remotionui("changelog-entry", "scene", { title: "Changelog" }),
  remotionui("logo-wall", "scene", {
    title: "Logo wall",
    defaults: { title: "Teams that ship with us" },
  }),
  remotionui("end-card", "scene", {
    title: "End card",
    defaults: {
      eyebrow: "",
      title: "Ready to try it?",
      subtitle: "Start free. No credit card needed.",
      cta: "Start free trial",
      url: "example.com/signup",
      handles: ["@acme", "github.com/acme"],
    },
  }),
  remotionui("code-reveal", "ui", {
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
  remotionui("code-diff-wipe", "ui", { title: "Code diff" }),
  remotionui("kanban-move", "ui", {
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
  remotionui("file-tree-reveal", "ui", {
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
  remotionui("drag-drop-flow", "ui", { title: "File upload" }),
  remotionui("commit-graph", "ui", { title: "Commit graph" }),
  remotionui("notification-stack", "overlay", { title: "Notification stack" }),
  remotionui("device-mockup-zoom", "device", {
    title: "Device mockup",
    kind: "wrapper",
    screen: true,
    defaults: { eyebrow: "", title: "", subtitle: "" },
  }),
  remotionui("multi-device-lineup", "device", {
    title: "Device lineup",
    kind: "wrapper",
    screen: true,
    center: true,
    defaults: { scale: 1.6 },
    fixedProps: {
      phone: { label: "Phone" },
      tablet: { label: "Tablet" },
      laptop: { label: "Laptop" },
    },
  }),
  remotionui("handwriting-text", "text", {
    title: "Handwriting",
    center: true,
    omit: ["durationInFrames"],
    defaults: { text: "Signed by hand", penColor: "#e8b86d" },
  }),
  remotionui("neon-flicker-text", "text", {
    title: "Neon sign",
    center: true,
    omit: ["durationInFrames"],
    defaults: { text: "Open all night" },
  }),
  remotionui("text-mask-video", "text", {
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
    },
  }),
  remotionui("aurora-bg", "background", { title: "Aurora" }),
  remotionui("particle-field", "background", { title: "Particle field" }),
];

function spotControls(
  label: string,
  x: number,
  y: number,
  prefix = "",
): Record<string, ControlSpec> {
  const xKey = prefix ? `${prefix}X` : "x";
  const yKey = prefix ? `${prefix}Y` : "y";
  const fraction = { type: "number", min: 0, max: 1, step: 0.01 } as const;
  return {
    [xKey]: {
      ...fraction,
      default: x,
      description: label,
      widget: "spot",
      help: "Click the frame to choose the spot.",
    },
    [yKey]: { ...fraction, default: y, description: `${label} vertical`, hidden: true },
  };
}

const NEUTRAL_DEFAULTS: Record<string, Record<string, unknown>> = {
  "staggered-fade-up": { text: "Ship your demo in minutes" },
  "inline-highlight": { before: "Ship faster with ", highlight: "Acme", after: "." },
  "terminal-simulator": { title: "~/projects/acme" },
  "search-reveal": { text: "acme" },
  "laptop-frame": { screenSrc: "app:placeholders/screen-wide.svg" },
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

const NEUTRAL_FIXED_PROPS: Record<string, Record<string, unknown>> = {
  "logo-assemble": {
    images: ["aurora", "bloom", "canyon", "chalk", "cobalt", "dusk", "ember", "flare"].map(
      (name) => `app:backdrops/${name}.jpg`,
    ),
  },
};

const PAINTED: Record<string, string> = {
  typewriter: "white",
  "shimmer-sweep": "white",
  "matrix-decode": "white",
  "slot-machine-roll": "white",
  "staggered-fade-up": "white",
  "tracking-in": "white",
  "rgb-glitch-text": "#fafafa",
  "strikethrough-replace": "white",
  "marker-highlight": "white",
  "inline-highlight": "white",
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
    controls: { ...spotControls("Where", 0.5, 0.62) },
  }),
  ...remocn("overlay", ["scribble-circle"], {
    place: "spot",
    controls: {
      ...spotControls("Around", 0.5, 0.5),
      width: {
        type: "number",
        default: 420,
        min: 20,
        max: 3840,
        step: 10,
        unit: "px",
        description: "Width",
      },
      height: {
        type: "number",
        default: 220,
        min: 20,
        max: 2160,
        step: 10,
        unit: "px",
        description: "Height",
      },
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
  ...remocn("effect", ["speed-lines", "spring-settle"]),
  ...remocn("ui", ["terminal-simulator"], {
    controls: {
      lines: {
        type: "rows",
        default: [
          { text: "deploy --production", type: "command", delay: 0 },
          { text: "Running tests...", type: "log", delay: 6 },
          { text: "128 tests passed", type: "success", delay: 10 },
          { text: "Building the image...", type: "log", delay: 8 },
          { text: "Image ready in 41s", type: "success", delay: 10 },
          { text: "Rolling out to everyone...", type: "log", delay: 8 },
          { text: "Live in production", type: "success", delay: 12 },
        ],
        description: "Lines",
        help: "Each line has text and a type: command, log, success or error. Lines ending in ... pause before the next one.",
      },
    },
  }),
  ...remocn("ui", [
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
  { registry: "remocn", name: "pixelate-region", category: "effect", installOnly: true },
  ...REMOTION_UI,
];

const OVERRIDES = { DESCRIPTIONS, NEUTRAL_DEFAULTS, NEUTRAL_FIXED_PROPS, PAINTED };

export function unknownOverrideNames(): string[] {
  const names = new Set(LIST.map((item) => item.name));
  return Object.entries(OVERRIDES).flatMap(([table, entries]) =>
    Object.keys(entries)
      .filter((name) => !names.has(name))
      .map((name) => `${table}: "${name}" names no element in the list`),
  );
}

export const CURATED: CuratedElement[] = LIST.map((item) => ({
  ...item,
  ...(NEUTRAL_DEFAULTS[item.name]
    ? { defaults: { ...item.defaults, ...NEUTRAL_DEFAULTS[item.name] } }
    : {}),
  ...(NEUTRAL_FIXED_PROPS[item.name]
    ? { fixedProps: { ...item.fixedProps, ...NEUTRAL_FIXED_PROPS[item.name] } }
    : {}),
  ...(PAINTED[item.name]
    ? {
        paint: PAINTED[item.name],
        controls: {
          ...item.controls,
          background: {
            type: "color",
            default: PAINTED[item.name],
            description: "Background",
            help: "Set it to transparent to lay the text over other layers.",
          },
        },
      }
    : {}),
}));
