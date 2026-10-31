import { defineConfig } from "oxlint";
import { lintConfig, runtimeNeutral } from "../../scripts/lib/oxlint-base.ts";

export default defineConfig(
  lintConfig(import.meta.dirname, [runtimeNeutral("apps/film", { relativeOnly: true })]),
);
