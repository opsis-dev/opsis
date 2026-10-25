import { defineConfig } from "oxlint";
import { lintConfig } from "../../scripts/lib/oxlint-base.ts";

export default defineConfig(lintConfig(import.meta.dirname));
