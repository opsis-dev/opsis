import { readFileSync } from "node:fs";
import path from "node:path";
import { generatedElementMeta } from "@opsis/core/elements/elements.generated";
import { BUNDLED_FONTS } from "@opsis/core/fonts/bundled.generated";
import { describe, expect, it } from "vitest";
import { FIXTURES_DIR, fixtureText, parityFixtures } from "../parity-fixtures";
import { ELEMENTS_JSON, elementsJsonText } from "./element-json";
import { SOURCE_LAYOUT_JSON, sourceLayoutJsonText } from "./source-layout-json";
import { TEMPLATES_JSON, templatesJsonText } from "./template-json";
import { ROOT } from "./tool-config";

const read = (file: string) => readFileSync(file, "utf8");

describe("files Rust reads", () => {
  it("hold the registry elements the ingest wrote", () => {
    expect(read(ELEMENTS_JSON)).toBe(elementsJsonText(generatedElementMeta));
  });

  it("hold the bundled fonts", () => {
    const file = path.join(ROOT, "crates", "compile", "src", "fonts", "bundled.generated.json");
    expect(read(file)).toBe(`${JSON.stringify(BUNDLED_FONTS, null, 2)}\n`);
  });

  it("hold where Export as code finds its sources", () => {
    expect(read(SOURCE_LAYOUT_JSON)).toBe(sourceLayoutJsonText());
  });

  it("hold the project templates", () => {
    expect(read(TEMPLATES_JSON)).toBe(templatesJsonText());
  });

  it("hold the parity fixtures the TS compile writes today", () => {
    for (const [name, value] of Object.entries(parityFixtures()))
      expect({ name, text: read(path.join(FIXTURES_DIR, name)) }).toEqual({
        name,
        text: fixtureText(value),
      });
  }, 120_000);
});
