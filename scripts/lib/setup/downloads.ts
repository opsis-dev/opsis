import { chmodSync, copyFileSync, createReadStream, createWriteStream } from "node:fs";
import path from "node:path";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";
import { writeViaTemp } from "../atomic";
import { verifiedDownload } from "../pins";
import { type Pins, platformName } from "./manifest";
import { downloadsDir, recipeHash, step, unpackInto } from "./store";

export type Made = { dir: string; path: string };

function zigProgramName(): string {
  return process.platform === "win32" ? "zig.exe" : "zig";
}

export async function setupZig(pins: Pins): Promise<Made> {
  const archive = pins.zig.archives[platformName()];
  if (!archive) throw new Error(`Zig ${pins.zig.version} isn't pinned for ${platformName()}.`);
  const dir = await step("zig", recipeHash(["zig", archive]), async (into) => {
    const file = await verifiedDownload(archive, downloadsDir());
    await unpackInto(file, path.join(into, "zig"));
  });
  return { dir, path: path.join(dir, "zig", zigProgramName()) };
}

export async function setupKddw(pins: Pins): Promise<Made> {
  const dir = await step("kddw", recipeHash(["kddw", pins.kddw]), async (into) => {
    const file = await verifiedDownload(pins.kddw, downloadsDir());
    await unpackInto(file, path.join(into, "source"));
  });
  return { dir, path: path.join(dir, "source") };
}

async function gunzipTo(archive: string, file: string): Promise<void> {
  await writeViaTemp(file, (temp) =>
    pipeline(createReadStream(archive), createGunzip(), createWriteStream(temp)),
  );
  chmodSync(file, 0o755);
}

export type FfmpegFiles = { dir: string; ffmpeg: string; ffprobe: string; licence: string };

export async function setupFfmpeg(pins: Pins, target: string): Promise<FfmpegFiles> {
  const pin = pins.ffmpeg.targets.find((t) => t.target === target);
  if (!pin) throw new Error(`There's no FFmpeg pinned for ${target}.`);
  const exe = target.includes("-windows-") ? ".exe" : "";
  function files(dir: string): FfmpegFiles {
    return {
      dir,
      ffmpeg: path.join(dir, `opsis-ffmpeg-${target}${exe}`),
      ffprobe: path.join(dir, `ffprobe${exe}`),
      licence: path.join(dir, "LICENSE.ffmpeg.txt"),
    };
  }
  const dir = await step("ffmpeg", recipeHash(["ffmpeg", pin]), async (into) => {
    const made = files(into);
    await gunzipTo(await verifiedDownload(pin.program, downloadsDir()), made.ffmpeg);
    await gunzipTo(await verifiedDownload(pin.probe, downloadsDir()), made.ffprobe);
    copyFileSync(await verifiedDownload(pin.licence, downloadsDir()), made.licence);
  });
  return files(dir);
}

export async function setupNpm(
  pins: Pins,
): Promise<{ dirs: string[]; paths: Record<string, string> }> {
  const dirs: string[] = [];
  const paths: Record<string, string> = {};
  for (const [name, pin] of Object.entries(pins.npm)) {
    const dir = await step("npm", recipeHash(["npm", name, pin]), async (into) => {
      const file = await verifiedDownload(pin, downloadsDir());
      await unpackInto(file, path.join(into, "package"));
    });
    dirs.push(dir);
    paths[name] = path.join(dir, "package");
  }
  return { dirs, paths };
}
