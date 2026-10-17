import { z } from "zod";
import { clip, elementRow, fileRow, missing, mutedIf, quote, row, type CardRow } from "./card";
import { defineNode, port } from "./define";
import {
  asset,
  beatFields,
  bool,
  choice,
  hoverMs,
  more,
  ms,
  optionalTarget,
  pointerFields,
  target,
  text,
} from "./params";
import { chordKeys, formatDuration, displayAddress } from "../format";
import { PAUSE_MARK } from "../vars";

const chain = {
  inputs: { session: port("session", "Previous") },
  outputs: { session: port("session", "Next") },
} as const;

const pause = (after: number) =>
  after > 0 ? row("timer", `Pause ${formatDuration(after)}`) : null;
const zoomRow = (p: { zoom?: boolean; cluster?: string }) =>
  p.zoom === false
    ? row("zoom-out", "No zoom", { tone: "muted" })
    : p.cluster?.trim()
      ? row("zoom-in", `Zoom group “${clip(p.cluster, 20)}”`)
      : null;
const base = { category: "Browser", phase: "capture", virtual: true } as const;

export const sessionNode = defineNode({
  ...base,
  type: "browser.session",
  title: "Open browser",
  icon: "globe",
  description:
    "Starts a browser window for the actions that follow: size, device, language, sign-in.",
  inputs: {},
  outputs: { session: port("session", "Next") },
  params: z.object({
    viewportWidth: z
      .number()
      .int()
      .min(320)
      .max(3840)
      .default(1920)
      .meta({
        description: "Window width",
        step: 10,
        unit: "px",
        presetsLabel: "Size",
        presets: [
          { label: "1280 × 720", values: { viewportWidth: 1280, viewportHeight: 720 } },
          { label: "1440 × 900", values: { viewportWidth: 1440, viewportHeight: 900 } },
          { label: "1920 × 1080", values: { viewportWidth: 1920, viewportHeight: 1080 } },
        ],
      }),
    viewportHeight: z
      .number()
      .int()
      .min(240)
      .max(2160)
      .default(1080)
      .meta({ description: "Window height", step: 10, unit: "px" }),
    dpr: z.number().min(1).max(4).default(2).meta({
      description: "Screenshot sharpness",
      help: "Only affects screenshots. 2× or more gives crisp 4K images.",
      step: 1,
      unit: "×",
    }),
    mobile: bool(false, "Mobile device"),
    userAgent: more(text("", "Browser identity"), {
      help: "Pretend to be a specific browser. Leave empty for the default.",
      advanced: true,
    }),
    storageState: more(text("", "Signed in as"), {
      widget: "secret",
      help: "A sign-in saved by a Sign in once node.",
    }),
    locale: more(text("en-US", "Language"), { help: "For example en-US, de-DE, ja-JP." }),
    timezone: more(text("", "Time zone"), { widget: "timezone" }),
    colorScheme: choice(["light", "dark", "no-preference"], "light", "Theme", {
      light: "Light",
      dark: "Dark",
      "no-preference": "System",
    }),
    captureScale: z.number().min(1).max(4).default(1).meta({
      description: "Recording sharpness",
      help: "Records at a higher resolution so zoomed-in shots stay sharp.",
      step: 0.25,
      unit: "×",
      advanced: true,
    }),
  }),
  card: ({ params: p }) => [
    row("monitor", `${p.viewportWidth} × ${p.viewportHeight}`),
    p.mobile && row("smartphone", "Mobile"),
    p.storageState.trim() && row("user-round", `Signed in as ${quote(p.storageState, 24)}`),
    p.colorScheme === "dark" && row("moon", "Dark theme"),
    p.locale && p.locale !== "en-US" && row("languages", p.locale),
  ],
});

export const navigateNode = defineNode({
  ...base,
  ...chain,
  type: "browser.navigate",
  title: "Go to page",
  icon: "link",
  description: "Opens a page address.",
  params: z.object({
    url: text("{{Site address}}", "Page address"),
    waitUntil: choice(
      ["load", "domcontentloaded", "networkidle", "commit"],
      "load",
      "Wait until the page has",
      {
        load: "Loaded",
        domcontentloaded: "Started showing",
        networkidle: "Settled",
        commit: "Responded",
      },
    ),
    after: more(ms(700, "Pause after"), { help: "How long the video lingers on the new page." }),
  }),
  card: ({ params: p }) => [
    p.url.trim()
      ? row("link", displayAddress(p.url, { keepQuery: true }), { title: p.url })
      : missing("link", "Add an address"),
    pause(p.after),
  ],
});

export const clickNode = defineNode({
  ...base,
  ...chain,
  type: "browser.click",
  title: "Click",
  icon: "mouse-pointer-click",
  description: "Moves the cursor to an element and clicks it.",
  params: z.object({
    target: target(),
    ...beatFields,
    ...pointerFields,
    hoverMs: hoverMs(220, "Hover before clicking"),
  }),
  card: ({ params: p, raw }) => [
    elementRow("mouse-pointer-click", p.target, raw),
    pause(p.after),
    zoomRow(p),
  ],
});

export const typeNode = defineNode({
  ...base,
  ...chain,
  type: "browser.type",
  title: "Type text",
  icon: "keyboard",
  description: "Types into a field, key by key.",
  params: z.object({
    target: target(),
    text: more(text("", "Text to type", "textarea"), {
      pauses: true,
      help: "Use Add pause to hold still for a moment in the middle of typing.",
    }),
    chunkPauseMs: ms(0, "Pause length", 10_000),
    pressEnter: bool(false, "Press Enter after typing"),
    ...beatFields,
    frame: pointerFields.frame,
  }),
  card: ({ params: p, raw }) => [
    elementRow("crosshair", p.target, raw),
    p.text.trim()
      ? row("keyboard", quote(p.text.replaceAll(PAUSE_MARK, " … ")), {
          title: p.text.replaceAll(PAUSE_MARK, " … "),
        })
      : missing("keyboard", "Add text to type"),
    p.pressEnter && row("corner-down-left", "Then Enter"),
    pause(p.after),
    zoomRow(p),
  ],
});

export const hoverNode = defineNode({
  ...base,
  ...chain,
  type: "browser.hover",
  title: "Hover",
  icon: "pointer",
  description:
    "Moves the cursor over an element without clicking, so a menu or tooltip can appear.",
  params: z.object({
    target: target(),
    ...beatFields,
    ...pointerFields,
    hoverMs: hoverMs(600, "Hover for"),
  }),
  card: ({ params: p, raw }) => [
    elementRow("mouse-pointer-2", p.target, raw),
    row("hourglass", `Hover ${formatDuration(p.hoverMs)}`),
    pause(p.after),
    zoomRow(p),
  ],
});

const circlePath = (radius: number, steps: number) =>
  Array.from({ length: steps + 1 }, (_, k) => {
    const angle = (k / steps) * 2 * Math.PI;
    return `${Math.round(radius - radius * Math.cos(angle))}, ${Math.round(-radius * Math.sin(angle))}`;
  }).join("\n");

const PATH_PRESETS = {
  Circle: circlePath(120, 16),
  Tick: "0, 0\n40, 40\n120, -80",
  Zigzag: "0, 0\n60, -60\n120, 0\n180, -60\n240, 0",
  Square: "0, 0\n200, 0\n200, 200\n0, 200\n0, 0",
};

const onto = (r: CardRow): CardRow => (r.tone === "warn" ? r : { ...r, text: `Onto ${r.text}` });

const pathPoints = (path: string) => path.split("\n").filter((line) => line.trim()).length;

function dropRow(
  p: {
    mode: "element" | "by" | "path";
    to: string;
    dx: number;
    dy: number;
    path: string;
  },
  raw: Record<string, unknown>,
): CardRow {
  if (p.mode === "element")
    return onto(elementRow("move", p.to, raw, "to", "Pick where to drop it"));
  if (p.mode === "by") return row("move", `By ${p.dx}, ${p.dy} px`);
  const points = pathPoints(p.path);
  return points
    ? row("spline", `Along ${points} points`)
    : missing("spline", "Add points to the path");
}

const distance = (description: string, def: number) =>
  z
    .number()
    .int()
    .min(-10_000)
    .max(10_000)
    .default(def)
    .meta({ description, step: 10, unit: "px", showIf: { mode: ["by"] } });

export const dragNode = defineNode({
  ...base,
  ...chain,
  type: "browser.drag",
  title: "Drag",
  icon: "grab",
  description:
    "Drags an element onto another one, by a distance, or along a path, like moving a card or drawing a shape.",
  params: z.object({
    target: target(
      "Element to drag",
      "Pick it on the page, type its visible text, or give a spot as across, down.",
    ),
    mode: choice(["element", "by", "path"], "element", "Drag", {
      element: "Onto an element",
      by: "By a distance",
      path: "Along a path",
    }),
    to: more(optionalTarget("Drop on"), { showIf: { mode: ["element"] } }),
    dx: distance("Across", 300),
    dy: distance("Down", 0),
    path: more(text("", "Path", "textarea"), {
      help: "One point per line, as across, down in pixels from where the drag starts.",
      presets: Object.entries(PATH_PRESETS).map(([label, path]) => ({
        label,
        values: { path },
      })),
      showIf: { mode: ["path"] },
    }),
    holdMs: ms(150, "Hold before moving", 5000),
    dragMs: more(ms(0, "Drag time", 10_000), { zeroLabel: "Auto" }),
    method: more(
      choice(["smooth", "quick"], "smooth", "How", {
        smooth: "Smooth, with the cursor",
        quick: "Quick, for pages that ignore the smooth drag",
      }),
      { advanced: true },
    ),
    ...beatFields,
    ...pointerFields,
    hoverMs: hoverMs(220, "Hover before picking it up"),
  }),
  card: ({ params: p, raw }) => [
    elementRow("grab", p.target, raw),
    dropRow(p, raw),
    pause(p.after),
    zoomRow(p),
  ],
});

export const scrollNode = defineNode({
  ...base,
  ...chain,
  type: "browser.scroll",
  title: "Scroll",
  icon: "arrow-down-up",
  description: "Scrolls to an element or by a distance.",
  params: z.object({
    mode: choice(["element", "by"], "element", "Scroll to", {
      element: "An element",
      by: "A distance",
    }),
    target: more(optionalTarget("Element"), { showIf: { mode: ["element"] } }),
    deltaY: z
      .number()
      .int()
      .min(-20_000)
      .max(20_000)
      .default(600)
      .meta({
        description: "Distance",
        help: "Negative scrolls up.",
        step: 50,
        unit: "px",
        showIf: { mode: ["by"] },
      }),
    smooth: bool(true, "Smooth"),
    durationMs: ms(900, "Scroll time", 10_000),
    after: beatFields.after,
  }),
  card: ({ params: p, raw }) => [
    p.mode === "element"
      ? elementRow("scroll", p.target, raw)
      : row("scroll", `${p.deltaY >= 0 ? "Down" : "Up"} ${Math.abs(p.deltaY)} px`),
    p.smooth && row("timer", `Smooth, ${formatDuration(p.durationMs)}`),
    pause(p.after),
  ],
});

export const selectNode = defineNode({
  ...base,
  ...chain,
  type: "browser.select",
  title: "Choose option",
  icon: "list-checks",
  description: "Picks an option in a dropdown.",
  params: z.object({ target: target(), value: text("", "Option"), ...beatFields }),
  card: ({ params: p, raw }) => [
    elementRow("list", p.target, raw),
    p.value.trim() ? row("check", quote(p.value)) : missing("check", "Choose an option"),
    pause(p.after),
  ],
});

export const uploadNode = defineNode({
  ...base,
  ...chain,
  type: "browser.upload",
  title: "Upload file",
  icon: "upload",
  description: "Attaches a file from your project to a file field.",
  params: z.object({
    target: target("File field", "Pick the upload field, or type its label."),
    file: asset("File"),
    ...beatFields,
  }),
  card: ({ params: p, raw }) => [
    elementRow("upload", p.target, raw),
    p.file ? fileRow("file", p.file) : missing("file", "Choose a file"),
    pause(p.after),
  ],
});

export const pressNode = defineNode({
  ...base,
  ...chain,
  type: "browser.press",
  title: "Press keys",
  icon: "command",
  description: "Presses a key or shortcut, like Ctrl+K. Shows in keystroke chips.",
  params: z.object({
    keys: more(text("Enter", "Keys"), {
      widget: "keys",
      help: "Click the field and press the shortcut.",
    }),
    after: beatFields.after,
  }),
  card: ({ params: p }) => [
    p.keys.trim()
      ? row("command", chordKeys(p.keys).join(" + "))
      : missing("command", "Press a shortcut"),
    pause(p.after),
  ],
});

function waitRow(
  p: { mode: "element" | "network-idle" | "url" | "ms"; target: string; url: string; ms: number },
  raw: Record<string, unknown>,
): CardRow {
  switch (p.mode) {
    case "element":
      return elementRow("eye", p.target, raw, "target", "Pick what to wait for");
    case "network-idle":
      return row("wifi", "Until the page is quiet");
    case "url":
      return p.url.trim()
        ? row("link", `Until ${displayAddress(p.url, { keepQuery: true })}`)
        : missing("link", "Add an address pattern");
    case "ms":
      return row("hourglass", formatDuration(p.ms));
  }
}

export const waitNode = defineNode({
  ...base,
  ...chain,
  type: "browser.wait",
  title: "Wait",
  icon: "hourglass",
  description:
    "Waits for an element, a quiet page, an address, or a fixed time. Waiting for something is more reliable than a fixed time.",
  params: z.object({
    mode: choice(["element", "network-idle", "url", "ms"], "element", "Wait for", {
      element: "An element",
      "network-idle": "A quiet page",
      url: "An address",
      ms: "A fixed time",
    }),
    target: more(optionalTarget("Element"), { showIf: { mode: ["element"] } }),
    url: more(text("", "Address pattern"), {
      help: "Use * as a wildcard, e.g. */dashboard*.",
      showIf: { mode: ["url"] },
    }),
    ms: more(ms(1000, "Time"), { showIf: { mode: ["ms"] } }),
    timeoutMs: ms(20_000, "Give up after"),
  }),
  card: ({ params: p, raw }) => [
    waitRow(p, raw),
    p.mode !== "ms" &&
      row("timer-off", `Give up after ${formatDuration(p.timeoutMs)}`, { tone: "muted" }),
  ],
});

export const assertNode = defineNode({
  ...base,
  ...chain,
  type: "browser.assert",
  title: "Check text",
  icon: "badge-check",
  description:
    "Stops the recording early if some text is not on the page, so you don't export a broken take.",
  params: z.object({
    text: text("", "Text"),
    target: optionalTarget("Only inside (optional)"),
    timeoutMs: ms(10_000, "Give up after"),
  }),
  card: ({ params: p }) => [
    p.text.trim()
      ? row("circle-check", quote(p.text))
      : missing("circle-check", "Add the text to look for"),
    row("timer-off", `Give up after ${formatDuration(p.timeoutMs)}`, { tone: "muted" }),
  ],
});

export const hoistNode = defineNode({
  ...base,
  ...chain,
  type: "browser.hoist",
  title: "Find element early",
  icon: "anchor",
  description:
    "Looks an element up ahead of time so a later action doesn't pause on screen while searching.",
  params: z.object({ target: target() }),
  card: ({ params: p, raw }) => [elementRow("crosshair", p.target, raw)],
});

export const setStateNode = defineNode({
  ...base,
  ...chain,
  type: "browser.setState",
  title: "Run page script",
  icon: "braces",
  description:
    "Runs a small script in the page, e.g. to skip onboarding or turn on a feature. Put it before Go to page to run it as the site first opens.",
  params: z.object({
    script: text("localStorage.setItem('onboarded', '1');", "Script", "code"),
    reload: bool(false, "Reload the page after"),
  }),
  card: ({ params: p }) => [
    row("code", clip(p.script.split("\n")[0], 36), { title: p.script }),
    p.reload && row("refresh-cw", "Then reload"),
  ],
});

export const mockRouteNode = defineNode({
  ...base,
  ...chain,
  type: "browser.mockRoute",
  title: "Fake a response",
  icon: "server",
  description:
    "Makes the site show fixed data for matching requests, so every take looks the same.",
  params: z.object({
    url: more(text("**/api/**", "Address pattern"), { help: "Use * as a wildcard." }),
    status: z.number().int().min(100).max(599).default(200).meta({ description: "Status code" }),
    contentType: text("application/json", "Type"),
    body: text('{"ok":true}', "Response", "code"),
  }),
  card: ({ params: p }) => [
    row("server", clip(p.url, 36), { title: p.url }),
    row(
      "reply",
      `${p.status} ${p.contentType.slice(p.contentType.lastIndexOf("/") + 1).toUpperCase()}`.trim(),
    ),
  ],
});

export const freezeClockNode = defineNode({
  ...base,
  ...chain,
  type: "browser.freezeClock",
  title: "Freeze time",
  icon: "clock",
  description:
    'Pins the date and time the page sees, so clocks and "x minutes ago" never change between takes.',
  params: z.object({
    time: more(text("2026-10-01T09:41:00", "Date and time"), { widget: "datetime" }),
    mode: choice(["fixed", "paused"], "fixed", "Clock", { fixed: "Frozen", paused: "Paused" }),
  }),
  card: ({ params: p }) => {
    const d = new Date(p.time);
    const when = Number.isNaN(d.getTime())
      ? p.time
      : d.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
    return [row("clock", `${when} · ${p.mode === "fixed" ? "Frozen" : "Paused"}`)];
  },
});

export const hideElementsNode = defineNode({
  ...base,
  ...chain,
  type: "browser.hideElements",
  title: "Hide clutter",
  icon: "eye-off",
  description: "Hides cookie banners, chat bubbles and anything else you list.",
  params: z.object({
    cookieBanners: bool(true, "Cookie banners"),
    chatWidgets: bool(true, "Chat widgets"),
    selectors: more(text("", "Also hide", "textarea"), {
      help: "CSS selectors, one per line.",
      advanced: true,
    }),
  }),
  card: ({ params: p }) => {
    const extra = p.selectors.split("\n").filter((l) => l.trim()).length;
    const parts = [
      p.cookieBanners && "Cookie banners",
      p.chatWidgets && "Chat widgets",
      extra && `+${extra} more`,
    ].filter(Boolean);
    return [
      row("eye-off", parts.length ? parts.join(" · ") : "Nothing yet", mutedIf(!parts.length)),
    ];
  },
});

export const chapterNode = defineNode({
  ...base,
  ...chain,
  type: "browser.chapter",
  title: "Chapter",
  icon: "bookmark",
  description: "Starts a new chapter. Can show a chapter card in the video.",
  params: z.object({
    title: text("Chapter", "Title"),
    description: text("", "Description"),
    showInVideo: bool(false, "Show a chapter card in the video"),
    durationMs: more(ms(2000, "Card shown for"), { showIf: { showInVideo: ["true"] } }),
  }),
  card: ({ params: p }) => [
    row("bookmark", quote(p.title)),
    p.showInVideo && row("presentation", `Card for ${formatDuration(p.durationMs)}`),
  ],
});

export const overlayHtmlNode = defineNode({
  ...base,
  ...chain,
  type: "browser.overlayHtml",
  title: "HTML overlay",
  icon: "code",
  description: "Shows your own HTML on top of the page while recording.",
  params: z.object({
    html: text('<div style="font:600 32px system-ui">Hello</div>', "HTML", "code"),
    durationMs: more(ms(2000, "Shown for"), {
      help: "The next actions carry on while it's shown.",
      zeroLabel: "Until the end",
    }),
  }),
  card: ({ params: p }) => [
    row("code-xml", clip(p.html.replace(/<[^>]+>/g, " "), 36) || "HTML", { title: p.html }),
    row("timer", p.durationMs ? formatDuration(p.durationMs) : "Until the end"),
  ],
});

export const browserNodes = [
  sessionNode,
  navigateNode,
  clickNode,
  typeNode,
  hoverNode,
  dragNode,
  scrollNode,
  selectNode,
  uploadNode,
  pressNode,
  waitNode,
  assertNode,
  hoistNode,
  setStateNode,
  mockRouteNode,
  freezeClockNode,
  hideElementsNode,
  chapterNode,
  overlayHtmlNode,
];

export const ACTION_NODE_TYPES: ReadonlySet<string> = new Set(
  [clickNode, typeNode, hoverNode, dragNode, selectNode, pressNode, scrollNode, uploadNode].map(
    (n) => n.type,
  ),
);
