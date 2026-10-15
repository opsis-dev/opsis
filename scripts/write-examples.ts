import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { workflowSchema } from "../src/shared/schema/workflow";
import { TEMPLATES } from "../src/shared/templates";

const ROOT = path.resolve(import.meta.dirname, "..");

for (const t of TEMPLATES.filter((x) => x.id !== "blank")) {
  const dir = path.join(ROOT, "examples", t.id);
  const wf = workflowSchema.parse({
    ...t.build(t.title),
    id: `example_${t.id.replace(/-/g, "_")}`,
  });
  await mkdir(path.join(dir, "assets"), { recursive: true });
  await writeFile(path.join(dir, "assets", ".gitkeep"), "");
  await writeFile(path.join(dir, "project.demoflow.json"), `${JSON.stringify(wf, null, 2)}\n`);
  await writeFile(path.join(dir, ".gitignore"), ".demoflow/\nsecrets/\n");
  console.log(`wrote examples/${t.id}/project.demoflow.json (${wf.nodes.length} nodes)`);
}
