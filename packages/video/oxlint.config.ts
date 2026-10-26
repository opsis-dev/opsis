import { defineConfig } from "oxlint";
import { lintConfig, runtimeNeutral } from "../../scripts/lib/oxlint-base.ts";

export default defineConfig(
  lintConfig(import.meta.dirname, [
    runtimeNeutral("packages/video", { relativeOnly: true }),
    { files: ["src/common/lists.ts"], rules: { "opsis/use-find-by-id": "off" } },
    { files: ["src/common/guards.ts"], rules: { "opsis/use-type-guards": "off" } },
    {
      files: ["src/**/*.ts", "src/**/*.tsx"],
      rules: {
        "opsis/no-animate-class": "error",
        "opsis/no-three-clock": "error",
      },
    },
  ]),
);
