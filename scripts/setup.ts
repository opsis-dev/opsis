import { homedir } from "node:os";
import path from "node:path";
import { hostTarget } from "./lib/ffmpeg";
import { FILAMENT_RECIPE, buildFilament, installedFilament } from "./lib/filament";
import { verifiedDownload } from "./lib/pins";
import { setupFfmpeg, setupKddw, setupNpm, setupZig } from "./lib/setup/downloads";
import { readPins } from "./lib/setup/manifest";
import { pin } from "./lib/setup/pin";
import { downloadsDir, prune, recipeHash, step, writePaths } from "./lib/setup/store";
import { checkSystem } from "./lib/setup/system";
import { SKIA_RECIPE, buildSkia } from "./lib/skia";
import { buildStaticQt, configureArgs } from "./lib/static-qt";
import { ROOT } from "./lib/tool-config";
import { runTool } from "./lib/tools";

const NO_ENGINE = "--no-engine";
const PRUNE = "--prune";
const CONTAINER = "--container";
const ENGINE_DIR = path.join(homedir(), ".config", "Opsis", "engine");
const QT_RECIPE = path.join(ROOT, "scripts", "lib", "static-qt.ts");
const SKIA_SCRIPT = path.join(ROOT, "scripts", "lib", "skia.ts");
const FILAMENT_SCRIPT = path.join(ROOT, "scripts", "lib", "filament.ts");
const GIT_SOURCE_SCRIPT = path.join(ROOT, "scripts", "lib", "git-source.ts");

function qtPlatform(): "linux" | "darwin" | "win32" {
  const platform = process.platform;
  if (platform === "linux" || platform === "darwin" || platform === "win32") return platform;
  throw new Error(`setup doesn't build for ${platform}.`);
}

function installEngine(zig: string): string {
  runTool(["install-engine", ENGINE_DIR], { release: false, env: { ...process.env, ZIG: zig } });
  return ENGINE_DIR;
}

function inContainer(args: string[]): boolean {
  if (!args.includes(CONTAINER)) return false;
  if (process.platform !== "linux")
    throw new Error(`${CONTAINER} builds Qt and Skia in a Linux container; this isn't Linux.`);
  return true;
}

async function setup(args: string[]): Promise<void> {
  const container = inContainer(args);
  checkSystem(container);
  const pins = readPins();
  const zig = await setupZig(pins);
  const kddw = await setupKddw(pins);
  const ffmpeg = await setupFfmpeg(pins, hostTarget());
  const npm = await setupNpm(pins);
  const qt = await step(
    "qt",
    recipeHash(["qt", pins.qt, configureArgs(qtPlatform()), { container }], [QT_RECIPE]),
    async (dir) => {
      const tarballs = [];
      for (const source of pins.qt.sources)
        tarballs.push(await verifiedDownload(source, downloadsDir()));
      buildStaticQt({
        version: pins.qt.version,
        tarballs,
        dir,
        mounts: [downloadsDir()],
        container,
      });
    },
  );
  const skia = await step(
    "skia",
    recipeHash(["skia", pins.skia, SKIA_RECIPE, { container }], [SKIA_SCRIPT, GIT_SOURCE_SCRIPT]),
    (dir) => buildSkia(dir, container),
  );
  const filament = await step(
    "filament",
    recipeHash(
      ["filament", pins.filament, FILAMENT_RECIPE, { container }],
      [FILAMENT_SCRIPT, GIT_SOURCE_SCRIPT],
    ),
    (dir) => {
      buildFilament(dir, container);
    },
  );
  const paths = {
    zig: zig.path,
    qt: path.join(qt, "prefix"),
    kddw: kddw.path,
    skia,
    filament: installedFilament(filament),
    ffmpeg: ffmpeg.ffmpeg,
    ffprobe: ffmpeg.ffprobe,
    ffmpegLicence: ffmpeg.licence,
    npm: npm.paths,
  };
  writePaths(paths);
  if (!args.includes(NO_ENGINE)) writePaths({ ...paths, engine: installEngine(zig.path) });
  if (args.includes(PRUNE)) prune([zig.dir, kddw.dir, filament, ffmpeg.dir, qt, skia, ...npm.dirs]);
  console.log("setup: done");
}

const args = process.argv.slice(2);
if (args[0] === "pin") await pin(args.slice(1));
else await setup(args);
