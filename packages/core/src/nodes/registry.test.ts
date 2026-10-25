import { describe, expect, it } from "vitest";
import { ELEMENTS } from "../elements";
import { CAPTIONS_ELEMENT } from "../elements/core";
import { FIELD_WIDGETS, fieldMeta, targetRoleKey } from "../elements/introspect";
import { defaultParams, NODE_DEFS } from "./registry";

const fieldsOf = (widget: string) =>
  NODE_DEFS.flatMap((def) =>
    Object.entries(def.params.shape)
      .filter(([, field]) => fieldMeta(field).widget === widget)
      .map(([key]) => ({ def, key })),
  );

describe("node registry", () => {
  it("has parseable defaults for every setting a node can start without", () => {
    const undefaulted = NODE_DEFS.flatMap((def) => {
      const params = defaultParams(def.type);
      return Object.entries(def.params.shape)
        .filter(([k, field]) => !(k in params) && !field.safeParse(undefined).success)
        .filter(([, field]) => fieldMeta(field).widget !== "target")
        .map(([k]) => `${def.type}.${k}`);
    });
    expect(undefaulted).toEqual([]);
  });

  it("declares a hidden role next to every picked element", () => {
    const missing = fieldsOf("target")
      .filter(({ def, key }) => {
        const role = def.params.shape[targetRoleKey(key)];
        return !role || !fieldMeta(role).hidden;
      })
      .map(({ def, key }) => `${def.type}.${key}`);
    expect(missing).toEqual([]);
  });

  it("generates one node per placeable registry element", () => {
    const placed = NODE_DEFS.flatMap((def) => (def.element ? [def.element] : []));
    const expected = ELEMENTS.filter((el) => el.key !== CAPTIONS_ELEMENT).map((el) => el.key);
    expect(placed.toSorted()).toEqual(expected.toSorted());
  });

  it("only uses widgets the inspector knows", () => {
    const known = new Set<string>(FIELD_WIDGETS);
    const unknown = NODE_DEFS.flatMap((def) =>
      Object.entries(def.params.shape).flatMap(([key, field]) => {
        const widget: string | undefined = fieldMeta(field).widget;
        return widget && !known.has(widget) ? [`${def.type}.${key}: ${widget}`] : [];
      }),
    );
    expect(unknown).toEqual([]);
  });
});
