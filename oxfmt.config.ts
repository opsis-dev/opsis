import { defineConfig } from "oxfmt";

export default defineConfig({
  printWidth: 100,
  sortPackageJson: false,
  ignorePatterns: [
    ".vite/**",
    "out/**",
    "remotion-bundle/**",
    "resources/**",
    "PLAN.md",
    "bun.lock",
    "src/renderer/routeTree.gen.ts",
    "src/video/elements/remocn/**",
    "src/video/elements/snap-cn/**",
    "src/video/lib/**",
    "src/renderer/components/ui/**",
    "src/shared/elements/elements.generated.ts",
    "src/video/registry/components.generated.ts",
  ],
});
