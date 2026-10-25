import { readFileSync } from "node:fs";
import path from "node:path";
import { formatHex } from "culori";
import { describe, expect, it } from "vitest";
import { CANVAS_BACKGROUND } from "./canvas-color";

describe("window background", () => {
  it("matches the canvas colour token, so no other colour shows while the page loads", () => {
    const css = readFileSync(
      path.resolve(import.meta.dirname, "../../apps/ui/src/index.css"),
      "utf8",
    );
    const root = css.slice(css.indexOf(":root {"));
    const [, token] = /--background: (oklch\([^)]*\))/.exec(root) ?? [];
    expect(token).toBeDefined();
    expect(formatHex(token)).toBe(CANVAS_BACKGROUND);
  });
});
