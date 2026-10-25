import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { localFonts } from "../../scripts/lib/vite-local-fonts.ts";

export default defineConfig({
  root: import.meta.dirname,
  publicDir: false,
  plugins: [localFonts(), react(), tailwindcss()],
  build: { outDir: "dist", emptyOutDir: true },
});
