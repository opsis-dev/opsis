import { z } from "zod";
import { itemClip } from "../compile/clip";
import { outputOf } from "../schema/values";
import { quote, row, warnIf } from "./card";
import { defineNode, port } from "./define";
import { secretFileName, secretName } from "../secrets";
import { bool, choice, keysOf, more, ms, optionalTarget, text, visualEnvelope } from "./params";
import { formatDuration, displayAddress } from "../format";
import { msToFrames } from "../timeline/time";

export const DEFAULT_VIEWPORT = { width: 1920, height: 1080 };

export const DEFAULT_TAKE_MS = 6000;

const CURSOR_LABELS = { synthetic: "Smooth", baked: "As on screen", none: "None" };

const CURSOR_ROWS: Record<keyof typeof CURSOR_LABELS, string> = {
  synthetic: "Smooth cursor",
  baked: "Cursor as on screen",
  none: "No cursor",
};

const REGION_LABELS = {
  viewport: "Visible part",
  "full-page": "Whole page",
  element: "One element",
};

export const recordNode = defineNode({
  type: "capture.record",
  title: "Record take",
  category: "Capture",
  icon: "video",
  phase: "capture",
  executes: true,
  description: "Plays all connected browser actions in one go and records the screen.",
  inputs: { session: port("session", "Previous") },
  outputs: { recording: port("recording", "Recording") },
  params: z.object({
    fps: z
      .union([z.literal(25), z.literal(30), z.literal(60)])
      .default(30)
      .meta({ description: "Frame rate", unit: "fps" }),
    quality: z
      .number()
      .int()
      .min(10)
      .max(100)
      .default(90)
      .meta({ description: "Recording quality", step: 5, unit: "%", advanced: true }),
    establishMs: ms(700, "Pause before the first action"),
    endTailMs: more(ms(2500, "Keep recording after the last action"), {
      help: "Gives the camera time to settle at the end.",
    }),
    cursor: choice(keysOf(CURSOR_LABELS), "synthetic", "Cursor", CURSOR_LABELS),
    showActions: bool(false, "Quick mode: show action labels on the video"),
    mutates: more(text("", "Don't record at the same time as"), {
      help: "Takes that share this name run one after another, e.g. when they change the same account.",
      advanced: true,
    }),
  }),
  card: ({ params: p, result }) => {
    const rec = outputOf(result, "recording", "recording");
    return [
      row("video", `${p.fps} fps`),
      row("mouse-pointer", CURSOR_ROWS[p.cursor]),
      rec?.src && row("clock", formatDuration(rec.durationMs)),
      rec?.log && row("list-checks", `${rec.log.actions.length} actions`),
    ];
  },
  mutates: (p) => p.mutates || undefined,
  compile: ({ nodeId, result }) => ({
    recording: outputOf(result, "recording", "recording") ?? {
      type: "recording",
      src: null,
      log: null,
      durationMs: DEFAULT_TAKE_MS,
      viewport: DEFAULT_VIEWPORT,
      nodeId,
    },
  }),
});

export const stillNode = defineNode({
  type: "capture.still",
  title: "Screenshot",
  category: "Capture",
  icon: "camera",
  phase: "capture",
  executes: true,
  description:
    "Plays the connected actions, then takes a sharp screenshot of the page or one element.",
  inputs: { session: port("session", "Previous") },
  outputs: { image: port("image", "Image"), clip: port("clip", "Clip") },
  params: z.object({
    region: choice(keysOf(REGION_LABELS), "viewport", "Capture", REGION_LABELS),
    target: more(optionalTarget("Element"), { showIf: { region: ["element"] } }),
    padding: z
      .number()
      .int()
      .min(0)
      .max(400)
      .default(24)
      .meta({
        description: "Space around",
        unit: "px",
        showIf: { region: ["element"] },
      }),
    look: choice(["window", "cutout", "bleed"], "window", "Style", {
      window: "Window",
      cutout: "Cut-out",
      bleed: "Full bleed",
    }),
    backdrop: more(text("glaze", "Backdrop"), { widget: "backdrop" }),
    durationMs: ms(3000, "Length"),
    envelope: visualEnvelope(),
  }),
  card: ({ params: p, result }) => {
    const image = outputOf(result, "image", "image");
    const unpicked = p.region === "element" && !p.target.trim();
    const region =
      p.region !== "element"
        ? REGION_LABELS[p.region]
        : unpicked
          ? "Pick an element"
          : quote(p.target, 30);
    return [
      row("scan", region, warnIf(unpicked)),
      image?.width
        ? row("image", `${Math.round(image.width)} × ${Math.round(image.height ?? 0)}`)
        : null,
      row("clock", formatDuration(p.durationMs)),
    ];
  },
  compile: ({ nodeId, result, params, settings }) => {
    const image = outputOf(result, "image", "image") ?? { type: "image", src: null };
    const [component, props]: [string, Record<string, unknown>] = image.src
      ? [
          "core/still",
          {
            src: image.src,
            look: params.look,
            backdrop: params.backdrop,
            ...(image.width && image.height
              ? {
                  shot: {
                    width: image.width,
                    height: image.height,
                    scale: image.scale ?? 1,
                    ...(image.body ? { body: image.body } : {}),
                  },
                }
              : {}),
          },
        ]
      : ["core/placeholder", { title: "Screenshot not taken yet", subtitle: "Run to take it" }];
    const clip = itemClip(nodeId, component, props, msToFrames(params.durationMs, settings.fps), {
      handles: { trimEnd: { nodeId, param: "durationMs", unit: "ms" } },
      envelope: params.envelope,
    });
    return { image, clip };
  },
});

export const loginOnceNode = defineNode({
  type: "capture.loginOnce",
  title: "Sign in once",
  category: "Capture",
  icon: "key-round",
  phase: "capture",
  executes: true,
  volatile: true,
  description:
    "Opens a window where you sign in yourself. Demoflow keeps the sign-in, encrypted, for later takes.",
  inputs: {},
  outputs: {},
  params: z.object({
    url: text("{{Site address}}/login", "Sign-in page"),
    secretName: more(text("default", "Save as"), {
      help: "Pick this name under Signed in as on an Open browser node.",
    }),
    doneWhen: choice(["url-change", "selector", "window-closed"], "window-closed", "Finish when", {
      "url-change": "The address changes",
      selector: "An element appears",
      "window-closed": "I close the window",
    }),
    selector: more(text("", "Element that appears after signing in"), {
      help: "A CSS selector, e.g. .account-menu.",
      showIf: { doneWhen: ["selector"] },
    }),
    timeoutSec: z.number().int().min(10).max(3600).default(300).meta({
      description: "Give up after",
      unit: "s",
    }),
  }),
  card: ({ params: p, secrets }) => {
    const name = secretName(p.secretName);
    const saved = secrets?.includes(secretFileName(name));
    return [
      row("key-round", `Saves as ${quote(name, 24)}`),
      p.url.trim() && row("link", displayAddress(p.url, { keepQuery: true }), { title: p.url }),
      secrets &&
        row(saved ? "shield-check" : "shield-alert", saved ? "Signed in" : "Not signed in yet", {
          tone: saved ? undefined : "warn",
        }),
    ];
  },
});

export const captureNodes = [recordNode, stillNode, loginOnceNode];
