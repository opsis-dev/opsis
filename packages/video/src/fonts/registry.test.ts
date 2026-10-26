import { afterEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_PROJECT_FONTS } from "#video/common/fonts.ts";
import {
  applyProjectFonts,
  fontFileFor,
  registerFaces,
  unregisterFaces,
} from "#video/fonts/registry.ts";
import type { FontFamilyDef } from "#video/generated/spec.ts";

class UnreadableFace {
  load(): Promise<never> {
    return Promise.reject(new Error("Not a font."));
  }
}

const broken: FontFamilyDef = {
  id: "broken-sans",
  family: "Broken Sans",
  category: "sans-serif",
  source: "file",
  faces: [
    { file: "project:assets/fonts/broken.woff2", weight: "400", style: "normal", format: "woff2" },
  ],
};

afterEach(() => {
  unregisterFaces("df-test");
  vi.unstubAllGlobals();
});

describe("registering font faces", () => {
  it("reports an unreadable font once, however often the same font is registered", async () => {
    vi.stubGlobal("document", { fonts: { add: () => undefined, delete: () => true } });
    vi.stubGlobal("FontFace", UnreadableFace);
    const warnings: string[] = [];
    const warn = () => warnings.push(broken.family);
    for (let i = 0; i < 3; i++) await registerFaces("df-test", broken, (ref) => ref, warn);
    expect(warnings).toEqual(["Broken Sans"]);
    const changed = { ...broken, faces: [{ ...broken.faces[0], weight: "700" }] };
    await registerFaces("df-test", changed, (ref) => ref, warn);
    expect(warnings).toEqual(["Broken Sans", "Broken Sans"]);
  });
});

describe("finding a font's file for 3D text", () => {
  const face = (file: string, weight: string, style: "normal" | "italic") => ({
    file: `project:assets/fonts/serif/${file}`,
    weight,
    style,
    format: "woff2" as const,
  });
  const serif: FontFamilyDef = {
    id: "test-serif",
    family: "Test Serif",
    category: "serif",
    source: "file",
    faces: [
      face("regular.woff2", "400", "normal"),
      face("bold.woff2", "700", "normal"),
      face("italic.woff2", "400", "italic"),
    ],
  };

  it("picks the face nearest the weight, in the asked style when the family has it", async () => {
    await applyProjectFonts(
      { ...DEFAULT_PROJECT_FONTS, heading: "Test Serif", families: [serif] },
      (ref) => ref.replace("project:", "/p/"),
      [],
    );
    expect(fontFileFor("Test Serif", 800, false)).toEqual({
      url: "/p/assets/fonts/serif/bold.woff2",
      italic: false,
    });
    expect(fontFileFor("Test Serif", 300, false)?.url).toBe("/p/assets/fonts/serif/regular.woff2");
    expect(fontFileFor("Test Serif", 700, true)).toEqual({
      url: "/p/assets/fonts/serif/italic.woff2",
      italic: true,
    });
    expect(fontFileFor("df-heading", 700, false)?.url).toBe("/p/assets/fonts/serif/bold.woff2");
  });

  it("finds a bundled variable font and nothing for an unknown one", async () => {
    await applyProjectFonts(DEFAULT_PROJECT_FONTS, (ref) => ref, []);
    expect(fontFileFor("Outfit", 700, true)).toEqual({
      url: "app:fonts/bundled/outfit/outfit-latin-wght-normal.woff2",
      italic: false,
    });
    expect(fontFileFor("No Such Font", 400, false)).toBeNull();
  });
});
