import { readFileSync } from "node:fs";

export const PROFILERS = ["none", "samply", "perf", "heaptrack"] as const;

export type Profiler = (typeof PROFILERS)[number];

type RunTarget = { id: string; type: string };

export type InsideConfig = {
  userData: string;
  project: string;
  out: string;
  sitePort: number;
  sampleMs: number;
  nodeTimeoutMs: number;
  profiler: Profiler;
  app: string;
  appEnv: Record<string, string>;
  targets: RunTarget[];
};

export const CONFIG_FILE = "config.json";

export function readConfig(file: string): InsideConfig {
  return JSON.parse(readFileSync(file, "utf8"));
}
