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
import {
  CURATED,
  DESCRIPTIONS,
  REGISTRIES,
  type ControlSpec,
  type CuratedElement,
} from "./registry-ingest.list.ts";
import { humanize } from "../src/shared/format.ts";
import { walkFiles } from "./lib/walk.ts";

const ROOT = path.resolve(import.meta.dirname, "..");
const SRC = path.join(ROOT, "src");
const VIDEO = path.join(SRC, "video");
const OUT_META = path.join(SRC, "shared", "elements", "elements.generated.ts");
const OUT_COMPONENTS = path.join(VIDEO, "registry", "components.generated.ts");
const skipAdd = process.argv.includes("--no-add");
const onlyIndex = process.argv.indexOf("--only");
const only = onlyIndex === -1 ? undefined : process.argv[onlyIndex + 1];

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

function relocate(): void {
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

  const nsDirs = (sub: string) =>
    existsSync(path.join(VIDEO, sub)) ? readdirSync(path.join(VIDEO, sub)) : [];
  const elementNs = new Set(nsDirs("elements"));
  const libNs = new Set(nsDirs("lib"));
  const hookNs = new Set(nsDirs("hooks"));

  const rewrite = (spec: string): string => {
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
    const [, kind, ns, rest = ""] = m;
    if ((kind === "components" || kind === "elements") && elementNs.has(ns))
      return `@/video/elements/${ns}${rest}`;
    if (kind === "lib" && libNs.has(ns)) return `@/video/lib/${ns}${rest}`;
    if (kind === "lib" && ns === "utils") return "@/renderer/lib/utils";
    if (kind === "hooks" && hookNs.has(ns)) return `@/video/hooks/${ns}${rest}`;
    return spec;
  };

  for (const file of walkFiles(VIDEO)) {
    if (!/\.(tsx?|css)$/.test(file) || file.includes(`${path.sep}registry${path.sep}`)) continue;
    const before = readFileSync(file, "utf8");
    const after = before
      .replace(/^"use client";\s*\n/m, "")
      .replace(
        /(from\s+|import\s*\(\s*|import\s+)(["'])(@\/[^"']+)\2/g,
        (_all, lead: string, q: string, spec: string) => `${lead}${q}${rewrite(spec)}${q}`,
      );
    if (after !== before) writeFileSync(file, after);
  }
}

const project = new Project({ useInMemoryFileSystem: true });

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
    if (owner && typeof owner === "object" && !Array.isArray(owner))
      return (owner as Record<string, unknown>)[node.getName()];
  }
  return undefined;
}

type ParsedConfig = {
  controls: Record<string, Control>;
  durationInFrames?: number;
  componentName?: string;
};

function parseConfig(source: string, name: string): ParsedConfig | null {
  const sf = project.createSourceFile(`${name}-${Math.random()}.ts`, source, { overwrite: true });
  for (const decl of sf.getVariableDeclarations()) {
    const init = decl.getInitializer();
    if (!init || !Node.isObjectLiteralExpression(init)) continue;
    const controlsProp = init.getProperty("controls");
    if (!controlsProp || !Node.isPropertyAssignment(controlsProp)) continue;
    const controls = (literal(controlsProp.getInitializer()) ?? {}) as Record<string, Control>;
    const dur = init.getProperty("durationInFrames");
    const comp = init.getProperty("componentName");
    return {
      controls: Object.fromEntries(
        Object.entries(controls).filter(
          ([, c]) => c && typeof c === "object" && typeof c.type === "string",
        ),
      ),
      durationInFrames:
        dur && Node.isPropertyAssignment(dur)
          ? (literal(dur.getInitializer()) as number | undefined)
          : undefined,
      componentName:
        comp && Node.isPropertyAssignment(comp)
          ? (literal(comp.getInitializer()) as string | undefined)
          : undefined,
    };
  }
  return null;
}

const LIBRARY_WORDS = /\b(remotion|remotion-?ui|remocn|snapcn|snap-cn|shadcn)\b/i;

async function fetchDescription(item: CuratedElement): Promise<string | undefined> {
  try {
    const res = await fetch(REGISTRIES[item.registry].itemUrl(item.name));
    if (!res.ok) return undefined;
    const json = (await res.json()) as { description?: string };
    const sentences = (json.description ?? "")
      .split(/(?<=[.!?])\s+/)
      .map((s) => s.trim())
      .filter((s) => s && !LIBRARY_WORDS.test(s));
    return sentences.slice(0, 2).join(" ") || undefined;
  } catch {
    return undefined;
  }
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

type PropDoc = {
  name: string;
  type: string;
  default?: string;
  required?: boolean;
  description?: string;
};

async function fetchPropDocs(url: string): Promise<PropDoc[]> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  return ((await res.json()) as { props?: PropDoc[] }).props ?? [];
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
        const sf = project.createSourceFile(`preview-${Math.random()}.ts`, text);
        const table = literal(sf.getVariableDeclaration("PREVIEW_META")?.getInitializer(), sf);
        const fallback = literal(
          sf.getVariableDeclaration("PREVIEW_DEFAULTS")?.getInitializer(),
          sf,
        ) as { durationInFrames?: number } | undefined;
        const out: Record<string, number> = { "*": fallback?.durationInFrames ?? 120 };
        for (const [k, v] of Object.entries((table ?? {}) as Record<string, unknown>)) {
          const d = (v as { durationInFrames?: unknown } | undefined)?.durationInFrames;
          if (typeof d === "number") out[k] = d;
        }
        return out;
      });
    durationTables.set(url, pending);
  }
  return pending;
}

function sourceDefaults(source: string, exportName: string): Map<string, unknown> {
  const sf = project.createSourceFile(`src-${Math.random()}.tsx`, source);
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
    if (name === "speed")
      return { ...base, type: "number", default: value, min: 0.25, max: 4, step: 0.25 };
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

function libraryWordsIn(value: unknown): string[] {
  if (typeof value === "string") return LIBRARY_WORDS.test(value) ? [value] : [];
  if (Array.isArray(value)) return value.flatMap(libraryWordsIn);
  if (value && typeof value === "object") return Object.values(value).flatMap(libraryWordsIn);
  return [];
}

function remoteRefsIn(value: unknown): string[] {
  if (typeof value === "string") return /^https?:\/\//.test(value) ? [value] : [];
  if (Array.isArray(value)) return value.flatMap(remoteRefsIn);
  if (value && typeof value === "object") return Object.values(value).flatMap(remoteRefsIn);
  return [];
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

async function generate(): Promise<void> {
  const entries: string[] = [];
  const componentEntries: string[] = [];
  const framing = new Set<string>();
  const problems: string[] = [];

  for (const item of CURATED) {
    const file = findElementFile(item);
    if (!file) {
      problems.push(`${item.registry}/${item.name}: not installed`);
      continue;
    }
    if (item.installOnly) continue;
    const reg = REGISTRIES[item.registry];
    const cfg = item.noConfig || reg.propsUrl ? null : await fetchConfig(item, file);
    const description =
      item.description ?? DESCRIPTIONS[item.name] ?? (await fetchDescription(item));
    const exported = exportedNames(file);
    const exportName =
      item.exportName ??
      (cfg?.componentName && exported.includes(cfg.componentName)
        ? cfg.componentName
        : undefined) ??
      exported.find((e) => e === pascal(item.name)) ??
      exported.find((e) => /^[A-Z]/.test(e));
    if (!exportName) {
      problems.push(`${item.registry}/${item.name}: no component export found`);
      continue;
    }
    const key = `${reg.keyPrefix}/${item.name}`;
    const rel =
      "../" +
      path
        .relative(VIDEO, file)
        .replace(/\\/g, "/")
        .replace(/(\/index)?\.tsx$/, "");

    const source = readFileSync(file, "utf8");
    const props = reg.propsUrl ? await propControls(item, file, exportName) : undefined;
    const controls: Record<string, Control> = {
      ...(props ? props.controls : cfg?.controls),
      ...item.controls,
    };
    if (/\bfontFamily\?\s*:/.test(source) && /^\s+fontFamily(\s*=[^=>]|\s*,|\s*$)/m.test(source))
      controls.fontFamily = FONT_CONTROL;
    else if (!props) delete controls.fontFamily;
    if (!props)
      for (const [k, v] of Object.entries(item.defaults ?? {}))
        if (controls[k]) controls[k] = { ...controls[k], default: v };
    if (
      /\bspeed\?\s*:/.test(source) &&
      !controls.speed &&
      item.category !== "transition" &&
      !(item.omit ?? []).includes("speed")
    ) {
      controls.speed = {
        type: "number",
        default: 1,
        min: 0.25,
        max: 4,
        step: 0.25,
        description: "Speed",
      };
    }
    const kept = Object.entries(controls).filter(([k]) => !(item.omit ?? []).includes(k));
    for (const [k, c] of kept) {
      for (const hit of libraryWordsIn(c.default))
        problems.push(`${key}: default for "${k}" names a library: ${q(hit).slice(0, 80)}`);
      for (const hit of remoteRefsIn(c.default))
        problems.push(`${key}: default for "${k}" loads from the internet: ${q(hit).slice(0, 80)}`);
    }
    for (const hit of [...libraryWordsIn(item.fixedProps), ...remoteRefsIn(item.fixedProps)])
      problems.push(
        `${key}: fixed prop names a library or loads from the internet: ${q(hit).slice(0, 80)}`,
      );
    const fields = kept
      .map(([k, c]) => [k, zodFor(k, c)] as const)
      .filter((f): f is readonly [string, string] => f[1] != null)
      .map(([k, z]) => `      ${/^[A-Za-z_$][\w$]*$/.test(k) ? k : q(k)}: ${z},`);

    const kind =
      item.kind ??
      (item.category === "transition"
        ? "presentation"
        : item.category === "effect"
          ? "wrapper"
          : "component");
    const durations = reg.durationsUrl ? await previewDurations(reg.durationsUrl) : undefined;
    const duration =
      item.durationInFrames ??
      cfg?.durationInFrames ??
      props?.durationInFrames ??
      (durations ? (durations[item.name] ?? durations["*"]) : undefined) ??
      90;
    entries.push(
      [
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
      ].join("\n"),
    );
    let impl = `m.${exportName} as unknown as ElementImpl`;
    if (item.atEnd) {
      impl = `playsAtEnd(m.${exportName} as unknown as ElementComponent)`;
      framing.add("playsAtEnd");
    } else if (item.paint) {
      impl = `painted(m.${exportName} as unknown as ElementComponent)`;
      framing.add("painted");
    } else if (item.place) {
      const wrap = item.place === "points" ? "pointsFromSpots" : "placedAt";
      impl = `${wrap}(m.${exportName} as unknown as ElementComponent)`;
      framing.add(wrap);
    } else if (item.screen || item.center) {
      impl = `m.${exportName} as unknown as ElementComponent`;
      if (item.screen) {
        impl = `fitScreen(${impl})`;
        framing.add("fitScreen");
      }
      if (item.center) {
        impl = `centered(${impl})`;
        framing.add("centered");
      }
    }
    componentEntries.push(`  ${q(key)}: () => import(${q(rel)}).then((m) => ${impl}),`);
  }

  const meta = [
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
  const components = [
    ...(framing.size
      ? [`import { ${[...framing].sort().join(", ")}, type ElementComponent } from "./framing";`]
      : []),
    'import type { ElementImpl } from "./types";',
    "",
    "export const generatedElementLoaders: Record<string, () => Promise<ElementImpl>> = {",
    ...componentEntries,
    "};",
    "",
  ].join("\n");
  for (const [file, text] of [
    [OUT_META, meta],
    [OUT_COMPONENTS, components],
  ] as const) {
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, text);
    console.log(`wrote ${path.relative(ROOT, file)}`);
  }
  console.log(`${entries.length} elements`);
  if (problems.length) {
    console.warn(`\n${problems.length} problem(s):\n  ${problems.join("\n  ")}`);
    process.exitCode = 1;
  }
}

const titleCase = (s: string) =>
  s
    .split("-")
    .map((w) => w[0].toUpperCase() + w.slice(1))
    .join(" ");

if (!skipAdd) {
  for (const registry of Object.keys(REGISTRIES)) {
    if (only && registry !== only) continue;
    const items = CURATED.filter((i) => i.registry === registry);
    if (items.length) install(items);
  }
  relocate();
}
await generate();
