import { openAsBlob } from "node:fs";
import { chmod, mkdir, symlink, writeFile } from "node:fs/promises";
import path from "node:path";
import { BlobReader, configure, TextWriter, Uint8ArrayWriter, ZipReader } from "@zip.js/zip.js";

configure({ useWebWorkers: false });

export async function unzipTo(zip: string, dir: string): Promise<void> {
  const root = path.resolve(dir);
  const reader = new ZipReader(new BlobReader(await openAsBlob(zip)));
  try {
    for (const entry of await reader.getEntries()) {
      const dest = path.join(root, entry.filename);
      if (!dest.startsWith(root + path.sep))
        throw new Error(`${zip} has an entry outside its folder: ${entry.filename}`);
      if (entry.directory) {
        await mkdir(dest, { recursive: true });
        continue;
      }
      await mkdir(path.dirname(dest), { recursive: true });
      if (entry.symlink) {
        await symlink(await entry.getData(new TextWriter()), dest);
        continue;
      }
      await writeFile(dest, await entry.getData(new Uint8ArrayWriter()));
      if (entry.executable) await chmod(dest, 0o755);
    }
  } finally {
    await reader.close();
  }
}
