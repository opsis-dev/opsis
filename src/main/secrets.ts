import { existsSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { safeStorage } from "electron";
import { readBytesIfPresent, readDirIfPresent } from "../engine/json";
import { SECRET_EXT, secretFile, secretsDir } from "../engine/secret-paths";

const fileFor = secretFile;

function requireEncryption(): void {
  if (!safeStorage.isEncryptionAvailable())
    throw new Error(
      "This computer has no secure storage available, so saved sign-ins can't be used.",
    );
}

export async function ensureSecretsDir(projectDir: string): Promise<void> {
  const dir = secretsDir(projectDir);
  await mkdir(dir, { recursive: true });
  if (!existsSync(path.join(dir, ".gitignore")))
    await writeFile(path.join(dir, ".gitignore"), "*\n!.gitignore\n");
}

export async function writeSecret(projectDir: string, name: string, value: string): Promise<void> {
  requireEncryption();
  await ensureSecretsDir(projectDir);
  await writeFile(fileFor(projectDir, name), safeStorage.encryptString(value));
}

export async function listSecrets(projectDir: string): Promise<string[]> {
  const names = await readDirIfPresent(secretsDir(projectDir));
  return names
    .filter((f) => f.endsWith(SECRET_EXT))
    .map((f) => f.slice(0, -SECRET_EXT.length))
    .sort();
}

export async function readSecret(projectDir: string, name: string): Promise<string | null> {
  const encrypted = await readBytesIfPresent(fileFor(projectDir, name));
  if (encrypted === null) return null;
  requireEncryption();
  return safeStorage.decryptString(encrypted);
}
