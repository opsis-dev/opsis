import path from "node:path";
import { defineConfig } from "vite";

const ORACLE_DIR = import.meta.dirname;
const ROOT = path.join(ORACLE_DIR, "..", "..");

export default defineConfig(({ mode }) => ({
  root: ORACLE_DIR,
  base: "./",
  publicDir: path.join(ROOT, "public"),
  clearScreen: false,
  build: {
    outDir: path.join(ROOT, "oracle-bundle"),
    emptyOutDir: true,
    sourcemap: mode !== "release",
  },
}));
