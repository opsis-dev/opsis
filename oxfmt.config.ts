import { defineConfig } from "oxfmt";
import { IGNORED_PATHS } from "./scripts/lib/tool-config.ts";

export default defineConfig({
  printWidth: 100,
  sortPackageJson: false,
  sortImports: { newlinesBetween: false },
  ignorePatterns: [...IGNORED_PATHS, "bun.lock"],
});
