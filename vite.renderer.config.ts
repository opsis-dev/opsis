import path from "node:path";
import { mergeConfig } from "vite";
import ui from "./apps/ui/vite.config.ts";

export default mergeConfig(ui, {
  build: { outDir: path.join(import.meta.dirname, ".vite", "renderer", "main_window") },
});
