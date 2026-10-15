import { z } from "zod";

export const TRACE_RETENTION = ["always", "failures", "last-n", "off"] as const;

export const appSettingsSchema = z.object({
  traces: z
    .object({
      retention: z.enum(TRACE_RETENTION).default("always"),
      keepLast: z.number().int().min(1).max(500).default(10),
    })
    .default({ retention: "always", keepLast: 10 }),
  live: z
    .object({
      fps: z.number().int().min(1).max(60).default(12),
      autoOpen: z.boolean().default(true),
    })
    .default({ fps: 12, autoOpen: true }),
});

export type AppSettings = z.infer<typeof appSettingsSchema>;

export const DEFAULT_SETTINGS: AppSettings = appSettingsSchema.parse({});
