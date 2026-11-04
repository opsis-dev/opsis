import path from "node:path";
import { defineConfig } from "vite";

const TERMINAL_DIR = import.meta.dirname;
const ROOT = path.join(TERMINAL_DIR, "..", "..");

export default defineConfig(({ mode }) => ({
  root: TERMINAL_DIR,
  base: "./",
  publicDir: false,
  clearScreen: false,
  build: {
    outDir: path.join(ROOT, "terminal-bundle"),
    emptyOutDir: true,
    sourcemap: mode !== "release",
  },
}));
