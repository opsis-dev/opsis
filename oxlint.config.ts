import { defineConfig } from "oxlint";
import { lintConfig } from "./scripts/lib/oxlint-base.ts";

export default defineConfig(
  lintConfig(import.meta.dirname, [
    {
      files: ["scripts/**/*.ts"],
      excludeFiles: ["scripts/lib/tool-config.ts"],
      rules: { "opsis/use-root": "error" },
    },
    { files: ["scripts/lib/common/lists.ts"], rules: { "opsis/use-find-by-id": "off" } },
    { files: ["scripts/lib/common/guards.ts"], rules: { "opsis/use-type-guards": "off" } },
  ]),
);
