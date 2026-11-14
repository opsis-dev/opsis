import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import path from "node:path";
import { ROOT, devCache } from "./tool-config";

export const FILAMENT_PIN_FILE = path.join(ROOT, "crates", "filament", "downloads.generated.json");

export type FilamentArchive = { url: string; size: number; sha256: string };
export type FilamentPins = { version: string; archives: Record<string, FilamentArchive> };

function platformName(): string {
  return `${process.platform}-${process.arch}`;
}

export function readFilamentPins(): FilamentPins {
  return JSON.parse(readFileSync(FILAMENT_PIN_FILE, "utf8")) as FilamentPins;
}

export function filamentDir(root: string, version: string): string {
  return path.join(root, version, platformName());
}

function sha256(file: string): string {
  return createHash("sha256").update(readFileSync(file)).digest("hex");
}

export function fetchFilament(pins: FilamentPins, root = devCache("filament")): string {
  const archive = pins.archives[platformName()];
  if (!archive) throw new Error(`Filament has no pinned archive for ${platformName()}`);
  const dir = filamentDir(root, pins.version);
  const unpacked = path.join(dir, "filament");
  if (existsSync(path.join(unpacked, "lib"))) return unpacked;
  mkdirSync(dir, { recursive: true });
  const file = path.join(dir, path.basename(archive.url));
  if (!existsSync(file) || sha256(file) !== archive.sha256) {
    execFileSync("curl", ["-fsSL", "--retry", "3", "-o", file, archive.url], { stdio: "inherit" });
  }
  const found = sha256(file);
  if (found !== archive.sha256) {
    rmSync(file);
    throw new Error(`${file} is ${found}, not the pinned ${archive.sha256}`);
  }
  execFileSync("tar", ["-xzf", file, "-C", dir], { stdio: "inherit" });
  return unpacked;
}
