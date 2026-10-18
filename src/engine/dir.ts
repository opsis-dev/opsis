import { readdir, rm, stat } from "node:fs/promises";
import path from "node:path";

type DatedFile = { name: string; file: string; mtimeMs: number | null };

export async function datedFiles(dir: string): Promise<DatedFile[]> {
  const names = await readdir(dir).catch((): string[] => []);
  return Promise.all(
    names.map(async (name) => {
      const file = path.join(dir, name);
      const info = await stat(file).catch(() => null);
      return { name, file, mtimeMs: info?.mtimeMs ?? null };
    }),
  );
}

type Prune = { keep?: (name: string) => boolean; newest?: number; unusedForMs?: number };

export async function pruneDir(
  dir: string,
  { keep = () => false, newest = 0, unusedForMs }: Prune,
): Promise<void> {
  const files = (await datedFiles(dir)).toSorted((a, b) => (b.mtimeMs ?? 0) - (a.mtimeMs ?? 0));
  const cutoff = unusedForMs === undefined ? null : Date.now() - unusedForMs;
  const unused = (mtimeMs: number | null) =>
    cutoff === null || (mtimeMs !== null && mtimeMs < cutoff);
  const gone = files.slice(newest).filter((f) => !keep(f.name) && unused(f.mtimeMs));
  await Promise.all(gone.map((f) => rm(f.file, { force: true })));
}

export async function fileIdentity(file: string): Promise<string> {
  const info = await stat(file);
  return `${info.size}:${Math.round(info.mtimeMs)}`;
}
