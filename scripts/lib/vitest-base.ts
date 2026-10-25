import { defineConfig } from "vitest/config";

type TestConfig = { alias?: Record<string, string>; nodeTests?: string[]; domTests?: string[] };

export const testConfig = ({
  alias = {},
  nodeTests = ["src/**/*.test.ts"],
  domTests = ["src/**/*.test.tsx"],
}: TestConfig = {}) =>
  defineConfig({
    resolve: { alias },
    test: {
      projects: [
        {
          extends: true,
          test: { name: "node", include: nodeTests, environment: "node" },
        },
        {
          extends: true,
          test: { name: "dom", include: domTests, environment: "happy-dom" },
        },
      ],
    },
  });
