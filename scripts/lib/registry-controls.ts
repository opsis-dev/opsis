import { readFileSync } from "node:fs";
import { Node } from "ts-morph";
import { z } from "zod";
import { humanize } from "../../src/shared/format";
import { hasStringType, isNumber, isRecord, isString, tryParseJson } from "../../src/shared/guards";
import { stringsIn } from "../../src/shared/vars";
import type { ControlSpec, CuratedElement } from "../registry-ingest.list";
import { js } from "./registry-codegen";
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

export function parseConfig(source: string, name: string): ParsedConfig | null {
  const sf = scratchFile(name, "ts", source);
  for (const decl of sf.getVariableDeclarations()) {
    const init = decl.getInitializer();
    if (!init || !Node.isObjectLiteralExpression(init)) continue;
    const controlsProp = init.getProperty("controls");
    if (!controlsProp || !Node.isPropertyAssignment(controlsProp)) continue;
    const controls = Object.entries(literalOf(controlsProp.getInitializer(), isRecord) ?? {});
    const dur = init.getProperty("durationInFrames");
    const comp = init.getProperty("componentName");
    return {
      controls: Object.fromEntries(
        controls.filter((entry): entry is [string, Control] => hasStringType(entry[1])),
      ),
      durationInFrames:
        dur && Node.isPropertyAssignment(dur)
          ? literalOf(dur.getInitializer(), isNumber)
          : undefined,
      componentName:
        comp && Node.isPropertyAssignment(comp)
          ? literalOf(comp.getInitializer(), isString)
          : undefined,
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

const SKIP_PROPS = new Set([
  "children",
  "frame",
  "className",
  "style",
  "delayInFrames",
  "exitAtInFrames",
  "exitInFrames",
  "exitStaggerInFrames",
  "mediaFilter",
  "gradient",
  "startFrom",
  "valueFormatter",
]);
const TRANSITION_INTERNALS = new Set(["durationInFrames", "variant"]);
const ASSET_ACCEPT: Record<string, string> = {
  src: "image/*",
  logoSrc: "image/*",
  backgroundSrc: "image/*",
  imageSrc: "image/*",
};
const ADVANCED_PROPS = new Set([
  "seed",
  "stagger",
  "rowStagger",
  "framesPerStep",
  "decimals",
  "edgeSoftness",
  "inkSoftness",
  "wobble",
  "penSize",
  "hum",
  "buzz",
  "offLevel",
  "overshoot",
  "warp",
  "originX",
  "originY",
  "striation",
  "horizonGlow",
  "depthBlur",
  "minSize",
  "drift",
  "zoom",
  "tailOpacity",
  "labelWidth",
  "rowHeight",
  "gap",
  "tickCount",
  "sweepInDegrees",
  "visibleRows",
  "slide",
  "panelWidth",
  "startLine",
  "language",
]);
const LABELS: Record<string, string> = {
  durationInFrames: "Animation length",
  src: "Media",
  logoSrc: "Logo",
  cta: "Button label",
  ctaLabel: "Button label",
  url: "Address",
  pathD: "Logo outline",
  viewBox: "Outline area",
  highlightedLines: "Highlighted lines",
  showLineNumbers: "Line numbers",
  framesPerStep: "Time per step",
  cols: "Columns",
  inkColor: "Text color",
  barInkColor: "Bar text color",
  eyebrow: "Small heading",
  insightEyebrow: "Takeaway heading",
  insight: "Takeaway",
  ctaTitle: "Closing title",
  showDropoff: "Show drop-off",
  dropoffColor: "Drop-off color",
  sweepInDegrees: "Sweep angle",
  originX: "Horizontal origin",
  originY: "Vertical origin",
  centerY: "Vertical position",
  seed: "Variation",
  stroke: "Line color",
  fill: "Fill color",
  offColor: "Unlit color",
  offLevel: "Unlit brightness",
  siblings: "Other files",
  activeLabel: "Label while dragging",
  doneLabel: "Label when done",
  sourceLabel: "Library title",
  selectedPath: "Highlighted file",
  highlightColumn: "Highlighted column",
  highlightLabel: "Highlight badge",
  laneColors: "Branch colors",
  toasts: "Notifications",
  align: "Position",
  showProgress: "Show timer bar",
  lifeSeconds: "Time on screen (seconds)",
  tier: "Plan name",
  period: "Billing period",
  note: "Small print",
  dealStaggerSeconds: "Card stagger (seconds)",
  moveSeconds: "Move time (seconds)",
  warmSeconds: "Colour fade (seconds)",
  minSize: "Smallest size",
  count: "Particles",
  angle: "Direction",
  maxCards: "Most cards",
  barData: "Chart data",
  steps: "Steps",
  wordmark: "Name",
  penColor: "Pen color",
  nodes: "Files",
  windowTitle: "Window title",
  tailOpacity: "Last bar opacity",
  slide: "Slide distance",
  media: "Fill",
  downColor: "Decrease color",
  showDelta: "Show change",
  delta: "Change",
  deltaSuffix: "Change suffix",
};

const isColorProp = (name: string) =>
  /colou?r$/i.test(name) || name === "stroke" || name === "fill";

function propLabel(name: string): string {
  if (LABELS[name]) return LABELS[name];
  const seconds = name.endsWith("Seconds");
  const label = humanize(name.replace(/InFrames$|Seconds$/, ""));
  return seconds ? `${label} (seconds)` : label;
}

const TECHNICAL_HELP = /`|\bpx\b|staticFile|\bprops?\b|\b[a-z]+[A-Z][A-Za-z]*\b/;

function cleanHelp(text: string | undefined): string | undefined {
  const sentences = (text ?? "")
    .replace(/,?\s*e\.g\.\s*`[^`]*`/g, "")
    .replace(/\be\.g\.\s/g, "e.g.\u00a0")
    .split(/(?<=[.!?])\s+/)
    .map((s) => s.trim().replace(/\u00a0/g, " "))
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

const SPEED_CONTROL: Control = { type: "number", default: 1, ...SPEED_RANGE, description: "Speed" };

const FONT_CONTROL: Control = {
  type: "font",
  default: "",
  description: "Font",
  help: "Project font follows the fonts set for the whole project.",
};

function propControl(
  name: string,
  type: string,
  value: unknown,
  help: string | undefined,
): Control | null {
  const base: Control = {
    type: "",
    description: propLabel(name),
    help,
    ...(name.endsWith("InFrames") || name === "framesPerStep" ? { unit: "frames" } : {}),
    ...(/InFrames$|Seconds$/.test(name) || ADVANCED_PROPS.has(name) ? { advanced: true } : {}),
  };
  if (/=>|ReactNode|ReactElement|CSSProperties|SpringConfig/.test(type)) return null;
  if (name === "fontFamily") return FONT_CONTROL;
  if (name in ASSET_ACCEPT)
    return {
      ...base,
      type: "asset",
      default: typeof value === "string" ? value : "",
      accept: ASSET_ACCEPT[name],
    };
  if (Array.isArray(value)) {
    if (value.every(isString)) return { ...base, type: "strings", default: value };
    if (value.every(isNumber)) return { ...base, type: "numbers", default: value };
    if (value.every(isRecord)) return { ...base, type: "rows", default: value };
    return null;
  }
  if (isRecord(value)) return { ...base, type: "object", default: value };
  if (type === "boolean" || typeof value === "boolean")
    return { ...base, type: "boolean", default: value === true };
  if (value === undefined) return null;
  const options = [...type.matchAll(/"([^"]*)"/g)].map((m) => m[1]);
  if (options.length >= 2 && /^\s*"[^"]*"(\s*\|\s*("[^"]*"|string))+\s*$/.test(type))
    return {
      ...base,
      type: "enum",
      options,
      default: typeof value === "string" && options.includes(value) ? value : options[0],
    };
  if (typeof value === "number") {
    if (name === "speed") return { ...base, type: "number", default: value, ...SPEED_RANGE };
    const step = Number.isInteger(value) ? undefined : Math.abs(value) < 1 ? 0.01 : 0.1;
    return { ...base, type: "number", default: value, ...(step ? { step } : {}) };
  }
  if (typeof value !== "string") return null;
  if (isColorProp(name)) return { ...base, type: "color", default: value };
  if (["code", "before", "after", "pathD"].includes(name))
    return { ...base, type: "code", default: value };
  return { ...base, type: "text", default: value };
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
    if (SKIP_PROPS.has(name) || (transition && TRANSITION_INTERNALS.has(name))) continue;
    const doc = docs.find((d) => d.name === name);
    const value =
      item.defaults && name in item.defaults
        ? item.defaults[name]
        : (fromSource.get(name) ?? parseDocDefault(doc?.default));
    const control = propControl(
      name,
      doc?.type ?? typeof value,
      value,
      cleanHelp(doc?.description),
    );
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
  const shown = (hit: string) => js(hit).slice(0, 80);
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
