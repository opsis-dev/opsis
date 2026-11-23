import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { z } from "zod";
import { readJsonWith } from "../read-json";
import { ROOT, devCache } from "../tool-config";
import { unzipTo } from "../unzip";

export const SETUP_DIR = path.join(ROOT, ".setup");
export const PATHS_FILE = path.join(SETUP_DIR, "paths.json");
export const ZIG_LINK = path.join(SETUP_DIR, "bin", "zig");

const COMPLETE = ".complete";
const HASH_LENGTH = 16;

const pathsSchema = z.object({
  zig: z.string(),
  qt: z.string(),
  kddw: z.string(),
  skia: z.string(),
  filament: z.string(),
  ffmpeg: z.string(),
  ffprobe: z.string(),
  ffmpegLicence: z.string(),
  npm: z.record(z.string(), z.string()),
  engine: z.string().optional(),
});

export type SetupPaths = z.infer<typeof pathsSchema>;

export function downloadsDir(): string {
  return devCache("downloads");
}

export function recipeHash(parts: unknown, files: string[] = []): string {
  const hash = createHash("sha256").update(JSON.stringify(parts));
  for (const file of files) {
    hash.update(path.relative(ROOT, file));
    hash.update(readFileSync(file));
  }
  return hash.digest("hex").slice(0, HASH_LENGTH);
}

export function itemDir(item: string, hash: string): string {
  return path.join(devCache(item), hash);
}

export function isComplete(dir: string): boolean {
  return existsSync(path.join(dir, COMPLETE));
}

export async function step(
  item: string,
  hash: string,
  make: (dir: string) => Promise<void> | void,
): Promise<string> {
  const dir = itemDir(item, hash);
  if (isComplete(dir)) {
    console.log(`${item}: ${dir}`);
    return dir;
  }
  console.log(`${item}: making ${dir}`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  await make(dir);
  writeFileSync(path.join(dir, COMPLETE), "");
  console.log(`${item}: ${dir}`);
  return dir;
}

export async function unpackInto(archive: string, into: string): Promise<void> {
  const unpacking = `${into}.partial`;
  rmSync(unpacking, { recursive: true, force: true });
  mkdirSync(unpacking, { recursive: true });
  if (archive.endsWith(".zip")) await unzipTo(archive, unpacking);
  else execFileSync("cmake", ["-E", "tar", "xf", archive], { cwd: unpacking, stdio: "inherit" });
  const entries = readdirSync(unpacking);
  const [only] = entries;
  const root = entries.length === 1 && only ? path.join(unpacking, only) : unpacking;
  rmSync(into, { recursive: true, force: true });
  renameSync(root, into);
  rmSync(unpacking, { recursive: true, force: true });
}

export function readPaths(): SetupPaths {
  if (!existsSync(PATHS_FILE))
    throw new Error(`There's no ${path.relative(ROOT, PATHS_FILE)}. Run bun run setup first.`);
  return readJsonWith(pathsSchema, PATHS_FILE);
}

export function npmPackage(name: string): string {
  const dir = readPaths().npm[name];
  if (!dir)
    throw new Error(`setup has no ${name}; pin it with bun run setup pin npm ${name}@<version>.`);
  return dir;
}

export function writePaths(paths: SetupPaths): void {
  mkdirSync(SETUP_DIR, { recursive: true });
  writeFileSync(PATHS_FILE, `${JSON.stringify(pathsSchema.parse(paths), null, 2)}\n`);
  mkdirSync(path.dirname(ZIG_LINK), { recursive: true });
  rmSync(ZIG_LINK, { force: true });
  symlinkSync(paths.zig, ZIG_LINK);
}

export function prune(kept: string[]): void {
  const keep = new Set(kept.map((dir) => path.resolve(dir)));
  for (const item of new Set(kept.map((dir) => path.dirname(path.resolve(dir))))) {
    for (const entry of readdirSync(item)) {
      const dir = path.join(item, entry);
      if (keep.has(dir)) continue;
      rmSync(dir, { recursive: true, force: true });
      console.log(`pruned ${dir}`);
    }
  }
}
