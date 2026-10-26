import { defineConfig } from "vitest/config";

type TestConfig = {
  alias?: Record<string, string>;
  nodeTests?: string[];
  domTests?: string[];
  setupFiles?: string[];
  globalSetup?: string[];
};

export const testConfig = ({
  alias = {},
  nodeTests = ["src/**/*.test.ts"],
  domTests = ["src/**/*.test.tsx"],
  setupFiles = [],
  globalSetup = [],
}: TestConfig = {}) =>
  defineConfig({
    resolve: { alias },
    test: {
      setupFiles,
      globalSetup,
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
