import path from "node:path";
import { installEngine, writeChromiumCredits } from "./lib/engine";
import { ROOT } from "./lib/tool-config";

await writeChromiumCredits(
  installEngine().browser,
  path.join(ROOT, "resources", "engine-licences", "LICENSES.chromium.html"),
);
