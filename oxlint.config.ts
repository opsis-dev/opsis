import { defineConfig } from "oxlint";

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
  ignorePatterns: [
    ".vite/**",
    "out/**",
    "remotion-bundle/**",
    "resources/**",
    "src/renderer/routeTree.gen.ts",
    "src/video/elements/remocn/**",
    "src/video/elements/snap-cn/**",
    "src/video/lib/**",
    "src/renderer/components/ui/**",
    "src/shared/elements/elements.generated.ts",
    "src/video/registry/components.generated.ts",
  ],
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
                message: "src/shared and src/video must stay runtime-neutral (PLAN.md §13).",
              },
            ],
            paths: WALL_CLOCK_LIBS.map((name) => ({
              name,
              message:
                "Animations in src/video must be a pure function of useCurrentFrame() (PLAN.md §8.1).",
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
      files: ["**/*.test.ts", "**/*.test.tsx"],
      rules: {
        "vitest/require-mock-type-parameters": "off",
      },
    },
  ],
});
