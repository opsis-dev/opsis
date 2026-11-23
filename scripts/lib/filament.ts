import { execFileSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import path from "node:path";
import { fetchGitSource } from "./git-source";
import { readPins } from "./setup/manifest";
import { msvcEnv } from "./static-qt";
import { ROOT } from "./tool-config";
import { runInTrixie } from "./trixie";

const BUILD_JOBS = "8";
const INSIDE_FLAG = "--inside";
const INSTALLED = "filament";
export const FILAMENT_LIBRARY_DIR = "opsis";
const MAC_VULKAN_LOADER = "libvulkan.dylib";

export type FilamentPlatform = "linux" | "darwin" | "win32";

const SHARED_OPTIONS: Record<string, string> = {
  CMAKE_BUILD_TYPE: "Release",
  FILAMENT_SKIP_SAMPLES: "ON",
  FILAMENT_SKIP_SDL2: "ON",
  FILAMENT_BUILD_TESTING: "OFF",
  FILAMENT_SUPPORTS_VULKAN: "ON",
  FILAMENT_SUPPORTS_OPENGL: "OFF",
  FILAMENT_SUPPORTS_WEBGPU: "OFF",
  DIST_DIR: FILAMENT_LIBRARY_DIR,
};

const PLATFORM_OPTIONS: Record<FilamentPlatform, Record<string, string>> = {
  linux: { CMAKE_C_COMPILER: "clang", CMAKE_CXX_COMPILER: "clang++" },
  darwin: { CMAKE_CXX_FLAGS: `-DFILAMENT_VKLIBRARY_PATH=\\"${MAC_VULKAN_LOADER}\\"` },
  win32: { CMAKE_C_COMPILER: "cl", CMAKE_CXX_COMPILER: "cl", USE_STATIC_CRT: "OFF" },
};

export function configureOptions(platform: FilamentPlatform): string[] {
  return Object.entries({ ...SHARED_OPTIONS, ...PLATFORM_OPTIONS[platform] }).map(
    ([key, value]) => `-D${key}=${value}`,
  );
}

export const FILAMENT_RECIPE = {
  shared: SHARED_OPTIONS,
  platforms: PLATFORM_OPTIONS,
};

function hostPlatform(): FilamentPlatform {
  const platform = process.platform;
  if (platform === "linux" || platform === "darwin" || platform === "win32") return platform;
  throw new Error(`Filament isn't built for ${platform}.`);
}

function workDir(platform: FilamentPlatform, dir: string): string {
  const runnerTemp = process.env.RUNNER_TEMP;
  return platform === "win32" && runnerTemp ? path.join(runnerTemp, "flb") : path.join(dir, "work");
}

function withoutMsys(env: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  const { MSYSTEM: _msys, ...rest } = env;
  return rest;
}

function cmake(args: string[], env: NodeJS.ProcessEnv) {
  execFileSync("cmake", args, { stdio: "inherit", env });
}

export function installedFilament(dir: string): string {
  return path.join(dir, INSTALLED);
}

function buildHere(dir: string) {
  const { filament } = readPins();
  const platform = hostPlatform();
  const work = workDir(platform, dir);
  const src = path.join(work, "src");
  const build = path.join(work, "build");
  const prefix = installedFilament(dir);
  fetchGitSource(src, path.join(src, "CMakeLists.txt"), filament, "Filament");
  const env = platform === "win32" ? withoutMsys(msvcEnv()) : process.env;
  rmSync(build, { recursive: true, force: true });
  rmSync(prefix, { recursive: true, force: true });
  cmake(
    [
      "-G",
      "Ninja",
      "-S",
      src,
      "-B",
      build,
      `-DCMAKE_INSTALL_PREFIX=${prefix}`,
      ...configureOptions(platform),
    ],
    env,
  );
  cmake(["--build", build, "--parallel", BUILD_JOBS], env);
  cmake(["--install", build], env);
  const matc = path.join(prefix, "bin", platform === "win32" ? "matc.exe" : "matc");
  if (!existsSync(matc)) throw new Error(`The Filament build in ${prefix} has no ${matc}.`);
  rmSync(work, { recursive: true, force: true });
  console.log(`Filament: built into ${prefix}`);
}

function bunProgram(): string {
  return execFileSync("which", ["bun"], { encoding: "utf8" }).trim();
}

export function buildFilament(dir: string, container: boolean): string {
  if (!container) {
    buildHere(dir);
    return installedFilament(dir);
  }
  const bun = bunProgram();
  runInTrixie({
    script: `cd ${ROOT}\n${bun} scripts/lib/filament.ts ${INSIDE_FLAG} ${dir}`,
    mounts: [dir, ROOT, path.dirname(bun)],
  });
  return installedFilament(dir);
}

const inside = process.argv.indexOf(INSIDE_FLAG);
if (import.meta.main && inside >= 0) {
  const dir = process.argv[inside + 1];
  if (!dir) throw new Error(`${INSIDE_FLAG} needs the folder to build into.`);
  buildHere(dir);
}
