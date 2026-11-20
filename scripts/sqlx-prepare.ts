import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { ROOT } from "./lib/tool-config";

const SCHEMAS = [
  path.join(ROOT, "crates", "persistence", "meta.sql"),
  path.join(ROOT, "crates", "store", "schema.sql"),
  path.join(ROOT, "crates", "agent", "schema.sql"),
];
const DIR = path.join(os.homedir(), ".cache", "opsis-sqlx");
const DATABASE = path.join(DIR, "schema.db");

rmSync(DIR, { recursive: true, force: true });
mkdirSync(DIR, { recursive: true });
const db = new DatabaseSync(DATABASE);
for (const schema of SCHEMAS) db.exec(readFileSync(schema, "utf8"));
db.close();

execFileSync("cargo", ["sqlx", "prepare", "--workspace", "--", "--all-targets"], {
  cwd: ROOT,
  stdio: "inherit",
  env: { ...process.env, DATABASE_URL: `sqlite://${DATABASE}`, SQLX_OFFLINE: "false" },
});
rmSync(DIR, { recursive: true, force: true });
console.log("Wrote the sqlx query cache in .sqlx.");
