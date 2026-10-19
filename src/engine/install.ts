import { chmod, mkdir, rename, rm } from "node:fs/promises";
import path from "node:path";
import { createGunzip } from "node:zlib";
import { ensureBrowser } from "@remotion/renderer";
import type { EngineInstallUpdate } from "../shared/engine";
import { formatBytes } from "../shared/format";
import { downloadTo } from "./download";
import { writeJson } from "./json";
import { enginePaths, FFMPEG_RELEASE_URL } from "./paths";

export type InstallProgress = (update: EngineInstallUpdate) => void;

const CALM_MS = 500;

const downloadedSize = (got: number, total: number | null): string =>
  total ? `${formatBytes(got)} of ${formatBytes(total)}` : formatBytes(got);

export async function installFfmpeg(
  dir: string,
  onProgress: InstallProgress,
  signal?: AbortSignal,
) {
  const p = enginePaths(dir);
  await mkdir(path.dirname(p.ffmpeg), { recursive: true });
  const target = `${process.platform}-${process.arch}`;
  const files: Array<[string, string]> = [
    [`${FFMPEG_RELEASE_URL}/ffmpeg-${target}.gz`, p.ffmpeg],
    [`${FFMPEG_RELEASE_URL}/ffprobe-${target}.gz`, p.ffprobe],
    [`${FFMPEG_RELEASE_URL}/${target}.LICENSE`, p.ffmpegLicense],
  ];
  for (const [i, [url, dest]] of files.entries()) {
    await downloadTo(url, dest, {
      signal,
      transform: url.endsWith(".gz") ? createGunzip() : undefined,
      onBytes: (got, total) =>
        onProgress({
          part: "ffmpeg",
          step: `download-${i + 1}`,
          progress: total ? got / total : null,
          message: `Downloading the video tools (${i + 1} of ${files.length}) · ${downloadedSize(got, total)}`,
        }),
    });
    if (dest !== p.ffmpegLicense) await chmod(dest, 0o755);
  }
  onProgress({ part: "ffmpeg", step: "ready", progress: 1, message: "Video tools ready" });
}

const shellDownload = (onProgress: InstallProgress, progress: number, size: string) =>
  onProgress({
    part: "headless-shell",
    step: "download",
    progress,
    message: `Downloading the recording browser · ${size}`,
  });

async function downloadShellInto(staging: string, onProgress: InstallProgress): Promise<string> {
  await mkdir(staging, { recursive: true });
  await writeJson(path.join(staging, "package.json"), {
    name: "demoflow-engine-download",
    private: true,
  });
  const previous = process.cwd();
  process.chdir(staging);
  try {
    await ensureBrowser({
      onBrowserDownload: () => {
        shellDownload(onProgress, 0, "starting");
        return {
          version: null,
          onProgress: (x) =>
            shellDownload(
              onProgress,
              x.percent,
              downloadedSize(x.downloadedBytes, x.totalSizeInBytes),
            ),
        };
      },
    });
  } finally {
    process.chdir(previous);
  }
  return path.join(staging, "node_modules", ".remotion", "chrome-headless-shell");
}

export async function installHeadlessShell(dir: string, onProgress: InstallProgress) {
  const p = enginePaths(dir);
  const downloaded = await downloadShellInto(p.downloadDir, onProgress);
  await rm(p.shellRoot, { recursive: true, force: true });
  await rename(downloaded, p.shellRoot);
  await rm(p.downloadDir, { recursive: true, force: true });
  onProgress({
    part: "headless-shell",
    step: "ready",
    progress: 1,
    message: "Recording browser ready",
  });
}

export function calmProgress(onProgress: InstallProgress): InstallProgress {
  let step = "";
  let shown: number | null = null;
  let lastAt = 0;
  return (update) => {
    const current = `${update.part}|${update.step}`;
    const percent = update.progress === null ? null : Math.floor(update.progress * 100);
    const now = Date.now();
    if (current === step && percent === shown && now - lastAt < CALM_MS) return;
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
