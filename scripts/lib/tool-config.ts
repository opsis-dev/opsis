import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export const SRC_ALIAS = { "@": path.join(ROOT, "src") };

export const IGNORED_PATHS = [
  ".vite/**",
  "out/**",
  "remotion-bundle/**",
  "resources/**",
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
];
