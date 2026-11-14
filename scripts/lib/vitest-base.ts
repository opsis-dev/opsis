import { defineConfig } from "vitest/config";

type TestConfig = {
  alias?: Record<string, string>;
  nodeTests?: string[];
  setupFiles?: string[];
  globalSetup?: string[];
};

export function testConfig({
  alias = {},
  nodeTests = ["src/**/*.test.ts"],
  setupFiles = [],
  globalSetup = [],
}: TestConfig = {}) {
  return defineConfig({
    resolve: { alias },
    test: {
      fsModuleCachePath: "node_modules/.vitest-cache",
      setupFiles,
      globalSetup,
      include: nodeTests,
      environment: "node",
    },
  });
}
