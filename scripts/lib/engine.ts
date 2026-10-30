import { execFileSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { z } from "zod";
import { ROOT } from "./tool-config";

export const STAGED_ENGINE = path.join(os.homedir(), ".cache", "opsis-stage-engine");

const installedSchema = z.object({ browser: z.string(), ffmpeg: z.string() });

export type InstalledEngine = z.infer<typeof installedSchema>;

function cargoRun(crate: string, bin: string, args: string[]): string {
  return execFileSync(
    "cargo",
    ["run", "-q", "--release", "-p", crate, "--bin", bin, "--", ...args],
    {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "inherit"],
    },
  );
}

export function installEngine(dir = STAGED_ENGINE): InstalledEngine {
  const lines = cargoRun("opsis-engine", "install-engine", [dir]).trim().split("\n");
  return installedSchema.parse(JSON.parse(lines.at(-1) ?? "{}"));
}

export function writeChromiumCredits(browser: string, output: string): void {
  process.stdout.write(cargoRun("opsis-capture", "chromium-credits", [browser, output]));
}
