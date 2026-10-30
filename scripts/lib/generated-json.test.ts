import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { PLAYWRIGHT_DIR, playwrightFiles } from "./playwright-scripts";

function read(file: string) {
  return readFileSync(file, "utf8");
}

describe("files Rust reads", () => {
  it("hold the page scripts and tables taken from the installed Playwright", () => {
    for (const [name, text] of Object.entries(playwrightFiles()))
      expect({ name, text: read(path.join(PLAYWRIGHT_DIR, name)) }).toEqual({ name, text });
  });
});
