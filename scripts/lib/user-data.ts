import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, symlinkSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { type CheckedBrowser, checkedBrowser } from "./browser-stamp";
import { ROOT } from "./tool-config";

export const DEV_ENGINE = path.join(os.homedir(), ".config", "Opsis", "engine");

const SCHEMA = path.join(ROOT, "crates", "store", "schema.sql");

const META_ID = 1;

function foldersIn(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(dir, entry.name));
}

export function browserIn(engine: string): string | null {
  const root = path.join(engine, "chrome-headless-shell");
  if (!existsSync(root)) return null;
  const shells = foldersIn(root)
    .flatMap(foldersIn)
    .map((build) => path.join(build, "chrome-headless-shell"));
  return shells.find((file) => existsSync(file)) ?? null;
}

function engineSettings(engine: string): Record<string, CheckedBrowser | null> {
  const browser = browserIn(engine);
  return { browser: browser ? checkedBrowser(browser) : null };
}

function seedDatabase(userData: string, settings: Record<string, unknown>): void {
  const sql = readFileSync(SCHEMA, "utf8");
  const db = new DatabaseSync(path.join(userData, "opsis.db"));
  db.exec(sql);
  db.prepare("INSERT INTO meta (id, schema_hash) VALUES (?, ?)").run(
    META_ID,
    createHash("sha256").update(sql).digest("hex"),
  );
  const row = db.prepare("INSERT INTO settings (section, value) VALUES (?, ?)");
  for (const [section, value] of Object.entries(settings)) row.run(section, JSON.stringify(value));
  db.close();
}

export function seedUserData(
  userData: string,
  settings: Record<string, unknown> = {},
  { engine: linked = true }: { engine?: boolean } = {},
): void {
  mkdirSync(userData, { recursive: true });
  const engine = path.join(userData, "engine");
  if (linked && existsSync(DEV_ENGINE)) symlinkSync(DEV_ENGINE, engine);
  seedDatabase(userData, {
    assistant: { enabled: true },
    engine: engineSettings(engine),
    ...settings,
  });
}
