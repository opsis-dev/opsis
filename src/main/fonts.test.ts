import path from "node:path";
import { must } from "@opsis/core/test-fixtures";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { tempDirs } from "../engine/test-temp-dirs";
import { closeDatabase, db, IN_MEMORY, openDatabase } from "./db";
import { fonts } from "./db/schema";
import {
  addFamilies,
  addInspectedFonts,
  inspectFontFiles,
  inspectInstalledFonts,
  listLibrary,
  releaseInspectedFonts,
  removeLibraryFont,
} from "./fonts";

const userData = vi.hoisted(() => ({ dir: "" }));
vi.mock("electron", () => ({ app: { getPath: () => userData.dir } }));

const tempDir = tempDirs("opsis-fonts-");

beforeEach(async () => {
  userData.dir = await tempDir();
  openDatabase(IN_MEMORY);
  return closeDatabase;
});

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

describe("inspected fonts", () => {
  const files = [bundled("dm-sans/dm-sans-latin-wght-normal.woff2")];
  const NOT_READY = "Those fonts are no longer ready to add. Choose them again.";

  it("lets go of fonts the review dialog closed without adding", async () => {
    const { token } = must(await inspectFontFiles(null, files), "inspected fonts");
    releaseInspectedFonts(token);
    await expect(addInspectedFonts(token, ["DM Sans 9pt"])).rejects.toThrow(NOT_READY);
  });

  it("keeps only the latest inspection", async () => {
    const first = must(await inspectFontFiles(null, files), "inspected fonts");
    await inspectFontFiles(null, files);
    await expect(addInspectedFonts(first.token, ["DM Sans 9pt"])).rejects.toThrow(NOT_READY);
  });
});

const oldFont = {
  family: "Old",
  category: "sans-serif",
  source: "file",
  files: [
    {
      name: "old.woff2",
      bytes: new Uint8Array([1]),
      face: { weight: "400", style: "normal", format: "woff2" },
    },
  ],
} as const;

describe("font library", () => {
  it("keeps entries it can't read when the library changes", async () => {
    const unreadable = JSON.stringify({ id: "old", family: "Old" });
    db().insert(fonts).values({ fontId: "old", def: unreadable }).run();
    await addFamilies([{ ...oldFont, files: [...oldFont.files] }]);
    expect(db().select().from(fonts).all()[0]).toMatchObject({ fontId: "old", def: unreadable });
    expect(listLibrary().map((f) => f.id)).toEqual(["old-2"]);
  });

  it("replaces a family added again and removes it by id", async () => {
    const [first] = await addFamilies([{ ...oldFont, files: [...oldFont.files] }]);
    await addFamilies([{ ...oldFont, files: [...oldFont.files] }]);
    expect(listLibrary().map((f) => f.id)).toEqual([first?.id]);
    await removeLibraryFont(must(first, "added family").id);
    expect(listLibrary()).toEqual([]);
  });
});
