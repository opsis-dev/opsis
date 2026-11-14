import { execFileSync } from "node:child_process";
import { homedir } from "node:os";
import path from "node:path";
import { hostTarget } from "./lib/ffmpeg";
import { verifiedDownload } from "./lib/pins";
import { setupFfmpeg, setupFilament, setupKddw, setupNpm, setupZig } from "./lib/setup/downloads";
import { readPins } from "./lib/setup/manifest";
import { pin } from "./lib/setup/pin";
import { downloadsDir, prune, recipeHash, step, writePaths } from "./lib/setup/store";
import { checkSystem } from "./lib/setup/system";
import { SKIA_RECIPE, buildSkia } from "./lib/skia";
import { buildStaticQt, configureArgs } from "./lib/static-qt";
import { ROOT } from "./lib/tool-config";

const NO_ENGINE = "--no-engine";
const PRUNE = "--prune";
const ENGINE_DIR = path.join(homedir(), ".config", "Opsis", "engine");
const QT_RECIPE = path.join(ROOT, "scripts", "lib", "static-qt.ts");
const SKIA_SCRIPT = path.join(ROOT, "scripts", "lib", "skia.ts");

function qtPlatform(): "linux" | "darwin" | "win32" {
  const platform = process.platform;
  if (platform === "linux" || platform === "darwin" || platform === "win32") return platform;
  throw new Error(`setup doesn't build for ${platform}.`);
}

function installEngine(zig: string): string {
  execFileSync(
    "cargo",
    ["run", "-q", "-p", "opsis-engine", "--bin", "install-engine", "--", ENGINE_DIR],
    { cwd: ROOT, stdio: "inherit", env: { ...process.env, ZIG: zig } },
  );
  return ENGINE_DIR;
}

async function setup(args: string[]): Promise<void> {
  checkSystem();
  const pins = readPins();
  const zig = await setupZig(pins);
  const kddw = await setupKddw(pins);
  const filament = await setupFilament(pins);
  const ffmpeg = await setupFfmpeg(pins, hostTarget());
  const npm = await setupNpm(pins);
  const qt = await step(
    "qt",
    recipeHash(["qt", pins.qt, configureArgs(qtPlatform())], [QT_RECIPE]),
    async (dir) => {
      const tarballs = [];
      for (const source of pins.qt.sources)
        tarballs.push(await verifiedDownload(source, downloadsDir()));
      buildStaticQt({
        version: pins.qt.version,
        tarballs,
        dir,
        mounts: [downloadsDir()],
      });
    },
  );
  const skia = await step(
    "skia",
    recipeHash(["skia", pins.skia, SKIA_RECIPE], [SKIA_SCRIPT]),
    (dir) => buildSkia(dir),
  );
  const paths = {
    zig: zig.path,
    qt: path.join(qt, "prefix"),
    kddw: kddw.path,
    skia,
    filament: filament.path,
    ffmpeg: ffmpeg.ffmpeg,
    ffprobe: ffmpeg.ffprobe,
    ffmpegLicence: ffmpeg.licence,
    npm: npm.paths,
  };
  writePaths(paths);
  if (!args.includes(NO_ENGINE)) writePaths({ ...paths, engine: installEngine(zig.path) });
  if (args.includes(PRUNE))
    prune([zig.dir, kddw.dir, filament.dir, ffmpeg.dir, qt, skia, ...npm.dirs]);
  console.log("setup: done");
}

const args = process.argv.slice(2);
if (args[0] === "pin") await pin(args.slice(1));
else await setup(args);
