import { defineConfig } from "oxfmt";

export function formatConfig(ignorePatterns: string[]) {
  return defineConfig({
    printWidth: 100,
    sortPackageJson: false,
    sortImports: { newlinesBetween: false },
    ignorePatterns,
  });
}
