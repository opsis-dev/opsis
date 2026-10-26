import path from "node:path";
import { devUserData } from "./lib/app-data";
import { writeChromiumCredits } from "./lib/engine/chromium-credits";
import {
  CHROMIUM_CREDITS_FILE,
  ENGINE_LICENCES_FOLDER,
  engineStatus,
  userEngineDir,
} from "./lib/engine/paths";
import { ROOT } from "./lib/tool-config";

const engine = process.argv[2] ?? userEngineDir(devUserData());
const { complete, headlessShell } = engineStatus(engine);
if (!complete || !headlessShell) {
  console.error(`No engine in ${engine}. Set it up from Settings → Engine first.`);
  process.exit(1);
}

const output = path.join(ROOT, "resources", ENGINE_LICENCES_FOLDER, CHROMIUM_CREDITS_FILE);
const credits = await writeChromiumCredits(headlessShell, output);
console.log(
  `Saved the licences of Chromium ${credits.version} (${credits.bytes} bytes) to ${path.relative(ROOT, output)}. Settings → Licences shows them from there.`,
);
