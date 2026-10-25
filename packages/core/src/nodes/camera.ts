import { z } from "zod";
import { baseName } from "#core/asset-ref";
import { BACKDROPS, DEFAULT_BACKDROP } from "#core/backdrops";
import { parseColor } from "#core/color";
import {
  clipFrameOfRecording,
  itemClip,
  patchRecordings,
  recordingClip,
  wrapClip,
} from "#core/compile/clip";
import {
  defaultRecordingProps,
  followCursorBeats,
  idleSpans,
  recordingSpeed,
} from "#core/compile/recording";
import { remapDuration, remapFromSegments } from "#core/compile/remap";
import {
  restyleWith,
  retimeWith,
  styleStep,
  type StyleSteps,
  type StyleTable,
} from "#core/compile/style";
import type { FieldMeta } from "#core/elements/introspect";
import { formatDuration, humanize, percentText, plural } from "#core/format";
import { MIN_CROP, MIN_SPEED } from "#core/limits";
import { EPSILON } from "#core/math";
import { missing, mutedIf, row, toggleRow, type CardRow, type InputTake } from "#core/nodes/card";
import { CURSOR_STYLE_TYPE, cursorStyles } from "#core/nodes/cursor";
import {
  CLIP_PORT,
  defineNode,
  modifierBase,
  nodeBase,
  opt,
  port,
  recordingModifier,
  RECORDING_PORT,
  STYLE_PORT,
  styleModifier,
  withClip,
} from "#core/nodes/define";
import { effectStyles } from "#core/nodes/effects";
import type { NodeIconName } from "#core/nodes/icon-name";
import {
  asset,
  bool,
  choice,
  choiceOf,
  fraction,
  more,
  ms,
  msAtLeast,
  num,
  rate,
  text,
  zoomFactor,
} from "#core/nodes/params";
import { textStyles } from "#core/nodes/text";
import { pointRect, type Action } from "#core/schema/actions";
import { CAMERA_STYLES, type RecordingProps } from "#core/schema/recording";
import { asValue, type ClipValue } from "#core/schema/values";
import { DEFAULT_TAKE_MS } from "#core/schema/workflow";
import { msToFrames, recordingMsToFrames, wholeFrames } from "#core/timeline/time";
import {
  DEFAULT_ADDRESS_BAR,
  DEFAULT_FRAME_THEME,
  DEFAULT_WINDOW_STYLE,
  FRAME_THEMES,
  hostWindowStyle,
  WINDOW_STYLE_LABELS,
  WINDOW_STYLES,
  type FrameTheme,
  type WindowStyle,
} from "#core/window-styles";

const base = nodeBase("Camera", "compile");
const modifier = modifierBase("Camera");
const styleNode = styleModifier("Camera");
const takeNode = recordingModifier("Camera");

const spotFields = (x: number, y: number, label: string, help: string) => ({
  x: more(fraction(x, label), { widget: "spot", timeKey: "atMs", help }),
  y: more(fraction(y, `${label} vertical`), { hidden: true }),
});

const zoomSpot = spotFields(0.5, 0.5, "Spot", "Click the frame to choose where to zoom.");
const panFrom = spotFields(0.3, 0.4, "From", "Click the frame to choose where to start.");
const panTo = spotFields(0.7, 0.6, "To", "Click the frame to choose where to end.");

const FRAME_LABELS = { none: "None", ...WINDOW_STYLE_LABELS };

const frameThemeField = (showIf: NonNullable<FieldMeta["showIf"]>) =>
  more(choice(FRAME_THEMES, DEFAULT_FRAME_THEME, "Frame colour"), { showIf });

const windowFrameProps = (
  windowStyle: WindowStyle,
  frameTheme: FrameTheme,
  addressBar: boolean,
): Pick<RecordingProps, "chrome" | "windowStyle" | "frameTheme" | "addressBar"> => ({
  chrome: true,
  windowStyle,
  frameTheme,
  addressBar,
});

const appendBeats = (props: RecordingProps, beats: Action[]): RecordingProps => ({
  ...props,
  extraBeats: [...props.extraBeats, ...beats],
});

const FOCUS_LEAD_MS = 400;

function takeRow(take: InputTake | undefined, atMs: number): CardRow | null {
  if (take && atMs >= take.durationMs)
    return row("triangle-alert", `After the take ends at ${formatDuration(take.durationMs)}`, {
      tone: "warn",
    });
  return null;
}

type FocusSpot = { id: string; t: number; x: number; y: number; tDepart?: number };

const focusBeats = (
  props: RecordingProps,
  nodeId: string,
  cluster: string,
  scale: number,
  spots: FocusSpot[],
): RecordingProps => {
  const vp = props.log.viewport;
  return appendBeats(
    props,
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
  description:
    "Turns a recording into a polished clip: the window floats on a backdrop and the camera zooms to every click.",
  inputs: {
    [RECORDING_PORT]: port("recording", "Recording"),
    [STYLE_PORT]: opt("recording-style", "Style"),
  },
  outputs: { clip: port("clip", "Clip") },
  card: ({ params: p, inputTakes, inputStyle }) => [
    inputTakes?.[RECORDING_PORT]?.cursorTrack === false &&
      inputStyle?.[STYLE_PORT]?.includes(followCursorNode.type) &&
      row("triangle-alert", "The take has no cursor to follow", {
        tone: "warn",
        title: "Set Cursor to Smooth on the Record take node.",
      }),
    inputTakes?.[RECORDING_PORT]?.cursorTrack === false &&
      inputStyle?.[STYLE_PORT]?.includes(CURSOR_STYLE_TYPE) &&
      row("triangle-alert", "The take films its own cursor", {
        tone: "warn",
        title:
          "The Cursor node's look, size and colours apply only to a drawn cursor. Set Cursor to Smooth on the Record take node.",
      }),
    p.zoom ? row("zoom-in", `Up to ${p.maxScale}×`) : row("zoom-out", "No zoom", { tone: "muted" }),
    p.zoom && row("hourglass", `Hold ${formatDuration(p.hold)}`),
    p.style === "classic" && row("image", `${humanize(p.backdrop)} backdrop`),
    p.speed !== 1 && row("gauge", `${p.speed}× speed`),
  ],
  params: z.object({
    zoom: bool(true, "Zoom to the clicks"),
    maxScale: more(zoomFactor(1.74, "Maximum zoom", 0.02), {
      help: "Higher zooms closer but can look soft.",
    }),
    hold: ms(1300, "Hold each zoom at least", 10_000),
    speed: rate(1.25, "Playback speed", { min: 0.25, max: 4 }),
    style: choice(CAMERA_STYLES, "classic", "Camera style"),
    backdrop: more(choice(BACKDROPS, DEFAULT_BACKDROP, "Backdrop"), {
      showIf: { style: ["classic"] },
    }),
    frame: choice(["none", ...WINDOW_STYLES], "none", "Window frame", FRAME_LABELS),
    frameTheme: frameThemeField({ frame: [...WINDOW_STYLES] }),
    addressBar: more(bool(DEFAULT_ADDRESS_BAR, "Address bar"), {
      help: "Shows the page's address in the title bar.",
      showIf: { frame: [...WINDOW_STYLES] },
    }),
    motionBlur: bool(true, "Motion blur"),
  }),
  compile: ({ nodeId, inputs, params, settings }) => {
    const rec = asValue(inputs[RECORDING_PORT], "recording");
    const entries = [
      ...(asValue(inputs[STYLE_PORT], "recording-style")?.entries ?? []),
      ...(rec?.style ?? []),
    ];
    const durationMs = rec?.durationMs ?? DEFAULT_TAKE_MS;
    const frames = msToFrames(durationMs / params.speed, settings.fps);
    if (!rec?.src || !rec.log) {
      const placeholder = itemClip(
        nodeId,
        "core/placeholder",
        {
          title: "Not recorded yet",
          subtitle: "Run to record the take. The camera moves appear here afterwards.",
        },
        frames,
      );
      return { clip: retimeWith(STYLE_TABLE, placeholder, entries, settings.fps) };
    }
    const own: RecordingProps = {
      ...defaultRecordingProps(),
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
    const props = restyleWith(STYLE_TABLE, own, entries);
    const clip = recordingClip(nodeId, props, frames, { recording: rec });
    return { clip: retimeWith(STYLE_TABLE, clip, entries, settings.fps) };
  },
});

const manualZoomNode = defineNode({
  ...takeNode,
  type: "camera.manualZoom",
  title: "Zoom here",
  description: "Zooms to a spot of your choice at a set time.",
  card: ({ params: p, inputTakes }) => [
    row("zoom-in", `${p.scale}× at ${formatDuration(p.atMs)}`),
    row("hourglass", `Hold ${formatDuration(p.holdMs)}`),
    takeRow(inputTakes?.[RECORDING_PORT], p.atMs),
  ],
  params: z.object({
    atMs: ms(1000, "At"),
    holdMs: msAtLeast(1500, "Hold", 100),
    ...zoomSpot,
    scale: zoomFactor(1.5),
  }),
});

const panNode = defineNode({
  ...takeNode,
  type: "camera.pan",
  title: "Pan",
  description: "Moves the camera from one spot to another.",
  card: ({ params: p, inputTakes }) => [
    row("move", `${formatDuration(p.atMs)} → ${formatDuration(p.atMs + p.durationMs)}`),
    row("zoom-in", `${p.scale}×`),
    takeRow(inputTakes?.[RECORDING_PORT], p.atMs),
  ],
  params: z.object({
    atMs: ms(1000, "Start at"),
    durationMs: msAtLeast(1500, "Pan for", 100),
    fromX: panFrom.x,
    fromY: panFrom.y,
    toX: panTo.x,
    toY: panTo.y,
    scale: zoomFactor(1.4),
  }),
});

const followCursorNode = defineNode({
  ...styleNode,
  type: "camera.followCursor",
  title: "Follow cursor",
  description: "Keeps the camera on the cursor, good for tall formats.",
  card: ({ params: p }) => [
    row("locate-fixed", `${p.scale}× · re-aim every ${formatDuration(p.everyMs)}`),
  ],
  params: z.object({
    scale: zoomFactor(1.35),
    everyMs: msAtLeast(700, "Re-aim every", 200, 5000),
  }),
});

const speedSegmentSchema = z.object({
  fromMs: ms(0, "From", Number.MAX_SAFE_INTEGER),
  toMs: ms(1000, "To", Number.MAX_SAFE_INTEGER),
  rate: rate(2, "Speed", { min: MIN_SPEED, step: 0.1 }),
});

type SpeedSegment = z.infer<typeof speedSegmentSchema>;

function remapClip(
  clip: ClipValue,
  nodeId: string,
  fps: number,
  segments: SpeedSegment[],
): ClipValue {
  const toFrames = (recordingMs: number) =>
    clipFrameOfRecording(clip.tracks, (rec) =>
      recordingMsToFrames(recordingMs, recordingSpeed(rec), fps),
    ) ?? recordingMsToFrames(recordingMs, 1, fps);
  const remap = remapFromSegments(
    clip.durationInFrames,
    segments.map((s) => ({ from: toFrames(s.fromMs), to: toFrames(s.toMs), rate: s.rate })),
  );
  return wrapClip(clip, nodeId, { remap }, wholeFrames(remapDuration(remap), 1));
}

const idleSpeedupNode = defineNode({
  ...styleNode,
  type: "camera.idleSpeedup",
  title: "Skip quiet parts",
  description: "Speeds through moments where nothing happens.",
  card: ({ params: p }) => [
    row("fast-forward", `${p.rate}× when quiet over ${formatDuration(p.thresholdMs)}`),
  ],
  params: z.object({
    thresholdMs: ms(1500, "Quiet for longer than", 20_000),
    padMs: ms(400, "Keep around each action", 5000),
    rate: rate(4, "Speed while quiet", { min: 1.5, step: 0.5 }),
  }),
});

const speedRampNode = defineNode({
  ...takeNode,
  type: "camera.speedRamp",
  title: "Speed ramp",
  description: "Speeds up or slows down chosen parts.",
  card: ({ params: p }) => [
    row("gauge", plural(p.segments.length, "part"), mutedIf(!p.segments.length)),
  ],
  params: z.object({
    segments: z
      .array(speedSegmentSchema)
      .default([{ fromMs: 0, toMs: 1000, rate: 2 }])
      .meta({ description: "Parts" }),
  }),
});

const cropNode = defineNode({
  ...modifier,
  type: "camera.crop",
  title: "Crop",
  description: "Cuts the recording down to an area.",
  card: ({ params: p }) => [
    row(
      "crop",
      `${percentText(p.w)} × ${percentText(p.h)} from ${percentText(p.x)}, ${percentText(p.y)}`,
    ),
  ],
  params: z
    .object({
      x: fraction(0.1, "Left"),
      y: fraction(0.1, "Top"),
      w: num(0.8, "Width", { min: MIN_CROP, max: 1, step: 0.01 }),
      h: num(0.8, "Height", { min: MIN_CROP, max: 1, step: 0.01 }),
    })
    .superRefine((p, ctx) => {
      if (p.x + p.w > 1 + EPSILON)
        ctx.addIssue({ code: "custom", message: "The area runs past the right edge", path: ["w"] });
      if (p.y + p.h > 1 + EPSILON)
        ctx.addIssue({
          code: "custom",
          message: "The area runs past the bottom edge",
          path: ["h"],
        });
    }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) => wrapClip(clip, nodeId, { crop: params })),
});

type Device = { label: string; icon: NodeIconName; component: string };

const DEVICES = {
  laptop: { label: "Laptop", icon: "laptop", component: "snapcn/laptop-frame" },
  phone: { label: "Phone", icon: "smartphone", component: "snapcn/phone-frame" },
} satisfies Record<string, Device>;

const ENTRANCES = { rise: "Slide in", open: "Open up", none: "None" };

const deviceFrameNode = defineNode({
  ...modifier,
  type: "camera.deviceFrame",
  title: "Device frame",
  description: "Puts the video inside a laptop or phone.",
  card: ({ params: p }) => [
    row(DEVICES[p.device].icon, DEVICES[p.device].label),
    p.device === "laptop" && p.entrance !== "none" && row("sparkles", ENTRANCES[p.entrance]),
  ],
  params: z.object({
    device: choiceOf(DEVICES, "laptop", "Device"),
    entrance: more(choiceOf(ENTRANCES, "open", "Entrance"), { showIf: { device: ["laptop"] } }),
  }),
  compile: ({ nodeId, inputs, params }) =>
    withClip(inputs, (clip) => {
      const laptop = params.device === "laptop";
      return wrapClip(
        patchRecordings(clip, (p) => ({ ...p, screen: true })),
        nodeId,
        {
          wrapper: {
            component: DEVICES[params.device].component,
            props: { mode: "dark", ...(laptop ? { entrance: params.entrance } : {}) },
          },
        },
        clip.durationInFrames,
      );
    }),
});

const backdropParams = z
  .object({
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
    gradient: more(text("linear-gradient(135deg, #1e1b4b 0%, #0f172a 100%)", "Gradient"), {
      widget: "gradient",
      showIf: { kind: ["gradient"] },
    }),
    asset: more(asset("Image", "image/*"), { showIf: { kind: ["asset"] } }),
  })
  .superRefine((p, ctx) => {
    const missingWhat =
      (p.kind === "solid" &&
        !parseColor(p.color.trim()) &&
        (["color", "Pick a colour"] as const)) ||
      (p.kind === "gradient" && !p.gradient.trim() && (["gradient", "Pick a gradient"] as const)) ||
      (p.kind === "asset" && !p.asset.trim() && (["asset", "Choose an image"] as const));
    if (missingWhat)
      ctx.addIssue({ code: "custom", message: missingWhat[1], path: [missingWhat[0]] });
  });

type BackdropParams = z.infer<typeof backdropParams>;

type BackdropKind = {
  row: (p: BackdropParams) => CardRow;
  override: (p: BackdropParams) => string | undefined;
};

const BACKDROP_KINDS: Record<BackdropParams["kind"], BackdropKind> = {
  image: { row: (p) => row("image", humanize(p.name)), override: (p) => p.name },
  solid: { row: (p) => row("palette", p.color), override: (p) => p.color },
  gradient: {
    row: (p) => row("blend", "Gradient", { title: p.gradient }),
    override: (p) => p.gradient,
  },
  asset: {
    row: (p) => (p.asset ? row("image", baseName(p.asset)) : missing("image", "Choose an image")),
    override: (p) => p.asset || undefined,
  },
};

const backdropNode = defineNode({
  ...styleNode,
  type: "camera.backdrop",
  title: "Backdrop",
  description: "The background behind the floating window: a studio image, colour or gradient.",
  card: ({ params: p }) => [BACKDROP_KINDS[p.kind].row(p)],
  params: backdropParams,
});

const motionBlurNode = defineNode({
  ...styleNode,
  type: "camera.motionBlur",
  title: "Motion blur",
  description: "Adds blur to fast camera moves.",
  card: ({ params: p }) => [toggleRow("wind", p.enabled)],
  params: z.object({ enabled: bool(true, "Motion blur") }),
});

const windowFrameNode = defineNode({
  ...styleNode,
  type: "camera.windowFrame",
  title: "Window frame",
  description: "Draws a browser window's title bar around the recording.",
  card: ({ params: p }) => [
    row("app-window", `${WINDOW_STYLE_LABELS[p.windowStyle]} · ${humanize(p.frameTheme)}`),
    row("link", p.addressBar ? "Address bar" : "No address bar", mutedIf(!p.addressBar)),
  ],
  params: z.object({
    windowStyle: choice(WINDOW_STYLES, DEFAULT_WINDOW_STYLE, "Window style", WINDOW_STYLE_LABELS),
    frameTheme: choice(FRAME_THEMES, DEFAULT_FRAME_THEME, "Frame colour"),
    addressBar: more(bool(DEFAULT_ADDRESS_BAR, "Address bar"), {
      help: "Shows the page's address in the title bar.",
    }),
  }),
  hostDefaults: (platform) => ({ windowStyle: hostWindowStyle(platform) }),
});

const cameraStyles: StyleSteps = [
  styleStep(manualZoomNode, {
    restyle: (props, p, nodeId) =>
      focusBeats(props, nodeId, "zoom", p.scale, [
        { id: "a", t: p.atMs, tDepart: Math.max(0, p.atMs - FOCUS_LEAD_MS), x: p.x, y: p.y },
        { id: "b", t: p.atMs + p.holdMs, x: p.x, y: p.y },
      ]),
  }),
  styleStep(panNode, {
    restyle: (props, p, nodeId) =>
      focusBeats(props, nodeId, "pan", p.scale, [
        { id: "a", t: p.atMs, x: p.fromX, y: p.fromY },
        { id: "b", t: p.atMs + p.durationMs, x: p.toX, y: p.toY },
      ]),
  }),
  styleStep(followCursorNode, {
    restyle: (props, p, nodeId) =>
      appendBeats(props, followCursorBeats(props.log, p.scale, p.everyMs, nodeId)),
  }),
  styleStep(idleSpeedupNode, {
    retime: (clip, p, { nodeId, fps }) => {
      const log = clip.recording?.log;
      if (!log) return clip;
      const spans = idleSpans(log, p.thresholdMs, p.padMs);
      return spans.length
        ? remapClip(
            clip,
            nodeId,
            fps,
            spans.map(([fromMs, toMs]) => ({ fromMs, toMs, rate: p.rate })),
          )
        : clip;
    },
  }),
  styleStep(speedRampNode, {
    retime: (clip, p, { nodeId, fps }) =>
      remapClip(
        clip,
        nodeId,
        fps,
        p.segments.toSorted((a, b) => a.fromMs - b.fromMs),
      ),
  }),
  styleStep(backdropNode, {
    restyle: (props, p) => {
      const override = BACKDROP_KINDS[p.kind].override(p);
      return override ? { ...props, backdrop: override, backdropChosen: true } : props;
    },
  }),
  styleStep(motionBlurNode, {
    restyle: (props, p) => ({ ...props, motionBlur: p.enabled }),
  }),
  styleStep(windowFrameNode, {
    restyle: (props, p) => ({
      ...props,
      ...windowFrameProps(p.windowStyle, p.frameTheme, p.addressBar),
    }),
  }),
];

const STYLE_TABLE: StyleTable = new Map([
  ...cursorStyles,
  ...textStyles,
  ...cameraStyles,
  ...effectStyles,
]);

const applyStyleNode = defineNode({
  ...base,
  type: "camera.applyStyle",
  title: "Apply style",
  description:
    "Gives every recording in a clip the same style, for takes joined in a Sequence or Layers.",
  inputs: {
    [CLIP_PORT]: port("clip", "Clip"),
    [STYLE_PORT]: port("recording-style", "Style"),
  },
  outputs: { [CLIP_PORT]: port("clip", "Clip") },
  card: ({ inputStyle, inputTakes }) => {
    const entries = inputStyle?.[STYLE_PORT]?.length ?? 0;
    return [
      row("palette", plural(entries, "style setting"), mutedIf(!entries)),
      inputTakes?.[CLIP_PORT] === null &&
        row("triangle-alert", "No recording in this clip", {
          tone: "warn",
          title: "A style changes only recordings.",
        }),
    ];
  },
  params: z.object({}),
  compile: ({ inputs, settings }) =>
    withClip(inputs, (clip) => {
      const entries = asValue(inputs[STYLE_PORT], "recording-style")?.entries ?? [];
      const styled = patchRecordings(clip, (props) => restyleWith(STYLE_TABLE, props, entries));
      return retimeWith(STYLE_TABLE, styled, entries, settings.fps);
    }),
});

export const cameraNodes = [
  autoZoomNode,
  applyStyleNode,
  manualZoomNode,
  panNode,
  followCursorNode,
  idleSpeedupNode,
  speedRampNode,
  cropNode,
  deviceFrameNode,
  windowFrameNode,
  backdropNode,
  motionBlurNode,
];
