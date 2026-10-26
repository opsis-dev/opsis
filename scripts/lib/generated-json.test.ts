import { readFileSync } from "node:fs";
import path from "node:path";
import { BUNDLED_FONTS } from "@opsis/video/common/bundled-fonts.generated";
import { describe, expect, it } from "vitest";
import { PLAYWRIGHT_DIR, playwrightFiles } from "./playwright-scripts";
import { SOURCE_LAYOUT_JSON, sourceLayoutJsonText } from "./source-layout-json";
import { ROOT } from "./tool-config";

const read = (file: string) => readFileSync(file, "utf8");

describe("files Rust reads", () => {
  it("hold the bundled fonts", () => {
    const file = path.join(ROOT, "crates", "compile", "src", "fonts", "bundled.generated.json");
    expect(read(file)).toBe(`${JSON.stringify(BUNDLED_FONTS, null, 2)}\n`);
  });

  it("hold where Export as code finds its sources", () => {
    expect(read(SOURCE_LAYOUT_JSON)).toBe(sourceLayoutJsonText());
  });

  it("hold the page scripts and tables taken from the installed Playwright", () => {
    for (const [name, text] of Object.entries(playwrightFiles()))
      expect({ name, text: read(path.join(PLAYWRIGHT_DIR, name)) }).toEqual({ name, text });
  });
});
