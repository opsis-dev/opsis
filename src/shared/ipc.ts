import type { LiveMessage } from "./live";
import type { RunEvent } from "./schema/events";
import type { LastRun } from "./schema/values";
import type { Workflow } from "./schema/workflow";
import type { AppSettings } from "./settings";
import type { TraceModel } from "./trace";

export type RecentProject = { dir: string; name: string; openedAt: number };

export type OpenProject = { dir: string; workflow: Workflow; lastRun: LastRun | null };

export type ProjectTemplateId = string;

export type RunRequest = {
  projectDir: string;
  workflow: Workflow;
  targetNodeIds?: string[];
  fromNodeId?: string;
};

export type EnvStatus = {
  platform: string;
  arch: string;
  isPackaged: boolean;
  edition: "full" | "lite" | "dev";
  userData: string;
  versions: { electron: string; chrome: string; node: string; app: string };
  engine: {
    dir: string;
    complete: boolean;
    headlessShell: string | null;
    ffmpeg: string | null;
    ffprobe: string | null;
  };
};

export type EngineInstallEvent = {
  part: "ffmpeg" | "headless-shell";
  progress: number | null;
  message: string;
  done?: boolean;
  error?: string;
};

export type PickRequest = { url: string; viewport?: { width: number; height: number } };
export type PickResult = {
  target: string;
  css: string;
  role?: string;
  selectorUsed: string;
} | null;

export type MenuCommand =
  | "project:new"
  | "project:open"
  | "project:save"
  | "edit:undo"
  | "edit:redo"
  | "edit:group"
  | "edit:layout"
  | "run:start"
  | "run:cancel"
  | "view:settings"
  | "view:projects"
  | "view:toggle-left"
  | "view:toggle-bottom"
  | "view:toggle-right";

export interface DemoflowApi {
  platform: NodeJS.Platform;
  listRecentProjects(): Promise<RecentProject[]>;
  createProject(opts: {
    name: string;
    template: ProjectTemplateId;
    dir?: string;
  }): Promise<OpenProject | null>;
  openProject(dir?: string): Promise<OpenProject | null>;
  saveWorkflow(dir: string, workflow: Workflow): Promise<void>;
  readLastRun(dir: string): Promise<LastRun | null>;
  importAssets(dir: string, paths?: string[]): Promise<string[]>;
  pathForFile(file: File): string;
  revealInFolder(dir: string, relativePath?: string): Promise<void>;
  runWorkflow(req: RunRequest): Promise<{ runId: string }>;
  cancelRun(): Promise<void>;
  runStatus(): Promise<{ runId: string | null; projectDir: string | null }>;
  onRunEvent(listener: (event: RunEvent) => void): () => void;
  pickElement(req: PickRequest): Promise<PickResult>;
  getEnvironment(): Promise<EnvStatus>;
  installEngine(): Promise<void>;
  onEngineInstall(listener: (event: EngineInstallEvent) => void): () => void;
  getSettings(): Promise<AppSettings>;
  setSettings(settings: AppSettings): Promise<AppSettings>;
  onLive(listener: (message: LiveMessage) => void): () => void;
  readTrace(dir: string, traceId: string): Promise<TraceModel>;
  listSecrets(dir: string): Promise<string[]>;
  clearCache(dir: string): Promise<void>;
  onMenuCommand(listener: (command: MenuCommand) => void): () => void;
  setTitle(title: string): void;
}

export const IPC = {
  listRecent: "demoflow:projects:recent",
  createProject: "demoflow:projects:create",
  openProject: "demoflow:projects:open",
  saveWorkflow: "demoflow:projects:save",
  readLastRun: "demoflow:projects:last-run",
  importAssets: "demoflow:projects:import-assets",
  reveal: "demoflow:shell:reveal",
  run: "demoflow:run:start",
  cancel: "demoflow:run:cancel",
  runStatus: "demoflow:run:status",
  runEvent: "demoflow:run:event",
  pick: "demoflow:editor:pick",
  env: "demoflow:env",
  installEngine: "demoflow:engine:install",
  engineEvent: "demoflow:engine:event",
  getSettings: "demoflow:settings:get",
  setSettings: "demoflow:settings:set",
  livePort: "demoflow:live:port",
  readTrace: "demoflow:trace:read",
  listSecrets: "demoflow:secrets:list",
  clearCache: "demoflow:cache:clear",
  menu: "demoflow:menu",
  setTitle: "demoflow:window:title",
} as const;

declare global {
  interface Window {
    demoflow: DemoflowApi;
  }
}
