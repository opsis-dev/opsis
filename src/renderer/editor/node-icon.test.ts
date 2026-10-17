import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { iconNames } from "lucide-react/dynamic";
import { describe, expect, it } from "vitest";
import { DEFAULT_WORKFLOW_SETTINGS as settings } from "@/shared/schema/workflow";
import type { CardRow } from "@/shared/nodes/card";
import { defaultParams, NODE_DEFS } from "@/shared/nodes/registry";
import { TEMPLATES } from "@/shared/templates";
import { hasIcon, nodeIconNames } from "./node-icon";
import { CATEGORY_STYLES } from "./ports";

const NODES_DIR = join(import.meta.dirname, "../../shared/nodes");
const FIRST_ARG_OF_ROW = /(?<!function )\b\w*(?:row|Row|missing)\(\s*([^,]*?),/gs;
const ICON_TABLE = /const \w*ICON\w*[^=]*=\s*\{([^}]*)\}/gs;
const QUOTED = /"([a-z0-9-]+)"/g;

function iconsInSource(): string[] {
  const files = readdirSync(NODES_DIR).filter((f) => f.endsWith(".ts") && !f.endsWith(".test.ts"));
  return files.flatMap((file) => {
    const text = readFileSync(join(NODES_DIR, file), "utf8");
    const chunks = [...text.matchAll(FIRST_ARG_OF_ROW), ...text.matchAll(ICON_TABLE)].map(
      (m) => m[1],
    );
    return chunks.flatMap((chunk) => [...chunk.matchAll(QUOTED)].map((m) => m[1]));
  });
}

function iconsInDefaultCards(): string[] {
  return NODE_DEFS.flatMap((def) =>
    (def.card?.({ params: defaultParams(def.type), settings, connected: [] }) ?? [])
      .filter((r): r is CardRow => !!r)
      .map((r) => r.icon),
  );
}

describe("node icons", () => {
  it("only lists real icon names", () => {
    const known = new Set<string>(iconNames);
    expect(nodeIconNames().filter((name) => !known.has(name))).toEqual([]);
  });

  it("has every icon the nodes, categories, templates and cards use", () => {
    const used = [
      ...NODE_DEFS.flatMap((d) => (d.icon ? [d.icon] : [])),
      ...Object.values(CATEGORY_STYLES).map((s) => s.icon),
      ...TEMPLATES.map((t) => t.icon),
      ...iconsInDefaultCards(),
      ...iconsInSource(),
    ];
    expect([...new Set(used)].filter((name) => !hasIcon(name))).toEqual([]);
  });
});
