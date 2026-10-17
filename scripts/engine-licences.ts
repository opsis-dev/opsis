import { readFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { z } from "zod";
import { writeChromiumCredits } from "../src/engine/chromium-credits";
import { CHROMIUM_CREDITS_FILE, ENGINE_LICENCES_FOLDER, engineStatus } from "../src/engine/paths";

const ROOT = path.resolve(import.meta.dirname, "..");

const appName = z
  .object({ productName: z.string() })
  .parse(JSON.parse(readFileSync(path.join(ROOT, "package.json"), "utf8"))).productName;

function electronAppData(): string {
  if (process.platform === "darwin")
    return path.join(os.homedir(), "Library", "Application Support");
  if (process.platform === "win32")
    return process.env.APPDATA ?? path.join(os.homedir(), "AppData", "Roaming");
  return process.env.XDG_CONFIG_HOME ?? path.join(os.homedir(), ".config");
}

const engine = process.argv[2] ?? path.join(electronAppData(), appName, "engine");
const { complete, headlessShell } = engineStatus(engine);
if (!complete || !headlessShell) {
  console.error(`No engine in ${engine}. Set it up from Settings → Engine first.`);
  process.exit(1);
}

const output = path.join(ROOT, "resources", ENGINE_LICENCES_FOLDER, CHROMIUM_CREDITS_FILE);
const credits = await writeChromiumCredits(headlessShell, output);
console.log(
  `Saved the licences of Chromium ${credits.version} (${credits.bytes} bytes) to ${path.relative(ROOT, output)}. The app copies them into the engine folder when it starts.`,
);
