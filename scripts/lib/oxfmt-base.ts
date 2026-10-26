import { defineConfig } from "oxfmt";

export const formatConfig = (ignorePatterns: string[]) =>
  defineConfig({
    printWidth: 100,
    sortPackageJson: false,
    sortImports: { newlinesBetween: false },
    ignorePatterns,
  });
