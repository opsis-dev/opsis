import { defineConfig } from "vitest/config";

type TestConfig = {
  alias?: Record<string, string>;
  nodeTests?: string[];
  domTests?: string[];
  setupFiles?: string[];
  globalSetup?: string[];
};

export function testConfig({
  alias = {},
  nodeTests = ["src/**/*.test.ts"],
  domTests = ["src/**/*.test.tsx"],
  setupFiles = [],
  globalSetup = [],
}: TestConfig = {}) {
  return defineConfig({
    resolve: { alias },
    test: {
      fsModuleCachePath: "node_modules/.vitest-cache",
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
}
