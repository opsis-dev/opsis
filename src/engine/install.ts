import { createWriteStream, existsSync } from "node:fs";
import { chmod, mkdir, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";
import { ensureBrowser } from "@remotion/renderer";
import { enginePaths, FFMPEG_RELEASE_URL } from "./paths";

export type InstallProgress = (
  part: "ffmpeg" | "headless-shell",
  progress: number | null,
  message: string,
) => void;

const ffmpegTarget = (platform: string, arch: string) => `${platform}-${arch}`;

async function download(
  url: string,
  dest: string,
  onBytes: (received: number, total: number | null) => void,
  signal?: AbortSignal,
) {
  const res = await fetch(url, { signal, redirect: "follow" });
  if (!res.ok || !res.body) throw new Error(`Download failed (${res.status}): ${url}`);
  const total = Number(res.headers.get("content-length")) || null;
  let received = 0;
  const counted = Readable.fromWeb(res.body as never).on("data", (chunk: Buffer) => {
    received += chunk.length;
    onBytes(received, total);
  });
  const tmp = `${dest}.part`;
  await pipeline(counted, ...(url.endsWith(".gz") ? [createGunzip()] : []), createWriteStream(tmp));
  await rename(tmp, dest);
}

export async function installFfmpeg(
  dir: string,
  onProgress: InstallProgress,
  signal?: AbortSignal,
  platform = process.platform,
  arch = process.arch,
) {
  const p = enginePaths(dir, platform, arch);
  await mkdir(path.dirname(p.ffmpeg), { recursive: true });
  const target = ffmpegTarget(platform, arch);
  const files: Array<[string, string]> = [
    [`${FFMPEG_RELEASE_URL}/ffmpeg-${target}.gz`, p.ffmpeg],
    [`${FFMPEG_RELEASE_URL}/ffprobe-${target}.gz`, p.ffprobe],
    [`${FFMPEG_RELEASE_URL}/${target}.LICENSE`, p.ffmpegLicense],
  ];
  for (const [i, [url, dest]] of files.entries()) {
    await download(
      url,
      dest,
      (got, total) =>
        onProgress(
          "ffmpeg",
          total ? (i + got / total) / files.length : null,
          `Downloading the video tools (${i + 1} of ${files.length})…`,
        ),
      signal,
    );
    if (dest !== p.ffmpegLicense) await chmod(dest, 0o755);
  }
  onProgress("ffmpeg", 1, "Video tools ready");
}

export async function installHeadlessShell(dir: string, onProgress: InstallProgress) {
  const p = enginePaths(dir);
  await mkdir(dir, { recursive: true });
  if (!existsSync(p.packageJson))
    await writeFile(
      p.packageJson,
      `${JSON.stringify({ name: "demoflow-engine", private: true }, null, 2)}\n`,
    );
  const previous = process.cwd();
  process.chdir(dir);
  try {
    await ensureBrowser({
      onBrowserDownload: () => {
        onProgress("headless-shell", 0, "Downloading the recording browser…");
        return {
          version: null,
          onProgress: (x) =>
            onProgress("headless-shell", x.percent / 100, "Downloading the recording browser…"),
        };
      },
    });
  } finally {
    process.chdir(previous);
  }
  onProgress("headless-shell", 1, "Recording browser ready");
}

export async function installEngine(
  dir: string,
  onProgress: InstallProgress,
  signal?: AbortSignal,
) {
  await installFfmpeg(dir, onProgress, signal);
  await installHeadlessShell(dir, onProgress);
}

export async function removeEngine(dir: string) {
  await rm(dir, { recursive: true, force: true });
}
