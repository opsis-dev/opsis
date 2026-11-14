import path from "node:path";
import { z } from "zod";
import { pinnedSchema, writeGeneratedJson } from "../pins";
import { readJsonWith } from "../read-json";
import { ROOT } from "../tool-config";

export const PINS_FILE = path.join(ROOT, "scripts", "lib", "setup", "pins.generated.json");

const ffmpegTargetSchema = z.object({
  target: z.string(),
  asset: z.string(),
  program: pinnedSchema,
  probe: pinnedSchema,
  licence: pinnedSchema,
});

export const pinsSchema = z.object({
  zig: z.object({ version: z.string(), archives: z.record(z.string(), pinnedSchema) }),
  qt: z.object({
    version: z.string(),
    sources: z.array(pinnedSchema.extend({ module: z.string() })),
  }),
  kddw: pinnedSchema.extend({ version: z.string() }),
  skia: z.object({ repo: z.string(), branch: z.string(), commit: z.string() }),
  filament: z.object({ version: z.string(), archives: z.record(z.string(), pinnedSchema) }),
  ffmpeg: z.object({ release: z.string(), targets: z.array(ffmpegTargetSchema) }),
  npm: z.record(z.string(), pinnedSchema.extend({ version: z.string() })),
});

export type Pins = z.infer<typeof pinsSchema>;

export type FfmpegPins = Pins["ffmpeg"];

export function readPins(): Pins {
  return readJsonWith(pinsSchema, PINS_FILE);
}

export function writePins(pins: Pins): void {
  writeGeneratedJson(PINS_FILE, pinsSchema.parse(pins));
}

export function platformName(
  platform: string = process.platform,
  arch: string = process.arch,
): string {
  return `${platform}-${arch}`;
}
