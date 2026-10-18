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
import type { CuratedElement } from "../registry-ingest.list";
import { ROOT, SRC_DIR, VIDEO_DIR } from "./tool-config";
import { walkFiles } from "./walk";

export function install(items: CuratedElement[]): void {
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

export function moveInstalled(): void {
  for (const [from, to] of MOVES) {
    const fromDir = path.join(SRC_DIR, from);
    if (!existsSync(fromDir)) continue;
    for (const ns of readdirSync(fromDir)) {
      const src = path.join(fromDir, ns);
      const dest = path.join(VIDEO_DIR, to, ns);
      mkdirSync(path.dirname(dest), { recursive: true });
      if (statSync(src).isDirectory()) moveTree(src, dest);
      else renameSync(src, dest);
    }
    rmSync(fromDir, { recursive: true, force: true });
  }

  for (const [from, to] of FLAT_MOVES) {
    const fromDir = path.join(SRC_DIR, from);
    if (existsSync(fromDir)) moveTree(fromDir, path.join(VIDEO_DIR, to));
  }
  for (const dir of ["remotion", "compositions"]) {
    const left = walkFiles(path.join(SRC_DIR, dir));
    if (left.length)
      throw new Error(`Unexpected files left after relocating:\n  ${left.join("\n  ")}`);
    rmSync(path.join(SRC_DIR, dir), { recursive: true, force: true });
  }
}

type Namespaces = { elements: Set<string>; lib: Set<string>; hooks: Set<string> };

function installedNamespaces(): Namespaces {
  const nsDirs = (sub: string) =>
    new Set(existsSync(path.join(VIDEO_DIR, sub)) ? readdirSync(path.join(VIDEO_DIR, sub)) : []);
  return { elements: nsDirs("elements"), lib: nsDirs("lib"), hooks: nsDirs("hooks") };
}

function rewriteSpec(spec: string, ns: Namespaces): string {
  if (spec === "@/lib/utils") return "@/renderer/lib/utils";
  if (spec.startsWith("@/components/ui/"))
    return spec.replace("@/components/ui/", "@/renderer/components/ui/");
  const ui = REMOTION_UI_SPEC.exec(spec);
  if (ui) {
    const kind = ui[1] ?? ui[2];
    const base = kind === "lib" || kind === "hooks" ? kind : "elements";
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

export function rewriteImports(): void {
  const ns = installedNamespaces();
  for (const file of walkFiles(VIDEO_DIR)) {
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
