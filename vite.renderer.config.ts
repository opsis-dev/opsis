import path from "node:path";
import {
  googleFontModuleFile,
  GOOGLE_FONT_IMPORT,
  GOOGLE_SHIM_SEGMENTS,
} from "@opsis/video/fonts/google-font-imports";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";
import { PACKAGE_SOURCES, UI_ALIAS, VIDEO_DIR } from "./scripts/lib/tool-config.ts";

const localFonts = (): Plugin => ({
  name: "opsis-local-fonts",
  enforce: "pre",
  resolveId(id) {
    const name = GOOGLE_FONT_IMPORT.exec(id)?.[1];
    return name ? path.join(VIDEO_DIR, ...GOOGLE_SHIM_SEGMENTS, googleFontModuleFile(name)) : null;
  },
});

export default defineConfig({
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      routesDirectory: "./packages/ui/src/routes",
      generatedRouteTree: "./packages/ui/src/routeTree.gen.ts",
    }),
    localFonts(),
    react(),
    tailwindcss(),
  ],
  resolve: { alias: { ...UI_ALIAS, ...PACKAGE_SOURCES } },
  server: { watch: { ignored: ["**/out/**"] } },
});
