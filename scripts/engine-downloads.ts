import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import path from "node:path";
import { ROOT } from "./lib/tool-config";

type ChromeUrl = (o: { platform: string; version: null; chromeMode: "headless-shell" }) => string;

const require = createRequire(path.join(ROOT, "apps", "sidecar", "package.json"));
const dir = path.dirname(require.resolve("@remotion/renderer/package.json"));
const remotion = require(path.join(dir, "dist", "browser", "get-chrome-download-url.js")) as {
  TESTED_VERSION: string;
  getChromeDownloadUrl: ChromeUrl;
};
const macArm = remotion.getChromeDownloadUrl({
  platform: "mac-arm64",
  version: null,
  chromeMode: "headless-shell",
});

execFileSync(
  "cargo",
  [
    "run",
    "-q",
    "--release",
    "-p",
    "opsis-engine",
    "--bin",
    "pin-downloads",
    "--",
    remotion.TESTED_VERSION,
    macArm,
  ],
  { cwd: ROOT, stdio: "inherit" },
);
