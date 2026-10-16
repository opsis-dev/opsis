import { describe, expect, it } from "vitest";
import { ELEMENTS, elementNodeType } from "../elements";
import { defaultParams, getNodeDef, migrateSoundPorts, NODE_DEFS, normalizeNode } from "./registry";

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

  it("covers the PLAN.md §14 node list", () => {
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

describe("old Add sound connections", () => {
  it("move onto the one Sounds port", () => {
    const nodes = [
      { id: "mix", type: "audio.mix", position: { x: 0, y: 0 }, params: {} },
      { id: "other", type: "timeline.sequence", position: { x: 0, y: 0 }, params: {} },
    ];
    const edges = [
      {
        id: "m.audio->mix.a2",
        source: "m",
        sourceHandle: "audio",
        target: "mix",
        targetHandle: "a2",
      },
      {
        id: "c.clip->other.in1",
        source: "c",
        sourceHandle: "clip",
        target: "other",
        targetHandle: "in1",
      },
    ];
    expect(migrateSoundPorts(edges, nodes).map((e) => [e.id, e.targetHandle])).toEqual([
      ["m.audio->mix.sounds", "sounds"],
      ["c.clip->other.in1", "in1"],
    ]);
  });
});

describe("old window title bar setting", () => {
  it("becomes the macOS frame when it was on, and no frame when off", () => {
    const node = (chrome: boolean) =>
      normalizeNode({
        id: "c",
        type: "camera.autoZoom",
        position: { x: 0, y: 0 },
        params: { chrome },
      });
    expect(node(true).params).toEqual({ frame: "macos" });
    expect(node(false).params).toEqual({ frame: "none" });
  });
});
