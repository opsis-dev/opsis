import { defineConfig } from "vite";
import { SRC_ALIAS } from "./scripts/lib/tool-config.ts";

export default defineConfig({
  resolve: { alias: SRC_ALIAS },
  define: {
    "import.meta.url": 'require("node:url").pathToFileURL(__filename).href',
    "process.env.WS_NO_BUFFER_UTIL": JSON.stringify("1"),
    "process.env.WS_NO_UTF_8_VALIDATE": JSON.stringify("1"),
  },
  build: {
    rollupOptions: {
      external: [
        "playwright-core",
        /^playwright-core\//,
        "@remotion/bundler",
        "@remotion/tailwind-v4",
        /^@remotion\/compositor-/,
        "esbuild",
      ],
    },
  },
});
