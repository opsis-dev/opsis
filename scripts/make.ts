import { execFileSync } from "node:child_process";
import path from "node:path";
import { ROOT } from "./lib/tool-config";

execFileSync("bun", ["run", "stage"], { cwd: ROOT, stdio: "inherit" });
execFileSync("tauri", ["build", "--config", "tauri.release.conf.json", ...process.argv.slice(2)], {
  cwd: path.join(ROOT, "crates", "desktop"),
  stdio: "inherit",
});
