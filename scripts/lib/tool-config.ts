import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export const PACKAGES_DIR = path.join(ROOT, "packages");

export const CORE_DIR = path.join(PACKAGES_DIR, "core", "src");

export const UI_DIR = path.join(PACKAGES_DIR, "ui", "src");

export const VIDEO_PACKAGE_DIR = path.join(PACKAGES_DIR, "video");

export const VIDEO_DIR = path.join(VIDEO_PACKAGE_DIR, "src");

export const UI_ALIAS = { "@": UI_DIR };

export const PACKAGE_SOURCES = { "@opsis/core": CORE_DIR, "@opsis/video": VIDEO_DIR };

export const IGNORED_PATHS = [
  ".vite/**",
  "out/**",
  "remotion-bundle/**",
  "resources/**",
  "public/three/**",
  "packages/*/dist/**",
  "packages/ui/src/routeTree.gen.ts",
  "packages/ui/src/components/ui/**",
  "packages/ui/src/components/*.tsx",
  "packages/ui/src/hooks/**",
  "packages/ui/src/editor/node-glyphs.generated.ts",
  "packages/video/src/elements/**",
  "packages/video/src/lib/**",
  "packages/video/src/hooks/**",
  "packages/video/src/fonts/google/**",
  "packages/video/src/registry/components.generated.ts",
  "packages/core/src/elements/elements.generated.ts",
  "packages/core/src/fonts/bundled.generated.ts",
];
