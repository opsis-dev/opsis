import path from "node:path";
import { z } from "zod";
import { pinnedSchema } from "./pins";
import { readJsonWith } from "./read-json";
import { ROOT } from "./tool-config";

export const SHELL_PLATFORMS = ["linux64", "linux-arm64", "mac-x64", "mac-arm64", "win64"] as const;

export type ShellPlatform = (typeof SHELL_PLATFORMS)[number];

export const ENGINE_DOWNLOADS = path.join(
  ROOT,
  "crates",
  "engine",
  "src",
  "downloads.generated.json",
);

export const engineDownloadsSchema = z.object({
  testedVersion: z.string(),
  shells: z.array(pinnedSchema.extend({ platform: z.enum(SHELL_PLATFORMS) })),
});

export type EngineDownloads = z.infer<typeof engineDownloadsSchema>;

export function shellUrl(version: string, platform: ShellPlatform): string {
  return `https://storage.googleapis.com/chrome-for-testing-public/${version}/${platform}/chrome-headless-shell-${platform}.zip`;
}

export function readEngineDownloads(): EngineDownloads {
  return readJsonWith(engineDownloadsSchema, ENGINE_DOWNLOADS);
}
