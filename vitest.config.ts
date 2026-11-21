import path from "node:path";
import { testConfig } from "./scripts/lib/vitest-base.ts";

export default testConfig({
  nodeTests: ["scripts/**/*.test.ts"],
  globalSetup: [path.join(import.meta.dirname, "scripts", "lib", "build-tools.ts")],
});
