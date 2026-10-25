import { GraphBuilder } from "@opsis/core/templates/build";
import { describe, expect, it } from "vitest";
import { editedNodes } from "./workflow-diff";

const film = () => {
  const g = new GraphBuilder();
  g.node("a", "fx.core.solid", 0, 0);
  g.node("b", "fx.core.solid", 0, 1);
  g.node("trim", "timeline.trim", 1, 0);
  g.node("out", "output.render", 2, 0);
  g.link("a", "trim").link("trim", "out");
  return g.build({ id: "x", name: "x" });
};

describe("editedNodes", () => {
  it("names a node whose settings changed and everything after it", () => {
    const before = film();
    const after = structuredClone(before);
    after.nodes[2].params = { startMs: 500 };
    expect(editedNodes(before, after).toSorted()).toEqual(["out", "trim"]);
  });

  it("names a node whose connections changed", () => {
    const before = film();
    const after = structuredClone(before);
    after.edges = after.edges.map((e) => (e.source === "a" ? { ...e, source: "b" } : e));
    expect(editedNodes(before, after).toSorted()).toEqual(["out", "trim"]);
  });

  it("ignores a node that only moved", () => {
    const before = film();
    const after = structuredClone(before);
    after.nodes[0].position = { x: 500, y: 500 };
    expect(editedNodes(before, after)).toEqual([]);
  });
});
