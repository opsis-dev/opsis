import { createWriteStream } from "node:fs";
import { chmod, mkdir, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { createGunzip } from "node:zlib";
import { ensureBrowser } from "@remotion/renderer";
import { enginePaths, FFMPEG_RELEASE_URL } from "./paths";
import { formatBytes } from "../shared/format";

export type InstallUpdate = {
  part: "ffmpeg" | "headless-shell";
  step: string;
  progress: number | null;
  message: string;
};

export type InstallProgress = (update: InstallUpdate) => void;

const ffmpegTarget = (platform: string, arch: string) => `${platform}-${arch}`;

export const downloadedSize = (got: number, total: number | null): string =>
  total ? `${formatBytes(got)} of ${formatBytes(total)}` : formatBytes(got);

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
        onProgress({
          part: "ffmpeg",
          step: `download-${i + 1}`,
          progress: total ? got / total : null,
          message: `Downloading the video tools (${i + 1} of ${files.length}) · ${downloadedSize(got, total)}`,
        }),
      signal,
    );
    if (dest !== p.ffmpegLicense) await chmod(dest, 0o755);
  }
  onProgress({ part: "ffmpeg", step: "ready", progress: 1, message: "Video tools ready" });
}

export async function installHeadlessShell(dir: string, onProgress: InstallProgress) {
  const p = enginePaths(dir);
  const staging = p.downloadDir;
  await mkdir(staging, { recursive: true });
  await writeFile(
    path.join(staging, "package.json"),
    `${JSON.stringify({ name: "demoflow-engine-download", private: true }, null, 2)}\n`,
  );
  const previous = process.cwd();
  process.chdir(staging);
  try {
    await ensureBrowser({
      onBrowserDownload: () => {
        onProgress({
          part: "headless-shell",
          step: "download",
          progress: 0,
          message: "Downloading the recording browser · starting",
        });
        return {
          version: null,
          onProgress: (x) =>
            onProgress({
              part: "headless-shell",
              step: "download",
              progress: x.percent,
              message: `Downloading the recording browser · ${downloadedSize(x.downloadedBytes, x.totalSizeInBytes)}`,
            }),
        };
      },
    });
  } finally {
    process.chdir(previous);
  }
  const downloaded = path.join(staging, "node_modules", ".remotion", "chrome-headless-shell");
  await rm(p.shellRoot, { recursive: true, force: true });
  await rename(downloaded, p.shellRoot);
  await rm(staging, { recursive: true, force: true });
  onProgress({
    part: "headless-shell",
    step: "ready",
    progress: 1,
    message: "Recording browser ready",
  });
}

export function calmProgress(onProgress: InstallProgress): InstallProgress {
  let step = "";
  let shown = -1;
  let lastAt = 0;
  return (update) => {
    const current = `${update.part}|${update.step}`;
    const percent = update.progress == null ? -1 : Math.floor(update.progress * 100);
    const now = Date.now();
    if (current === step && percent === shown && now - lastAt < 500) return;
    step = current;
    shown = percent;
    lastAt = now;
    onProgress(update);
  };
}

export async function installEngine(
  dir: string,
  onProgress: InstallProgress,
  signal?: AbortSignal,
) {
  const report = calmProgress(onProgress);
  await installFfmpeg(dir, report, signal);
  await installHeadlessShell(dir, report);
}

export async function removeEngine(dir: string) {
  await rm(dir, { recursive: true, force: true });
}
