import { defineConfig } from "oxlint";
import { lintConfig, runtimeNeutral } from "../../scripts/lib/oxlint-base.ts";

export default defineConfig(
  lintConfig(import.meta.dirname, [
    runtimeNeutral("packages/core"),
    { files: ["src/lists.ts"], rules: { "opsis/use-find-by-id": "off" } },
    { files: ["src/errors.ts"], rules: { "opsis/use-on-abort": "off" } },
    { files: ["src/guards.ts"], rules: { "opsis/use-type-guards": "off" } },
  ]),
);
