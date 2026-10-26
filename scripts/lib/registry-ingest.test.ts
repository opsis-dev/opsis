import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { parsePatch } from "diff";
import { describe, expect, it } from "vitest";
import type { CuratedElement } from "../registry-ingest.list";
import { fieldSchema } from "./registry-codegen";
import { inferColumns, isColorList, rowTypeName } from "./registry-columns";
import { namesLibrary, themeFallbacks } from "./registry-controls";
import { hasReason, patchSource } from "./registry-patches";
import { componentProps, hiddenDefaultTexts, hiddenPropProblems } from "./registry-props";
import { bindingDefaults } from "./ts-literal";

const COLOUR = { type: "string", "x-check": { rule: "colour", message: "Pick a colour" } };

describe("fieldSchema", () => {
  it("writes a palette with its length limits", () => {
    expect(
      fieldSchema("colors", { type: "colors", min: 2, max: 7, default: ["#000", "#fff"] }),
    ).toEqual({
      default: ["#000", "#fff"],
      description: "colors",
      minItems: 2,
      maxItems: 7,
      type: "array",
      items: COLOUR,
    });
  });

  it("writes rows as objects built from their columns", () => {
    const rows = fieldSchema("steps", {
      type: "rows",
      default: [{ at: 0, code: "a" }],
      description: "Versions",
      columns: {
        code: { type: "code", default: "", description: "Code" },
        at: { type: "number", default: 0, description: "Show at", unit: "frames" },
      },
    });
    expect(rows).toEqual({
      default: [{ at: 0, code: "a" }],
      description: "Versions",
      type: "array",
      items: {
        type: "object",
        properties: {
          code: { default: "", description: "Code", "x-widget": "code", type: "string" },
          at: { default: 0, description: "Show at", "x-unit": "frames", type: "number" },
        },
      },
    });
  });

  it("refuses rows without columns", () => {
    expect(fieldSchema("rows", { type: "rows", default: [] })).toBeNull();
  });

  it("writes optional settings and colours that may be empty", () => {
    expect(fieldSchema("max", { type: "number", optional: true, emptyLabel: "Auto" })).toEqual({
      description: "max",
      "x-zero-label": "Auto",
      type: "number",
    });
    expect(fieldSchema("bg", { type: "color", default: "", emptyLabel: "Theme" })).toEqual({
      default: "",
      description: "bg",
      "x-widget": "color",
      "x-zero-label": "Theme",
      type: "string",
    });
  });

  it("writes gradients and themes with their widgets", () => {
    expect(fieldSchema("g", { type: "gradient", default: "linear-gradient(red, blue)" })).toEqual({
      default: "linear-gradient(red, blue)",
      description: "g",
      "x-widget": "gradient",
      type: "string",
    });
    const themes = { light: { background: "#fff" }, dark: { background: "#000" } };
    expect(
      fieldSchema("theme", { type: "theme", default: "light", description: "Theme", themes }),
    ).toMatchObject({
      default: { mode: "light", colors: {} },
      description: "Theme",
      "x-widget": "theme",
      "x-themes": themes,
      type: "object",
      required: ["mode", "colors"],
    });
  });

  it("puts the default first, then the field's keywords, then its type", () => {
    const schema = fieldSchema("size", { type: "number", default: 4, min: 1, step: 1 });
    expect(Object.keys(schema ?? {})).toEqual([
      "default",
      "description",
      "x-step",
      "type",
      "minimum",
    ]);
  });
});

describe("inferColumns", () => {
  const members = new Map([
    ["kind", { type: '"added" | "fixed"', optional: false }],
    ["moveTo", { type: "number", optional: true }],
  ]);

  it("types columns from the default rows and the row type", () => {
    const { columns, problems } = inferColumns(
      [
        { kind: "added", text: "One", tint: "#ff0000", trend: [1, 2] },
        { kind: "fixed", text: "Two" },
      ],
      members,
    );
    expect(problems).toEqual([]);
    expect(columns.kind).toMatchObject({ type: "enum", options: ["added", "fixed"] });
    expect(columns.text).toMatchObject({ type: "text", default: "Two" });
    expect(columns.tint).toMatchObject({ type: "color", optional: true, emptyLabel: "None" });
    expect(columns.trend).toMatchObject({ type: "numbers", optional: true });
    expect(columns.moveTo).toMatchObject({ type: "number", optional: true });
  });

  it("keeps explicit columns when the default list is empty", () => {
    const { columns } = inferColumns([], new Map(), { image: { type: "asset", default: "" } });
    expect(Object.keys(columns)).toEqual(["image"]);
  });

  it("reports values it can't show", () => {
    expect(inferColumns([{ cells: ["a", true] }], new Map()).problems).toHaveLength(1);
  });

  it("reads the row type's name from a prop type", () => {
    expect(rowTypeName("ChangeRow[]")).toBe("ChangeRow");
    expect(rowTypeName("Array<Toast>")).toBe("Toast");
    expect(rowTypeName("string[]")).toBeUndefined();
  });
});

const COMPONENT = `
import { Props as Shared } from "./shared";
const NEUTRAL = ["#111", "#222"];
const SAMPLE = [{ label: "Remotion", mark: <svg aria-label="snapcn" /> }];
export interface WidgetProps { colors?: string[]; title: string; items?: unknown[]; width: number }
export function Widget({ colors = NEUTRAL, title = "Hi", items = SAMPLE, width }: WidgetProps) {
  const { speed = 2 } = {};
  return null;
}
export const Alias = Widget;
`;

describe("componentProps", () => {
  it("reads the props type, the destructured defaults and an alias", () => {
    const read = componentProps(COMPONENT, "Alias");
    expect(read?.props.map((p) => [p.name, p.required])).toEqual([
      ["colors", false],
      ["title", false],
      ["items", false],
      ["width", true],
    ]);
  });

  it("follows a props type imported from a sibling file", () => {
    const dir = mkdtempSync(path.join(tmpdir(), "opsis-props-"));
    writeFileSync(path.join(dir, "types.ts"), "export type TeaserProps = { brand?: string };");
    const file = path.join(dir, "index.tsx");
    const source = `import type { TeaserProps } from "./types";\nexport function Teaser(props: TeaserProps) { return null; }`;
    writeFileSync(file, source);
    expect(componentProps(source, "Teaser", file)?.props.map((p) => p.name)).toEqual(["brand"]);
  });

  it("reads defaults from any destructure in the file", () => {
    const defaults = bindingDefaults(COMPONENT);
    expect(defaults.get("colors")).toEqual(["#111", "#222"]);
    expect(defaults.get("speed")).toBe(2);
  });
});

describe("hiddenDefaultTexts", () => {
  it("finds text and labels in hidden defaults, through the constants they name", () => {
    expect(hiddenDefaultTexts(COMPONENT, ["items"]).get("items")).toEqual(["Remotion", "snapcn"]);
  });
});

describe("hiddenPropProblems", () => {
  it("names every prop that is neither shown nor omitted, and required ones without a value", () => {
    const dir = mkdtempSync(path.join(tmpdir(), "opsis-props-"));
    const file = path.join(dir, "widget.tsx");
    writeFileSync(file, COMPONENT);
    const item: CuratedElement = {
      registry: "remocn",
      name: "widget",
      category: "ui",
      omit: { colors: "internal" },
    };
    const problems = hiddenPropProblems({
      key: "remocn/widget",
      item,
      file,
      exportName: "Widget",
      shown: new Set(["title"]),
    });
    expect(problems).toEqual([
      'remocn/widget: the hidden default for "items" names a library, company or product: "Remotion"',
      'remocn/widget: the hidden default for "items" names a library, company or product: "snapcn"',
      'remocn/widget: prop "items" (default SAMPLE) is not a setting, a fixed prop or omitted with a reason',
      'remocn/widget: required prop "width" has no setting, fixed value or default',
    ]);
  });
});

describe("default checks", () => {
  it("catch libraries, companies and products", () => {
    expect(namesLibrary("AirPods Connected")).toBe(true);
    expect(namesLibrary("github.com/acme")).toBe(true);
    expect(namesLibrary("Built with remocn")).toBe(true);
    expect(namesLibrary("Recording started")).toBe(false);
  });

  it("find the theme token a prop falls back to", () => {
    expect([...themeFallbacks("const paper = ground ?? t.background; const x = y ?? z;")]).toEqual([
      ["ground", "background"],
    ]);
  });

  it("treat colour-named lists of colours as palettes", () => {
    expect(isColorList("laneColors", ["#fff", "rgba(0,0,0,0.5)"])).toBe(true);
    expect(isColorList("colors", ["#fff", "not a colour"])).toBe(false);
    expect(isColorList("labels", ["#fff"])).toBe(false);
  });
});

describe("element patches", () => {
  const [patch] = parsePatch(`--- a/x.tsx
+++ b/x.tsx
@@ -1,3 +1,3 @@
 a
-scale(1.05))
+scale(1.05)
 c
`);

  it("applies a patch that fits", () => {
    expect(patchSource("a\nscale(1.05))\nc\n", patch, "p")).toEqual({
      text: "a\nscale(1.05)\nc\n",
    });
  });

  it("leaves an applied patch alone, so the ingest can run again", () => {
    expect(patchSource("a\nscale(1.05)\nc\n", patch, "p")).toEqual({ text: "a\nscale(1.05)\nc\n" });
  });

  it("fails when the element changed upstream", () => {
    expect(patchSource("a\nscale(2)\nc\n", patch, "p").problem).toMatch(/no longer fits/);
  });

  it("needs each patch file to start with why it is needed", () => {
    expect(hasReason("The scale had a stray bracket.\n\ndiff --git a/x b/x\n")).toBe(true);
    expect(hasReason("diff --git a/x b/x\n")).toBe(false);
  });
});
