import { defineConfig } from "vitest/config";
import { SRC_ALIAS } from "./scripts/lib/tool-config.ts";

export default defineConfig({
  resolve: { alias: SRC_ALIAS },
  test: {
    projects: [
      { extends: true, test: { name: "node", include: ["src/**/*.test.ts"], environment: "node" } },
      {
        extends: true,
        test: { name: "dom", include: ["src/**/*.test.tsx"], environment: "happy-dom" },
      },
    ],
  },
});
