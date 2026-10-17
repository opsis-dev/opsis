import path from "node:path";
import { describe, expect, it } from "vitest";
import { inspectFontFiles, inspectInstalledFonts } from "./fonts";

const bundled = (file: string) => path.resolve("public/fonts/bundled", file);

describe("inspectFontFiles", () => {
  it("reads the family, weight range and style of each face and finds the licence", async () => {
    const inspected = await inspectFontFiles(null, [
      bundled("dm-sans/dm-sans-latin-wght-normal.woff2"),
      bundled("dm-sans/dm-sans-latin-wght-italic.woff2"),
      bundled("dm-sans/LICENSE"),
    ]);
    const face = { family: "DM Sans 9pt", weight: "100 1000", variable: true, format: "woff2" };
    expect(inspected).toMatchObject({
      families: [
        {
          family: "DM Sans 9pt",
          faces: [
            { ...face, name: "dm-sans-latin-wght-normal.woff2", style: "normal" },
            { ...face, name: "dm-sans-latin-wght-italic.woff2", style: "italic" },
          ],
        },
      ],
      license: "LICENSE",
      skipped: 0,
      unreadable: 0,
    });
  });
});

describe("inspectInstalledFonts", () => {
  it("counts faces it can't read", () => {
    const inspected = inspectInstalledFonts([
      { postscriptName: "Broken", data: new Uint8Array([1, 2, 3, 4]).buffer },
    ]);
    expect(inspected).toMatchObject({ families: [], skipped: 0, unreadable: 1 });
  });
});
