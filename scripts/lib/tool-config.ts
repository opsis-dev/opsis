import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export const SRC_DIR = path.join(ROOT, "src");

export const VIDEO_DIR = path.join(SRC_DIR, "video");

export const SRC_ALIAS = { "@": SRC_DIR };

export const IGNORED_PATHS = [
  ".vite/**",
  "out/**",
  "remotion-bundle/**",
  "resources/**",
  "src/renderer/routeTree.gen.ts",
  "public/three/**",
  "src/video/elements/**",
  "src/video/lib/**",
  "src/video/hooks/**",
  "src/renderer/components/ui/**",
  "src/renderer/components/*.tsx",
  "src/renderer/hooks/**",
  "src/shared/elements/elements.generated.ts",
  "src/shared/fonts/bundled.generated.ts",
  "src/renderer/editor/node-glyphs.generated.ts",
  "src/video/fonts/google/**",
  "src/video/registry/components.generated.ts",
];
