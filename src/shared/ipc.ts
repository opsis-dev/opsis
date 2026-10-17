import type { LiveMessage } from "./live";
import type { WindowRole } from "./menu";
import type { RunEvent } from "./schema/events";
import type { RunProgress } from "./run-progress";
import type { LastRun } from "./schema/values";
import type { Workflow } from "./schema/workflow";
import type { AppSettings, SettingsPatch } from "./settings";
import type { ApiEditorReply, ApiEditorRequest } from "./api/types";
import type { TraceModel } from "./trace";
import type { FontFamilyDef } from "./fonts/types";
import type { PlannedFamily } from "./fonts/plan";
import type { TimelineSpec } from "./schema/timeline";
import type { AnalysisRequest, AnalysisResult } from "./mix/result";

export type RecentProject = { dir: string; name: string; openedAt: number };

export type OpenProject = {
  dir: string;
  workflow: Workflow;
  lastRun: LastRun | null;
  version: string | null;
};

export type SaveResult =
  | { ok: true; version: string }
  | { ok: false; version: string; workflow: Workflow | null };

export type ProjectFileChange = { dir: string; workflow: Workflow; version: string };

export type ProjectLocations = { defaultDir: string; suggestedDir: string };

export const SETTINGS_PAGES = [
  "general",
  "projects",
  "engine",
  "recording",
  "fonts",
  "export",
  "api",
  "licences",
  "about",
] as const;

export type SettingsPage = (typeof SETTINGS_PAGES)[number];

export type LicenceEntry = {
  id: string;
  title: string;
  summary: string;
  text?: string;
  file?: string;
};

export type ApiStatus = {
  enabled: boolean;
  listening: boolean;
  address: string;
  error?: string;
};

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
  step: string;
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
  | "view:toggle-right"
  | "view:toggle-minimap"
  | "view:focus-preview"
  | "view:focus-timeline"
  | "view:focus-mixer"
  | "view:focus-run"
  | "edit:add-node";

export type DockPanelName = "preview" | "timeline" | "mixer" | "run";

export type CloseRequest = { ask: boolean; busy: string[] };
export type CloseDecision = { close: boolean; dontAsk: boolean };

export type InspectedFamily = PlannedFamily;

export type InspectedFonts = {
  token: string;
  families: InspectedFamily[];
  license?: string;
  skipped: number;
};

export type FontCatalogEntry = {
  id: string;
  family: string;
  category: string;
  variable: boolean;
  styles: string[];
  weights: number[];
  license: string;
  type: string;
};

export type FontInstalledFace = {
  postscriptName: string;
  data: ArrayBuffer;
};

export type StillRequest = {
  projectDir: string;
  nodeId: string;
  key: string;
  spec: TimelineSpec;
  frame: number;
  priority: boolean;
};

export type StillResult = { hash: string; ready: boolean; unavailable?: boolean };

export type StillReady = { projectDir: string; nodeId: string; hash: string };

export type WindowState = { maximized: boolean; fullscreen: boolean; focused: boolean };

export interface DemoflowApi {
  platform: NodeJS.Platform;
  listRecentProjects(): Promise<RecentProject[]>;
  createProject(opts: {
    name: string;
    template: ProjectTemplateId;
    dir?: string;
  }): Promise<OpenProject | null>;
  openProject(dir?: string): Promise<OpenProject | null>;
  saveWorkflow(dir: string, workflow: Workflow, base?: string | null): Promise<SaveResult>;
  projectLocations(): Promise<ProjectLocations>;
  chooseFolder(opts: { title: string; defaultPath?: string }): Promise<string | null>;
  watchProject(dir: string | null): Promise<void>;
  onProjectFileChanged(listener: (change: ProjectFileChange) => void): () => void;
  readLastRun(dir: string): Promise<LastRun | null>;
  importAssets(dir: string, paths?: string[]): Promise<string[]>;
  pathForFile(file: File): string;
  revealInFolder(dir: string, relativePath?: string): Promise<void>;
  runWorkflow(req: RunRequest): Promise<{ runId: string }>;
  cancelRun(): Promise<void>;
  runStatus(): Promise<{
    runId: string | null;
    projectDir: string | null;
    progress: RunProgress;
  }>;
  onRunEvent(listener: (event: RunEvent) => void): () => void;
  pickElement(req: PickRequest): Promise<PickResult>;
  getEnvironment(): Promise<EnvStatus>;
  installEngine(): Promise<void>;
  onEngineInstall(listener: (event: EngineInstallEvent) => void): () => void;
  getSettings(): Promise<AppSettings>;
  setSettings(settings: AppSettings): Promise<AppSettings>;
  patchSettings(patch: SettingsPatch): Promise<AppSettings>;
  onSettingsChanged(listener: (settings: AppSettings) => void): () => void;
  openSettings(page?: SettingsPage): Promise<void>;
  sendMenuToEditor(command: MenuCommand): void;
  listLicences(): Promise<LicenceEntry[]>;
  openLicenceFile(file: string): Promise<void>;
  apiStatus(): Promise<ApiStatus>;
  onApiStatus(listener: (status: ApiStatus) => void): () => void;
  onApiRequest(listener: (request: ApiEditorRequest) => Promise<ApiEditorReply>): () => void;
  onLive(listener: (message: LiveMessage) => void): () => void;
  readTrace(dir: string, traceId: string): Promise<TraceModel>;
  listSecrets(dir: string): Promise<string[]>;
  clearCache(dir: string): Promise<void>;
  onMenuCommand(listener: (command: MenuCommand) => void): () => void;
  setTitle(title: string): void;
  windowRole(role: WindowRole): Promise<void>;
  getWindowState(): Promise<WindowState>;
  requestStill(request: StillRequest): Promise<StillResult>;
  listStills(projectDir: string): Promise<Record<string, string>>;
  analyze(request: AnalysisRequest): Promise<AnalysisResult>;
  openPanelWindow(panel: DockPanelName, projectDir: string): Promise<void>;
  focusPanelWindow(panel: DockPanelName): Promise<void>;
  closePanelWindow(panel: DockPanelName): Promise<void>;
  closePanelWindows(): Promise<void>;
  listPanelWindows(): Promise<DockPanelName[]>;
  onPanelWindows(listener: (open: DockPanelName[]) => void): () => void;
  reportUnsaved(unsaved: boolean): void;
  onFlushSave(listener: () => Promise<void>): () => void;
  onConfirmClose(listener: (request: CloseRequest) => void): () => void;
  decideClose(decision: CloseDecision): void;
  closeShown(): void;
  onStillReady(listener: (still: StillReady) => void): () => void;
  onWindowState(listener: (state: WindowState) => void): () => void;
  dispatchMenuCommand(command: MenuCommand): void;
  listFonts(): Promise<FontFamilyDef[]>;
  inspectFontFiles(paths?: string[]): Promise<InspectedFonts | null>;
  inspectInstalledFonts(faces: FontInstalledFace[]): Promise<InspectedFonts>;
  addInspectedFonts(token: string, families: string[]): Promise<FontFamilyDef[]>;
  removeFont(id: string): Promise<void>;
  fontCatalog(): Promise<FontCatalogEntry[]>;
  downloadFont(id: string): Promise<FontFamilyDef>;
  useFontInProject(dir: string, fontId: string): Promise<FontFamilyDef>;
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
  windowRole: "demoflow:window:role",
  getWindowState: "demoflow:window:get-state",
  requestStill: "demoflow:stills:request",
  listStills: "demoflow:stills:list",
  stillReady: "demoflow:stills:ready",
  windowState: "demoflow:window:state",
  listFonts: "demoflow:fonts:list",
  inspectFontFiles: "demoflow:fonts:inspect-files",
  inspectInstalledFonts: "demoflow:fonts:inspect-installed",
  addInspectedFonts: "demoflow:fonts:add",
  removeFont: "demoflow:fonts:remove",
  fontCatalog: "demoflow:fonts:catalog",
  downloadFont: "demoflow:fonts:download",
  useFontInProject: "demoflow:fonts:use-in-project",
  analyze: "demoflow:analysis:run",
  openPanelWindow: "demoflow:panels:open",
  focusPanelWindow: "demoflow:panels:focus",
  closePanelWindow: "demoflow:panels:close",
  closePanelWindows: "demoflow:panels:close-all",
  listPanelWindows: "demoflow:panels:list",
  panelWindows: "demoflow:panels:changed",
  reportUnsaved: "demoflow:exit:unsaved",
  flushSave: "demoflow:exit:flush-save",
  saveFlushed: "demoflow:exit:save-flushed",
  confirmClose: "demoflow:exit:confirm",
  closeDecision: "demoflow:exit:decision",
  closeShown: "demoflow:exit:shown",
  projectLocations: "demoflow:projects:locations",
  chooseFolder: "demoflow:projects:choose-folder",
  watchProject: "demoflow:projects:watch",
  projectFileChanged: "demoflow:projects:file-changed",
  patchSettings: "demoflow:settings:patch",
  settingsChanged: "demoflow:settings:changed",
  openSettings: "demoflow:settings:open-window",
  menuToEditor: "demoflow:menu:to-editor",
  listLicences: "demoflow:licences:list",
  openLicenceFile: "demoflow:licences:open",
  apiStatus: "demoflow:api:status",
  apiStatusChanged: "demoflow:api:status-changed",
  apiRequest: "demoflow:api:request",
  apiReply: "demoflow:api:reply",
} as const;

declare global {
  interface Window {
    demoflow: DemoflowApi;
  }
}
