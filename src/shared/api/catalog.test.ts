import { describe, expect, it } from "vitest";
import { catalogEntry, nodeCatalog } from "./catalog";

describe("node catalogue", () => {
  it("describes every visible node with ports and a settings schema", () => {
    const nodes = nodeCatalog(true);
    expect(nodes.length).toBeGreaterThan(100);
    for (const node of nodes) {
      expect(node.title).toBeTruthy();
      expect(node.params).toMatchObject({ type: "object" });
      for (const port of [...Object.values(node.inputs), ...Object.values(node.outputs)])
        expect(port.type).toBeTruthy();
    }
  });

  it("returns null for an unknown type", () => {
    expect(catalogEntry("nope.nothing")).toBeNull();
    expect(catalogEntry("browser.click")?.inputs.session?.type).toBe("session");
  });
});
