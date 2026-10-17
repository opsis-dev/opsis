import { readdir, stat } from "node:fs/promises";
import path from "node:path";

const readdirOrEmpty = (dir: string): Promise<string[]> => readdir(dir).catch((): string[] => []);

type DatedFile = { name: string; file: string; mtimeMs: number | null };

export async function datedFiles(dir: string): Promise<DatedFile[]> {
  const names = await readdirOrEmpty(dir);
  return Promise.all(
    names.map(async (name) => {
      const file = path.join(dir, name);
      const info = await stat(file).catch(() => null);
      return { name, file, mtimeMs: info?.mtimeMs ?? null };
    }),
  );
}
