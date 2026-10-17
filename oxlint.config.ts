import { defineConfig } from "oxlint";
import { IGNORED_PATHS } from "./scripts/lib/tool-config.ts";

const WALL_CLOCK_LIBS = ["framer-motion", "motion", "motion/react", "motion/react-client", "gsap"];

export default defineConfig({
  plugins: ["typescript", "react", "unicorn", "oxc", "import", "vitest"],
  jsPlugins: ["./scripts/oxlint-plugin-demoflow.ts"],
  categories: {
    correctness: "error",
    suspicious: "warn",
  },
  env: {
    browser: true,
    node: true,
    es2024: true,
  },
  ignorePatterns: IGNORED_PATHS,
  rules: {
    "react/react-in-jsx-scope": "off",
    "react/exhaustive-deps": "warn",
    "react/rules-of-hooks": "error",
    "typescript/consistent-type-imports": "error",
    "typescript/no-explicit-any": "warn",
    "unicorn/no-array-sort": "off",
    "unicorn/consistent-function-scoping": "off",
    "unicorn/require-post-message-target-origin": "off",
    "no-underscore-dangle": ["warn", { allow: ["_zod"] }],
    "import/no-unassigned-import": ["warn", { allow: ["**/*.css"] }],
    "react/hooks": "off",
    "import/no-cycle": "off",
    "unicorn/filename-case": ["error", { case: "kebabCase", ignore: ["^__root\\.tsx$"] }],
    "demoflow/use-find-by-id": "error",
    "demoflow/use-on-abort": "error",
    "demoflow/use-parse-json-with": "error",
    "demoflow/use-percent-text": "error",
    "demoflow/use-type-guards": "error",
  },
  overrides: [
    {
      files: ["src/shared/**/*.ts", "src/video/**/*.ts", "src/video/**/*.tsx"],
      excludeFiles: ["**/*.test.ts"],
      rules: {
        "no-restricted-imports": [
          "error",
          {
            patterns: [
              {
                group: [
                  "node:*",
                  "electron",
                  "playwright-core",
                  "@remotion/renderer",
                  "@remotion/bundler",
                ],
                message:
                  "src/shared and src/video must stay runtime-neutral (AGENTS.md, Runtime boundaries).",
              },
            ],
            paths: WALL_CLOCK_LIBS.map((name) => ({
              name,
              message:
                "Animations in src/video must be a pure function of useCurrentFrame() (AGENTS.md).",
            })),
          },
        ],
      },
    },
    {
      files: ["src/video/**/*.ts", "src/video/**/*.tsx"],
      rules: {
        "demoflow/no-animate-class": "error",
      },
    },
    {
      files: ["scripts/**/*.ts"],
      excludeFiles: ["scripts/lib/tool-config.ts"],
      rules: { "demoflow/use-root": "error" },
    },
    { files: ["src/shared/lists.ts"], rules: { "demoflow/use-find-by-id": "off" } },
    { files: ["src/shared/errors.ts"], rules: { "demoflow/use-on-abort": "off" } },
    { files: ["src/shared/guards.ts"], rules: { "demoflow/use-type-guards": "off" } },
    {
      files: ["**/*.test.ts", "**/*.test.tsx"],
      rules: {
        "vitest/require-mock-type-parameters": "off",
      },
    },
  ],
});
