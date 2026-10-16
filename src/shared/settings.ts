import { z } from "zod";

export const TRACE_RETENTION = ["always", "failures", "last-n", "off"] as const;

export const START_OPTIONS = ["start-screen", "last-project", "new-project"] as const;
export type StartOption = (typeof START_OPTIONS)[number];

export const CANVAS_MODES = ["mouse", "touchpad"] as const;
export type CanvasMode = (typeof CANVAS_MODES)[number];

export const PROJECTS_FOLDER_NAME = "DemoflowProjects";

export const appSettingsSchema = z.object({
  app: z
    .object({
      onStart: z.enum(START_OPTIONS).default("start-screen"),
      confirmExit: z.boolean().default(true),
    })
    .default({ onStart: "start-screen", confirmExit: true }),
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
  render: z
    .object({
      licenseKey: z.string().max(200).default(""),
    })
    .default({ licenseKey: "" }),
  projects: z
    .object({
      defaultDir: z.string().default(""),
      lastDir: z.string().default(""),
    })
    .default({ defaultDir: "", lastDir: "" }),
  editor: z
    .object({
      canvasMode: z.enum(CANVAS_MODES).default("mouse"),
    })
    .default({ canvasMode: "mouse" }),
  api: z
    .object({
      enabled: z.boolean().default(false),
    })
    .default({ enabled: false }),
});

export type AppSettings = z.infer<typeof appSettingsSchema>;

export const DEFAULT_SETTINGS: AppSettings = appSettingsSchema.parse({});

export type SettingsPatch = { [K in keyof AppSettings]?: Partial<AppSettings[K]> };
