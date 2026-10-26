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

function headlessShells(dir: string): string[] {
  const rp = remotionPlatform();
  const folder = path.join(dir, "chrome-headless-shell", rp, `chrome-headless-shell-${rp}`);
  const exe = process.platform === "win32" ? ".exe" : "";
  return [path.join(folder, "headless_shell"), path.join(folder, `chrome-headless-shell${exe}`)];
}

export function engineStatus(dir: string): { complete: boolean; headlessShell: string | null } {
  const exe = process.platform === "win32" ? ".exe" : "";
  const headlessShell = headlessShells(dir).find((file) => existsSync(file)) ?? null;
  const ffmpeg = ["ffmpeg", "ffprobe"].every((tool) =>
    existsSync(path.join(dir, "ffmpeg", `${tool}${exe}`)),
  );
  return { complete: !!headlessShell && ffmpeg, headlessShell };
}

export const userEngineDir = (userData: string): string => path.join(userData, "engine");

export const bundledEngineDir = (resourcesPath: string): string =>
  path.join(resourcesPath, "engine");
