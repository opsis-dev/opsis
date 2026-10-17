import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { safeStorage } from "electron";
import { nullIfMissing } from "../engine/json";
import { secretFileName } from "../shared/secrets";

const SECRETS_DIR = "secrets";
const SECRET_EXT = ".json.enc";

const secretsDir = (projectDir: string) => path.join(projectDir, SECRETS_DIR);

const fileFor = (projectDir: string, name: string) =>
  path.join(secretsDir(projectDir), `${secretFileName(name)}${SECRET_EXT}`);

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
  const names = (await nullIfMissing(readdir(secretsDir(projectDir)))) ?? [];
  return names
    .filter((f) => f.endsWith(SECRET_EXT))
    .map((f) => f.slice(0, -SECRET_EXT.length))
    .sort();
}

export async function readSecret(projectDir: string, name: string): Promise<string | null> {
  const encrypted = await nullIfMissing(readFile(fileFor(projectDir, name)));
  if (encrypted === null) return null;
  requireEncryption();
  return safeStorage.decryptString(encrypted);
}
