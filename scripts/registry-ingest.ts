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
import { Node, Project, SyntaxKind, type Expression } from "ts-morph";
import { CURATED, DESCRIPTIONS, REGISTRIES, type CuratedElement } from "./registry-ingest.list.ts";

const ROOT = path.resolve(import.meta.dirname, "..");
const SRC = path.join(ROOT, "src");
const VIDEO = path.join(SRC, "video");
const OUT_META = path.join(SRC, "shared", "elements", "elements.generated.ts");
const OUT_COMPONENTS = path.join(VIDEO, "registry", "components.generated.ts");
const skipAdd = process.argv.includes("--no-add");

type Control = {
  type: string;
  default?: unknown;
  min?: number;
  max?: number;
  step?: number;
  label?: string;
  description?: string;
  options?: string[];
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

function walk(dir: string, out: string[] = []): string[] {
  if (!existsSync(dir)) return out;
  for (const entry of readdirSync(dir)) {
    const p = path.join(dir, entry);
    if (statSync(p).isDirectory()) walk(p, out);
    else out.push(p);
  }
  return out;
}

function relocate(): void {
  for (const [from, to] of MOVES) {
    const fromDir = path.join(SRC, from);
    if (!existsSync(fromDir)) continue;
    for (const ns of readdirSync(fromDir)) {
      const src = path.join(fromDir, ns);
      const dest = path.join(VIDEO, to, ns);
      mkdirSync(path.dirname(dest), { recursive: true });
      if (statSync(src).isDirectory()) {
        for (const file of walk(src)) {
          const target = path.join(dest, path.relative(src, file));
          mkdirSync(path.dirname(target), { recursive: true });
          renameSync(file, target);
        }
      } else {
        renameSync(src, dest);
      }
    }
    rmSync(fromDir, { recursive: true, force: true });
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

  for (const file of walk(VIDEO)) {
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

function literal(node: Expression | undefined): unknown {
  if (!node) return undefined;
  if (Node.isStringLiteral(node) || Node.isNoSubstitutionTemplateLiteral(node))
    return node.getLiteralValue();
  if (Node.isNumericLiteral(node)) return node.getLiteralValue();
  if (Node.isPrefixUnaryExpression(node) && node.getOperatorToken() === SyntaxKind.MinusToken) {
    const v = literal(node.getOperand() as Expression);
    return typeof v === "number" ? -v : undefined;
  }
  if (node.getKind() === SyntaxKind.TrueKeyword) return true;
  if (node.getKind() === SyntaxKind.FalseKeyword) return false;
  if (
    Node.isAsExpression(node) ||
    Node.isSatisfiesExpression(node) ||
    Node.isParenthesizedExpression(node)
  )
    return literal(node.getExpression());
  if (Node.isArrayLiteralExpression(node)) return node.getElements().map((e) => literal(e));
  if (Node.isObjectLiteralExpression(node)) {
    const out: Record<string, unknown> = {};
    for (const prop of node.getProperties()) {
      if (Node.isPropertyAssignment(prop)) {
        const name = prop
          .getNameNode()
          .getText()
          .replace(/^["']|["']$/g, "");
        out[name] = literal(prop.getInitializer());
      }
    }
    return out;
  }
  if (Node.isCallExpression(node) && node.getExpression().getText() === "enumVariants") {
    const arr = literal(node.getArguments()[0] as Expression);
    if (Array.isArray(arr)) return Object.fromEntries(arr.map((v) => [String(v), {}]));
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

const LIBRARY_WORDS = /\b(remotion|remocn|snapcn|shadcn)\b/i;

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

const q = (v: unknown) => JSON.stringify(v);

function zodFor(key: string, c: Control): string | null {
  const desc = q(c.description ?? c.label ?? key);
  const m = (extra = "") => `.meta({ description: ${desc}${extra} })`;
  switch (c.type) {
    case "text":
    case "text-content":
      return `z.string().default(${q(String(c.default ?? ""))})${m(String(c.default ?? "").includes("\\n") || String(c.default ?? "").length > 60 ? ', widget: "textarea"' : "")}`;
    case "image":
      return `z.string().default(${q(String(c.default ?? ""))})${m(', widget: "asset"')}`;
    case "number":
    case "number-input": {
      let s = "z.number()";
      if (typeof c.min === "number") s += `.min(${c.min})`;
      if (typeof c.max === "number") s += `.max(${c.max})`;
      s += `.default(${typeof c.default === "number" ? c.default : 0})`;
      return s + m(typeof c.step === "number" ? `, step: ${c.step}` : "");
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
    default:
      return null;
  }
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
  const problems: string[] = [];

  for (const item of CURATED) {
    const file = findElementFile(item);
    if (!file) {
      problems.push(`${item.registry}/${item.name}: not installed`);
      continue;
    }
    const cfg = item.noConfig ? null : await fetchConfig(item, file);
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
    const reg = REGISTRIES[item.registry];
    const key = `${reg.keyPrefix}/${item.name}`;
    const rel =
      "../" +
      path
        .relative(VIDEO, file)
        .replace(/\\/g, "/")
        .replace(/(\/index)?\.tsx$/, "");

    const source = readFileSync(file, "utf8");
    const controls: Record<string, Control> = { ...cfg?.controls, ...item.controls };
    if (/\bspeed\?\s*:/.test(source) && !controls.speed && item.category !== "transition") {
      controls.speed = {
        type: "number",
        default: 1,
        min: 0.25,
        max: 4,
        step: 0.25,
        description: "Speed",
      };
    }
    const fields = Object.entries(controls)
      .filter(([k]) => !(item.omit ?? []).includes(k))
      .map(([k, c]) => [k, zodFor(k, c)] as const)
      .filter((f): f is readonly [string, string] => f[1] != null)
      .map(([k, z]) => `      ${/^[A-Za-z_$][\w$]*$/.test(k) ? k : q(k)}: ${z},`);

    const kind =
      item.category === "transition"
        ? "presentation"
        : item.category === "effect"
          ? "wrapper"
          : "component";
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
        `    defaultDurationInFrames: ${item.durationInFrames ?? cfg?.durationInFrames ?? 90},`,
        `  },`,
      ].join("\n"),
    );
    componentEntries.push(
      `  ${q(key)}: () => import(${q(rel)}).then((m) => m.${exportName} as unknown as ElementImpl),`,
    );
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
    const items = CURATED.filter((i) => i.registry === registry);
    if (items.length) install(items);
  }
  relocate();
}
await generate();
