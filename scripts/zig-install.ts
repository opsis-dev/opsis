import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, renameSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { type Pinned, verifiedDownload } from "./lib/pins";
import { unzipTo } from "./lib/unzip";

const ZIG_VERSION = "0.17.0";
const ARCHIVES: Partial<Record<string, Pinned>> = {
  "x86_64-linux": {
    url: "https://ziglang.org/download/0.17.0/zig-x86_64-linux-0.17.0.tar.xz",
    sha256: "1cbe9df9f27e6b78d14ccbca43b6703a404ef79ef1c463de901d7f088d4e2026",
    size: 57332648,
  },
  "aarch64-linux": {
    url: "https://ziglang.org/download/0.17.0/zig-aarch64-linux-0.17.0.tar.xz",
    sha256: "9e8d11661d4ae3bd57702a3832781e23ad151dde5798e16a5ccd503f65234ff8",
    size: 52877280,
  },
  "x86_64-windows": {
    url: "https://ziglang.org/download/0.17.0/zig-x86_64-windows-0.17.0.zip",
    sha256: "b5663f69581dcf391293fbf16c06cb80d81d806545ce618b4d0bab7f0eb8c428",
    size: 100268137,
  },
  "aarch64-macos": {
    url: "https://ziglang.org/download/0.17.0/zig-aarch64-macos-0.17.0.tar.xz",
    sha256: "b607e9b9234790a008116ae5bdb71c6243b84b9fb42a53a9e70fde41c06c536a",
    size: 53985220,
  },
  "x86_64-macos": {
    url: "https://ziglang.org/download/0.17.0/zig-x86_64-macos-0.17.0.tar.xz",
    sha256: "4f9a1c5269aa17ebda5e6d3c2b89d6cbf36f7d2b22a0306e9ab98f25f95529c6",
    size: 59317572,
  },
};
const ARCHES: Partial<Record<NodeJS.Architecture, string>> = { x64: "x86_64", arm64: "aarch64" };
const SYSTEMS: Partial<Record<NodeJS.Platform, string>> = {
  linux: "linux",
  darwin: "macos",
  win32: "windows",
};

function cacheDir(): string {
  return path.join(homedir(), ".cache", "opsis-zig");
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
