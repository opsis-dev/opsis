import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export const PACKAGES_DIR = path.join(ROOT, "packages");

export const CORE_DIR = path.join(PACKAGES_DIR, "core", "src");

export const APPS_DIR = path.join(ROOT, "apps");

export const UI_DIR = path.join(APPS_DIR, "ui", "src");

export const VIDEO_PACKAGE_DIR = path.join(PACKAGES_DIR, "video");

export const VIDEO_DIR = path.join(VIDEO_PACKAGE_DIR, "src");

export const IGNORED_PATHS = [
  ".vite/**",
  "out/**",
  "remotion-bundle/**",
  "resources/**",
  "public/three/**",
  "packages/*/dist/**",
  "apps/ui/src/routeTree.gen.ts",
  "apps/ui/src/components/ui/**",
  "apps/ui/src/components/*.tsx",
  "apps/ui/src/hooks/**",
  "apps/ui/src/editor/node-glyphs.generated.ts",
  "packages/video/src/elements/**",
  "packages/video/src/lib/**",
  "packages/video/src/hooks/**",
  "packages/video/src/fonts/google/**",
  "packages/video/src/registry/components.generated.ts",
  "packages/core/src/elements/elements.generated.ts",
  "packages/core/src/fonts/bundled.generated.ts",
];
