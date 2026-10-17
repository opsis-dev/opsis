import { z } from "zod";
import { BACKDROPS, DEFAULT_BACKDROP } from "../backdrops";
import {
  itemClip,
  patchRecordings,
  placeholderClip,
  recordingClip,
  recordingProps,
  wrapClip,
} from "../compile/clip";
import { remapDuration, remapFromSegments } from "../compile/remap";
import {
  CAMERA_STYLES,
  DEFAULT_RECORDING_PROPS,
  followCursorBeats,
  idleSpans,
  type RecordingProps,
} from "../compile/recording";
import { pointRect, type Action } from "../schema/actions";
import { asValue, type ClipValue } from "../schema/values";
import { mutedIf, row, toggleRow, warnIf, type CardRow } from "./card";
import { clipModifier, defineNode, port, withClip } from "./define";
import type { NodeIconName } from "./icon-name";
import {
  asset,
  bool,
  choice,
  fraction,
  keysOf,
  MIN_SPEED,
  more,
  ms,
  rate,
  text,
  zoomScale,
} from "./params";
import { DEFAULT_TAKE_MS, DEFAULT_VIEWPORT } from "./capture";
import { formatDuration, humanize, percentText, plural } from "../format";
import { msToFrames } from "../timeline/time";
import { baseName } from "../asset-ref";

const base = { category: "Camera", phase: "compile" } as const;
const modifier = { ...base, ...clipModifier };

const spot = { widget: "spot", timeKey: "atMs" } as const;

const WINDOW_STYLES = ["macos", "windows", "linux"] as const;
type WindowStyle = (typeof WINDOW_STYLES)[number];
const FRAME_LABELS = { none: "None", macos: "macOS", windows: "Windows", linux: "Linux" };

const FRAME_THEMES = ["light", "dark"] as const;
const frameThemeField = (showIf: Record<string, string[]>) =>
  more(choice(FRAME_THEMES, "light", "Frame colour"), { showIf });

const windowFrameProps = (
  windowStyle: WindowStyle,
  frameTheme: (typeof FRAME_THEMES)[number],
  addressBar: boolean,
): Pick<RecordingProps, "chrome" | "windowStyle" | "frameTheme" | "addressBar"> => ({
  chrome: true,
  windowStyle,
  frameTheme,
  addressBar,
});

const appendBeats = (clip: ClipValue, beats: Action[]) =>
  patchRecordings(clip, (p) => ({ ...p, extraBeats: [...p.extraBeats, ...beats] }));

const FOCUS_LEAD_MS = 400;

type FocusSpot = { id: string; t: number; x: number; y: number; tDepart?: number };

const focusBeats = (
  clip: ClipValue,
  nodeId: string,
  cluster: string,
  scale: number,
  spots: FocusSpot[],
): ClipValue => {
  const vp = clip.recording?.viewport ?? DEFAULT_VIEWPORT;
  return appendBeats(
    clip,
    spots.map(({ id, x, y, ...timing }): Action => {
      const point = { x: x * vp.width, y: y * vp.height };
      return {
        ...timing,
        id: `${nodeId}:${id}`,
        kind: "focus",
        point,
        rect: pointRect(point),
        cluster: `${nodeId}:${cluster}`,
        zoomScale: scale,
        nodeId,
      };
    }),
  );
};

const autoZoomNode = defineNode({
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
    maxScale: more(zoomScale(1.74, "Maximum zoom", 0.02), {
      help: "Higher zooms closer but can look soft.",
    }),
    hold: ms(1300, "Hold each zoom at least", 10_000),
    speed: rate(1.25, "Playback speed", { min: 0.25, max: 4 }),
    style: choice(CAMERA_STYLES, "classic", "Camera style"),
    backdrop: choice(BACKDROPS, DEFAULT_BACKDROP, "Backdrop"),
    frame: choice(["none", ...WINDOW_STYLES], "none", "Window frame", FRAME_LABELS),
    frameTheme: frameThemeField({ frame: [...WINDOW_STYLES] }),
    addressBar: more(bool(true, "Address bar"), {
      help: "Shows the page's address in the title bar.",
      showIf: { frame: [...WINDOW_STYLES] },
    }),
    motionBlur: bool(true, "Motion blur"),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const rec = asValue(inputs.recording, "recording");
    const durationMs = rec?.durationMs ?? DEFAULT_TAKE_MS;
    const frames = msToFrames(durationMs / params.speed, settings.fps);
    if (!rec?.src || !rec.log) {
      return {
        clip: placeholderClip(
          nodeId,
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
      ...(params.frame === "none"
        ? { chrome: false }
        : windowFrameProps(params.frame, params.frameTheme, params.addressBar)),
      backdrop: params.backdrop,
    };
    return { clip: recordingClip(nodeId, props, frames, { recording: rec }) };
  },
});

const manualZoomNode = defineNode({
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
    scale: zoomScale(1.5),
  }),
  compile: ({ nodeId, inputs, params: p }) =>
    withClip(inputs, (clip) =>
      focusBeats(clip, nodeId, "zoom", p.scale, [
        { id: "a", t: p.atMs, tDepart: Math.max(0, p.atMs - FOCUS_LEAD_MS), x: p.x, y: p.y },
        { id: "b", t: p.atMs + p.holdMs, x: p.x, y: p.y },
      ]),
    ),
});

const panNode = defineNode({
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
    scale: zoomScale(1.4),
  }),
  compile: ({ nodeId, inputs, params: p }) =>
    withClip(inputs, (clip) =>
      focusBeats(clip, nodeId, "pan", p.scale, [
        { id: "a", t: p.atMs, x: p.fromX, y: p.fromY },
        { id: "b", t: p.atMs + p.durationMs, x: p.toX, y: p.toY },
      ]),
    ),
});

const followCursorNode = defineNode({
  ...modifier,
  type: "camera.followCursor",
  title: "Follow cursor",
  icon: "crosshair",
  description: "Keeps the camera on the cursor, good for tall formats.",
  card: ({ params: p }) => [
    row("locate-fixed", `${p.scale}× · re-aim every ${formatDuration(p.everyMs)}`),
  ],
  params: z.object({
    scale: zoomScale(1.35),
    everyMs: ms(700, "Re-aim every", 5000),
  }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) =>
      clip.recording?.log
        ? appendBeats(
            clip,
            followCursorBeats(clip.recording.log, params.scale, params.everyMs, nodeId),
          )
        : clip,
    ),
});

const lastRecordingSpeed = (clip: ClipValue): number => recordingProps(clip).at(-1)?.speed ?? 1;

function remapClip(
  clip: ClipValue,
  nodeId: string,
  fps: number,
  segmentsMs: Array<[number, number, number]>,
): ClipValue {
  const playback = clip.recording ? lastRecordingSpeed(clip) : 1;
  const toFrames = (msv: number) => (msv / 1000 / playback) * fps;
  const remap = remapFromSegments(
    clip.durationInFrames,
    segmentsMs.map(([a, b, r]) => [toFrames(a), toFrames(b), r]),
  );
  return wrapClip(clip, nodeId, { remap }, Math.max(1, Math.round(remapDuration(remap))));
}

const idleSpeedupNode = defineNode({
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
    rate: rate(4, "Speed while quiet", { min: 1.5, step: 0.5 }),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    withClip(inputs, (clip) => {
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

const speedRampNode = defineNode({
  ...modifier,
  type: "camera.speedRamp",
  title: "Speed ramp",
  icon: "gauge",
  description: "Speeds up or slows down chosen parts.",
  card: ({ params: p }) => [
    row("gauge", plural(p.segments.length, "part"), mutedIf(!p.segments.length)),
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
          rate: rate(2, "Speed", { min: MIN_SPEED, step: 0.1 }),
        }),
      )
      .default([{ fromMs: 0, toMs: 1000, rate: 2 }])
      .meta({ description: "Parts" }),
  }),
  compile: ({ nodeId, inputs, params, settings }) =>
    withClip(inputs, (clip) =>
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

const cropNode = defineNode({
  ...modifier,
  type: "camera.crop",
  title: "Crop",
  icon: "crop",
  description: "Cuts the recording down to an area.",
  card: ({ params: p }) => [
    row(
      "crop",
      `${percentText(p.w)} × ${percentText(p.h)} from ${percentText(p.x)}, ${percentText(p.y)}`,
    ),
  ],
  params: z.object({
    x: fraction(0.1, "Left"),
    y: fraction(0.1, "Top"),
    w: fraction(0.8, "Width"),
    h: fraction(0.8, "Height"),
  }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) => wrapClip(clip, nodeId, { crop: params })),
});

const DEVICE_LABELS = { browser: "Browser window", laptop: "Laptop", phone: "Phone" };

const DEVICE_ICONS: Record<keyof typeof DEVICE_LABELS, NodeIconName> = {
  browser: "app-window",
  laptop: "laptop",
  phone: "smartphone",
};

const HOST_WINDOW_STYLES: Partial<Record<NodeJS.Platform, WindowStyle>> = {
  win32: "windows",
  linux: "linux",
};

const deviceFrameNode = defineNode({
  ...modifier,
  type: "camera.deviceFrame",
  title: "Device frame",
  icon: "laptop",
  description: "Puts the video inside a browser window, laptop or phone.",
  card: ({ params: p }) => [
    row(DEVICE_ICONS[p.device], humanize(p.device)),
    p.device === "laptop" &&
      p.entrance !== "none" &&
      row("sparkles", p.entrance === "open" ? "Opens up" : "Slides in"),
  ],
  params: z.object({
    device: choice(keysOf(DEVICE_LABELS), "browser", "Device", DEVICE_LABELS),
    windowStyle: more(choice(WINDOW_STYLES, "macos", "Window style", FRAME_LABELS), {
      showIf: { device: ["browser"] },
    }),
    frameTheme: frameThemeField({ device: ["browser"] }),
    entrance: more(
      choice(["rise", "open", "none"], "open", "Entrance", {
        rise: "Slide in",
        open: "Open up",
        none: "None",
      }),
      { showIf: { device: ["laptop"] } },
    ),
  }),
  hostDefaults: (platform) => ({ windowStyle: HOST_WINDOW_STYLES[platform] ?? "macos" }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) => {
      if (params.device === "browser")
        return patchRecordings(clip, (p) => ({
          ...p,
          ...windowFrameProps(params.windowStyle, params.frameTheme, p.addressBar ?? true),
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

const backdropParams = z.object({
  kind: choice(["image", "solid", "gradient", "asset"], "image", "Kind", {
    image: "Studio image",
    solid: "Colour",
    gradient: "Gradient",
    asset: "Your image",
  }),
  name: z
    .enum(BACKDROPS)
    .default(DEFAULT_BACKDROP)
    .meta({ description: "Studio image", showIf: { kind: ["image"] } }),
  color: more(text("#0b0b10", "Colour"), { widget: "color", showIf: { kind: ["solid"] } }),
  gradient: more(text("linear-gradient(135deg, #1e1b4b, #0f172a)", "Gradient"), {
    help: "A CSS gradient.",
    showIf: { kind: ["gradient"] },
  }),
  asset: more(asset("Image", "image/*"), { showIf: { kind: ["asset"] } }),
});

type BackdropParams = z.infer<typeof backdropParams>;

function backdropOf(p: BackdropParams): { row: CardRow; value: string | undefined } {
  switch (p.kind) {
    case "image":
      return { row: row("image", humanize(p.name)), value: p.name };
    case "solid":
      return { row: row("palette", p.color), value: p.color };
    case "gradient":
      return { row: row("blend", "Gradient", { title: p.gradient }), value: p.gradient };
    case "asset":
      return {
        row: row("image", p.asset ? baseName(p.asset) : "Choose an image", warnIf(!p.asset)),
        value: p.asset || undefined,
      };
  }
}

const backdropNode = defineNode({
  ...modifier,
  type: "camera.backdrop",
  title: "Backdrop",
  icon: "image",
  description: "The background behind the floating window: a studio image, colour or gradient.",
  card: ({ params: p }) => [backdropOf(p).row],
  params: backdropParams,
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
      patchRecordings(clip, (p) => ({ ...p, backdrop: backdropOf(params).value ?? p.backdrop })),
    ),
});

const motionBlurNode = defineNode({
  ...modifier,
  type: "camera.motionBlur",
  title: "Motion blur",
  icon: "wind",
  description: "Adds blur to fast camera moves.",
  card: ({ params: p }) => [toggleRow("wind", p.enabled)],
  params: z.object({ enabled: bool(true, "Motion blur") }),
  compile: ({ inputs, params }) =>
    withClip(inputs, (clip) =>
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
