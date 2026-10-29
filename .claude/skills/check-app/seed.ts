import { Database } from "bun:sqlite";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";

const ROOT = path.resolve(import.meta.dirname, "..", "..", "..");
const CHECK_DIR = path.join(os.homedir(), ".cache", "opsis-check");
const USER_DATA = path.join(CHECK_DIR, "user-data");
const PROJECTS = path.join(CHECK_DIR, "projects");
const DEV_ENGINE = path.join(os.homedir(), ".config", "Opsis", "engine");
const SHELL_NAMES = ["headless_shell", "chrome-headless-shell"];

const foldersIn = (dir: string): string[] =>
  readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(dir, entry.name));

function browserIn(engine: string): string | null {
  const root = path.join(engine, "chrome-headless-shell");
  if (!existsSync(root)) return null;
  const shells = foldersIn(root)
    .flatMap(foldersIn)
    .flatMap((build) => SHELL_NAMES.map((name) => path.join(build, name)));
  return shells.find((file) => existsSync(file)) ?? null;
}

function engineSettings(engine: string): Record<string, { path: string } | null> {
  const tool = (file: string) => (existsSync(file) ? { path: file } : null);
  const browser = browserIn(engine);
  return {
    browser: browser ? { path: browser } : null,
    ffmpeg: tool(path.join(engine, "ffmpeg", "ffmpeg")),
  };
}

function seedDatabase(settings: Record<string, unknown>): void {
  const sql = readFileSync(path.join(ROOT, "crates", "store", "schema.sql"), "utf8");
  const db = new Database(path.join(USER_DATA, "opsis.db"));
  db.exec(sql);
  db.run("INSERT INTO meta (id, schema_hash) VALUES (?, ?)", [
    1,
    createHash("sha256").update(sql).digest("hex"),
  ]);
  for (const [section, value] of Object.entries(settings))
    db.run("INSERT INTO settings (section, value) VALUES (?, ?)", [section, JSON.stringify(value)]);
  db.close();
}

rmSync(CHECK_DIR, { recursive: true, force: true });
mkdirSync(USER_DATA, { recursive: true });
const engine = path.join(USER_DATA, "engine");
if (existsSync(DEV_ENGINE)) symlinkSync(DEV_ENGINE, engine);
seedDatabase({ api: { enabled: true }, engine: engineSettings(engine) });
cpSync(path.join(ROOT, "examples"), PROJECTS, { recursive: true });
console.log(USER_DATA);
