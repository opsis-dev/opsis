import { testConfig } from "./scripts/lib/vitest-base.ts";

export default testConfig({ nodeTests: ["src/**/*.test.ts", "scripts/**/*.test.ts"] });
