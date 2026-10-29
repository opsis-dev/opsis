import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { usedGlyphs } from "@opsis/frontend/nodes/looks";
import { z } from "zod";
import { NOTICES, loadGlyph } from "./lib/glyph-sets";
import { ROOT, FRONTEND_DIR } from "./lib/tool-config";

const OUT = path.join(FRONTEND_DIR, "editor", "node-glyphs.generated.ts");
const lookSchema = z.object({ glyph: z.string(), morph: z.string().optional() });
const catalogSchema = z.object({ nodes: z.array(z.object({ look: lookSchema.optional() })) });

function catalogLooks(): Array<z.infer<typeof lookSchema>> {
  const text = execFileSync(
    "cargo",
    ["run", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data", "catalog"],
    { cwd: ROOT, encoding: "utf8", maxBuffer: 1 << 26 },
  );
  return catalogSchema
    .parse(JSON.parse(text))
    .nodes.flatMap((node) => (node.look ? [node.look] : []));
}

async function main(): Promise<void> {
  const glyphs = usedGlyphs(catalogLooks());
  const notices = readFileSync(NOTICES, "utf8");
  const lines = await Promise.all(
    glyphs.map(
      async (glyph) =>
        `  ${JSON.stringify(glyph)}: ${JSON.stringify(await loadGlyph(glyph, notices))},`,
    ),
  );
  writeFileSync(
    OUT,
    [
      "export const NODE_GLYPHS: Record<string, ReadonlyArray<readonly [string, Record<string, string>]>> = {",
      ...lines,
      "};",
      "",
    ].join("\n"),
  );
  console.log(`${glyphs.length} glyphs`);
}

await main();
