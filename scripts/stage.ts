import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import { installEngine, writeChromiumCredits } from "./lib/engine";
import { ROOT } from "./lib/tool-config";

const RESOURCES = path.join(ROOT, "resources");

function stageBrowserCredits() {
  writeChromiumCredits(
    installEngine().browser,
    path.join(RESOURCES, "engine-licences", "LICENSES.chromium.html"),
  );
}

rmSync(RESOURCES, { recursive: true, force: true });
mkdirSync(RESOURCES, { recursive: true });
execFileSync("bun", ["run", "film:bundle", "--mode", "release"], { cwd: ROOT, stdio: "inherit" });
stageBrowserCredits();
if (!existsSync(path.join(ROOT, "film-bundle", "index.html")))
  throw new Error("The film page is missing.");
console.log(`Staged the app's resources in ${path.relative(ROOT, RESOURCES)}.`);
