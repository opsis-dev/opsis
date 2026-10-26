import { describe, expect, it, vi } from "vitest";
import { asSchema, fieldMeta, shapeEntries } from "@/core/schema";
import { catalogNodes } from "@/store/catalog";
import { typedField } from "./form/schema-kind";
import { widgetTakes } from "./schema-form";

vi.hoisted(() => Object.assign(window, { opsis: { platform: "linux" } }));

describe("Inspector widgets", () => {
  it("can draw every setting that names them", () => {
    const misfits = catalogNodes().flatMap((def) =>
      shapeEntries(asSchema(def.params)).flatMap(([key, field]) => {
        const { widget } = fieldMeta(field);
        const kind = typedField(field, undefined)?.kind;
        return widget && kind && !widgetTakes(widget, kind) ? [`${def.type}.${key}`] : [];
      }),
    );
    expect(misfits).toEqual([]);
  });

  it("knows a field kind for every setting", () => {
    const unknown = catalogNodes().flatMap((def) =>
      shapeEntries(asSchema(def.params)).flatMap(([key, field]) =>
        fieldMeta(field).hidden || typedField(field, field.default) ? [] : [`${def.type}.${key}`],
      ),
    );
    expect(unknown).toEqual([]);
  });
});
