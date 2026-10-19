import { z } from "zod";
import { DEFAULT_BACKDROP } from "../backdrops";
import { actionLogSchema, type Action, type ActionLog } from "../schema/actions";
import { FRAME_THEMES, WINDOW_STYLES } from "../window-styles";

export const CAMERA_STYLES = ["classic", "proof", "ledger", "stage"] as const;

const ANNOTATION_KINDS = ["highlight", "spotlight", "tooltip", "callout"] as const;
export type AnnotationKind = (typeof ANNOTATION_KINDS)[number];

const annotationSpecSchema = z.object({
  kind: z.enum(ANNOTATION_KINDS),
  match: z.string(),
  text: z.string(),
  color: z.string(),
  padding: z.number(),
  leadMs: z.number(),
  holdMs: z.number(),
  textColor: z.string().optional(),
  fontSize: z.number().optional(),
  font: z.string().optional(),
});
export type AnnotationSpec = z.infer<typeof annotationSpecSchema>;

const KEYSTROKE_POSITIONS = ["bottom", "top"] as const;
export type KeystrokePosition = (typeof KEYSTROKE_POSITIONS)[number];

const cursorLookSchema = z.object({ fill: z.string(), edge: z.string() });
export type CursorLook = z.infer<typeof cursorLookSchema>;

const rippleLookSchema = z.object({ color: z.string(), size: z.number(), width: z.number() });
export type RippleLook = z.infer<typeof rippleLookSchema>;

const keystrokeLookSchema = z.object({
  position: z.enum(KEYSTROKE_POSITIONS),
  typed: z.boolean(),
  chords: z.boolean(),
  keycap: z.string(),
  ink: z.string(),
  size: z.number(),
});
export type KeystrokeLook = z.infer<typeof keystrokeLookSchema>;

export const DEFAULT_CURSOR_LOOK: CursorLook = { fill: "#111418", edge: "#ffffff" };

export const DEFAULT_RIPPLE: RippleLook = { color: "#0f172a", size: 1, width: 2 };

export const DEFAULT_KEYCAP = { keycap: "#0a0a0a", ink: "#ffffff" } as const;

export const recordingPropsSchema = z.object({
  src: z.string(),
  proxySrc: z.string().optional(),
  log: actionLogSchema,
  name: z.string(),
  speed: z.number(),
  zoom: z.boolean(),
  tuning: z.object({ sMax: z.number().optional(), holdMinS: z.number().optional() }),
  motionBlur: z.boolean(),
  style: z.enum(CAMERA_STYLES),
  chrome: z.boolean(),
  windowStyle: z.enum(WINDOW_STYLES).optional(),
  frameTheme: z.enum(FRAME_THEMES).optional(),
  addressBar: z.boolean().optional(),
  backdrop: z.string(),
  cursor: z.boolean(),
  cursorScale: z.number(),
  cursorLook: cursorLookSchema,
  ripple: rippleLookSchema.nullable(),
  extraBeats: actionLogSchema.shape.actions,
  keystrokes: keystrokeLookSchema.optional(),
  annotations: z.array(annotationSpecSchema),
});
export type RecordingProps = z.infer<typeof recordingPropsSchema>;

export const recordingSpeed = (props: Pick<RecordingProps, "speed">): number => props.speed || 1;

export const defaultRecordingProps = (): Omit<
  RecordingProps,
  "src" | "proxySrc" | "log" | "name"
> => ({
  speed: 1.25,
  zoom: true,
  tuning: {},
  motionBlur: true,
  style: "classic",
  chrome: false,
  backdrop: DEFAULT_BACKDROP,
  cursor: true,
  cursorScale: 1,
  cursorLook: DEFAULT_CURSOR_LOOK,
  ripple: DEFAULT_RIPPLE,
  extraBeats: [],
  annotations: [],
});

export const POINTER_KINDS = new Set<Action["kind"]>(["click", "type", "hover", "focus", "drag"]);

export function idleSpans(
  log: ActionLog,
  thresholdMs: number,
  padMs = 400,
): Array<[number, number]> {
  const beats = [...log.actions].sort((a, b) => a.t - b.t);
  const spans: Array<[number, number]> = [];
  let cursor = 0;
  for (const a of beats) {
    const start = (a.tDepart ?? a.t) - padMs;
    if (start - cursor >= thresholdMs) spans.push([cursor, start]);
    cursor = Math.max(cursor, (a.tEnd ?? a.t) + padMs);
  }
  const end = log.durationMs - padMs;
  if (end - cursor >= thresholdMs) spans.push([cursor, end]);
  return spans;
}

export function followCursorBeats(
  log: ActionLog,
  scale: number,
  everyMs: number,
  nodeId: string,
): Action[] {
  const track = log.cursorTrack ?? [];
  const beats: Action[] = [];
  let next = 0;
  for (const s of track) {
    if (s.t < next) continue;
    beats.push({
      id: `${nodeId}:f${beats.length}`,
      kind: "focus",
      t: s.t,
      point: { x: s.x, y: s.y },
      cluster: `${nodeId}:follow`,
      zoomScale: scale,
      nodeId,
    });
    next = s.t + everyMs;
  }
  return beats;
}
