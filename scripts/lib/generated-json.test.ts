import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { PLAYWRIGHT_DIR, playwrightFiles } from "./playwright-scripts";
import { SOURCE_LAYOUT_JSON, sourceLayoutJsonText } from "./source-layout-json";

const read = (file: string) => readFileSync(file, "utf8");

describe("files Rust reads", () => {
  it("hold where Export as code finds its sources", () => {
    expect(read(SOURCE_LAYOUT_JSON)).toBe(sourceLayoutJsonText());
  });

  it("hold the page scripts and tables taken from the installed Playwright", () => {
    for (const [name, text] of Object.entries(playwrightFiles()))
      expect({ name, text: read(path.join(PLAYWRIGHT_DIR, name)) }).toEqual({ name, text });
  });
});
