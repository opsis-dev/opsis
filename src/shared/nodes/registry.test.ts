import { describe, expect, it } from "vitest";
import { CAPTIONS_ELEMENT, ELEMENTS, elementNodeType } from "../elements";
import { defaultParams, getNodeDef, NODE_DEFS } from "./registry";

const REQUIRED_KEYS = new Set(["target"]);

describe("node registry", () => {
  it("has parseable defaults for every setting a node can start without", () => {
    const undefaulted = NODE_DEFS.flatMap((def) => {
      const params = defaultParams(def.type);
      return Object.keys(def.params.shape)
        .filter((k) => !(k in params) && !REQUIRED_KEYS.has(k))
        .map((k) => `${def.type}.${k}`);
    });
    expect(undefaulted).toEqual([]);
  });

  it("generates one node per placeable registry element", () => {
    const nodeTypeFor = (key: string, kind: string) =>
      kind === "presentation" ? `transition.${key.replace("/", ".")}` : elementNodeType(key);
    const missing = ELEMENTS.filter(
      (el) => el.key !== CAPTIONS_ELEMENT && !getNodeDef(nodeTypeFor(el.key, el.kind)),
    ).map((el) => el.key);
    expect(missing).toEqual([]);
  });
});
