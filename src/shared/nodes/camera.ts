import { z } from "zod";
import { BACKDROPS } from "../../video/camera/lib/backdrop";
import {
  isClip,
  itemClip,
  patchRecordings,
  remapDuration,
  remapFromSegments,
  wrapClip,
} from "../compile/clip";
import {
  CAMERA_STYLES,
  DEFAULT_RECORDING_PROPS,
  followCursorBeats,
  idleSpans,
  type RecordingProps,
} from "../compile/recording";
import type { Action } from "../schema/actions";
import type { ClipValue, CompileValue, RecordingValue } from "../schema/values";
import { row } from "./card";
import { defineNode, port, type CompileOutputs } from "./define";
import { bool, choice, fraction, MAX_SPEED, more, ms, text } from "./params";
import { formatDuration, humanize } from "../format";
import { msToFrames } from "../timeline/time";
import { baseName } from "../asset-ref";

const base = { category: "Camera", phase: "compile" } as const;
const modifier = {
  inputs: { clip: port("clip", "Clip") },
  outputs: { clip: port("clip", "Clip") },
} as const;

const spot = { widget: "spot" } as const;

export const WINDOW_STYLES = ["macos", "windows", "linux"] as const;
const FRAME_LABELS = { none: "None", macos: "macOS", windows: "Windows", linux: "Linux" };

const clipIn = (inputs: Partial<Record<string, CompileValue>>): ClipValue | undefined =>
  isClip(inputs.clip) ? inputs.clip : undefined;

const passthrough = (
  inputs: Partial<Record<string, CompileValue>>,
  fn: (clip: ClipValue) => ClipValue,
): CompileOutputs => {
  const clip = clipIn(inputs);
  return clip ? { clip: fn(clip) } : {};
};

const appendBeats = (clip: ClipValue, beats: Action[]) =>
  patchRecordings(clip, (p) => ({
    ...p,
    extraBeats: [...((p.extraBeats as Action[]) ?? []), ...beats],
  }));

export const autoZoomNode = defineNode({
  ...base,
  type: "camera.autoZoom",
  title: "Auto zoom",
  icon: "scan-search",
  description:
    "Turns a recording into a polished clip: the window floats on a backdrop and the camera zooms to every click.",
  inputs: { recording: port("recording", "Recording") },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p }) => [
    p.zoom ? row("zoom-in", `Up to ${p.maxScale}×`) : row("zoom-out", "No zoom", { tone: "muted" }),
    p.zoom && row("hourglass", `Hold ${formatDuration(p.hold)}`),
    row("image", `${humanize(p.backdrop)} backdrop`),
    p.speed !== 1 && row("gauge", `${p.speed}× speed`),
  ],
  params: z.object({
    zoom: bool(true, "Zoom to the clicks"),
    maxScale: z.number().min(1.1).max(2.5).default(1.74).meta({
      description: "Maximum zoom",
      help: "Higher zooms closer but can look soft.",
      step: 0.02,
      unit: "×",
    }),
    hold: ms(1300, "Hold each zoom at least", 10_000),
    speed: z
      .number()
      .min(0.25)
      .max(4)
      .default(1.25)
      .meta({ description: "Playback speed", step: 0.05, unit: "×" }),
    style: choice(CAMERA_STYLES, "classic", "Camera style"),
    backdrop: z.enum(BACKDROPS).default("glaze").meta({ description: "Backdrop" }),
    frame: choice(["none", ...WINDOW_STYLES], "none", "Window frame", FRAME_LABELS),
    frameTheme: more(choice(["light", "dark"], "light", "Frame colour"), {
      showIf: { frame: [...WINDOW_STYLES] },
    }),
    addressBar: more(bool(true, "Address bar"), {
      help: "Shows the page's address in the title bar.",
      showIf: { frame: [...WINDOW_STYLES] },
    }),
    motionBlur: bool(true, "Motion blur"),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const rec = inputs.recording as RecordingValue | undefined;
    const durationMs = rec?.durationMs ?? 6000;
    const frames = msToFrames(durationMs / params.speed, settings.fps);
    if (!rec?.src || !rec.log) {
      return {
        clip: itemClip(
          nodeId,
          "core/placeholder",
          {
            title: "Not recorded yet",
            subtitle: "Run to record the take. The camera moves appear here afterwards.",
          },
          frames,
        ),
      };
    }
    const props: RecordingProps = {
      ...DEFAULT_RECORDING_PROPS,
      src: rec.src,
      ...(rec.proxySrc ? { proxySrc: rec.proxySrc } : {}),
      log: rec.log,
      name: rec.nodeId,
      speed: params.speed,
      zoom: params.zoom,
      tuning: { sMax: params.maxScale, holdMinS: params.hold / 1000 },
      motionBlur: params.motionBlur,
      style: params.style,
      chrome: params.frame !== "none",
      ...(params.frame !== "none"
        ? {
            windowStyle: params.frame,
            frameTheme: params.frameTheme,
            addressBar: params.addressBar,
          }
        : {}),
      backdrop: params.backdrop,
    };
    return { clip: itemClip(nodeId, "core/recording", props, frames, { recording: rec }) };
  },
});

export const manualZoomNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.manualZoom",
  title: "Zoom here",
  icon: "zoom-in",
  description: "Zooms to a spot of your choice at a set time.",
  card: ({ params: p }) => [
    row("zoom-in", `${p.scale}× at ${formatDuration(p.atMs)}`),
    row("hourglass", `Hold ${formatDuration(p.holdMs)}`),
  ],
  params: z.object({
    atMs: ms(1000, "At"),
    holdMs: ms(1500, "Hold"),
    x: more(fraction(0.5, "Spot"), { ...spot, help: "Click the frame to choose where to zoom." }),
    y: more(fraction(0.5, "Spot vertical"), { hidden: true }),
    scale: z
      .number()
      .min(1.1)
      .max(2.5)
      .default(1.5)
      .meta({ description: "Zoom", step: 0.05, unit: "×" }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    passthrough(inputs, (clip) => {
      const vp = clip.recording?.viewport ?? { width: 1920, height: 1080 };
      const point = { x: params.x * vp.width, y: params.y * vp.height };
      const rect = { x: point.x - 60, y: point.y - 24, w: 120, h: 48 };
      const common = {
        kind: "focus" as const,
        point,
        rect,
        cluster: `${nodeId}:zoom`,
        zoomScale: params.scale,
        nodeId,
      };
      return appendBeats(clip, [
        { ...common, id: `${nodeId}:a`, t: params.atMs, tDepart: Math.max(0, params.atMs - 400) },
        { ...common, id: `${nodeId}:b`, t: params.atMs + params.holdMs },
      ]);
    }),
});

export const panNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.pan",
  title: "Pan",
  icon: "move",
  description: "Moves the camera from one spot to another.",
  card: ({ params: p }) => [
    row("move", `${formatDuration(p.atMs)} → ${formatDuration(p.atMs + p.durationMs)}`),
    row("zoom-in", `${p.scale}×`),
  ],
  params: z.object({
    atMs: ms(1000, "Start at"),
    durationMs: ms(1500, "Pan for"),
    fromX: more(fraction(0.3, "From"), {
      ...spot,
      help: "Click the frame to choose where to start.",
    }),
    fromY: more(fraction(0.4, "From vertical"), { hidden: true }),
    toX: more(fraction(0.7, "To"), { ...spot, help: "Click the frame to choose where to end." }),
    toY: more(fraction(0.6, "To vertical"), { hidden: true }),
    scale: z
      .number()
      .min(1.1)
      .max(2.5)
      .default(1.4)
      .meta({ description: "Zoom", step: 0.05, unit: "×" }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    passthrough(inputs, (clip) => {
      const vp = clip.recording?.viewport ?? { width: 1920, height: 1080 };
      const beat = (id: string, t: number, fx: number, fy: number): Action => {
        const point = { x: fx * vp.width, y: fy * vp.height };
        return {
          id: `${nodeId}:${id}`,
          kind: "focus",
          t,
          point,
          rect: { x: point.x - 60, y: point.y - 24, w: 120, h: 48 },
          cluster: `${nodeId}:pan`,
          zoomScale: params.scale,
          nodeId,
        };
      };
      return appendBeats(clip, [
        beat("a", params.atMs, params.fromX, params.fromY),
        beat("b", params.atMs + params.durationMs, params.toX, params.toY),
      ]);
    }),
});

export const followCursorNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.followCursor",
  title: "Follow cursor",
  icon: "crosshair",
  description: "Keeps the camera on the cursor, good for tall formats.",
  card: ({ params: p }) => [
    row("locate-fixed", `${p.scale}× · re-aim every ${formatDuration(p.everyMs)}`),
  ],
  params: z.object({
    scale: z
      .number()
      .min(1.1)
      .max(2.5)
      .default(1.35)
      .meta({ description: "Zoom", step: 0.05, unit: "×" }),
    everyMs: ms(700, "Re-aim every", 5000),
  }),
  compile: ({ nodeId, inputs, params }) =>
    passthrough(inputs, (clip) =>
      clip.recording?.log
        ? appendBeats(
            clip,
            followCursorBeats(clip.recording.log, params.scale, params.everyMs, nodeId),
          )
        : clip,
    ),
});

function remapClip(
  clip: ClipValue,
  nodeId: string,
  fps: number,
  segmentsMs: Array<[number, number, number]>,
): ClipValue {
  const playback = clip.recording ? recordingSpeed(clip) : 1;
  const toFrames = (msv: number) => (msv / 1000 / playback) * fps;
  const remap = remapFromSegments(
    clip.durationInFrames,
    segmentsMs.map(([a, b, r]) => [toFrames(a), toFrames(b), r]),
  );
  return wrapClip(clip, nodeId, { remap }, Math.max(1, Math.round(remapDuration(remap))));
}

function recordingSpeed(clip: ClipValue): number {
  let speed = 1;
  patchRecordings(clip, (p) => {
    speed = (p.speed as number) ?? 1;
    return p;
  });
  return speed;
}

export const idleSpeedupNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.idleSpeedup",
  title: "Skip quiet parts",
  icon: "fast-forward",
  description: "Speeds through moments where nothing happens.",
  card: ({ params: p }) => [
    row("fast-forward", `${p.rate}× when quiet over ${formatDuration(p.thresholdMs)}`),
  ],
  params: z.object({
    thresholdMs: ms(1500, "Quiet for longer than", 20_000),
    padMs: ms(400, "Keep around each action", 5000),
    rate: z
      .number()
      .min(1.5)
      .max(MAX_SPEED)
      .default(4)
      .meta({ description: "Speed while quiet", step: 0.5, unit: "×" }),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    passthrough(inputs, (clip) => {
      const log = clip.recording?.log;
      if (!log) return clip;
      const spans = idleSpans(log, params.thresholdMs, params.padMs);
      return spans.length
        ? remapClip(
            clip,
            nodeId,
            settings.fps,
            spans.map(([a, b]) => [a, b, params.rate]),
          )
        : clip;
    }),
});

export const speedRampNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.speedRamp",
  title: "Speed ramp",
  icon: "gauge",
  description: "Speeds up or slows down chosen parts.",
  card: ({ params: p }) => [
    row(
      "gauge",
      p.segments.length === 1 ? "1 part" : `${p.segments.length} parts`,
      p.segments.length ? {} : { tone: "muted" },
    ),
  ],
  params: z.object({
    segments: z
      .array(
        z.object({
          fromMs: z
            .number()
            .int()
            .min(0)
            .default(0)
            .meta({ description: "From", unit: "ms", step: 50 }),
          toMs: z
            .number()
            .int()
            .min(0)
            .default(1000)
            .meta({ description: "To", unit: "ms", step: 50 }),
          rate: z
            .number()
            .min(0.1)
            .max(MAX_SPEED)
            .default(2)
            .meta({ description: "Speed", step: 0.1, unit: "×" }),
        }),
      )
      .default([{ fromMs: 0, toMs: 1000, rate: 2 }])
      .meta({ description: "Parts" }),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    passthrough(inputs, (clip) =>
      remapClip(
        clip,
        nodeId,
        settings.fps,
        [...params.segments]
          .sort((a, b) => a.fromMs - b.fromMs)
          .map((s) => [s.fromMs, s.toMs, s.rate]),
      ),
    ),
});

export const cropNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.crop",
  title: "Crop",
  icon: "crop",
  description: "Cuts the recording down to an area.",
  card: ({ params: p }) => [
    row(
      "crop",
      `${Math.round(p.w * 100)} % × ${Math.round(p.h * 100)} % from ${Math.round(p.x * 100)} %, ${Math.round(p.y * 100)} %`,
    ),
  ],
  params: z.object({
    x: fraction(0.1, "Left"),
    y: fraction(0.1, "Top"),
    w: fraction(0.8, "Width"),
    h: fraction(0.8, "Height"),
  }),
  compile: ({ nodeId, inputs, params }) =>
    passthrough(inputs, (clip) => wrapClip(clip, nodeId, { crop: params })),
});

export const deviceFrameNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.deviceFrame",
  title: "Device frame",
  icon: "laptop",
  description: "Puts the video inside a browser window, laptop or phone.",
  card: ({ params: p }) => [
    row(
      p.device === "phone" ? "smartphone" : p.device === "laptop" ? "laptop" : "app-window",
      humanize(p.device),
    ),
    p.device === "laptop" &&
      p.entrance !== "none" &&
      row("sparkles", p.entrance === "open" ? "Opens up" : "Slides in"),
  ],
  params: z.object({
    device: choice(["browser", "laptop", "phone"], "browser", "Device", {
      browser: "Browser window",
      laptop: "Laptop",
      phone: "Phone",
    }),
    windowStyle: more(choice(WINDOW_STYLES, "macos", "Window style", FRAME_LABELS), {
      showIf: { device: ["browser"] },
    }),
    frameTheme: more(choice(["light", "dark"], "light", "Frame colour"), {
      showIf: { device: ["browser"] },
    }),
    entrance: choice(["rise", "open", "none"], "open", "Entrance", {
      rise: "Slide in",
      open: "Open up",
      none: "None",
    }).meta({
      description: "Entrance",
      labels: { rise: "Slide in", open: "Open up", none: "None" },
      showIf: { device: ["laptop"] },
    }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    passthrough(inputs, (clip) => {
      if (params.device === "browser")
        return patchRecordings(clip, (p) => ({
          ...p,
          chrome: true,
          windowStyle: params.windowStyle,
          frameTheme: params.frameTheme,
          addressBar: p.addressBar ?? true,
        }));
      const src = clip.recording?.src;
      if (!src) return clip;
      const component = params.device === "laptop" ? "snapcn/laptop-frame" : "snapcn/phone-frame";
      return itemClip(
        nodeId,
        component,
        {
          mode: "dark",
          screenSrc: src,
          ...(params.device === "laptop" ? { entrance: params.entrance } : {}),
        },
        clip.durationInFrames,
        { recording: clip.recording, audio: clip.audio },
      );
    }),
});

export const backdropNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.backdrop",
  title: "Backdrop",
  icon: "image",
  description: "The background behind the floating window: a studio image, colour or gradient.",
  card: ({ params: p }) => [
    p.kind === "image"
      ? row("image", humanize(p.name))
      : p.kind === "solid"
        ? row("palette", p.color)
        : p.kind === "gradient"
          ? row("blend", "Gradient", { title: p.gradient })
          : row(
              "image",
              p.asset ? baseName(p.asset) : "Choose an image",
              p.asset ? {} : { tone: "warn" },
            ),
  ],
  params: z.object({
    kind: choice(["image", "solid", "gradient", "asset"], "image", "Kind", {
      image: "Studio image",
      solid: "Colour",
      gradient: "Gradient",
      asset: "Your image",
    }),
    name: z
      .enum(BACKDROPS)
      .default("glaze")
      .meta({ description: "Studio image", showIf: { kind: ["image"] } }),
    color: text("#0b0b10", "Colour").meta({
      description: "Colour",
      widget: "color",
      showIf: { kind: ["solid"] },
    }),
    gradient: text("linear-gradient(135deg, #1e1b4b, #0f172a)", "Gradient").meta({
      description: "Gradient",
      help: "A CSS gradient.",
      showIf: { kind: ["gradient"] },
    }),
    asset: text("", "Image").meta({
      description: "Image",
      widget: "asset",
      accept: "image/*",
      showIf: { kind: ["asset"] },
    }),
  }),
  compile: ({ inputs, params }) =>
    passthrough(inputs, (clip) =>
      patchRecordings(clip, (p) => ({
        ...p,
        backdrop:
          params.kind === "image"
            ? params.name
            : params.kind === "solid"
              ? params.color
              : params.kind === "gradient"
                ? params.gradient
                : params.asset || p.backdrop,
      })),
    ),
});

export const motionBlurNode = defineNode({
  ...base,
  ...modifier,
  type: "camera.motionBlur",
  title: "Motion blur",
  icon: "wind",
  description: "Adds blur to fast camera moves.",
  card: ({ params: p }) => [
    row("wind", p.enabled ? "On" : "Off", p.enabled ? {} : { tone: "muted" }),
  ],
  params: z.object({ enabled: bool(true, "Motion blur") }),
  compile: ({ inputs, params }) =>
    passthrough(inputs, (clip) =>
      patchRecordings(clip, (p) => ({ ...p, motionBlur: params.enabled })),
    ),
});

export const cameraNodes = [
  autoZoomNode,
  manualZoomNode,
  panNode,
  followCursorNode,
  idleSpeedupNode,
  speedRampNode,
  cropNode,
  deviceFrameNode,
  backdropNode,
  motionBlurNode,
];

export { passthrough };
