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
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { DEV_ENGINE, PROJECTS, ROOT, RUN_DIR, USER_DATA } from "./paths";

const SCHEMA = path.join(ROOT, "crates", "store", "schema.sql");

const META_ID = 1;

function seedDatabase(settings: Record<string, unknown>): void {
  const sql = readFileSync(SCHEMA, "utf8");
  const db = new DatabaseSync(path.join(USER_DATA, "opsis.db"));
  db.exec(sql);
  db.prepare("INSERT INTO meta (id, schema_hash) VALUES (?, ?)").run(
    META_ID,
    createHash("sha256").update(sql).digest("hex"),
  );
  const row = db.prepare("INSERT INTO settings (section, value) VALUES (?, ?)");
  for (const [section, value] of Object.entries(settings)) row.run(section, JSON.stringify(value));
  db.close();
}

const EXE = process.platform === "win32" ? ".exe" : "";
const SHELL_NAME = `chrome-headless-shell${EXE}`;

function foldersIn(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => path.join(dir, entry.name));
}

function downloadedBrowser(engine: string): string | null {
  const root = path.join(engine, "chrome-headless-shell");
  if (!existsSync(root)) return null;
  const builds = foldersIn(root).flatMap(foldersIn);
  const shells = builds.map((build) => path.join(build, SHELL_NAME));
  return shells.find((file) => existsSync(file)) ?? null;
}

function engineSettings(engine: string): Record<string, { path: string } | null> {
  function tool(file: string) {
    return existsSync(file) ? { path: file } : null;
  }
  const browser = downloadedBrowser(engine);
  return {
    browser: browser ? { path: browser } : null,
    ffmpeg: tool(path.join(engine, "ffmpeg", `ffmpeg${EXE}`)),
  };
}

export function freshRun(): void {
  rmSync(RUN_DIR, { recursive: true, force: true });
  mkdirSync(USER_DATA, { recursive: true });
  const engine = path.join(USER_DATA, "engine");
  if (existsSync(DEV_ENGINE)) symlinkSync(DEV_ENGINE, engine);
  seedDatabase({ api: { enabled: true }, engine: engineSettings(engine) });
  cpSync(path.join(ROOT, "examples"), PROJECTS, { recursive: true });
}
