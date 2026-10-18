import { existsSync } from "node:fs";
import path from "node:path";
import { ENGINE_MISSING_MESSAGE, type EngineStatus } from "../shared/engine";

const FFMPEG_RELEASE = "b6.1.1";
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
  headlessShells: [remotionBuild: string, chromeForTesting: string];
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
  const shellFolder = path.join(shellRoot, rp, `chrome-headless-shell-${rp}`);
  const exe = platform === "win32" ? ".exe" : "";
  return {
    dir,
    downloadDir: path.join(dir, ".download"),
    shellRoot,
    shellFolder,
    headlessShells: [
      path.join(shellFolder, "headless_shell"),
      path.join(shellFolder, `chrome-headless-shell${exe}`),
    ],
    ffmpeg: path.join(dir, "ffmpeg", `ffmpeg${exe}`),
    ffprobe: path.join(dir, "ffmpeg", `ffprobe${exe}`),
    ffmpegLicense: path.join(dir, "ffmpeg", "LICENSE.txt"),
  };
}

const findHeadlessShell = (paths: Pick<EnginePaths, "headlessShells">): string | null =>
  paths.headlessShells.find((file) => existsSync(file)) ?? null;

export function requireEngineFile(file: string | null): string {
  if (!file) throw new Error(ENGINE_MISSING_MESSAGE);
  return file;
}

export const requireHeadlessShell = (paths: Pick<EnginePaths, "headlessShells">): string =>
  requireEngineFile(findHeadlessShell(paths));

export function engineStatus(dir: string): EngineStatus {
  const p = enginePaths(dir);
  const has = (f: string) => (existsSync(f) ? f : null);
  const status = {
    dir,
    headlessShell: findHeadlessShell(p),
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
