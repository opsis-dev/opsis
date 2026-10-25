import { writeFileSync } from "node:fs";
import path from "node:path";
import { workflowSchema } from "@opsis/core/schema/workflow";
import { TEMPLATES } from "@opsis/core/templates";
import { ROOT } from "./tool-config";

export const TEMPLATES_JSON = path.join(
  ROOT,
  "crates",
  "project",
  "src",
  "templates.generated.json",
);

export const templatesJsonText = (): string =>
  `${JSON.stringify(
    TEMPLATES.map((t) => ({
      id: t.id,
      title: t.title,
      description: t.description,
      icon: t.icon,
      workflow: workflowSchema.parse({ ...t.build(t.title), id: `template_${t.id}` }),
    })),
    null,
    1,
  )}\n`;

export function writeTemplatesJson(): void {
  writeFileSync(TEMPLATES_JSON, templatesJsonText());
  console.log(`wrote ${path.relative(ROOT, TEMPLATES_JSON)}`);
}
