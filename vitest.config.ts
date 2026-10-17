import { defineConfig } from "vitest/config";
import { SRC_ALIAS } from "./scripts/lib/tool-config.ts";

export default defineConfig({
  resolve: { alias: SRC_ALIAS },
  test: {
    include: ["src/**/*.test.ts", "src/**/*.test.tsx"],
    environment: "node",
  },
});
