import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { safeStorage } from "electron";
import { secretFileName } from "../shared/secrets";

const fileFor = (projectDir: string, name: string) =>
  path.join(projectDir, "secrets", `${secretFileName(name)}.json.enc`);

export async function ensureSecretsDir(projectDir: string): Promise<void> {
  const dir = path.join(projectDir, "secrets");
  await mkdir(dir, { recursive: true });
  if (!existsSync(path.join(dir, ".gitignore")))
    await writeFile(path.join(dir, ".gitignore"), "*\n!.gitignore\n");
}

export async function writeSecret(projectDir: string, name: string, value: string): Promise<void> {
  await ensureSecretsDir(projectDir);
  const data = safeStorage.isEncryptionAvailable()
    ? safeStorage.encryptString(value)
    : Buffer.from(value, "utf8");
  await writeFile(fileFor(projectDir, name), data);
}

export async function listSecrets(projectDir: string): Promise<string[]> {
  const dir = path.join(projectDir, "secrets");
  if (!existsSync(dir)) return [];
  return (await readdir(dir))
    .filter((f) => f.endsWith(".json.enc"))
    .map((f) => f.slice(0, -".json.enc".length))
    .sort();
}

export async function readSecret(projectDir: string, name: string): Promise<string | null> {
  const file = fileFor(projectDir, name);
  if (!existsSync(file)) return null;
  const data = await readFile(file);
  return safeStorage.isEncryptionAvailable()
    ? safeStorage.decryptString(data)
    : data.toString("utf8");
}
