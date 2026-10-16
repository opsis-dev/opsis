import { existsSync } from "node:fs";
import path from "node:path";

export const FFMPEG_RELEASE = "b6.1.1";
export const FFMPEG_RELEASE_URL = `https://github.com/eugeneware/ffmpeg-static/releases/download/${FFMPEG_RELEASE}`;

export const CHROMIUM_CREDITS_FILE = "LICENSES.chromium.html";
export const ENGINE_LICENCES_FOLDER = "engine-licences";

export type RemotionPlatform = "linux64" | "linux-arm64" | "mac-x64" | "mac-arm64" | "win64";

export function remotionPlatform(
  platform = process.platform,
  arch = process.arch,
): RemotionPlatform {
  if (platform === "linux") return arch === "arm64" ? "linux-arm64" : "linux64";
  if (platform === "darwin") return arch === "arm64" ? "mac-arm64" : "mac-x64";
  if (platform === "win32") return "win64";
  throw new Error(`Unsupported platform ${platform}/${arch}`);
}

export type EnginePaths = {
  dir: string;
  downloadDir: string;
  shellRoot: string;
  shellFolder: string;
  headlessShell: string;
  ffmpeg: string;
  ffprobe: string;
  ffmpegLicense: string;
};

export function enginePaths(
  dir: string,
  platform = process.platform,
  arch = process.arch,
): EnginePaths {
  const rp = remotionPlatform(platform, arch);
  const shellRoot = path.join(dir, "chrome-headless-shell");
  const current = path.join(shellRoot, rp, `chrome-headless-shell-${rp}`);
  const legacy = path.join(
    dir,
    "node_modules",
    ".remotion",
    "chrome-headless-shell",
    rp,
    `chrome-headless-shell-${rp}`,
  );
  const shellFolder = !existsSync(current) && existsSync(legacy) ? legacy : current;
  const remotionBuild = path.join(shellFolder, "headless_shell");
  const exe = platform === "win32" ? ".exe" : "";
  return {
    dir,
    downloadDir: path.join(dir, ".download"),
    shellRoot,
    shellFolder,
    headlessShell: existsSync(remotionBuild)
      ? remotionBuild
      : path.join(shellFolder, `chrome-headless-shell${exe}`),
    ffmpeg: path.join(dir, "ffmpeg", `ffmpeg${exe}`),
    ffprobe: path.join(dir, "ffmpeg", `ffprobe${exe}`),
    ffmpegLicense: path.join(dir, "ffmpeg", "LICENSE.txt"),
  };
}

export type EngineStatus = {
  dir: string;
  headlessShell: string | null;
  ffmpeg: string | null;
  ffprobe: string | null;
  complete: boolean;
};

export function engineStatus(dir: string): EngineStatus {
  const p = enginePaths(dir);
  const has = (f: string) => (existsSync(f) ? f : null);
  const status = {
    dir,
    headlessShell: has(p.headlessShell),
    ffmpeg: has(p.ffmpeg),
    ffprobe: has(p.ffprobe),
  };
  return { ...status, complete: !!(status.headlessShell && status.ffmpeg && status.ffprobe) };
}

export function resolveEngineDir(opts: {
  isPackaged: boolean;
  resourcesPath: string;
  userData: string;
}): string {
  const bundled = path.join(opts.resourcesPath, "engine");
  if (opts.isPackaged && engineStatus(bundled).complete) return bundled;
  return path.join(opts.userData, "engine");
}
