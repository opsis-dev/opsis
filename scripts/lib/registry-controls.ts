import { readFileSync } from "node:fs";
import path from "node:path";
import { parseColor } from "@opsis/core/color";
import type { FieldMeta } from "@opsis/core/elements/introspect";
import { humanize } from "@opsis/core/format";
import { isNumber, isRecord, isString, tryParseJson } from "@opsis/core/guards";
import type { StepRange } from "@opsis/core/math";
import { stringsIn } from "@opsis/core/vars";
import { Node, type ObjectLiteralExpression } from "ts-morph";
import { z } from "zod";
import {
  CONTROL_TYPES,
  FONT_WEIGHTS,
  HIDDEN_PROPS,
  ANIMATION_FRAMES,
  ANIMATION_SECONDS,
  SPEED_RANGE,
  numberControl,
  type ControlSpec,
  type ControlType,
  type CuratedElement,
} from "../registry-ingest.list";
import { zodFor } from "./registry-codegen";
import {
  inferColumns,
  isColorList,
  isColorProp,
  rowMembers,
  rowTypeName,
  unionOptions,
} from "./registry-columns";
import { VIDEO_DIR } from "./tool-config";
import { bindingDefaults, literalOf, scratchFile, sourceDefaults } from "./ts-literal";

export type Control = ControlSpec & {
  label?: string;
  variants?: Record<string, unknown>;
  themes?: FieldMeta["themes"];
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

const BRAND_WORDS =
  /\b(apple|airpods|iphone|ipad|macbook|google|chrome|safari|firefox|mozilla|webkit|microsoft|github|gitlab|slack|notion|figma|stripe|vercel|netlify|react|next\.js|tailwind|openai|chatgpt|claude|anthropic|grok|amazon|youtube|twitter|linkedin|discord|spotify|netflix|shopify|paypal)\b/i;

export const namesLibrary = (text: string) => LIBRARY_WORDS.test(text) || BRAND_WORDS.test(text);
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

type PropOverride = {
  label?: string;
  type?: "code";
  accept?: string;
  unit?: "frames";
  range?: StepRange;
  advanced?: true;
};

const PROP_OVERRIDES: Partial<Record<string, PropOverride>> = {
  src: { label: "Media", accept: "image/*" },
  logoSrc: { label: "Logo", accept: "image/*" },
  backgroundSrc: { accept: "image/*" },
  imageSrc: { accept: "image/*" },
  durationInFrames: { label: "Animation length", range: ANIMATION_FRAMES },
  rollSeconds: { range: ANIMATION_SECONDS },
  wipeSeconds: { range: ANIMATION_SECONDS },
  stepSeconds: { range: ANIMATION_SECONDS },
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
  moveSeconds: { label: "Move time (seconds)", range: ANIMATION_SECONDS },
  warmSeconds: { label: "Colour fade (seconds)", range: ANIMATION_SECONDS },
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
  upColor: { label: "Increase color" },
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

const SPEED_CONTROL: Control = numberControl("Speed", 1, SPEED_RANGE);

const FONT_CONTROL: Control = {
  type: "font",
  default: "",
  description: "Font",
  help: "Project font follows the fonts set for the whole project.",
};

const UNEDITABLE_TYPE = /=>|ReactNode|ReactElement|CSSProperties|SpringConfig/;

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

function listFromValue(name: string, base: ControlBase, value: unknown[]): Control | null {
  if (isColorList(name, value)) return { ...base, type: "colors", min: 1, default: value };
  if (value.every(isString)) return { ...base, type: "strings", default: value };
  if (value.every(isNumber)) return { ...base, type: "numbers", default: value };
  if (value.every(isRecord)) return { ...base, type: "rows", default: value };
  return null;
}

function enumFromDoc(base: ControlBase, docType: string, value: unknown): Control | null {
  const options = unionOptions(docType, { open: true }) ?? [];
  if (options.length < 2) return null;
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
  if (Array.isArray(value)) return listFromValue(name, base, value);
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
    if (Object.hasOwn(HIDDEN_PROPS, name) || (transition && TRANSITION_INTERNALS.has(name)))
      continue;
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

const omitted = (item: CuratedElement, key: string) => Object.hasOwn(item.omit ?? {}, key);

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
  if (
    declaresSpeed(source) &&
    !controls.speed &&
    item.category !== "transition" &&
    !omitted(item, "speed")
  )
    controls.speed = SPEED_CONTROL;
  return Object.entries(controls).filter(([k]) => !omitted(item, k));
}

const LIST_TYPES: ReadonlySet<ControlType> = new Set(["colors", "rows", "strings", "numbers"]);

function withFontWeights(key: string, c: Control): Control {
  const known = c.options?.length || Object.keys(c.variants ?? {}).length;
  if (key !== "fontWeight" || known || (c.type !== "enum" && c.type !== "select")) return c;
  return { ...c, options: FONT_WEIGHTS, default: String(c.default ?? "400") };
}

type Resolution = { controls: Array<[string, Control]>; problems: string[] };

function themePresets(item: CuratedElement): Control["themes"] {
  if (!item.theme) return undefined;
  const source = readFileSync(path.join(VIDEO_DIR, "lib", item.theme.presets), "utf8");
  const colorsOf = (name: string) =>
    Object.fromEntries(
      Object.entries(
        literalOf(
          scratchFile("theme", "ts", source).getVariableDeclaration(name)?.getInitializer(),
          isRecord,
        ) ?? {},
      ).filter((entry): entry is [string, string] => isString(entry[1])),
    );
  return {
    light: colorsOf("defaultLightTheme"),
    dark: colorsOf("defaultDarkTheme"),
  };
}

function withColumns(
  key: string,
  control: Control,
  file: string,
  docType: string | undefined,
): { control: Control; problems: string[] } {
  if (control.type !== "rows") return { control, problems: [] };
  const rows = Array.isArray(control.default) ? control.default : [];
  const members = rowMembers(file, rowTypeName(docType));
  const { columns, problems } = inferColumns(rows, members, control.columns);
  const nestedProblems = Object.entries(columns).flatMap(([column, spec]) =>
    spec.type === "rows" && !spec.columns
      ? [`column "${column}" is a list of rows without columns`]
      : [],
  );
  const all = [...problems, ...nestedProblems];
  if (Object.keys(columns).length === 0) all.push("rows without columns");
  return { control: { ...control, columns }, problems: all.map((p) => `"${key}": ${p}`) };
}

const TEXT_TYPES: ReadonlySet<ControlType> = new Set(["text", "text-content"]);

const asColor = (key: string, control: Control): Control =>
  TEXT_TYPES.has(control.type) &&
  isColorProp(key) &&
  isString(control.default) &&
  parseColor(control.default)
    ? { ...control, type: "color" }
    : control;

const THEME_FALLBACK = /\b(\w+) \?\? t\.(\w+)\b/g;

export const themeFallbacks = (source: string): Map<string, string> =>
  new Map([...source.matchAll(THEME_FALLBACK)].map((m) => [m[1], m[2]]));

function colorFallback(
  control: Control,
  token: string | undefined,
  preset: Record<string, string> | undefined,
  fromComponent: unknown,
): Control {
  if (control.type !== "color") return control;
  if (token) {
    const followsTheme = control.default === undefined || control.default === preset?.[token];
    return { ...control, emptyLabel: "Theme", default: followsTheme ? "" : control.default };
  }
  if (control.default === undefined && isString(fromComponent))
    return { ...control, default: fromComponent };
  return control.default === undefined
    ? { ...control, default: "", emptyLabel: "Default" }
    : control;
}

export function resolveControls(
  item: CuratedElement,
  file: string,
  controls: Array<[string, Control]>,
  docs: PropDoc[] = [],
): Resolution {
  const source = readFileSync(file, "utf8");
  const fromComponent = bindingDefaults(source);
  const themes = themePresets(item);
  const fallbacks = item.theme ? themeFallbacks(source) : new Map<string, string>();
  const preset = item.theme ? themes?.[item.theme.mode] : undefined;
  const problems: string[] = [];
  const resolved = controls.map(([key, c]): [string, Control] => {
    let control: Control = withFontWeights(key, c);
    if (control.default === undefined && LIST_TYPES.has(control.type) && fromComponent.has(key))
      control = { ...control, default: fromComponent.get(key) };
    control = colorFallback(
      asColor(key, control),
      fallbacks.get(key),
      preset,
      fromComponent.get(key),
    );
    if (control.type === "theme")
      control = { ...control, themes, default: control.default ?? item.theme?.mode };
    const withRows = withColumns(key, control, file, docs.find((d) => d.name === key)?.type);
    problems.push(...withRows.problems);
    control = withRows.control;
    if ((control.type === "colors" || control.type === "rows") && !Array.isArray(control.default))
      problems.push(`"${key}": no default list; give one in the list or the component`);
    if (zodFor(key, control) === null)
      problems.push(
        `"${key}": a ${control.type} setting the ingest can't write; give it in the list`,
      );
    return [key, control];
  });
  return { controls: resolved, problems };
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
        (hit) => `${key}: default for "${k}" names a library, company or product: ${shown(hit)}`,
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
