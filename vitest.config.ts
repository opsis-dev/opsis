import { testConfig } from "./scripts/lib/vitest-base.ts";

export default testConfig({ nodeTests: ["scripts/**/*.test.ts"] });
