import { zColor } from "@remotion/zod-types";
import { describe, expect, it } from "vitest";
import { z } from "zod";
import { typedField } from "./schema-kind";

describe("typedField", () => {
  it("picks the list inputs from plain schemas", () => {
    expect(typedField(z.array(z.string()).default([]), undefined)?.kind).toBe("stringList");
    expect(typedField(z.array(z.number()).default([1]), [1, 2])?.kind).toBe("numberList");
    const palette = typedField(z.array(zColor()).min(2).max(10), ["#000", "#fff"]);
    expect(palette).toMatchObject({ kind: "colorList", min: 2, max: 10 });
  });

  it("reads gradient and theme widgets", () => {
    const gradient = z.string().meta({ widget: "gradient" });
    expect(typedField(gradient, "linear-gradient(red, blue)")?.kind).toBe("gradient");
    const theme = z
      .object({ mode: z.enum(["light", "dark"]), colors: z.record(z.string(), z.string()) })
      .meta({ widget: "theme" });
    expect(typedField(theme, { mode: "dark", colors: {} })?.kind).toBe("theme");
    expect(typedField(theme, { mode: "dim" })).toBeNull();
  });
});
