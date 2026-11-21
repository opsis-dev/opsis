import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { presets, REGISTRY_URL, registry } from "./lib/agent-registry";
import { fetchOk } from "./lib/download";
import { ROOT } from "./lib/tool-config";

const OUT = path.join(ROOT, "crates", "session", "src", "agents.generated.json");

const text = await (await fetchOk(REGISTRY_URL)).text();
const source = registry.parse(JSON.parse(text));
const snapshot = {
  source: REGISTRY_URL,
  sha256: createHash("sha256").update(text).digest("hex"),
  registryVersion: source.version,
  presets: presets(source),
};
writeFileSync(OUT, `${JSON.stringify(snapshot, null, 2)}\n`);
console.log(`Wrote ${snapshot.presets.length} agents to ${path.relative(ROOT, OUT)}.`);
