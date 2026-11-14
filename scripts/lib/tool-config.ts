import { homedir } from "node:os";
import path from "node:path";

export const ROOT = path.resolve(import.meta.dirname, "..", "..");

export function devCache(name: string): string {
  return path.join(homedir(), ".cache", "opsis-dev", name);
}

export const IGNORED_PATHS = [
  "target/**",
  ".sqlx/**",
  "fixtures/**",
  "crates/terminal/tests/casts/**",
  "examples/**",
  "crates/compile/src/fonts/bundled.generated.json",
  "crates/compile/src/terminal/themes.generated.json",
  "crates/script/src/themes.generated.json",
  "crates/script/src/highlights.generated.scm",
  "crates/engine/src/downloads.generated.json",
  "crates/ui/glyphs.generated.json",
  "QT_NOTICES.generated.md",
  "scripts/lib/ffmpeg.generated.json",
  "crates/capture/src/playwright/*.generated.*",
  "resources/**",
  "crates/compile/src/elements/camera/mac-cursors.generated.json",
  "scripts/lib/generated/**",
];
