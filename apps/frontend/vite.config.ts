import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const FRONTEND_DIR = import.meta.dirname;
const FRONTEND_DEV_PORT = 5173;

export default defineConfig({
  root: FRONTEND_DIR,
  publicDir: path.join(FRONTEND_DIR, "public"),
  plugins: [
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
      routesDirectory: path.join(FRONTEND_DIR, "src", "routes"),
      generatedRouteTree: path.join(FRONTEND_DIR, "src", "routeTree.gen.ts"),
    }),
    react(),
    tailwindcss(),
  ],
  resolve: { alias: { "@": path.join(FRONTEND_DIR, "src") }, preserveSymlinks: false },
  clearScreen: false,
  server: { port: FRONTEND_DEV_PORT, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true },
});
