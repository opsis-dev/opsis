import path from "node:path";
import type { OxlintConfig } from "oxlint";
import { IGNORED_PATHS, ROOT } from "./tool-config.ts";

type Override = NonNullable<OxlintConfig["overrides"]>[number];

export function lintConfig(dir: string, overrides: Override[] = []): OxlintConfig {
  const prefix = path.relative(dir, ROOT).split(path.sep).join("/");
  function fromRoot(rel: string) {
    return prefix ? `${prefix}/${rel}` : `./${rel}`;
  }
  const own = path.relative(ROOT, dir).split(path.sep).join("/");
  return {
    plugins: ["typescript", "unicorn", "oxc", "import", "vitest"],
    jsPlugins: [fromRoot("scripts/oxlint-plugin-opsis.ts")],
    categories: {
      correctness: "error",
      suspicious: "warn",
    },
    env: {
      browser: true,
      node: true,
      es2024: true,
    },
    ignorePatterns: own
      ? IGNORED_PATHS.filter((p) => p.startsWith(`${own}/`)).map((p) => p.slice(own.length + 1))
      : IGNORED_PATHS,
    rules: {
      "typescript/consistent-type-imports": "error",
      "typescript/no-explicit-any": "warn",
      "unicorn/no-array-sort": "off",
      "unicorn/consistent-function-scoping": "off",
      "unicorn/require-post-message-target-origin": "off",
      "no-underscore-dangle": ["warn", { allow: ["_zod"] }],
      "import/no-cycle": "off",
      "unicorn/filename-case": ["error", { case: "kebabCase" }],
      "opsis/use-find-by-id": "error",
      "opsis/use-function-declarations": "error",
      "opsis/use-parse-json-with": "error",
      "opsis/use-type-guards": "error",
    },
    overrides: [
      ...overrides,
      {
        files: ["**/*.test.ts"],
        rules: {
          "vitest/require-mock-type-parameters": "off",
        },
      },
    ],
  };
}
