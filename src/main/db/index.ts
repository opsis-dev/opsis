import { mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";
import { eq } from "drizzle-orm";
import { drizzle } from "drizzle-orm/node-sqlite";
import { app } from "electron";
import { sha256Hex } from "../../engine/hash";
import { logWarning } from "../../engine/log";
import { meta } from "./schema";
import ddl from "./schema.generated.sql?raw";

const DB_FILE = "opsis.db";
export const IN_MEMORY = ":memory:";
const BUSY_TIMEOUT_MS = 5000;
const META_ID = 1;
export const SCHEMA_HASH = sha256Hex(ddl);

function connect(file: string) {
  if (file !== IN_MEMORY) mkdirSync(path.dirname(file), { recursive: true });
  const client = new DatabaseSync(file);
  client.exec(
    `PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = ${BUSY_TIMEOUT_MS};`,
  );
  return drizzle({ client });
}

export type Database = ReturnType<typeof connect>;

function storedHash(database: Database): string | null {
  try {
    return database.select().from(meta).where(eq(meta.id, META_ID)).get()?.schemaHash ?? null;
  } catch {
    return null;
  }
}

const hasTables = (database: Database): boolean =>
  database.$client.prepare("SELECT 1 FROM sqlite_schema LIMIT 1").get() !== undefined;

function createSchema(database: Database): Database {
  database.transaction(() => {
    database.$client.exec(ddl);
    database.insert(meta).values({ id: META_ID, schemaHash: SCHEMA_HASH }).run();
  });
  return database;
}

function removeDatabaseFiles(file: string): void {
  for (const suffix of ["", "-wal", "-shm"]) rmSync(`${file}${suffix}`, { force: true });
}

function openAt(file: string): Database {
  const database = connect(file);
  if (storedHash(database) === SCHEMA_HASH) return database;
  if (!hasTables(database)) return createSchema(database);
  logWarning("database", `${file} was made for another schema and was replaced`);
  database.$client.close();
  if (file !== IN_MEMORY) removeDatabaseFiles(file);
  return createSchema(connect(file));
}

let current: Database | null = null;

export function openDatabase(file = path.join(app.getPath("userData"), DB_FILE)): void {
  current?.$client.close();
  current = openAt(file);
}

export function closeDatabase(): void {
  current?.$client.close();
  current = null;
}

export function db(): Database {
  if (!current) throw new Error("The app's saved state isn't open yet.");
  return current;
}
