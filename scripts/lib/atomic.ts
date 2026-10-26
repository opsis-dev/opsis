import { randomUUID } from "node:crypto";
import { rename, rm } from "node:fs/promises";
import path from "node:path";

export async function writeViaTemp<T>(
  file: string,
  make: (temp: string) => Promise<T>,
): Promise<T> {
  const temp = `${file}.${randomUUID()}.partial${path.extname(file)}`;
  try {
    const value = await make(temp);
    await rename(temp, file);
    return value;
  } finally {
    await rm(temp, { force: true });
  }
}
