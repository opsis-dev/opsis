import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";
import { fontShimFile, GOOGLE_FONT_IMPORT } from "./src/shared/fonts/google-shim.ts";
import { SRC_ALIAS } from "./scripts/lib/tool-config.ts";

const localFonts = (): Plugin => ({
  name: "demoflow-local-fonts",
  enforce: "pre",
  resolveId(id) {
    const name = GOOGLE_FONT_IMPORT.exec(id)?.[1];
    return name
      ? path.resolve(import.meta.dirname, "src/video/fonts/google", fontShimFile(name))
      : null;
  },
});

export default defineConfig({
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      routesDirectory: "./src/renderer/routes",
      generatedRouteTree: "./src/renderer/routeTree.gen.ts",
    }),
    localFonts(),
    react(),
    tailwindcss(),
  ],
  resolve: { alias: SRC_ALIAS },
});
