import { execFileSync } from "node:child_process";
import path from "node:path";
import { ROOT } from "./tool-config";

const TOOLS = path.join(ROOT, "target", "release", "opsis-tools");

type ToolRun = { release?: boolean; env?: NodeJS.ProcessEnv };

export function buildTools(): void {
  execFileSync("cargo", ["build", "-q", "--release", "-p", "opsis-tools"], {
    cwd: ROOT,
    stdio: "inherit",
  });
}

export function runTool(
  args: string[],
  { release = true, env = process.env }: ToolRun = {},
): string {
  const profile = release ? ["--release"] : [];
  return execFileSync("cargo", ["run", "-q", ...profile, "-p", "opsis-tools", "--", ...args], {
    cwd: ROOT,
    env,
    encoding: "utf8",
    maxBuffer: 1 << 26,
    stdio: ["ignore", "pipe", "inherit"],
  });
}

export function editorCatalog(): unknown {
  return JSON.parse(execFileSync(TOOLS, ["catalog"], { encoding: "utf8", maxBuffer: 1 << 26 }));
}
