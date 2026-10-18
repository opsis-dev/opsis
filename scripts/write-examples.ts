import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { writeJson } from "../src/engine/json";
import { workflowSchema } from "../src/shared/schema/workflow";
import { TEMPLATES } from "../src/shared/templates";
import { ROOT } from "./lib/tool-config";

for (const t of TEMPLATES.filter((x) => x.id !== "blank")) {
  const dir = path.join(ROOT, "examples", t.id);
  const wf = workflowSchema.parse({
    ...t.build(t.title),
    id: `example_${t.id.replace(/-/g, "_")}`,
  });
  await mkdir(path.join(dir, "assets"), { recursive: true });
  await writeFile(path.join(dir, "assets", ".gitkeep"), "");
  await writeJson(path.join(dir, "project.demoflow.json"), wf);
  await writeFile(path.join(dir, ".gitignore"), ".demoflow/\nsecrets/\n");
  console.log(`wrote examples/${t.id}/project.demoflow.json (${wf.nodes.length} nodes)`);
}
