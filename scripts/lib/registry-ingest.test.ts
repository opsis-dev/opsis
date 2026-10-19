import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import type { CuratedElement } from "../registry-ingest.list";
import { zodFor } from "./registry-codegen";
import { inferColumns, isColorList, rowTypeName } from "./registry-columns";
import { namesLibrary, themeFallbacks } from "./registry-controls";
import { componentProps, hiddenDefaultTexts, hiddenPropProblems } from "./registry-props";
import { bindingDefaults } from "./ts-literal";

describe("zodFor", () => {
  it("writes a palette with its length limits", () => {
    expect(zodFor("colors", { type: "colors", min: 2, max: 7, default: ["#000", "#fff"] })).toBe(
      'z.array(zColor()).min(2).max(7).default(["#000","#fff"]).meta({ description: "colors" })',
    );
  });

  it("writes rows as objects built from their columns", () => {
    const rows = zodFor("steps", {
      type: "rows",
      default: [{ at: 0, code: "a" }],
      description: "Versions",
      columns: {
        code: { type: "code", default: "", description: "Code" },
        at: { type: "number", default: 0, description: "Show at", unit: "frames" },
      },
    });
    expect(rows).toBe(
      'z.array(z.object({ code: z.string().default("").meta({ description: "Code", widget: "code" }), at: z.number().default(0).meta({ description: "Show at", unit: "frames" }) })).default([{"at":0,"code":"a"}]).meta({ description: "Versions" })',
    );
  });

  it("refuses rows without columns", () => {
    expect(zodFor("rows", { type: "rows", default: [] })).toBeNull();
  });

  it("writes optional settings and colours that may be empty", () => {
    expect(zodFor("max", { type: "number", optional: true, emptyLabel: "Auto" })).toBe(
      'z.number().optional().meta({ description: "max", zeroLabel: "Auto" })',
    );
    expect(zodFor("bg", { type: "color", default: "", emptyLabel: "Theme" })).toBe(
      'z.string().default("").meta({ description: "bg", widget: "color", zeroLabel: "Theme" })',
    );
  });

  it("writes gradients and themes with their widgets", () => {
    expect(zodFor("g", { type: "gradient", default: "linear-gradient(red, blue)" })).toBe(
      'z.string().default("linear-gradient(red, blue)").meta({ description: "g", widget: "gradient" })',
    );
    expect(
      zodFor("theme", {
        type: "theme",
        default: "light",
        description: "Theme",
        themes: { light: { background: "#fff" }, dark: { background: "#000" } },
      }),
    ).toBe(
      'themeValueSchema.default({"mode":"light","colors":{}}).meta({ description: "Theme", widget: "theme", themes: {"light":{"background":"#fff"},"dark":{"background":"#000"}} })',
    );
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
