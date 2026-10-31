import { execFileSync } from "node:child_process";
import path from "node:path";
import { withFfmpeg } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";

execFileSync("tauri", ["dev", ...process.argv.slice(2)], {
  cwd: path.join(ROOT, "crates", "desktop"),
  stdio: "inherit",
  env: await withFfmpeg(),
});
