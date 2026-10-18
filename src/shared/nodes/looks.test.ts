import { describe, expect, it } from "vitest";
import { NODE_LOOKS, NODE_MOTIONS, type NodeLook } from "./looks";
import { NODE_DEFS } from "./registry";

const appearance = (look: NodeLook): string =>
  `${look.glyph} ${look.morph ? `morph ${look.morph}` : (look.motion ?? "still")}`;

describe("node looks", () => {
  it("gives every node type a look, and only node types", () => {
    const types = new Set(NODE_DEFS.map((def) => def.type));
    expect([...types].filter((type) => !NODE_LOOKS[type])).toEqual([]);
    expect(Object.keys(NODE_LOOKS).filter((type) => !types.has(type))).toEqual([]);
  });

  it("tells nodes that share an icon apart by how it moves", () => {
    const first = new Map<string, string>();
    const clashes: string[] = [];
    for (const [type, look] of Object.entries(NODE_LOOKS)) {
      const key = appearance(look);
      const other = first.get(key);
      if (other) clashes.push(`${other} and ${type}`);
      else first.set(key, type);
    }
    expect(clashes).toEqual([]);
  });

  it("uses every listed motion, and never a motion and a morph together", () => {
    const looks = Object.entries(NODE_LOOKS);
    const used = new Set(looks.map(([, look]) => look.motion));
    expect(NODE_MOTIONS.filter((motion) => !used.has(motion))).toEqual([]);
    expect(looks.filter(([, look]) => look.motion && look.morph).map(([type]) => type)).toEqual([]);
  });
});
