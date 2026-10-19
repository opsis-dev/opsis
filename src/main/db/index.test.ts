import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { getTableName } from "drizzle-orm";
import { afterEach, describe, expect, it, vi } from "vitest";
import { closeDatabase, db, IN_MEMORY, openDatabase, SCHEMA_HASH } from ".";
import { tempDirs } from "../../engine/test-temp-dirs";
import * as schema from "./schema";

vi.mock("electron", () => ({ app: { getPath: () => "" } }));

const tempDir = tempDirs("opsis-db-");

afterEach(closeDatabase);

const tableNames = () =>
  db()
    .$client.prepare("SELECT name FROM sqlite_schema WHERE type = 'table' ORDER BY name")
    .all()
    .map((row) => row.name);

describe("openDatabase", () => {
  it("creates every table of the schema", () => {
    openDatabase(IN_MEMORY);
    expect(tableNames()).toEqual(Object.values(schema).map(getTableName).sort());
    expect(db().select().from(schema.meta).get()?.schemaHash).toBe(SCHEMA_HASH);
  });

  it("keeps a database made for this schema", async () => {
    const file = path.join(await tempDir(), "opsis.db");
    openDatabase(file);
    db().insert(schema.settings).values({ section: "app", value: {} }).run();
    openDatabase(file);
    expect(db().select().from(schema.settings).all()).toHaveLength(1);
  });

  it("replaces a database made for another schema", async () => {
    const file = path.join(await tempDir(), "opsis.db");
    openDatabase(file);
    db().insert(schema.settings).values({ section: "app", value: {} }).run();
    db().update(schema.meta).set({ schemaHash: "older" }).run();
    openDatabase(file);
    expect(db().select().from(schema.settings).all()).toEqual([]);
    expect(db().select().from(schema.meta).get()?.schemaHash).toBe(SCHEMA_HASH);
  });

  it("leaves nothing behind when a transaction fails", () => {
    openDatabase(IN_MEMORY);
    expect(() =>
      db().transaction((tx) => {
        tx.insert(schema.settings).values({ section: "app", value: {} }).run();
        tx.insert(schema.settings).values({ section: "app", value: {} }).run();
      }),
    ).toThrow(/^Failed query: insert into "settings"/);
    expect(db().select().from(schema.settings).all()).toEqual([]);
  });
});

describe("schema.generated.sql", () => {
  it("matches what drizzle-kit exports for the schema", () => {
    const exported = execFileSync(
      process.execPath,
      [path.resolve("node_modules/drizzle-kit/bin.cjs"), "export", "--config", "drizzle.config.ts"],
      { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    );
    expect(readFileSync(path.join(import.meta.dirname, "schema.generated.sql"), "utf8")).toBe(
      exported,
    );
  });
});
