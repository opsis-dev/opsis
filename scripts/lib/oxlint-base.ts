import path from "node:path";
import type { OxlintConfig } from "oxlint";
import { IGNORED_PATHS, ROOT } from "./tool-config.ts";

type Override = NonNullable<OxlintConfig["overrides"]>[number];

const WALL_CLOCK_LIBS = ["framer-motion", "motion", "motion/react", "motion/react-client", "gsap"];

const NODE_ONLY = ["node:*", "playwright-core"];

export function lintConfig(dir: string, overrides: Override[] = []): OxlintConfig {
  const prefix = path.relative(dir, ROOT).split(path.sep).join("/");
  function fromRoot(rel: string) {
    return prefix ? `${prefix}/${rel}` : `./${rel}`;
  }
  const own = path.relative(ROOT, dir).split(path.sep).join("/");
  return {
    plugins: ["typescript", "react", "unicorn", "oxc", "import", "vitest"],
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
      : [...IGNORED_PATHS, "packages/**", "apps/**"],
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
      "opsis/use-find-by-id": "error",
      "opsis/use-function-components": "error",
      "opsis/use-function-declarations": "error",
      "opsis/use-parse-json-with": "error",
      "opsis/use-percent-text": "error",
      "opsis/use-type-guards": "error",
    },
    overrides: [
      ...overrides,
      {
        files: ["**/*.test.ts", "**/*.test.tsx"],
        rules: {
          "vitest/require-mock-type-parameters": "off",
        },
      },
    ],
  };
}

export function runtimeNeutral(
  who: string,
  { relativeOnly = false }: { relativeOnly?: boolean } = {},
): Override {
  return {
    files: ["src/**/*.ts", "src/**/*.tsx"],
    excludeFiles: ["**/*.test.ts", "**/*.test.tsx"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: NODE_ONLY,
              message: `${who} must stay runtime-neutral (AGENTS.md, Runtime boundaries).`,
            },
            ...(relativeOnly
              ? [
                  {
                    group: ["@/**"],
                    message:
                      "packages/video imports its own files by relative path: the UI compiles them too, with its own @/ alias.",
                  },
                ]
              : []),
          ],
          paths: [
            ...WALL_CLOCK_LIBS.map((name) => ({
              name,
              message:
                "Animations in packages/video must be a pure function of the runtime's useFrame() (AGENTS.md).",
            })),
            {
              name: "@react-three/fiber",
              importNames: ["useFrame"],
              message:
                "Fiber's useFrame runs on the wall clock. Derive the scene from the runtime's useFrame() instead (AGENTS.md).",
            },
          ],
        },
      ],
    },
  };
}
