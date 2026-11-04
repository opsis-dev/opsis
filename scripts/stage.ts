import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import { installEngine, writeChromiumCredits } from "./lib/engine";
import { FFMPEG_LICENCE, stageFfmpeg } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";

const RESOURCES = path.join(ROOT, "resources");

function stageBrowserCredits() {
  return writeChromiumCredits(
    installEngine().browser,
    path.join(RESOURCES, "engine-licences", "LICENSES.chromium.html"),
  );
}

rmSync(RESOURCES, { recursive: true, force: true });
mkdirSync(RESOURCES, { recursive: true });
execFileSync("bun", ["run", "film:bundle", "--mode", "release"], { cwd: ROOT, stdio: "inherit" });
execFileSync("bun", ["run", "terminal:bundle", "--mode", "release"], {
  cwd: ROOT,
  stdio: "inherit",
});
await stageBrowserCredits();
const ffmpeg = await stageFfmpeg();
if (!existsSync(path.join(ROOT, "film-bundle", "index.html")))
  throw new Error("The film page is missing.");
if (!existsSync(path.join(ROOT, "terminal-bundle", "index.html")))
  throw new Error("The terminal page is missing.");
if (!existsSync(ffmpeg) || !existsSync(FFMPEG_LICENCE))
  throw new Error("The bundled FFmpeg or its licence is missing.");
console.log(`Staged the app's resources in ${path.relative(ROOT, RESOURCES)}.`);
