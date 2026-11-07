import { execFileSync } from "node:child_process";
import { ROOT } from "./lib/tool-config";

execFileSync("bun", ["run", "stage"], { cwd: ROOT, stdio: "inherit" });
execFileSync("cargo", ["build", "--release", "-p", "opsis-ui", ...process.argv.slice(2)], {
  cwd: ROOT,
  stdio: "inherit",
});
console.log("Built target/release/opsis. Installers come with the packaging step.");
