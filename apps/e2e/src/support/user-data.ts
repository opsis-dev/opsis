import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, symlinkSync } from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { DEV_ENGINE, PROJECTS, ROOT, RUN_DIR, USER_DATA } from "./paths";

const SCHEMA = path.join(ROOT, "crates", "store", "db", "schema.generated.sql");

const META_ID = 1;

function seedDatabase(settings: Record<string, unknown>): void {
  const sql = readFileSync(SCHEMA, "utf8");
  const db = new DatabaseSync(path.join(USER_DATA, "opsis.db"));
  db.exec(sql);
  db.prepare("INSERT INTO meta (id, schemaHash) VALUES (?, ?)").run(
    META_ID,
    createHash("sha256").update(sql).digest("hex"),
  );
  const row = db.prepare("INSERT INTO settings (section, value) VALUES (?, ?)");
  for (const [section, value] of Object.entries(settings)) row.run(section, JSON.stringify(value));
  db.close();
}

export function freshRun(): void {
  rmSync(RUN_DIR, { recursive: true, force: true });
  mkdirSync(USER_DATA, { recursive: true });
  if (existsSync(DEV_ENGINE)) symlinkSync(DEV_ENGINE, path.join(USER_DATA, "engine"));
  seedDatabase({ api: { enabled: true } });
  cpSync(path.join(ROOT, "examples"), PROJECTS, { recursive: true });
}
