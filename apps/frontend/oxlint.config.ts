import { defineConfig } from "oxlint";
import { lintConfig } from "../../scripts/lib/oxlint-base.ts";

export default defineConfig(
  lintConfig(import.meta.dirname, [
    {
      files: ["src/**/*.ts", "src/**/*.tsx"],
      rules: {
        "no-restricted-imports": [
          "error",
          {
            patterns: [
              {
                group: ["../../**"],
                message: "Import from two or more folders up through @/ (AGENTS.md).",
              },
            ],
          },
        ],
      },
    },
  ]),
);
