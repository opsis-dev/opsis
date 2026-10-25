import { fieldMeta, shapeEntries } from "@opsis/core/elements/introspect";
import { NODE_DEFS } from "@opsis/core/nodes/registry";
import { describe, expect, it, vi } from "vitest";
import { typedField } from "./form/schema-kind";
import { widgetTakes } from "./zod-form";

vi.hoisted(() => Object.assign(window, { opsis: { platform: "linux" } }));

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
