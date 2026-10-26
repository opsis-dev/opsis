import { readFileSync } from "node:fs";
import path from "node:path";
import { opaqueHex, parseColor } from "@opsis/video/common/color";
import { describe, expect, it } from "vitest";

const ROOT = path.resolve(import.meta.dirname, "..", "..", "..", "..");

const read = (...parts: string[]) => readFileSync(path.join(ROOT, ...parts), "utf8");

const hexByte = (text: string) => Number.parseInt(text, 16).toString(16).padStart(2, "0");

describe("window background", () => {
  it("matches the canvas colour token, so no other colour shows while the page loads", () => {
    const css = read("apps", "ui", "src", "index.css");
    const [, token = ""] =
      /--background: (oklch\([^)]*\))/.exec(css.slice(css.indexOf(":root {"))) ?? [];
    const [, r = "", g = "", b = ""] =
      /const BACKGROUND: Color = Color\(0x(\w\w), 0x(\w\w), 0x(\w\w), 0xff\);/.exec(
        read("crates", "desktop", "src", "windows.rs"),
      ) ?? [];
    const color = parseColor(token);
    expect(color).not.toBeNull();
    expect(color && opaqueHex(color)).toBe(`#${hexByte(r)}${hexByte(g)}${hexByte(b)}`);
  });
});
