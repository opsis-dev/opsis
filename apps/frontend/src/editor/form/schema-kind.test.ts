import { describe, expect, it } from "vitest";
import { typedField } from "./schema-kind";

const colour = { type: "string", "x-check": { rule: "colour", message: "Pick a colour" } };

describe("typedField", () => {
  it("picks the list inputs from plain schemas", () => {
    expect(typedField({ type: "array", items: { type: "string" } }, undefined)?.kind).toBe(
      "stringList",
    );
    expect(typedField({ type: "array", items: { type: "number" } }, [1, 2])?.kind).toBe(
      "numberList",
    );
    const palette = typedField({ type: "array", items: colour, minItems: 2, maxItems: 10 }, [
      "#000",
      "#fff",
    ]);
    expect(palette).toMatchObject({ kind: "colorList", min: 2, max: 10 });
  });

  it("reads gradient and theme widgets", () => {
    const gradient = { type: "string", "x-widget": "gradient" };
    expect(typedField(gradient, "linear-gradient(red, blue)")?.kind).toBe("gradient");
    const theme = { type: "object", "x-widget": "theme" };
    expect(typedField(theme, { mode: "dark", colors: {} })?.kind).toBe("theme");
    expect(typedField(theme, { mode: "dim" })).toBeNull();
  });

  it("lets a number that takes null be cleared", () => {
    expect(typedField({ type: ["integer", "null"] }, null)).toMatchObject({
      kind: "optionalNumber",
      value: undefined,
    });
  });
});
