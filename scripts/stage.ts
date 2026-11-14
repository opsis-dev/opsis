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
await stageBrowserCredits();
const ffmpeg = await stageFfmpeg();
if (!existsSync(ffmpeg) || !existsSync(FFMPEG_LICENCE))
  throw new Error("The bundled FFmpeg or its licence is missing.");
console.log(`Staged the app's resources in ${path.relative(ROOT, RESOURCES)}.`);
