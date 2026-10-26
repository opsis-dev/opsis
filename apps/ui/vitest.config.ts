import path from "node:path";
import { testConfig } from "../../scripts/lib/vitest-base.ts";

const src = path.join(import.meta.dirname, "src");

export default testConfig({
  alias: { "@": src },
  setupFiles: [path.join(src, "test", "setup.ts")],
  globalSetup: [path.join(src, "test", "build-editor-data.ts")],
});
