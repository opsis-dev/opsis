import { execFileSync } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { getCompositorPackage } from "@opsis/sidecar/compositor-package";
import {
  GPL_2,
  appVersion,
  installedRemotion,
  readCompositorSources,
  sourcesNote,
} from "./lib/compositor-sources";
import { installEngine, writeChromiumCredits } from "./lib/engine";
import { ROOT } from "./lib/tool-config";

const RESOURCES = path.join(ROOT, "resources");
const SIDECAR = "opsis-render";

function run(program: string, args: string[]) {
  return execFileSync(program, args, { cwd: ROOT, stdio: "inherit" });
}

function output(program: string, args: string[]) {
  return execFileSync(program, args, { cwd: ROOT, encoding: "utf8" });
}

function targetTriple(): string {
  const host = /^host: (.+)$/m.exec(output("rustc", ["-vV"]))?.[1];
  if (!host) throw new Error("rustc didn't name its host.");
  return host.trim();
}

const BUN_ARCH: Record<string, string> = { x86_64: "x64", aarch64: "arm64" };
const BUN_OS: Record<string, string> = { linux: "linux", darwin: "darwin", windows: "windows" };

function bunTarget(triple: string): string {
  const [arch] = triple.split("-");
  const system = Object.keys(BUN_OS).find((name) => triple.includes(name));
  if (!arch || !BUN_ARCH[arch] || !system) throw new Error(`Bun can't compile for ${triple}.`);
  const musl = triple.endsWith("-musl") ? "-musl" : "";
  return `bun-${BUN_OS[system]}-${BUN_ARCH[arch]}${musl}`;
}

function compileSidecar() {
  const triple = targetTriple();
  const out = path.join(RESOURCES, "binaries", `${SIDECAR}-${triple}`);
  run("bun", [
    "build",
    "--compile",
    `--target=${bunTarget(triple)}`,
    "--minify",
    path.join("apps", "sidecar", "src", "main.ts"),
    "--outfile",
    out,
  ]);
}

function stageCompositor() {
  const fromSidecar = createRequire(path.join(ROOT, "apps", "sidecar", "package.json"));
  const fromRenderer = createRequire(fromSidecar.resolve("@remotion/renderer/package.json"));
  const pkg = getCompositorPackage({ arch: process.arch, platform: process.platform });
  const folder = path.dirname(fromRenderer.resolve(`${pkg}/package.json`));
  const compositor = path.join(RESOURCES, "compositor");
  cpSync(folder, compositor, { recursive: true, dereference: true });
  const sources = readCompositorSources();
  if (sources.remotion !== installedRemotion())
    throw new Error(
      `licences/compositor-sources.json names Remotion ${sources.remotion}, but ${installedRemotion()} is installed.`,
    );
  copyFileSync(GPL_2, path.join(compositor, "COPYING.GPLv2"));
  writeFileSync(path.join(compositor, "SOURCES.md"), sourcesNote(sources, appVersion()));
}

function stageBrowserCredits() {
  writeChromiumCredits(
    installEngine().browser,
    path.join(RESOURCES, "engine-licences", "LICENSES.chromium.html"),
  );
}

rmSync(RESOURCES, { recursive: true, force: true });
mkdirSync(RESOURCES, { recursive: true });
run("bun", ["run", "video:bundle", "--", "--release"]);
compileSidecar();
stageCompositor();
stageBrowserCredits();
if (!existsSync(path.join(ROOT, "remotion-bundle")))
  throw new Error("The video bundle is missing.");
console.log(`Staged the app's resources in ${path.relative(ROOT, RESOURCES)}.`);
