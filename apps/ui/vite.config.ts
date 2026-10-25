import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { localFonts } from "../../scripts/lib/vite-local-fonts.ts";

const UI_DIR = import.meta.dirname;

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
