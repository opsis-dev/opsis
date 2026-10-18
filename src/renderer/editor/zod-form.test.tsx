import { describe, expect, it, vi } from "vitest";
import { fieldMeta, shapeEntries } from "@/shared/elements/introspect";
import { NODE_DEFS } from "@/shared/nodes/registry";
import { typedField } from "./form/schema-kind";
import { widgetTakes } from "./zod-form";

vi.hoisted(() => Object.assign(window, { demoflow: { platform: "linux" } }));

describe("Inspector widgets", () => {
  it("can draw every setting that names them", () => {
    const misfits = NODE_DEFS.flatMap((def) =>
      shapeEntries(def.params).flatMap(([key, field]) => {
        const { widget } = fieldMeta(field);
        const kind = typedField(field, undefined)?.kind;
        return widget && kind && !widgetTakes(widget, kind) ? [`${def.type}.${key}`] : [];
      }),
    );
    expect(misfits).toEqual([]);
  });
});
