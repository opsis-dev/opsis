import path from "node:path";
import {
  googleFontModuleFile,
  GOOGLE_FONT_IMPORT,
  GOOGLE_SHIM_SEGMENTS,
} from "@opsis/video/fonts/google-font-imports";
import type { Plugin } from "vite";
import { VIDEO_DIR } from "./tool-config";

export const localFonts = (): Plugin => ({
  name: "opsis-local-fonts",
  enforce: "pre",
  resolveId(id) {
    const name = GOOGLE_FONT_IMPORT.exec(id)?.[1];
    return name ? path.join(VIDEO_DIR, ...GOOGLE_SHIM_SEGMENTS, googleFontModuleFile(name)) : null;
  },
});
