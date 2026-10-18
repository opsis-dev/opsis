import { z } from "zod";
import { chordKeys } from "../chord";
import { formatDuration, displayAddress } from "../format";
import { nonBlankLines } from "../text-input";
import { PAUSE_MARK } from "../vars";
import {
  CODE_TEXT_MAX,
  elementRow,
  fileRow,
  LABELLED_TEXT_MAX,
  missing,
  mutedIf,
  quote,
  quotedRow,
  row,
  TAG_TEXT_MAX,
  truncate,
  type CardRow,
} from "./card";
import { defineNode, port } from "./define";
import {
  asset,
  beatFields,
  bool,
  choice,
  choiceOf,
  int,
  more,
  ms,
  optionalTarget,
  pickedRole,
  pointerFields,
  target,
  text,
} from "./params";

const chain = {
  inputs: { session: port("session", "Previous") },
  outputs: { session: port("session", "Next") },
} as const;

const pause = (after: number) =>
  after > 0 ? row("timer", `Pause ${formatDuration(after)}`) : null;
const giveUpRow = (timeoutMs: number) =>
  row("timer-off", `Give up after ${formatDuration(timeoutMs)}`, { tone: "muted" });
const zoomRow = (p: { zoom?: boolean; cluster?: string }) =>
  p.zoom === false
    ? row("zoom-out", "No zoom", { tone: "muted" })
    : p.cluster?.trim()
      ? row("zoom-in", `Zoom group “${truncate(p.cluster, TAG_TEXT_MAX)}”`)
      : null;
const base = { category: "Browser", phase: "capture", virtual: true } as const;
const action = { ...base, ...chain, action: true } as const;

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
    viewportWidth: int(1920, "Window width", {
      min: 320,
      max: 3840,
      step: 10,
      unit: "px",
      presetsLabel: "Size",
      presets: [
        { label: "1280 × 720", values: { viewportWidth: 1280, viewportHeight: 720 } },
        { label: "1440 × 900", values: { viewportWidth: 1440, viewportHeight: 900 } },
        { label: "1920 × 1080", values: { viewportWidth: 1920, viewportHeight: 1080 } },
      ],
    }),
    viewportHeight: int(1080, "Window height", { min: 240, max: 2160, step: 10, unit: "px" }),
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
    p.storageState.trim() &&
      row("user-round", `Signed in as ${quote(p.storageState, LABELLED_TEXT_MAX)}`),
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
  ...action,
  type: "browser.click",
  title: "Click",
  icon: "mouse-pointer-click",
  description: "Moves the cursor to an element and clicks it.",
  params: z.object({
    target: target(),
    targetRole: pickedRole(),
    ...beatFields,
    ...pointerFields,
    hoverMs: ms(220, "Hover before clicking", 10_000),
  }),
  card: ({ params: p }) => [
    elementRow("mouse-pointer-click", p.target, p.targetRole),
    pause(p.after),
    zoomRow(p),
  ],
});

export const typeNode = defineNode({
  ...action,
  type: "browser.type",
  title: "Type text",
  icon: "keyboard",
  description: "Types into a field, key by key.",
  params: z.object({
    target: target(),
    targetRole: pickedRole(),
    text: more(text("", "Text to type", "textarea"), {
      pauses: true,
      help: "Use Add pause to hold still for a moment in the middle of typing.",
    }),
    chunkPauseMs: ms(0, "Pause length", 10_000),
    pressEnter: bool(false, "Press Enter after typing"),
    ...beatFields,
    frame: pointerFields.frame,
    frameRole: pointerFields.frameRole,
  }),
  card: ({ params: p }) => {
    const typed = p.text.replaceAll(PAUSE_MARK, " … ");
    return [
      elementRow("crosshair", p.target, p.targetRole),
      quotedRow("keyboard", typed, "Add text to type", { title: typed }),
      p.pressEnter && row("corner-down-left", "Then Enter"),
      pause(p.after),
      zoomRow(p),
    ];
  },
});

export const hoverNode = defineNode({
  ...action,
  type: "browser.hover",
  title: "Hover",
  icon: "pointer",
  description:
    "Moves the cursor over an element without clicking, so a menu or tooltip can appear.",
  params: z.object({
    target: target(),
    targetRole: pickedRole(),
    ...beatFields,
    ...pointerFields,
    hoverMs: ms(600, "Hover for", 10_000),
  }),
  card: ({ params: p }) => [
    elementRow("mouse-pointer-2", p.target, p.targetRole),
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

const distance = (description: string, def: number) =>
  int(def, description, {
    min: -10_000,
    max: 10_000,
    step: 10,
    unit: "px",
    showIf: { mode: ["by"] },
  });

const dragParams = z.object({
  target: target(
    "Element to drag",
    "Pick it on the page, type its visible text, or give a spot as across, down.",
  ),
  targetRole: pickedRole(),
  mode: choice(["element", "by", "path"], "element", "Drag", {
    element: "Onto an element",
    by: "By a distance",
    path: "Along a path",
  }),
  to: more(optionalTarget("Drop on"), { showIf: { mode: ["element"] } }),
  toRole: pickedRole(),
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
  hoverMs: ms(220, "Hover before picking it up", 10_000),
});

type DropParams = Pick<z.infer<typeof dragParams>, "mode" | "to" | "toRole" | "dx" | "dy" | "path">;

function dropRow(p: DropParams): CardRow {
  if (p.mode === "element")
    return onto(elementRow("move", p.to, p.toRole, "Pick where to drop it"));
  if (p.mode === "by") return row("move", `By ${p.dx}, ${p.dy} px`);
  const points = nonBlankLines(p.path).length;
  return points
    ? row("spline", `Along ${points} points`)
    : missing("spline", "Add points to the path");
}

export const dragNode = defineNode({
  ...action,
  type: "browser.drag",
  title: "Drag",
  icon: "grab",
  description:
    "Drags an element onto another one, by a distance, or along a path, like moving a card or drawing a shape.",
  params: dragParams,
  card: ({ params: p }) => [
    elementRow("grab", p.target, p.targetRole),
    dropRow(p),
    pause(p.after),
    zoomRow(p),
  ],
});

export const scrollNode = defineNode({
  ...action,
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
    targetRole: pickedRole(),
    deltaY: int(600, "Distance", {
      min: -20_000,
      max: 20_000,
      help: "Negative scrolls up.",
      step: 50,
      unit: "px",
      showIf: { mode: ["by"] },
    }),
    smooth: bool(true, "Smooth"),
    durationMs: ms(900, "Scroll time", 10_000),
    after: beatFields.after,
  }),
  card: ({ params: p }) => [
    p.mode === "element"
      ? elementRow("scroll", p.target, p.targetRole)
      : row("scroll", `${p.deltaY >= 0 ? "Down" : "Up"} ${Math.abs(p.deltaY)} px`),
    p.smooth && row("timer", `Smooth, ${formatDuration(p.durationMs)}`),
    pause(p.after),
  ],
});

export const selectNode = defineNode({
  ...action,
  type: "browser.select",
  title: "Choose option",
  icon: "list-checks",
  description: "Picks an option in a dropdown.",
  params: z.object({
    target: target(),
    targetRole: pickedRole(),
    value: text("", "Option"),
    ...beatFields,
  }),
  card: ({ params: p }) => [
    elementRow("list", p.target, p.targetRole),
    quotedRow("check", p.value, "Choose an option"),
    pause(p.after),
  ],
});

export const uploadNode = defineNode({
  ...action,
  type: "browser.upload",
  title: "Upload file",
  icon: "upload",
  description: "Attaches a file from your project to a file field.",
  params: z.object({
    target: target("File field", "Pick the upload field, or type its label."),
    targetRole: pickedRole(),
    file: asset("File"),
    ...beatFields,
  }),
  card: ({ params: p }) => [
    elementRow("upload", p.target, p.targetRole),
    p.file ? fileRow("file", p.file) : missing("file", "Choose a file"),
    pause(p.after),
  ],
});

export const pressNode = defineNode({
  ...action,
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

const waitParams = z.object({
  mode: choice(["element", "network-idle", "url", "ms"], "element", "Wait for", {
    element: "An element",
    "network-idle": "A quiet page",
    url: "An address",
    ms: "A fixed time",
  }),
  target: more(optionalTarget("Element"), { showIf: { mode: ["element"] } }),
  targetRole: pickedRole(),
  url: more(text("", "Address pattern"), {
    help: "Use * as a wildcard, e.g. */dashboard*.",
    showIf: { mode: ["url"] },
  }),
  ms: more(ms(1000, "Time"), { showIf: { mode: ["ms"] } }),
  timeoutMs: ms(20_000, "Give up after"),
});

function waitRow(p: z.infer<typeof waitParams>): CardRow {
  switch (p.mode) {
    case "element":
      return elementRow("eye", p.target, p.targetRole, "Pick what to wait for");
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
  params: waitParams,
  card: ({ params: p }) => [waitRow(p), p.mode !== "ms" && giveUpRow(p.timeoutMs)],
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
    targetRole: pickedRole(),
    timeoutMs: ms(10_000, "Give up after"),
  }),
  card: ({ params: p }) => [
    quotedRow("circle-check", p.text, "Add the text to look for"),
    giveUpRow(p.timeoutMs),
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
  params: z.object({ target: target(), targetRole: pickedRole() }),
  card: ({ params: p }) => [elementRow("crosshair", p.target, p.targetRole)],
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
    row("code", truncate(p.script.split("\n")[0], CODE_TEXT_MAX), { title: p.script }),
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
    status: int(200, "Status code", { min: 100, max: 599 }),
    contentType: text("application/json", "Type"),
    body: text('{"ok":true}', "Response", "code"),
  }),
  card: ({ params: p }) => {
    const subtype = p.contentType.slice(p.contentType.lastIndexOf("/") + 1).toUpperCase();
    return [
      row("server", truncate(p.url, CODE_TEXT_MAX), { title: p.url }),
      row("reply", subtype ? `${p.status} ${subtype}` : String(p.status)),
    ];
  },
});

const CLOCK_MODES = { fixed: "Frozen", paused: "Paused" };

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
    mode: choiceOf(CLOCK_MODES, "fixed", "Clock"),
  }),
  card: ({ params: p }) => {
    const d = new Date(p.time);
    const when = Number.isNaN(d.getTime())
      ? p.time
      : d.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
    return [row("clock", `${when} · ${CLOCK_MODES[p.mode]}`)];
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
    const extra = nonBlankLines(p.selectors).length;
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
    row("code-xml", truncate(p.html.replace(/<[^>]+>/g, " "), CODE_TEXT_MAX) || "HTML", {
      title: p.html,
    }),
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
  browserNodes.filter((n) => n.action).map((n) => n.type),
);
