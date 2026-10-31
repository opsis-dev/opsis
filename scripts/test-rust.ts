import { execFileSync } from "node:child_process";
import { withFfmpeg } from "./lib/ffmpeg";
import { ROOT } from "./lib/tool-config";

const args = process.argv.slice(2);

execFileSync("cargo", ["test", ...(args.length > 0 ? args : ["--workspace"])], {
  cwd: ROOT,
  stdio: "inherit",
  env: await withFfmpeg(),
});
