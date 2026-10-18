import { z } from "zod";

export const TRACE_RETENTION = ["always", "failures", "last-n", "off"] as const;
type TraceRetention = (typeof TRACE_RETENTION)[number];

export const TRACE_RETENTION_LABELS: Record<TraceRetention, string> = {
  always: "Always",
  failures: "Failures only",
  "last-n": "Last N",
  off: "Off",
};

export const KEEP_LAST_TRACES = { min: 1, max: 500 };

export const LIVE_FPS = { min: 1, max: 30 };

export const START_OPTIONS = ["start-screen", "last-project", "new-project"] as const;
type StartOption = (typeof START_OPTIONS)[number];

export const START_OPTION_LABELS: Record<StartOption, string> = {
  "start-screen": "Show the start screen",
  "last-project": "Open the last project",
  "new-project": "Start a new project",
};

export const CANVAS_MODES = ["mouse", "touchpad"] as const;
export type CanvasMode = (typeof CANVAS_MODES)[number];

export const PROJECTS_FOLDER_NAME = "DemoflowProjects";

export const appSettingsSchema = z.object({
  app: z
    .object({
      onStart: z.enum(START_OPTIONS).default("start-screen"),
      confirmExit: z.boolean().default(true),
    })
    .prefault({}),
  traces: z
    .object({
      retention: z.enum(TRACE_RETENTION).default("always"),
      keepLast: z.number().int().min(KEEP_LAST_TRACES.min).max(KEEP_LAST_TRACES.max).default(10),
    })
    .prefault({}),
  live: z
    .object({
      fps: z.number().int().min(LIVE_FPS.min).max(LIVE_FPS.max).default(12),
      autoOpen: z.boolean().default(true),
    })
    .prefault({}),
  render: z
    .object({
      licenseKey: z.string().max(200).default(""),
    })
    .prefault({}),
  projects: z
    .object({
      defaultDir: z.string().default(""),
      lastDir: z.string().default(""),
    })
    .prefault({}),
  editor: z
    .object({
      canvasMode: z.enum(CANVAS_MODES).default("mouse"),
    })
    .prefault({}),
  api: z
    .object({
      enabled: z.boolean().default(false),
    })
    .prefault({}),
});

export type AppSettings = z.infer<typeof appSettingsSchema>;

export const DEFAULT_SETTINGS: AppSettings = appSettingsSchema.parse({});

export type SettingsPatch = { [K in keyof AppSettings]?: Partial<AppSettings[K]> };

export const mergeSettings = (current: AppSettings, patch: SettingsPatch): AppSettings => ({
  app: { ...current.app, ...patch.app },
  traces: { ...current.traces, ...patch.traces },
  live: { ...current.live, ...patch.live },
  render: { ...current.render, ...patch.render },
  projects: { ...current.projects, ...patch.projects },
  editor: { ...current.editor, ...patch.editor },
  api: { ...current.api, ...patch.api },
});
