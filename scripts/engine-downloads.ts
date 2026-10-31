import { execFileSync } from "node:child_process";
import { ROOT } from "./lib/tool-config";

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
    ...process.argv.slice(2),
  ],
  { cwd: ROOT, stdio: "inherit" },
);
