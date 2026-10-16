import { describe, expect, it } from "vitest";
import { ELEMENTS, elementNodeType } from "../elements";
import { defaultParams, getNodeDef, NODE_DEFS } from "./registry";

describe("node registry", () => {
  it("has unique types and parseable defaults", () => {
    expect(new Set(NODE_DEFS.map((d) => d.type)).size).toBe(NODE_DEFS.length);
    const undefaulted = NODE_DEFS.flatMap((def) => {
      const params = defaultParams(def.type);
      return Object.keys(def.params.shape)
        .filter((k) => !(k in params) && k !== "target")
        .map((k) => `${def.type}.${k}`);
    });
    expect(undefaulted).toEqual([]);
  });

  it("generates one node per placeable registry element", () => {
    const nodeTypeFor = (key: string, kind: string) =>
      kind === "presentation" ? `transition.${key.replace("/", ".")}` : elementNodeType(key);
    const missing = ELEMENTS.filter(
      (el) => el.key !== "core/captions" && !getNodeDef(nodeTypeFor(el.key, el.kind)),
    ).map((el) => el.key);
    expect(missing).toEqual([]);
  });

  it("covers the core node list", () => {
    for (const type of [
      "browser.session",
      "browser.navigate",
      "browser.click",
      "browser.type",
      "browser.hover",
      "browser.scroll",
      "browser.wait",
      "capture.record",
      "capture.still",
      "camera.autoZoom",
      "fx.core.title",
      "timeline.sequence",
      "output.render",
    ]) {
      expect(getNodeDef(type)?.type).toBe(type);
    }
  });
});
