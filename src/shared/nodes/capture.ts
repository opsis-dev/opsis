import { z } from "zod";
import { DEFAULT_BACKDROP } from "../backdrops";
import { itemClip, placeholderClip, type ItemExtra } from "../compile/clip";
import { formatDuration, displayAddress } from "../format";
import type { Bounds } from "../schema/actions";
import { outputOf } from "../schema/values";
import { DEFAULT_TAKE_MS, DEFAULT_VIEWPORT } from "../schema/workflow";
import { secretFileName, secretName } from "../secrets";
import { msToFrames } from "../timeline/time";
import { elementRow, LABELLED_TEXT_MAX, quote, row, warnIf } from "./card";
import { defineNode, nodeBase, port } from "./define";
import {
  bool,
  choice,
  choiceOf,
  int,
  more,
  ms,
  optionalTarget,
  pickedRole,
  text,
  visualEnvelope,
} from "./params";

const CURSORS = {
  synthetic: { label: "Smooth", row: "Smooth cursor" },
  baked: { label: "As on screen", row: "Cursor as on screen" },
  none: { label: "None", row: "No cursor" },
};

const base = { ...nodeBase("Capture", "capture"), executes: true };

const REGION_LABELS = {
  viewport: "Visible part",
  "full-page": "Whole page",
  element: "One element",
};

export const recordNode = defineNode({
  ...base,
  type: "capture.record",
  title: "Record take",
  icon: "video",
  description: "Plays all connected browser actions in one go and records the screen.",
  inputs: { session: port("session", "Previous") },
  outputs: { recording: port("recording", "Recording") },
  params: z.object({
    fps: z
      .union([z.literal(25), z.literal(30), z.literal(60)])
      .default(30)
      .meta({ description: "Frame rate", unit: "fps" }),
    quality: int(90, "Recording quality", {
      min: 10,
      max: 100,
      step: 5,
      unit: "%",
      advanced: true,
    }),
    establishMs: ms(700, "Pause before the first action"),
    endTailMs: more(ms(2500, "Keep recording after the last action"), {
      help: "Gives the camera time to settle at the end.",
    }),
    cursor: choiceOf(CURSORS, "synthetic", "Cursor"),
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
      row("mouse-pointer", CURSORS[p.cursor].row),
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

type ShotSize = { width?: number; height?: number; scale?: number; body?: Bounds };

function shotOf(image: ShotSize) {
  if (!image.width || !image.height) return undefined;
  return { width: image.width, height: image.height, scale: image.scale ?? 1, body: image.body };
}

export const stillNode = defineNode({
  ...base,
  type: "capture.still",
  title: "Screenshot",
  icon: "camera",
  description:
    "Plays the connected actions, then takes a sharp screenshot of the page or one element.",
  inputs: { session: port("session", "Previous") },
  outputs: { image: port("image", "Image"), clip: port("clip", "Clip") },
  params: z.object({
    region: choiceOf(REGION_LABELS, "viewport", "Capture"),
    target: more(optionalTarget("Element"), { showIf: { region: ["element"] } }),
    targetRole: pickedRole(),
    padding: int(24, "Space around", {
      min: 0,
      max: 400,
      unit: "px",
      showIf: { region: ["element"] },
    }),
    look: choice(["window", "cutout", "bleed"], "window", "Style", {
      window: "Window",
      cutout: "Cut-out",
      bleed: "Full bleed",
    }),
    backdrop: more(text(DEFAULT_BACKDROP, "Backdrop"), { widget: "backdrop" }),
    durationMs: ms(3000, "Length"),
    envelope: visualEnvelope(),
  }),
  card: ({ params: p, result }) => {
    const image = outputOf(result, "image", "image");
    const shot = image && shotOf(image);
    return [
      p.region === "element"
        ? elementRow("scan", p.target, p.targetRole)
        : row("scan", REGION_LABELS[p.region]),
      shot && row("image", `${Math.round(shot.width)} × ${Math.round(shot.height)}`),
      row("clock", formatDuration(p.durationMs)),
    ];
  },
  compile: ({ nodeId, result, params, settings }) => {
    const image = outputOf(result, "image", "image") ?? { type: "image", src: null };
    const frames = msToFrames(params.durationMs, settings.fps);
    const extra: ItemExtra = {
      handles: { trimEnd: { nodeId, param: "durationMs", unit: "ms" } },
      envelope: params.envelope,
    };
    const clip = image.src
      ? itemClip(
          nodeId,
          "core/still",
          {
            src: image.src,
            look: params.look,
            backdrop: params.backdrop,
            shot: shotOf(image),
          },
          frames,
          extra,
        )
      : placeholderClip(
          nodeId,
          { title: "Screenshot not taken yet", subtitle: "Run to take it" },
          frames,
          extra,
        );
    return { image, clip };
  },
});

export const loginOnceNode = defineNode({
  ...base,
  type: "capture.loginOnce",
  title: "Sign in once",
  icon: "key-round",
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
    timeoutSec: int(300, "Give up after", { min: 10, max: 3600, unit: "s" }),
  }),
  card: ({ params: p, secrets }) => {
    const name = secretName(p.secretName);
    const saved = secrets?.includes(secretFileName(name));
    return [
      row("key-round", `Saves as ${quote(name, LABELLED_TEXT_MAX)}`),
      p.url.trim() && row("link", displayAddress(p.url, { keepQuery: true }), { title: p.url }),
      secrets &&
        row(
          saved ? "shield-check" : "shield-alert",
          saved ? "Signed in" : "Not signed in yet",
          warnIf(!saved),
        ),
    ];
  },
});

export const captureNodes = [recordNode, stillNode, loginOnceNode];
