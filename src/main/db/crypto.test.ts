import { DatabaseSync } from "node:sqlite";
import { drizzle } from "drizzle-orm/node-sqlite";
import { integer, sqliteTable } from "drizzle-orm/sqlite-core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { encryptedText } from "./crypto";

const keyring = vi.hoisted(() => ({ available: true, backend: "gnome_libsecret" }));

vi.mock("electron", () => ({
  safeStorage: {
    isEncryptionAvailable: () => keyring.available,
    getSelectedStorageBackend: () => keyring.backend,
    encryptString: (value: string) => Buffer.from(`sealed:${value}`),
    decryptString: (bytes: Buffer) => {
      const text = bytes.toString();
      if (!text.startsWith("sealed:")) throw new Error("Error while decrypting the ciphertext");
      return text.slice("sealed:".length);
    },
  },
}));

const keys = sqliteTable("keys", { id: integer().primaryKey(), value: encryptedText().notNull() });

function database() {
  const client = new DatabaseSync(":memory:");
  client.exec("CREATE TABLE keys (id integer PRIMARY KEY, value text NOT NULL)");
  return drizzle({ client });
}

beforeEach(() => {
  keyring.available = true;
  keyring.backend = "gnome_libsecret";
});

describe("encryptedText", () => {
  it("stores the value sealed and reads it back", () => {
    const db = database();
    db.insert(keys).values({ id: 1, value: "sk-test" }).run();
    expect(db.$client.prepare("SELECT value FROM keys").get()?.value).not.toContain("sk-test");
    expect(db.select().from(keys).get()?.value).toBe("sk-test");
  });

  it("reports a value it can't decrypt", () => {
    const db = database();
    db.$client.exec("INSERT INTO keys VALUES (1, 'not sealed')");
    expect(() => db.select().from(keys).get()).toThrow(
      "This key can't be read on this computer. Enter it again.",
    );
  });

  it("refuses to save without secure storage", () => {
    keyring.available = false;
    expect(() => database().insert(keys).values({ id: 1, value: "sk-test" }).run()).toThrow(
      "This computer has no secure storage available, so keys can't be saved.",
    );
  });

  it.runIf(process.platform === "linux")("refuses to save when the keyring is plain text", () => {
    keyring.backend = "basic_text";
    expect(() => database().insert(keys).values({ id: 1, value: "sk-test" }).run()).toThrow(
      /no keyring to protect keys/,
    );
  });
});
