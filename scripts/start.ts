import { execFileSync } from "node:child_process";
import { withFfmpeg } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";

execFileSync("cargo", ["build", "-p", "opsis-effects", "--bin", "opsis-renderer"], {
  cwd: ROOT,
  stdio: "inherit",
});
execFileSync("cargo", ["run", "-p", "opsis-ui", "--bin", "opsis", "--", ...process.argv.slice(2)], {
  cwd: ROOT,
  stdio: "inherit",
  env: await withFfmpeg(),
});
