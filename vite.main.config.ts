import { defineConfig } from "vite";
import { SRC_ALIAS } from "./scripts/lib/tool-config.ts";

export default defineConfig({
  resolve: { alias: SRC_ALIAS },
});
