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
    "src/video/elements/remotion-ui/**",
    "src/video/lib/**",
    "src/video/hooks/**",
    "src/renderer/components/ui/**",
    "src/renderer/hooks/**",
    "src/shared/elements/elements.generated.ts",
    "src/shared/fonts/bundled.generated.ts",
    "src/video/fonts/google/**",
    "src/video/registry/components.generated.ts",
  ],
});
