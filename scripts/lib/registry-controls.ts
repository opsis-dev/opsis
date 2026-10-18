import { readFileSync } from "node:fs";
import { Node, type ObjectLiteralExpression } from "ts-morph";
import { z } from "zod";
import { humanize } from "../../src/shared/format";
import { isNumber, isRecord, isString, tryParseJson } from "../../src/shared/guards";
import { stringsIn } from "../../src/shared/vars";
import {
  CONTROL_TYPES,
  numberControl,
  type ControlSpec,
  type ControlType,
  type CuratedElement,
} from "../registry-ingest.list";
import { literalOf, scratchFile, sourceDefaults } from "./ts-literal";

export type Control = ControlSpec & {
  label?: string;
  variants?: Record<string, unknown>;
};

export type ParsedConfig = {
  controls: Record<string, Control>;
  durationInFrames?: number;
  componentName?: string;
};

const isControlType = (value: unknown): value is ControlType =>
  CONTROL_TYPES.some((type) => type === value);

const isControl = (value: unknown): value is Control =>
  isRecord(value) && isControlType(value.type);

function assignment(init: ObjectLiteralExpression, name: string) {
  const prop = init.getProperty(name);
  return prop && Node.isPropertyAssignment(prop) ? prop : undefined;
}

const literalProp = <T>(
  init: ObjectLiteralExpression,
  name: string,
  guard: (value: unknown) => value is T,
): T | undefined => literalOf(assignment(init, name)?.getInitializer(), guard);

export function parseConfig(source: string, name: string): ParsedConfig | null {
  const sf = scratchFile(name, "ts", source);
  for (const decl of sf.getVariableDeclarations()) {
    const init = decl.getInitializer();
    if (!init || !Node.isObjectLiteralExpression(init) || !assignment(init, "controls")) continue;
    const controls = Object.entries(literalProp(init, "controls", isRecord) ?? {});
    return {
      controls: Object.fromEntries(
        controls.filter((entry): entry is [string, Control] => isControl(entry[1])),
      ),
      durationInFrames: literalProp(init, "durationInFrames", isNumber),
      componentName: literalProp(init, "componentName", isString),
    };
  }
  return null;
}

const LIBRARY_WORDS = /\b(remotion|remotion-?ui|remocn|snapcn|snap-cn|shadcn)\b/i;

export const namesLibrary = (text: string) => LIBRARY_WORDS.test(text);
const loadsRemote = (text: string) => /^https?:\/\//.test(text);

const optionalText = z
  .string()
  .nullish()
  .transform((value) => value ?? undefined);

export const propDocsSchema = z.object({
  props: z
    .array(
      z.object({
        name: z.string(),
        type: optionalText,
        default: optionalText,
        description: optionalText,
      }),
    )
    .nullish()
    .transform((props) => props ?? []),
});

export type PropDoc = z.infer<typeof propDocsSchema>["props"][number];

const TRANSITION_INTERNALS = new Set(["durationInFrames", "variant"]);

const isColorProp = (name: string) =>
  /colou?r$/i.test(name) || name === "stroke" || name === "fill";

function propLabel(name: string): string {
  const label = PROP_OVERRIDES[name]?.label;
  if (label) return label;
  const words = humanize(name.replace(/InFrames$|Seconds$/, ""));
  return name.endsWith("Seconds") ? `${words} (seconds)` : words;
}

const TECHNICAL_HELP = /`|\bpx\b|staticFile|\bprops?\b|\b[a-z]+[A-Z][A-Za-z]*\b/;

export const splitSentences = (text: string): string[] => text.split(/(?<=[.!?])(?<!\be\.g\.)\s+/);

function cleanHelp(text: string | undefined): string | undefined {
  const sentences = splitSentences((text ?? "").replace(/,?\s*e\.g\.\s*`[^`]*`/g, ""))
    .map((s) => s.trim())
    .filter((s) => s && !TECHNICAL_HELP.test(s) && !LIBRARY_WORDS.test(s));
  return sentences.join(" ") || undefined;
}

function parseDocDefault(text: string | undefined): unknown {
  if (text === undefined) return undefined;
  const parsed = tryParseJson(text);
  if (parsed.ok) return parsed.value;
  return /^(#|rgba?\(|hsla?\()/.test(text.trim()) ? text.trim() : undefined;
}

const SPEED_RANGE = { min: 0.25, max: 4, step: 0.25 } as const;

const SPEED_CONTROL: Control = numberControl("Speed", 1, SPEED_RANGE);

type PropOverride = {
  skip?: true;
  label?: string;
  type?: "code";
  accept?: string;
  unit?: "frames";
  range?: typeof SPEED_RANGE;
  advanced?: true;
};

const PROP_OVERRIDES: Partial<Record<string, PropOverride>> = {
  children: { skip: true },
  frame: { skip: true },
  className: { skip: true },
  style: { skip: true },
  delayInFrames: { skip: true },
  exitAtInFrames: { skip: true },
  exitInFrames: { skip: true },
  exitStaggerInFrames: { skip: true },
  mediaFilter: { skip: true },
  gradient: { skip: true },
  startFrom: { skip: true },
  valueFormatter: { skip: true },
  src: { label: "Media", accept: "image/*" },
  logoSrc: { label: "Logo", accept: "image/*" },
  backgroundSrc: { accept: "image/*" },
  imageSrc: { accept: "image/*" },
  durationInFrames: { label: "Animation length" },
  cta: { label: "Button label" },
  ctaLabel: { label: "Button label" },
  url: { label: "Address" },
  pathD: { label: "Logo outline", type: "code" },
  viewBox: { label: "Outline area" },
  highlightedLines: { label: "Highlighted lines" },
  showLineNumbers: { label: "Line numbers" },
  framesPerStep: { label: "Time per step", unit: "frames", advanced: true },
  cols: { label: "Columns" },
  inkColor: { label: "Text color" },
  barInkColor: { label: "Bar text color" },
  eyebrow: { label: "Small heading" },
  insightEyebrow: { label: "Takeaway heading" },
  insight: { label: "Takeaway" },
  ctaTitle: { label: "Closing title" },
  showDropoff: { label: "Show drop-off" },
  dropoffColor: { label: "Drop-off color" },
  sweepInDegrees: { label: "Sweep angle", advanced: true },
  originX: { label: "Horizontal origin", advanced: true },
  originY: { label: "Vertical origin", advanced: true },
  centerY: { label: "Vertical position" },
  seed: { label: "Variation", advanced: true },
  stroke: { label: "Line color" },
  fill: { label: "Fill color" },
  offColor: { label: "Unlit color" },
  offLevel: { label: "Unlit brightness", advanced: true },
  siblings: { label: "Other files" },
  activeLabel: { label: "Label while dragging" },
  doneLabel: { label: "Label when done" },
  sourceLabel: { label: "Library title" },
  selectedPath: { label: "Highlighted file" },
  highlightColumn: { label: "Highlighted column" },
  highlightLabel: { label: "Highlight badge" },
  laneColors: { label: "Branch colors" },
  toasts: { label: "Notifications" },
  align: { label: "Position" },
  showProgress: { label: "Show timer bar" },
  lifeSeconds: { label: "Time on screen (seconds)" },
  tier: { label: "Plan name" },
  period: { label: "Billing period" },
  note: { label: "Small print" },
  dealStaggerSeconds: { label: "Card stagger (seconds)" },
  moveSeconds: { label: "Move time (seconds)" },
  warmSeconds: { label: "Colour fade (seconds)" },
  minSize: { label: "Smallest size", advanced: true },
  count: { label: "Particles" },
  angle: { label: "Direction" },
  maxCards: { label: "Most cards" },
  barData: { label: "Chart data" },
  steps: { label: "Steps" },
  wordmark: { label: "Name" },
  penColor: { label: "Pen color" },
  nodes: { label: "Files" },
  windowTitle: { label: "Window title" },
  tailOpacity: { label: "Last bar opacity", advanced: true },
  slide: { label: "Slide distance", advanced: true },
  media: { label: "Fill" },
  downColor: { label: "Decrease color" },
  showDelta: { label: "Show change" },
  delta: { label: "Change" },
  deltaSuffix: { label: "Change suffix" },
  stagger: { advanced: true },
  rowStagger: { advanced: true },
  decimals: { advanced: true },
  edgeSoftness: { advanced: true },
  inkSoftness: { advanced: true },
  wobble: { advanced: true },
  penSize: { advanced: true },
  hum: { advanced: true },
  buzz: { advanced: true },
  overshoot: { advanced: true },
  warp: { advanced: true },
  striation: { advanced: true },
  horizonGlow: { advanced: true },
  depthBlur: { advanced: true },
  drift: { advanced: true },
  zoom: { advanced: true },
  labelWidth: { advanced: true },
  rowHeight: { advanced: true },
  gap: { advanced: true },
  tickCount: { advanced: true },
  visibleRows: { advanced: true },
  panelWidth: { advanced: true },
  startLine: { advanced: true },
  language: { advanced: true },
  code: { type: "code" },
  before: { type: "code" },
  after: { type: "code" },
  speed: { range: SPEED_RANGE },
};

const FONT_CONTROL: Control = {
  type: "font",
  default: "",
  description: "Font",
  help: "Project font follows the fonts set for the whole project.",
};

const UNEDITABLE_TYPE = /=>|ReactNode|ReactElement|CSSProperties|SpringConfig/;
const STRING_UNION = /^\s*"[^"]*"(\s*\|\s*("[^"]*"|string))+\s*$/;
const QUOTED = /"([^"]*)"/g;

type ControlBase = Omit<Control, "type" | "default">;

function baseControl(name: string, override: PropOverride | undefined, help?: string): ControlBase {
  const unit = override?.unit ?? (name.endsWith("InFrames") ? "frames" : undefined);
  return {
    description: propLabel(name),
    help,
    ...(unit ? { unit } : {}),
    ...(/InFrames$|Seconds$/.test(name) || override?.advanced ? { advanced: true } : {}),
  };
}

function listFromValue(base: ControlBase, value: unknown[]): Control | null {
  if (value.every(isString)) return { ...base, type: "strings", default: value };
  if (value.every(isNumber)) return { ...base, type: "numbers", default: value };
  if (value.every(isRecord)) return { ...base, type: "rows", default: value };
  return null;
}

function enumFromDoc(base: ControlBase, docType: string, value: unknown): Control | null {
  const options = [...docType.matchAll(QUOTED)].map((m) => m[1]);
  if (options.length < 2 || !STRING_UNION.test(docType)) return null;
  return {
    ...base,
    type: "enum",
    options,
    default: typeof value === "string" && options.includes(value) ? value : options[0],
  };
}

function numberFromValue(base: ControlBase, value: number, override?: PropOverride): Control {
  if (override?.range) return { ...base, type: "number", default: value, ...override.range };
  const step = Number.isInteger(value) ? undefined : Math.abs(value) < 1 ? 0.01 : 0.1;
  return { ...base, type: "number", default: value, ...(step ? { step } : {}) };
}

function scalarFromValue(
  name: string,
  base: ControlBase,
  value: unknown,
  override?: PropOverride,
): Control | null {
  if (typeof value === "number") return numberFromValue(base, value, override);
  if (typeof value !== "string") return null;
  if (isColorProp(name)) return { ...base, type: "color", default: value };
  return { ...base, type: override?.type ?? "text", default: value };
}

function propControl(
  name: string,
  docType: string | undefined,
  value: unknown,
  help: string | undefined,
): Control | null {
  const type = docType ?? "";
  if (UNEDITABLE_TYPE.test(type)) return null;
  if (name === "fontFamily") return FONT_CONTROL;
  const override = PROP_OVERRIDES[name];
  const base = baseControl(name, override, help);
  if (override?.accept)
    return {
      ...base,
      type: "asset",
      default: typeof value === "string" ? value : "",
      accept: override.accept,
    };
  if (Array.isArray(value)) return listFromValue(base, value);
  if (isRecord(value)) return { ...base, type: "object", default: value };
  if (type === "boolean" || typeof value === "boolean")
    return { ...base, type: "boolean", default: value === true };
  if (value === undefined) return null;
  return enumFromDoc(base, type, value) ?? scalarFromValue(name, base, value, override);
}

export function propControls(
  item: CuratedElement,
  file: string,
  exportName: string,
  docs: PropDoc[],
): { controls: Record<string, Control>; durationInFrames?: number } {
  const fromSource = sourceDefaults(readFileSync(file, "utf8"), exportName);
  const transition = item.category === "transition";
  const names = [...new Set([...docs.map((d) => d.name), ...fromSource.keys()])];
  const out: Record<string, Control> = {};
  for (const name of names) {
    if (PROP_OVERRIDES[name]?.skip || (transition && TRANSITION_INTERNALS.has(name))) continue;
    const doc = docs.find((d) => d.name === name);
    const value =
      item.defaults && name in item.defaults
        ? item.defaults[name]
        : (fromSource.get(name) ?? parseDocDefault(doc?.default));
    const control = propControl(name, doc?.type, value, cleanHelp(doc?.description));
    if (control) out[name] = control;
  }
  const length =
    fromSource.get("durationInFrames") ??
    parseDocDefault(docs.find((d) => d.name === "durationInFrames")?.default);
  return {
    controls: out,
    durationInFrames: transition && typeof length === "number" ? length : undefined,
  };
}

const declaresFontFamily = (source: string) =>
  /\bfontFamily\?\s*:/.test(source) && /^\s+fontFamily(\s*=[^=>]|\s*,|\s*$)/m.test(source);

const declaresSpeed = (source: string) => /\bspeed\?\s*:/.test(source);

export function controlsFor(
  item: CuratedElement,
  source: string,
  found: Record<string, Control> | undefined,
  fromPropDocs: boolean,
): Array<[string, Control]> {
  const controls: Record<string, Control> = { ...found, ...item.controls };
  if (declaresFontFamily(source)) controls.fontFamily = FONT_CONTROL;
  else if (!fromPropDocs) delete controls.fontFamily;
  if (!fromPropDocs)
    for (const [k, v] of Object.entries(item.defaults ?? {}))
      if (controls[k]) controls[k] = { ...controls[k], default: v };
  const omit = item.omit ?? [];
  if (
    declaresSpeed(source) &&
    !controls.speed &&
    item.category !== "transition" &&
    !omit.includes("speed")
  )
    controls.speed = SPEED_CONTROL;
  return Object.entries(controls).filter(([k]) => !omit.includes(k));
}

export function defaultProblems(
  key: string,
  controls: Array<[string, Control]>,
  fixedProps: Record<string, unknown> | undefined,
): string[] {
  const shown = (hit: string) => JSON.stringify(hit).slice(0, 80);
  return [
    ...controls.flatMap(([k, c]) => [
      ...stringsIn(c.default, namesLibrary).map(
        (hit) => `${key}: default for "${k}" names a library: ${shown(hit)}`,
      ),
      ...stringsIn(c.default, loadsRemote).map(
        (hit) => `${key}: default for "${k}" loads from the internet: ${shown(hit)}`,
      ),
    ]),
    ...[...stringsIn(fixedProps, namesLibrary), ...stringsIn(fixedProps, loadsRemote)].map(
      (hit) => `${key}: fixed prop names a library or loads from the internet: ${shown(hit)}`,
    ),
  ];
}
