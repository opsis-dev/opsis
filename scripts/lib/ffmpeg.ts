import { execFileSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  createReadStream,
  createWriteStream,
  existsSync,
  mkdirSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";
import { z } from "zod";
import { writeViaTemp } from "./atomic";
import { pinnedSchema, verifiedDownload } from "./pins";
import { readJsonWith } from "./read-json";
import { ROOT } from "./tool-config";

export const FFMPEG_RELEASE = "b6.1.1";

export const FFMPEG_TARGETS = [
  ["x86_64-unknown-linux-gnu", "linux-x64"],
  ["aarch64-unknown-linux-gnu", "linux-arm64"],
  ["x86_64-apple-darwin", "darwin-x64"],
  ["aarch64-apple-darwin", "darwin-arm64"],
  ["x86_64-pc-windows-msvc", "win32-x64"],
] as const;

export const FFMPEG_PINS = path.join(ROOT, "scripts", "lib", "ffmpeg.generated.json");

export const FFMPEG_CACHE = path.join(os.homedir(), ".cache", "opsis-ffmpeg");

export const FFMPEG_LICENCE = path.join(ROOT, "resources", "LICENSE.ffmpeg.txt");

const BINARIES = path.join(ROOT, "crates", "desktop", "binaries");

const ffmpegPinsSchema = z.object({
  release: z.string(),
  targets: z.array(
    z.object({
      target: z.string(),
      asset: z.string(),
      program: pinnedSchema,
      licence: pinnedSchema,
    }),
  ),
});

export type FfmpegPins = z.infer<typeof ffmpegPinsSchema>;

function releaseUrl(file: string): string {
  return `https://github.com/eugeneware/ffmpeg-static/releases/download/${FFMPEG_RELEASE}/${file}`;
}

export function ffmpegProgramUrl(asset: string): string {
  return releaseUrl(`ffmpeg-${asset}.gz`);
}

export function ffmpegLicenceUrl(asset: string): string {
  return releaseUrl(`${asset}.LICENSE`);
}

export function readFfmpegPins(): FfmpegPins {
  return readJsonWith(ffmpegPinsSchema, FFMPEG_PINS);
}

export function sidecarFileName(target: string): string {
  const exe = target.includes("-windows-") ? ".exe" : "";
  return `opsis-ffmpeg-${target}${exe}`;
}

function hostTarget(): string {
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
  return path.join(BINARIES, sidecarFileName(target));
}

export async function stageFfmpeg(target = hostTarget()): Promise<string> {
  const pins = readFfmpegPins().targets.find((pin) => pin.target === target);
  if (!pins) throw new Error(`There's no FFmpeg pinned for ${target}.`);
  const archive = await verifiedDownload(pins.program, FFMPEG_CACHE);
  const licence = await verifiedDownload(pins.licence, FFMPEG_CACHE);
  mkdirSync(BINARIES, { recursive: true });
  const ffmpeg = stagedFfmpeg(target);
  await writeViaTemp(ffmpeg, (temp) =>
    pipeline(createReadStream(archive), createGunzip(), createWriteStream(temp)),
  );
  chmodSync(ffmpeg, 0o755);
  mkdirSync(path.dirname(FFMPEG_LICENCE), { recursive: true });
  copyFileSync(licence, FFMPEG_LICENCE);
  return ffmpeg;
}

export async function ensureFfmpeg(): Promise<string> {
  const target = hostTarget();
  const staged = stagedFfmpeg(target);
  return existsSync(staged) && existsSync(FFMPEG_LICENCE) ? staged : stageFfmpeg(target);
}

export async function withFfmpeg(): Promise<NodeJS.ProcessEnv> {
  return { ...process.env, OPSIS_FFMPEG: await ensureFfmpeg() };
}
