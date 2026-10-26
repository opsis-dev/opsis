import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export const PACKAGES_DIR = path.join(ROOT, "packages");

export const APPS_DIR = path.join(ROOT, "apps");

export const UI_DIR = path.join(APPS_DIR, "ui", "src");

export const VIDEO_PACKAGE_DIR = path.join(PACKAGES_DIR, "video");

export const VIDEO_DIR = path.join(VIDEO_PACKAGE_DIR, "src");

export const IGNORED_PATHS = [
  "target/**",
  ".sqlx/**",
  "fixtures/**",
  "examples/**",
  "crates/compile/src/elements/registry.generated.json",
  "crates/compile/src/fonts/bundled.generated.json",
  "crates/engine/src/downloads.generated.json",
  "crates/render/src/source-layout.generated.json",
  "crates/capture/src/playwright/*.generated.*",
  "apps/ui/src/generated/bindings.ts",
  "apps/sidecar/src/generated/protocol.ts",
  "remotion-bundle/**",
  "resources/**",
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
  "packages/video/src/generated/**",
  "packages/video/src/registry/length-cases.generated.json",
];
