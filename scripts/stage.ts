import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { getCompositorPackage } from "@opsis/sidecar/compositor-package";
import { z } from "zod";
import {
  GPL_2,
  appVersion,
  installedRemotion,
  readCompositorSources,
  sourcesNote,
} from "./lib/compositor-sources";
import { writeChromiumCredits } from "./lib/engine/chromium-credits";
import { CHROMIUM_CREDITS_FILE, ENGINE_LICENCES_FOLDER, engineStatus } from "./lib/engine/paths";
import { ROOT } from "./lib/tool-config";

const RESOURCES = path.join(ROOT, "resources");
const STAGED_ENGINE = path.join(os.homedir(), ".cache", "opsis-stage-engine");
const SIDECAR = "opsis-render";

const run = (program: string, args: string[]) =>
  execFileSync(program, args, { cwd: ROOT, stdio: "inherit" });

const output = (program: string, args: string[]) =>
  execFileSync(program, args, { cwd: ROOT, encoding: "utf8" });

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

const sourceSchema = z.object({ from: z.array(z.string()), to: z.string() });
const layoutSchema = z.object({
  exportedSources: z.array(sourceSchema),
  dependencyManifests: z.array(sourceSchema),
});

function stageExportSources() {
  const layout = layoutSchema.parse(
    JSON.parse(
      readFileSync(
        path.join(ROOT, "crates", "render", "src", "source-layout.generated.json"),
        "utf8",
      ),
    ),
  );
  for (const source of [...layout.exportedSources, ...layout.dependencyManifests])
    cpSync(path.join(ROOT, ...source.from), path.join(RESOURCES, "export-src", source.to), {
      recursive: true,
      filter: (file) => !file.includes(`${path.sep}node_modules`),
    });
}

async function stageBrowserCredits() {
  run("cargo", [
    "run",
    "-q",
    "--release",
    "-p",
    "opsis-engine",
    "--bin",
    "install-engine",
    "--",
    STAGED_ENGINE,
  ]);
  const { complete, headlessShell } = engineStatus(STAGED_ENGINE);
  if (!complete || !headlessShell)
    throw new Error(`The engine in ${STAGED_ENGINE} isn't complete.`);
  const credits = await writeChromiumCredits(
    headlessShell,
    path.join(RESOURCES, ENGINE_LICENCES_FOLDER, CHROMIUM_CREDITS_FILE),
  );
  console.log(`Chromium ${credits.version} licences: ${credits.bytes} bytes`);
}

rmSync(RESOURCES, { recursive: true, force: true });
mkdirSync(RESOURCES, { recursive: true });
run("bun", ["run", "video:bundle", "--", "--release"]);
compileSidecar();
stageCompositor();
stageExportSources();
await stageBrowserCredits();
if (!existsSync(path.join(ROOT, "remotion-bundle")))
  throw new Error("The video bundle is missing.");
console.log(`Staged the app's resources in ${path.relative(ROOT, RESOURCES)}.`);
