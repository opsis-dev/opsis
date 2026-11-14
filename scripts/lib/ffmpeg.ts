import { execFileSync } from "node:child_process";
import path from "node:path";
import { type FfmpegPins, readPins } from "./setup/manifest";
import { readPaths } from "./setup/store";
import { ROOT } from "./tool-config";

export const FFMPEG_LICENCE = path.join(ROOT, "resources", "LICENSE.ffmpeg.txt");

const RELEASES = "https://github.com/eugeneware/ffmpeg-static/releases";

export function ffmpegReleaseUrl(release: string): string {
  return `${RELEASES}/tag/${release}`;
}

export function readFfmpegPins(): FfmpegPins {
  return readPins().ffmpeg;
}

export function sidecarFileName(target: string): string {
  const exe = target.includes("-windows-") ? ".exe" : "";
  return `opsis-ffmpeg-${target}${exe}`;
}

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

export function stagedFfmpeg(): string {
  return readPaths().ffmpeg;
}

export function stagedFfprobe(): string {
  return readPaths().ffprobe;
}

export function stagedFfmpegLicence(): string {
  return readPaths().ffmpegLicence;
}

export function withFfmpeg(): NodeJS.ProcessEnv {
  return { ...process.env, OPSIS_FFMPEG: stagedFfmpeg() };
}
