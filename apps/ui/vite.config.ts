import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  googleFontModuleFile,
  GOOGLE_FONT_IMPORT,
  GOOGLE_SHIM_SEGMENTS,
} from "@opsis/video/fonts/google-font-imports";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

const UI_DIR = import.meta.dirname;
const VIDEO_DIR = path.dirname(fileURLToPath(import.meta.resolve("@opsis/video/root")));

const localFonts = (): Plugin => ({
  name: "opsis-local-fonts",
  enforce: "pre",
  resolveId(id) {
    const name = GOOGLE_FONT_IMPORT.exec(id)?.[1];
    return name ? path.join(VIDEO_DIR, ...GOOGLE_SHIM_SEGMENTS, googleFontModuleFile(name)) : null;
  },
});

export default defineConfig({
  root: UI_DIR,
  publicDir: path.join(UI_DIR, "..", "..", "public"),
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      routesDirectory: path.join(UI_DIR, "src", "routes"),
      generatedRouteTree: path.join(UI_DIR, "src", "routeTree.gen.ts"),
    }),
    localFonts(),
    react(),
    tailwindcss(),
  ],
  resolve: { alias: { "@": path.join(UI_DIR, "src") }, preserveSymlinks: false },
});
