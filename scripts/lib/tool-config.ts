import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

const PACKAGES_DIR = path.join(ROOT, "packages");

const VIDEO_PACKAGE_DIR = path.join(PACKAGES_DIR, "video");

export const VIDEO_DIR = path.join(VIDEO_PACKAGE_DIR, "src");

export const IGNORED_PATHS = [
  "target/**",
  ".sqlx/**",
  "fixtures/**",
  "crates/terminal/tests/casts/**",
  "examples/**",
  "crates/compile/src/fonts/bundled.generated.json",
  "crates/compile/src/terminal/themes.generated.json",
  "crates/engine/src/downloads.generated.json",
  "crates/ui/glyphs.generated.json",
  "scripts/lib/ffmpeg.generated.json",
  "crates/capture/src/playwright/*.generated.*",
  "film-bundle/**",
  "terminal-bundle/**",
  "resources/**",
  "packages/*/dist/**",
  "packages/video/src/camera/lib/mac-cursors.generated.ts",
  "packages/video/src/generated/**",
  "packages/video/src/registry/length-cases.generated.json",
  "packages/video/src/registry/default-props.generated.json",
];
