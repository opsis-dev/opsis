import { describe, expect, it } from "vitest";
import { catalogEntry, nodeCatalog } from "#core/api/catalog";
import { NODE_DEFS } from "#core/nodes/registry";

describe("node catalogue", () => {
  it("lists the visible node types without their settings", () => {
    const nodes = nodeCatalog();
    expect(nodes.map((n) => n.type)).toEqual(NODE_DEFS.filter((d) => !d.hidden).map((d) => d.type));
    expect(nodes.find((n) => n.type === "browser.click")).not.toHaveProperty("params");
  });

  it("describes one node type with its ports and settings schema", () => {
    expect(catalogEntry("browser.click")).toMatchObject({
      title: "Click",
      runs: true,
      inputs: {
        session: { type: "browser-session", label: "Browser", optional: false, many: false },
      },
      outputs: { session: { type: "browser-session", label: "Browser" } },
      params: { type: "object", properties: { target: expect.anything() } },
    });
    for (const node of nodeCatalog())
      expect(catalogEntry(node.type)?.params).toMatchObject({ type: "object" });
  });

  it("returns null for an unknown type", () => {
    expect(catalogEntry("nope.nothing")).toBeNull();
  });
});
