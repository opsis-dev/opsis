import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { z } from "zod";
import { cargoRun } from "./engine";
import { ROOT } from "./tool-config";

export const FFMPEG_LICENCE = path.join(ROOT, "resources", "LICENSE.ffmpeg.txt");

const BINARIES = path.join(ROOT, "crates", "desktop", "binaries");

const stagedSchema = z.object({ ffmpeg: z.string(), licence: z.string() });

export function hostTarget(): string {
  const info = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const host = info
    .split("\n")
    .find((line) => line.startsWith("host: "))
    ?.slice("host: ".length)
    .trim();
  if (!host) throw new Error("rustc didn't name the target it builds for.");
  return host;
}

export function stagedFfmpeg(target = hostTarget()): string {
  const exe = target.includes("-windows-") ? ".exe" : "";
  return path.join(BINARIES, `opsis-ffmpeg-${target}${exe}`);
}

export function stageFfmpeg(target = hostTarget()): string {
  const lines = cargoRun("opsis-engine", "stage-ffmpeg", [target]).trim().split("\n");
  return stagedSchema.parse(JSON.parse(lines.at(-1) ?? "{}")).ffmpeg;
}

export function ensureFfmpeg(): string {
  const target = hostTarget();
  const staged = stagedFfmpeg(target);
  return existsSync(staged) && existsSync(FFMPEG_LICENCE) ? staged : stageFfmpeg(target);
}

export function withFfmpeg(): NodeJS.ProcessEnv {
  return { ...process.env, OPSIS_FFMPEG: ensureFfmpeg() };
}
