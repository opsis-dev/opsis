import type { AppSettings, CanvasMode, StartOption, TraceRetention } from "@/generated/bindings";

export const TRACE_RETENTION: TraceRetention[] = ["always", "failures", "last-n", "off"];

export const TRACE_RETENTION_LABELS: Record<TraceRetention, string> = {
  always: "Always",
  failures: "Failures only",
  "last-n": "Last N",
  off: "Off",
};

export const KEEP_LAST_TRACES = { min: 1, max: 500 };

export const LIVE_FPS = { min: 1, max: 30 };

export const START_OPTIONS: StartOption[] = ["start-screen", "last-project", "new-project"];

export const START_OPTION_LABELS: Record<StartOption, string> = {
  "start-screen": "Show the start screen",
  "last-project": "Open the last project",
  "new-project": "Start a new project",
};

export const CANVAS_MODES: CanvasMode[] = ["mouse", "touchpad"];

export const DEFAULT_SETTINGS: AppSettings = {
  app: { onStart: "start-screen", confirmExit: true, setupSkipped: false },
  traces: { retention: "always", keepLast: 10 },
  live: { fps: 12, autoOpen: true },
  render: { licenseKey: "" },
  projects: { defaultDir: "", lastDir: "" },
  editor: { canvasMode: "mouse" },
  api: { enabled: false },
  engine: { browser: null, ffmpeg: null, ffprobe: null },
};

type EditableSection = Exclude<keyof AppSettings, "engine">;

export type SettingsPatch = { [K in EditableSection]?: Partial<AppSettings[K]> };

export const mergeSettings = (current: AppSettings, patch: SettingsPatch): AppSettings => ({
  app: { ...current.app, ...patch.app },
  traces: { ...current.traces, ...patch.traces },
  live: { ...current.live, ...patch.live },
  render: { ...current.render, ...patch.render },
  projects: { ...current.projects, ...patch.projects },
  editor: { ...current.editor, ...patch.editor },
  api: { ...current.api, ...patch.api },
  engine: current.engine,
});
