import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { ROOT } from "./tool-config";

describe("schema.generated.sql", () => {
  it("matches what drizzle-kit exports for the schema", () => {
    const exported = execFileSync(
      process.execPath,
      [
        path.join(ROOT, "node_modules/drizzle-kit/bin.cjs"),
        "export",
        "--config",
        "drizzle.config.ts",
      ],
      { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    );
    const file = path.join(ROOT, "crates", "store", "db", "schema.generated.sql");
    expect(readFileSync(file, "utf8")).toBe(exported);
  });
});
