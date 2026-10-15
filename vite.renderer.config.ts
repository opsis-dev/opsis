import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

const GOOGLE_FONT = /^@remotion\/google-fonts\/([A-Za-z]+)$/;
const kebab = (name: string) => name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase();

const localFonts = (): Plugin => ({
  name: "demoflow-local-fonts",
  enforce: "pre",
  resolveId(id) {
    const name = GOOGLE_FONT.exec(id)?.[1];
    return name
      ? path.resolve(import.meta.dirname, "src/video/fonts/google", `${kebab(name)}.ts`)
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
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
});
