import { execSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { Node, Project, SyntaxKind, type SourceFile } from "ts-morph";
import { z } from "zod";
import {
  CURATED,
  DESCRIPTIONS,
  REGISTRIES,
  unknownOverrideNames,
  type ControlSpec,
  type CuratedElement,
} from "./registry-ingest.list";
import type { ElementKind } from "../src/shared/elements/types";
import { errorMessage } from "../src/shared/errors";
import { humanize } from "../src/shared/format";
import { walkFiles } from "./lib/walk";

const ROOT = path.resolve(import.meta.dirname, "..");
const SRC = path.join(ROOT, "src");
const VIDEO = path.join(SRC, "video");
const OUT_META = path.join(SRC, "shared", "elements", "elements.generated.ts");
const OUT_COMPONENTS = path.join(VIDEO, "registry", "components.generated.ts");
const skipAdd = process.argv.includes("--no-add");
const onlyIndex = process.argv.indexOf("--only");
const only = onlyIndex === -1 ? undefined : process.argv[onlyIndex + 1];
if (only && skipAdd)
  throw new Error("--only picks the registry to install, so it does nothing with --no-add.");

type Control = ControlSpec & {
  label?: string;
  variants?: Record<string, unknown>;
};

function install(items: CuratedElement[]): void {
  const names = items.map((i) => `@${i.registry}/${i.name}`);
  execSync(`yes n | bunx --bun shadcn@latest add ${names.join(" ")}`, {
    cwd: ROOT,
    stdio: "inherit",
    shell: "/bin/bash",
  });
}

const MOVES: Array<[string, string]> = [
  ["components", "elements"],
  ["lib", "lib"],
  ["hooks", "hooks"],
];

const FLAT_MOVES: Array<[string, string]> = [
  ["remotion/primitives", "elements/remotion-ui"],
  ["remotion/scenes", "elements/remotion-ui"],
  ["compositions", "elements/remotion-ui"],
  ["remotion/lib", "lib/remotion-ui"],
  ["remotion/hooks", "hooks/remotion-ui"],
];

function moveTree(from: string, dest: string): void {
  for (const file of walkFiles(from)) {
    const target = path.join(dest, path.relative(from, file));
    mkdirSync(path.dirname(target), { recursive: true });
    renameSync(file, target);
  }
}

const REMOTION_UI_SPEC =
  /^@\/(?:remotion\/(primitives|scenes|lib|hooks)|(compositions))\/([^/]+)(\/.*)?$/;

function moveInstalled(): void {
  for (const [from, to] of MOVES) {
    const fromDir = path.join(SRC, from);
    if (!existsSync(fromDir)) continue;
    for (const ns of readdirSync(fromDir)) {
      const src = path.join(fromDir, ns);
      const dest = path.join(VIDEO, to, ns);
      mkdirSync(path.dirname(dest), { recursive: true });
      if (statSync(src).isDirectory()) moveTree(src, dest);
      else renameSync(src, dest);
    }
    rmSync(fromDir, { recursive: true, force: true });
  }

  for (const [from, to] of FLAT_MOVES) {
    const fromDir = path.join(SRC, from);
    if (existsSync(fromDir)) moveTree(fromDir, path.join(VIDEO, to));
  }
  for (const dir of ["remotion", "compositions"]) {
    const left = walkFiles(path.join(SRC, dir));
    if (left.length)
      throw new Error(`Unexpected files left after relocating:\n  ${left.join("\n  ")}`);
    rmSync(path.join(SRC, dir), { recursive: true, force: true });
  }
}

type Namespaces = { elements: Set<string>; lib: Set<string>; hooks: Set<string> };

function installedNamespaces(): Namespaces {
  const nsDirs = (sub: string) =>
    new Set(existsSync(path.join(VIDEO, sub)) ? readdirSync(path.join(VIDEO, sub)) : []);
  return { elements: nsDirs("elements"), lib: nsDirs("lib"), hooks: nsDirs("hooks") };
}

function rewriteSpec(spec: string, ns: Namespaces): string {
  if (spec === "@/lib/utils") return "@/renderer/lib/utils";
  if (spec.startsWith("@/components/ui/"))
    return spec.replace("@/components/ui/", "@/renderer/components/ui/");
  const ui = REMOTION_UI_SPEC.exec(spec);
  if (ui) {
    const kind = ui[1] ?? ui[2];
    const base = kind === "lib" ? "lib" : kind === "hooks" ? "hooks" : "elements";
    return `@/video/${base}/remotion-ui/${ui[3]}${ui[4] ?? ""}`;
  }
  const m = /^@\/(?:renderer\/|video\/)?(components|elements|lib|hooks)\/([^/]+)(\/.*)?$/.exec(
    spec,
  );
  if (!m) return spec;
  const [, kind, name, rest = ""] = m;
  if ((kind === "components" || kind === "elements") && ns.elements.has(name))
    return `@/video/elements/${name}${rest}`;
  if (kind === "lib" && ns.lib.has(name)) return `@/video/lib/${name}${rest}`;
  if (kind === "lib" && name === "utils") return "@/renderer/lib/utils";
  if (kind === "hooks" && ns.hooks.has(name)) return `@/video/hooks/${name}${rest}`;
  return spec;
}

function rewriteImports(): void {
  const ns = installedNamespaces();
  for (const file of walkFiles(VIDEO)) {
    if (!/\.(tsx?|css)$/.test(file) || file.includes(`${path.sep}registry${path.sep}`)) continue;
    const before = readFileSync(file, "utf8");
    const after = before
      .replace(/^"use client";\s*\n/m, "")
      .replace(
        /(from\s+|import\s*\(\s*|import\s+)(["'])(@\/[^"']+)\2/g,
        (_all, lead: string, q: string, spec: string) => `${lead}${q}${rewriteSpec(spec, ns)}${q}`,
      );
    if (after !== before) writeFileSync(file, after);
  }
}

const project = new Project({ useInMemoryFileSystem: true });
let scratchFiles = 0;

const scratchFile = (name: string, ext: string, source: string) =>
  project.createSourceFile(`${name}-${++scratchFiles}.${ext}`, source);

const isRecord = (value: unknown): value is Record<string, unknown> =>
  !!value && typeof value === "object" && !Array.isArray(value);

function literal(node: Node | undefined, scope?: SourceFile, depth = 0): unknown {
  if (!node || depth > 8) return undefined;
  const next = (n: Node | undefined) => literal(n, scope, depth + 1);
  if (Node.isStringLiteral(node) || Node.isNoSubstitutionTemplateLiteral(node))
    return node.getLiteralValue();
  if (Node.isNumericLiteral(node)) return node.getLiteralValue();
  if (Node.isPrefixUnaryExpression(node) && node.getOperatorToken() === SyntaxKind.MinusToken) {
    const v = next(node.getOperand());
    return typeof v === "number" ? -v : undefined;
  }
  if (node.getKind() === SyntaxKind.TrueKeyword) return true;
  if (node.getKind() === SyntaxKind.FalseKeyword) return false;
  if (
    Node.isAsExpression(node) ||
    Node.isSatisfiesExpression(node) ||
    Node.isParenthesizedExpression(node)
  )
    return next(node.getExpression());
  if (Node.isArrayLiteralExpression(node)) return node.getElements().map((e) => next(e));
  if (Node.isObjectLiteralExpression(node)) {
    const out: Record<string, unknown> = {};
    for (const prop of node.getProperties()) {
      if (Node.isPropertyAssignment(prop)) {
        const name = prop
          .getNameNode()
          .getText()
          .replace(/^["']|["']$/g, "");
        out[name] = next(prop.getInitializer());
      }
    }
    return out;
  }
  if (Node.isCallExpression(node) && node.getExpression().getText() === "enumVariants") {
    const arr = next(node.getArguments()[0]);
    if (Array.isArray(arr)) return Object.fromEntries(arr.map((v) => [String(v), {}]));
  }
  if (scope && Node.isIdentifier(node)) {
    const decl = scope.getVariableDeclaration(node.getText());
    return decl ? next(decl.getInitializer()) : undefined;
  }
  if (scope && Node.isPropertyAccessExpression(node)) {
    const owner = next(node.getExpression());
    if (isRecord(owner)) return owner[node.getName()];
  }
  return undefined;
}

function literalNumber(node: Node | undefined, scope?: SourceFile): number | undefined {
  const value = literal(node, scope);
  return typeof value === "number" ? value : undefined;
}

function literalString(node: Node | undefined, scope?: SourceFile): string | undefined {
  const value = literal(node, scope);
  return typeof value === "string" ? value : undefined;
}

function literalRecord(node: Node | undefined, scope?: SourceFile): Record<string, unknown> {
  const value = literal(node, scope);
  return isRecord(value) ? value : {};
}

const isControl = (value: unknown): value is Control =>
  isRecord(value) && typeof value.type === "string";

type ParsedConfig = {
  controls: Record<string, Control>;
  durationInFrames?: number;
  componentName?: string;
};

function parseConfig(source: string, name: string): ParsedConfig | null {
  const sf = scratchFile(name, "ts", source);
  for (const decl of sf.getVariableDeclarations()) {
    const init = decl.getInitializer();
    if (!init || !Node.isObjectLiteralExpression(init)) continue;
    const controlsProp = init.getProperty("controls");
    if (!controlsProp || !Node.isPropertyAssignment(controlsProp)) continue;
    const controls = Object.entries(literalRecord(controlsProp.getInitializer()));
    const dur = init.getProperty("durationInFrames");
    const comp = init.getProperty("componentName");
    return {
      controls: Object.fromEntries(
        controls.filter((entry): entry is [string, Control] => isControl(entry[1])),
      ),
      durationInFrames:
        dur && Node.isPropertyAssignment(dur) ? literalNumber(dur.getInitializer()) : undefined,
      componentName:
        comp && Node.isPropertyAssignment(comp) ? literalString(comp.getInitializer()) : undefined,
    };
  }
  return null;
}

const LIBRARY_WORDS = /\b(remotion|remotion-?ui|remocn|snapcn|snap-cn|shadcn)\b/i;

const namesLibrary = (text: string) => LIBRARY_WORDS.test(text);
const loadsRemote = (text: string) => /^https?:\/\//.test(text);

const registryItemSchema = z.object({ description: z.string().nullish() });

async function fetchDescription(item: CuratedElement): Promise<string> {
  const url = REGISTRIES[item.registry].itemUrl(item.name);
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  const { description } = registryItemSchema.parse(await res.json());
  return (description ?? "")
    .split(/(?<=[.!?])\s+/)
    .map((s) => s.trim())
    .filter((s) => s && !namesLibrary(s))
    .slice(0, 2)
    .join(" ");
}

async function descriptionOf(
  item: CuratedElement,
  key: string,
  problems: string[],
): Promise<string | undefined> {
  const known = item.description ?? DESCRIPTIONS[item.name];
  if (known) return known;
  try {
    const fetched = await fetchDescription(item);
    if (fetched) return fetched;
    problems.push(`${key}: the registry gives no usable description; add one to DESCRIPTIONS`);
  } catch (error) {
    problems.push(`${key}: no description (${errorMessage(error)}); add one to DESCRIPTIONS`);
  }
  return undefined;
}

async function fetchConfig(item: CuratedElement, installed: string): Promise<ParsedConfig | null> {
  const reg = REGISTRIES[item.registry];
  for (const dir of reg.configDirs) {
    const url = `${reg.rawBase}/${dir}/${item.name}/config.ts`;
    const res = await fetch(url);
    if (res.ok) return parseConfig(await res.text(), item.name);
  }
  return parseConfig(readFileSync(installed, "utf8"), item.name);
}

const optionalText = z
  .string()
  .nullish()
  .transform((value) => value ?? undefined);

const propDocsSchema = z.object({
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

type PropDoc = z.infer<typeof propDocsSchema>["props"][number];

async function fetchPropDocs(url: string): Promise<PropDoc[]> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  return propDocsSchema.parse(await res.json()).props;
}

const durationTables = new Map<string, Promise<Record<string, number>>>();

function previewDurations(url: string): Promise<Record<string, number>> {
  let pending = durationTables.get(url);
  if (!pending) {
    pending = fetch(url)
      .then((res) => {
        if (!res.ok) throw new Error(`${url}: ${res.status}`);
        return res.text();
      })
      .then((text) => {
        const sf = scratchFile("preview", "ts", text);
        const table = literalRecord(
          sf.getVariableDeclaration("PREVIEW_META")?.getInitializer(),
          sf,
        );
        const fallback = literalRecord(
          sf.getVariableDeclaration("PREVIEW_DEFAULTS")?.getInitializer(),
          sf,
        );
        const out: Record<string, number> = {
          "*": typeof fallback.durationInFrames === "number" ? fallback.durationInFrames : 120,
        };
        for (const [k, v] of Object.entries(table)) {
          const d = isRecord(v) ? v.durationInFrames : undefined;
          if (typeof d === "number") out[k] = d;
        }
        return out;
      });
    durationTables.set(url, pending);
  }
  return pending;
}

function sourceDefaults(source: string, exportName: string): Map<string, unknown> {
  const sf = scratchFile("src", "tsx", source);
  const out = new Map<string, unknown>();
  const init = sf.getVariableDeclaration(exportName)?.getInitializer();
  const fn =
    init && (Node.isArrowFunction(init) || Node.isFunctionExpression(init))
      ? init
      : sf.getFunction(exportName);
  const binding = fn?.getParameters()[0]?.getNameNode();
  if (!binding || !Node.isObjectBindingPattern(binding)) return out;
  for (const el of binding.getElements()) {
    if (el.getDotDotDotToken()) continue;
    const key = el.getPropertyNameNode()?.getText() ?? el.getName();
    out.set(key, literal(el.getInitializer(), sf));
  }
  return out;
}

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
  try {
    return JSON.parse(text);
  } catch {
    return /^(#|rgba?\(|hsla?\()/.test(text.trim()) ? text.trim() : undefined;
  }
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
    if (value.every((v) => typeof v === "string"))
      return { ...base, type: "strings", default: value };
    if (value.every((v) => typeof v === "number"))
      return { ...base, type: "numbers", default: value };
    if (value.every((v) => v && typeof v === "object" && !Array.isArray(v)))
      return { ...base, type: "rows", default: value };
    return null;
  }
  if (value && typeof value === "object") return { ...base, type: "object", default: value };
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

async function propControls(
  item: CuratedElement,
  file: string,
  exportName: string,
): Promise<{ controls: Record<string, Control>; durationInFrames?: number }> {
  const reg = REGISTRIES[item.registry];
  const docs = reg.propsUrl ? await fetchPropDocs(reg.propsUrl(item.name)) : [];
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

const q = (v: unknown) => JSON.stringify(v);

function metaOf(key: string, c: Control, extra: Record<string, unknown> = {}): string {
  const meta: Record<string, unknown> = {
    description: c.description ?? c.label ?? key,
    ...extra,
    ...(typeof c.step === "number" ? { step: c.step } : {}),
    ...(c.unit ? { unit: c.unit } : {}),
    ...(c.help ? { help: c.help } : {}),
    ...(c.advanced ? { advanced: true } : {}),
    ...(c.widget ? { widget: c.widget } : {}),
    ...(c.hidden ? { hidden: true } : {}),
  };
  const body = Object.entries(meta)
    .map(([k, v]) => `${k}: ${q(v)}`)
    .join(", ");
  return `.meta({ ${body} })`;
}

function zodFor(key: string, c: Control): string | null {
  const m = (extra: Record<string, unknown> = {}) => metaOf(key, c, extra);
  const text = String(c.default ?? "");
  switch (c.type) {
    case "text":
    case "text-content":
      return `z.string().default(${q(text)})${m(text.includes("\n") || text.length > 60 ? { widget: "textarea" } : {})}`;
    case "code":
      return `z.string().default(${q(text)})${m({ widget: "code" })}`;
    case "font":
      return `z.string().default("")${m({ widget: "font" })}`;
    case "image":
      return `z.string().default(${q(text)})${m({ widget: "asset" })}`;
    case "asset":
      return `z.string().default(${q(text)})${m({ widget: "asset", ...(c.accept ? { accept: c.accept } : {}) })}`;
    case "number":
    case "number-input": {
      let s = "z.number()";
      if (typeof c.min === "number") s += `.min(${c.min})`;
      if (typeof c.max === "number") s += `.max(${c.max})`;
      s += `.default(${typeof c.default === "number" ? c.default : 0})`;
      return s + m();
    }
    case "color":
      return `zColor().default(${q(String(c.default ?? "#000000"))})${m()}`;
    case "boolean":
      return `z.boolean().default(${c.default === true})${m()}`;
    case "enum":
    case "select": {
      const opts = c.options ?? Object.keys(c.variants ?? {});
      if (opts.length === 0) return null;
      const def =
        c.default != null && opts.includes(String(c.default)) ? String(c.default) : opts[0];
      return `z.enum(${q(opts.map(String))}).default(${q(def)})${m()}`;
    }
    case "strings":
      return `z.array(z.string()).default(${q(c.default ?? [])})${m()}`;
    case "numbers":
      return `z.array(z.number()).default(${q(c.default ?? [])})${m()}`;
    case "rows":
      return `z.array(z.record(z.string(), z.unknown())).default(${q(c.default ?? [])})${m()}`;
    case "object":
      return `z.record(z.string(), z.unknown()).default(${q(c.default ?? {})})${m()}`;
    default:
      return null;
  }
}

function stringsIn(value: unknown, test: (text: string) => boolean): string[] {
  if (typeof value === "string") return test(value) ? [value] : [];
  if (Array.isArray(value)) return value.flatMap((v) => stringsIn(v, test));
  if (value && typeof value === "object")
    return Object.values(value).flatMap((v) => stringsIn(v, test));
  return [];
}

function defaultProblems(
  key: string,
  controls: Array<[string, Control]>,
  fixedProps: Record<string, unknown> | undefined,
): string[] {
  const shown = (hit: string) => q(hit).slice(0, 80);
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

function exportedNames(file: string): string[] {
  const text = readFileSync(file, "utf8");
  return [...text.matchAll(/export\s+(?:default\s+)?(?:function|const)\s+([A-Za-z0-9_]+)/g)].map(
    (m) => m[1],
  );
}

function findElementFile(item: CuratedElement): string | null {
  const dir = path.join(VIDEO, "elements", REGISTRIES[item.registry].installDir(item));
  const candidates = [`${dir}/${item.name}.tsx`, `${dir}/${item.name}/index.tsx`];
  return candidates.find((c) => existsSync(c)) ?? null;
}

const pascal = (s: string) =>
  s.replace(/(^|[-_])([a-z0-9])/g, (_m, _p, c: string) => c.toUpperCase());

const titleCase = (s: string) =>
  s
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");

const declaresFontFamily = (source: string) =>
  /\bfontFamily\?\s*:/.test(source) && /^\s+fontFamily(\s*=[^=>]|\s*,|\s*$)/m.test(source);

const declaresSpeed = (source: string) => /\bspeed\?\s*:/.test(source);

function exportNameOf(
  item: CuratedElement,
  file: string,
  cfg: ParsedConfig | null,
): string | undefined {
  const exported = exportedNames(file);
  return (
    item.exportName ??
    (cfg?.componentName && exported.includes(cfg.componentName) ? cfg.componentName : undefined) ??
    exported.find((e) => e === pascal(item.name)) ??
    exported.find((e) => /^[A-Z]/.test(e))
  );
}

function controlsFor(
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

const kindOf = (item: CuratedElement): ElementKind =>
  item.kind ??
  (item.category === "transition"
    ? "presentation"
    : item.category === "effect"
      ? "wrapper"
      : "component");

async function registryDuration(item: CuratedElement): Promise<number | undefined> {
  const url = REGISTRIES[item.registry].durationsUrl;
  if (!url) return undefined;
  const durations = await previewDurations(url);
  return durations[item.name] ?? durations["*"];
}

const WRAPPERS: Array<[applies: (item: CuratedElement) => boolean, wrapper: string]> = [
  [(item) => !!item.screen, "fitScreen"],
  [(item) => !!item.center, "centered"],
  [(item) => !!item.atEnd, "playsAtEnd"],
  [(item) => !!item.paint, "painted"],
  [(item) => item.place === "spot", "placedAt"],
  [(item) => item.place === "points", "pointsFromSpots"],
];

type Ingested = { meta: string; loader: string; helpers: string[] };

async function ingest(item: CuratedElement, problems: string[]): Promise<Ingested | null> {
  const file = findElementFile(item);
  if (!file) {
    problems.push(`${item.registry}/${item.name}: not installed`);
    return null;
  }
  if (item.installOnly) return null;
  const reg = REGISTRIES[item.registry];
  const key = `${reg.keyPrefix}/${item.name}`;
  const cfg = item.noConfig || reg.propsUrl ? null : await fetchConfig(item, file);
  const description = await descriptionOf(item, key, problems);
  const exportName = exportNameOf(item, file, cfg);
  if (!exportName) {
    problems.push(`${item.registry}/${item.name}: no component export found`);
    return null;
  }
  const source = readFileSync(file, "utf8");
  const props = reg.propsUrl ? await propControls(item, file, exportName) : undefined;
  const controls = controlsFor(item, source, props ? props.controls : cfg?.controls, !!props);
  problems.push(...defaultProblems(key, controls, item.fixedProps));
  const fields = controls
    .map(([k, c]) => [k, zodFor(k, c)] as const)
    .filter((f): f is readonly [string, string] => f[1] != null)
    .map(([k, schema]) => `      ${/^[A-Za-z_$][\w$]*$/.test(k) ? k : q(k)}: ${schema},`);
  const kind = kindOf(item);
  const duration =
    item.durationInFrames ??
    cfg?.durationInFrames ??
    props?.durationInFrames ??
    (await registryDuration(item)) ??
    90;
  const meta = [
    `  {`,
    `    key: ${q(key)},`,
    `    title: ${q(item.title ?? titleCase(item.name))},`,
    ...(description ? [`    description: ${q(description)},`] : []),
    `    category: ${q(item.category)},`,
    `    kind: ${q(kind)},`,
    `    source: { registry: ${q(reg.label)}, url: ${q(`${reg.docs}/${item.name}`)}, license: ${q(reg.license)} },`,
    `    schema: z.object({`,
    ...fields,
    `    }),`,
    `    fixedProps: ${q(item.fixedProps ?? {})},`,
    `    defaultDurationInFrames: ${duration},`,
    `  },`,
  ].join("\n");
  const base = kind === "presentation" ? "presentation" : "component";
  const wrappers = WRAPPERS.filter(([applies]) => applies(item)).map(([, wrapper]) => wrapper);
  const impl = wrappers.reduce(
    (inner, wrapper) => `${wrapper}(${inner})`,
    `${base}(m.${exportName})`,
  );
  const rel =
    "../" +
    path
      .relative(VIDEO, file)
      .replace(/\\/g, "/")
      .replace(/(\/index)?\.tsx$/, "");
  return {
    meta,
    loader: `  ${q(key)}: () => import(${q(rel)}).then((m) => ${impl}),`,
    helpers: [base, ...wrappers],
  };
}

const renderMeta = (entries: string[]) =>
  [
    'import { zColor } from "@remotion/zod-types";',
    'import { z } from "zod";',
    'import { defineElementMeta, type ElementMetaInput } from "./types";',
    "",
    "const defs: ElementMetaInput[] = [",
    ...entries,
    "];",
    "",
    "export const generatedElementMeta = defs.map(defineElementMeta);",
    "",
  ].join("\n");

const renderLoaders = (loaders: string[], helpers: Set<string>) =>
  [
    `import { ${[...helpers].sort().join(", ")} } from "./framing";`,
    'import type { ElementImpl } from "./types";',
    "",
    "export const generatedElementLoaders: Record<string, () => Promise<ElementImpl>> = {",
    ...loaders,
    "};",
    "",
  ].join("\n");

async function generate(): Promise<void> {
  const problems: string[] = [...unknownOverrideNames()];
  const ingested: Ingested[] = [];
  for (const item of CURATED) {
    const result = await ingest(item, problems);
    if (result) ingested.push(result);
  }
  const helpers = new Set(ingested.flatMap((i) => i.helpers));
  for (const [file, text] of [
    [OUT_META, renderMeta(ingested.map((i) => i.meta))],
    [
      OUT_COMPONENTS,
      renderLoaders(
        ingested.map((i) => i.loader),
        helpers,
      ),
    ],
  ] as const) {
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, text);
    console.log(`wrote ${path.relative(ROOT, file)}`);
  }
  console.log(`${ingested.length} elements`);
  if (problems.length) {
    console.warn(`\n${problems.length} problem(s):\n  ${problems.join("\n  ")}`);
    process.exitCode = 1;
  }
}

if (!skipAdd) {
  for (const registry of Object.keys(REGISTRIES)) {
    if (only && registry !== only) continue;
    const items = CURATED.filter((i) => i.registry === registry);
    if (items.length) install(items);
  }
  moveInstalled();
  rewriteImports();
}
await generate();
