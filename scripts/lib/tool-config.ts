import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export const PACKAGES_DIR = path.join(ROOT, "packages");

export const APPS_DIR = path.join(ROOT, "apps");

export const FRONTEND_DIR = path.join(APPS_DIR, "frontend", "src");

export const VIDEO_PACKAGE_DIR = path.join(PACKAGES_DIR, "video");

export const VIDEO_DIR = path.join(VIDEO_PACKAGE_DIR, "src");

export const IGNORED_PATHS = [
  "target/**",
  ".sqlx/**",
  "fixtures/**",
  "crates/terminal/tests/casts/**",
  "examples/**",
  "crates/compile/src/fonts/bundled.generated.json",
  "crates/engine/src/downloads.generated.json",
  "scripts/lib/ffmpeg.generated.json",
  "crates/capture/src/playwright/*.generated.*",
  "apps/frontend/src/generated/bindings.ts",
  "film-bundle/**",
  "resources/**",
  "packages/*/dist/**",
  "apps/frontend/src/routeTree.gen.ts",
  "apps/frontend/src/components/ui/**",
  "apps/frontend/src/components/*.tsx",
  "apps/frontend/src/hooks/**",
  "apps/frontend/src/editor/node-glyphs.generated.ts",
  "packages/video/src/camera/lib/mac-cursors.generated.ts",
  "packages/video/src/generated/**",
  "packages/video/src/registry/length-cases.generated.json",
  "packages/video/src/registry/default-props.generated.json",
];
