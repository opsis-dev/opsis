import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, renameSync, rmSync } from "node:fs";
import path from "node:path";
import { type Pinned, verifiedDownload } from "./lib/pins";
import { devCache } from "./lib/tool-config";
import { unzipTo } from "./lib/unzip";

const ZIG_VERSION = "0.17.1";
const ARCHIVES: Partial<Record<string, Pinned>> = {
  "x86_64-linux": {
    url: "https://ziglang.org/download/0.17.1/zig-x86_64-linux-0.17.1.tar.xz",
    sha256: "3581863e3cbcfd0fce0efc682ce7e9e93da96668a977791d356a26d2a7e9b17a",
    size: 57411036,
  },
  "aarch64-linux": {
    url: "https://ziglang.org/download/0.17.1/zig-aarch64-linux-0.17.1.tar.xz",
    sha256: "a1546d5646eb184437918d6635ca6b762a0e889eac09cef720ee7a1ed3fe7795",
    size: 52977364,
  },
  "x86_64-windows": {
    url: "https://ziglang.org/download/0.17.1/zig-x86_64-windows-0.17.1.zip",
    sha256: "b03f477a1943673e02aca0f570e8030dffd536ce1f45cbc4d3326ec31159562e",
    size: 100343666,
  },
  "aarch64-macos": {
    url: "https://ziglang.org/download/0.17.1/zig-aarch64-macos-0.17.1.tar.xz",
    sha256: "75f1bc7a1e9852f17ec9a0c1895f767c4ec56d46bc8a834920eece1d0359dd8d",
    size: 54001240,
  },
  "x86_64-macos": {
    url: "https://ziglang.org/download/0.17.1/zig-x86_64-macos-0.17.1.tar.xz",
    sha256: "9af07b20319fbd0c92ee46ad23e81d6a782b1aaea2cfb8dcbff35f1d2c7ddbb7",
    size: 59496824,
  },
};
const ARCHES: Partial<Record<NodeJS.Architecture, string>> = { x64: "x86_64", arm64: "aarch64" };
const SYSTEMS: Partial<Record<NodeJS.Platform, string>> = {
  linux: "linux",
  darwin: "macos",
  win32: "windows",
};

function cacheDir(): string {
  return devCache("zig");
}

function zigTarget(): string {
  const arch = ARCHES[process.arch];
  const system = SYSTEMS[process.platform];
  if (!arch || !system)
    throw new Error(`Zig isn't pinned for ${process.platform} ${process.arch}.`);
  return `${arch}-${system}`;
}

function zigProgram(folder: string): string {
  return path.join(folder, process.platform === "win32" ? "zig.exe" : "zig");
}

async function unpack(archive: string, into: string): Promise<void> {
  if (archive.endsWith(".zip")) await unzipTo(archive, into);
  else execFileSync("tar", ["-xJf", archive, "-C", into], { stdio: "inherit" });
}

async function installZig(): Promise<string> {
  const target = zigTarget();
  const archive = ARCHIVES[target];
  if (!archive) throw new Error(`Zig ${ZIG_VERSION} isn't pinned for ${target}.`);
  const name = `zig-${target}-${ZIG_VERSION}`;
  const folder = path.join(cacheDir(), name);
  if (existsSync(zigProgram(folder))) return folder;
  const downloaded = await verifiedDownload(archive, path.join(cacheDir(), "downloads"));
  const unpacking = `${folder}.partial`;
  rmSync(unpacking, { recursive: true, force: true });
  mkdirSync(unpacking, { recursive: true });
  await unpack(downloaded, unpacking);
  rmSync(folder, { recursive: true, force: true });
  renameSync(path.join(unpacking, name), folder);
  rmSync(unpacking, { recursive: true, force: true });
  if (!existsSync(zigProgram(folder))) throw new Error(`${downloaded} has no Zig program.`);
  return folder;
}

console.log(`Zig ${ZIG_VERSION}: ${await installZig()}`);
