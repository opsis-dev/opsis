import { z } from "zod";
import { DEFAULT_BACKDROP } from "#core/backdrops";
import { itemClip, type ItemExtra } from "#core/compile/clip";
import { formatDuration, plural } from "#core/format";
import { sessionPort, startRecordingNode } from "#core/nodes/browser";
import { elementRow, row } from "#core/nodes/card";
import { defineNode, nodeBase, port } from "#core/nodes/define";
import {
  bool,
  choice,
  choiceOf,
  int,
  more,
  ms,
  msAtLeast,
  optionalTarget,
  pickedRole,
  text,
  visualEnvelope,
} from "#core/nodes/params";
import { SESSION_PORT } from "#core/nodes/ports";
import type { Bounds } from "#core/schema/actions";
import { outputOf } from "#core/schema/values";
import { DEFAULT_STILL_MS, DEFAULT_TAKE_MS, DEFAULT_VIEWPORT } from "#core/schema/workflow";
import { msToFrames } from "#core/timeline/time";

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
  description: "Plays all connected browser actions in one go and records the screen.",
  inputs: { [SESSION_PORT]: sessionPort() },
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
  card: ({ params: p, result, inputSteps }) => {
    const rec = outputOf(result, "recording", "recording");
    const steps = inputSteps?.[SESSION_PORT];
    return [
      row("video", `${p.fps} fps`),
      row("mouse-pointer", CURSORS[p.cursor].row),
      steps &&
        !steps.includes(startRecordingNode.type) &&
        row("clapperboard", "Starts with the first action", { tone: "muted" }),
      rec?.src &&
        rec.log &&
        row(
          "clock",
          `${formatDuration(rec.durationMs)} · ${plural(rec.log.actions.length, "action")}`,
        ),
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
      style: [],
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
  usesProjectSize: true,
  description:
    "Plays the connected actions, then takes a sharp screenshot of the page or one element.",
  inputs: { [SESSION_PORT]: sessionPort() },
  outputs: { image: port("image", "Image"), clip: port("clip", "Clip") },
  params: z
    .object({
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
      durationMs: msAtLeast(DEFAULT_STILL_MS, "Length", 100),
      envelope: visualEnvelope(),
    })
    .superRefine((p, ctx) => {
      if (p.region === "element" && !p.target.trim())
        ctx.addIssue({ code: "custom", message: "Pick an element", path: ["target"] });
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
      : itemClip(
          nodeId,
          "core/placeholder",
          { title: "Screenshot not taken yet", subtitle: "Run to take it" },
          frames,
          extra,
        );
    return { image, clip };
  },
});

export const captureNodes = [recordNode, stillNode];
