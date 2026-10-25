import path from "node:path";
import { testConfig } from "../../scripts/lib/vitest-base.ts";

export default testConfig({ alias: { "@": path.join(import.meta.dirname, "src") } });
