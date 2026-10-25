import { describe, expect, it } from "vitest";
import { findFamily, firstFamily, fontSlug, fontStack, roleFamilyName } from "#core/fonts";
import { BUNDLED_FONTS } from "#core/fonts/bundled.generated";
import { familyNameFrom, planFontFiles, type FontCandidate } from "#core/fonts/plan";
import { DEFAULT_PROJECT_FONTS, type FontFamilyDef } from "#core/fonts/types";

const custom: FontFamilyDef = {
  id: "acme-sans",
  family: "Acme Sans",
  category: "sans-serif",
  source: "file",
  faces: [
    {
      file: "project:assets/fonts/acme-sans/a.woff2",
      weight: "400",
      style: "normal",
      format: "woff2",
    },
  ],
};

describe("font lookup", () => {
  it("bundles Latin files only", () => {
    expect(BUNDLED_FONTS.length).toBeGreaterThan(0);
    for (const f of BUNDLED_FONTS)
      for (const face of f.faces)
        expect(face.file).toMatch(/^app:fonts\/bundled\/.+-latin-.+\.woff2$/);
  });

  it("finds project fonts first, then bundled ones, and maps Google Sans", () => {
    const fonts = { ...DEFAULT_PROJECT_FONTS, families: [custom] };
    expect(findFamily("acme sans", fonts)?.id).toBe("acme-sans");
    expect(findFamily("Inter", fonts)?.source).toBe("bundled");
    expect(findFamily("Google Sans", fonts)?.family).toBe("Google Sans Flex");
    expect(findFamily("Nope", fonts)).toBeUndefined();
  });

  it("uses the text font for headings unless a heading font is set", () => {
    expect(roleFamilyName("heading", { ...DEFAULT_PROJECT_FONTS, main: "Manrope" })).toBe(
      "Manrope",
    );
    expect(
      roleFamilyName("heading", { ...DEFAULT_PROJECT_FONTS, main: "Manrope", heading: "Fraunces" }),
    ).toBe("Fraunces");
  });

  it("builds stacks with the right fallback and reads them back", () => {
    expect(fontStack("Fraunces", { families: [] })).toBe('"Fraunces", serif');
    expect(fontStack("JetBrains Mono", { families: [] })).toBe('"JetBrains Mono", monospace');
    expect(firstFamily('"Plus Jakarta Sans", sans-serif')).toBe("Plus Jakarta Sans");
    expect(fontSlug("IBM Plex Sans!")).toBe("ibm-plex-sans");
  });
});

const c = (name: string, extra: Partial<FontCandidate>): FontCandidate => ({
  name,
  family: "General Sans",
  weight: "400",
  style: "normal",
  variable: false,
  format: "truetype",
  size: 1,
  ...extra,
});

describe("planning font files from a download", () => {
  it("keeps one file per style, preferring variable fonts and woff2", () => {
    const plan = planFontFiles([
      c("TTF/GeneralSans-Variable.ttf", { weight: "200 700", variable: true }),
      c("WEB/GeneralSans-Variable.woff2", { weight: "200 700", variable: true, format: "woff2" }),
      c("WEB/GeneralSans-VariableItalic.woff2", {
        weight: "200 700",
        variable: true,
        style: "italic",
        format: "woff2",
      }),
      c("OTF/GeneralSans-Bold.otf", { weight: "700", format: "opentype" }),
      c("WEB/GeneralSans-Bold.woff2", { weight: "700", format: "woff2" }),
    ]);
    expect(plan).toHaveLength(1);
    expect(plan[0].faces.map((f) => f.name)).toEqual([
      "WEB/GeneralSans-Variable.woff2",
      "WEB/GeneralSans-VariableItalic.woff2",
    ]);
  });

  it("falls back to static weights in order when there is no variable font", () => {
    const plan = planFontFiles([
      c("Bold.ttf", { weight: "700" }),
      c("Regular.otf", { weight: "400", format: "opentype" }),
      c("Regular.woff", { weight: "400", format: "woff" }),
      c("Other.ttf", { family: "Other Mono" }),
    ]);
    expect(plan.map((p) => p.family)).toEqual(["General Sans", "Other Mono"]);
    expect(plan[0].faces.map((f) => f.name)).toEqual(["Regular.woff", "Bold.ttf"]);
  });
});

describe("family names inside font files", () => {
  it("uses the typographic family and drops style words otherwise", () => {
    expect(familyNameFrom("General Sans", "General Sans Medium")).toBe("General Sans");
    expect(familyNameFrom(null, "General Sans Variable")).toBe("General Sans");
    expect(familyNameFrom(null, "Acme Sans Semi Bold Italic")).toBe("Acme Sans");
    expect(familyNameFrom(null, "Inter")).toBe("Inter");
    expect(familyNameFrom(null, "Bold")).toBe("Bold");
  });
});
