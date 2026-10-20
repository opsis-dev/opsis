import { describe, expect, it } from "vitest";
import { findById } from "../../shared/lists";
import { GraphBuilder } from "../../shared/templates/build";
import { must } from "../../shared/test-fixtures";
import { hashNodes } from "./runner";

describe("cache keys of skipped nodes", () => {
  it("change everything after a node when Skip is turned on or off", () => {
    const g = new GraphBuilder();
    g.node("bg", "fx.core.solid", 0, 0);
    g.node("spot", "cursor.spotlight", 1, 0);
    g.node("out", "output.render", 2, 0);
    g.link("bg", "spot").link("spot", "out", "clip", "timeline");
    const wf = g.build({ id: "x", name: "x" });
    const keyOf = () => hashNodes(wf, wf.nodes, {}, "/nowhere", "test").get("out");
    const shown = keyOf();
    must(findById(wf.nodes, "spot"), "spot").disabled = true;
    const skipped = keyOf();
    must(findById(wf.nodes, "spot"), "spot").disabled = false;
    expect(skipped).not.toBe(shown);
    expect(keyOf()).toBe(shown);
  });
});
