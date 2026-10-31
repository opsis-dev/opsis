import path from "node:path";
import { defineConfig } from "vite";

const FILM_DIR = import.meta.dirname;
const ROOT = path.join(FILM_DIR, "..", "..");

export default defineConfig(({ mode }) => ({
  root: FILM_DIR,
  base: "./",
  publicDir: path.join(ROOT, "public"),
  clearScreen: false,
  build: {
    outDir: path.join(ROOT, "film-bundle"),
    emptyOutDir: true,
    sourcemap: mode !== "release",
  },
}));
