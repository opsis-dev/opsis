import { Channel } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { ApiEditorReply } from "@/core/api";
import { platform } from "@/lib/platform";
import {
  commands,
  events,
  type AnalysisRequest,
  type AnalysisResult,
  type ApiServerStatus,
  type AppSettings,
  type CloseDecision,
  type CloseRequest,
  type DockPanelName,
  type EditorCompile,
  type EngineInstallEvent,
  type EnvStatus,
  type FontCatalogEntry,
  type FontFamilyDef,
  type InspectedFonts,
  type InstalledFamily,
  type LastRun,
  type LicenceEntry,
  type LiveEvent,
  type MenuCommand,
  type MenuSection,
  type OpenProject,
  type PickRequest,
  type PickResult,
  type PlayRange,
  type PreviewReported,
  type EditorRequest,
  type ProjectFileChange,
  type ProjectFilesChange,
  type ProjectLocations,
  type RecentProject,
  type Relayed,
  type RelayTarget,
  type RunEvent,
  type RunRequest,
  type RunStatus,
  type SaveResult,
  type SettingsPage,
  type TimelineSpec,
  type TraceModel,
  type Value,
  type ViewEvent,
  type ViewInput,
  type ViewPrefChange,
  type ViewSize,
  type WindowRole,
  type WindowState,
  type Workflow,
} from "../generated/bindings";

type Result<T> = Promise<{ status: "ok"; data: T } | { status: "error"; error: string }>;

type Listenable<T> = {
  listen: (cb: (event: { payload: T }) => void) => Promise<UnlistenFn>;
};

export type FrameHeader = {
  width: number;
  height: number;
  t: number;
  captureId?: string;
  nodeId?: string;
};

export type StreamFrame = { header: FrameHeader; jpeg: Uint8Array<ArrayBuffer> };

export type LiveMessage =
  | {
      type: "frame";
      captureId: string;
      nodeId: string;
      jpeg: Uint8Array<ArrayBuffer>;
      width: number;
      height: number;
      t: number;
    }
  | { type: "event"; captureId: string; nodeId: string; t: number; event: LiveEvent };

async function unwrap<T>(result: Result<T>): Promise<T> {
  const done = await result;
  if (done.status === "error") throw new Error(done.error);
  return done.data;
}

function on<T>(source: Listenable<T>, listener: (payload: T) => void): () => void {
  let off: UnlistenFn | null = null;
  let gone = false;
  void source
    .listen((event) => listener(event.payload))
    .then((unlisten) => {
      if (gone) unlisten();
      else off = unlisten;
    });
  return () => {
    gone = true;
    off?.();
  };
}

const here = () => getCurrentWebviewWindow();

const decoder = new TextDecoder();

export function readFrame(buffer: ArrayBuffer): StreamFrame {
  const length = new DataView(buffer).getUint32(0, true);
  const header = JSON.parse(decoder.decode(new Uint8Array(buffer, 4, length))) as FrameHeader;
  return { header, jpeg: new Uint8Array(buffer.slice(4 + length)) };
}

function frameChannel(listener: (frame: StreamFrame) => void): Channel<ArrayBuffer> {
  return new Channel<ArrayBuffer>((buffer) => listener(readFrame(buffer)));
}

const droppedPaths = new Map<string, string>();

function rememberDrops(): void {
  void getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type !== "drop") return;
    for (const path of event.payload.paths)
      droppedPaths.set(path.split(/[\\/]/).at(-1) ?? path, path);
  });
}

export function createOpsis() {
  if (platform !== "win32") rememberDrops();
  return {
    platform,
    listRecentProjects: (): Promise<RecentProject[]> => unwrap(commands.listRecentProjects()),
    createProject: (opts: {
      name: string;
      template: string;
      dir?: string;
    }): Promise<OpenProject | null> =>
      unwrap(commands.createProject(opts.name, opts.template, opts.dir ?? null)),
    pickProjectDir: (): Promise<string | null> => unwrap(commands.pickProjectDir()),
    projectFolder: (parent: string, name: string): Promise<string> =>
      unwrap(commands.projectFolder(parent, name)),
    openProject: (dir: string): Promise<OpenProject> => unwrap(commands.openProject(dir)),
    saveWorkflow: (dir: string, workflow: Workflow, base: string | null): Promise<SaveResult> =>
      unwrap(commands.saveWorkflow(dir, workflow, base)),
    projectLocations: (): Promise<ProjectLocations> => unwrap(commands.projectLocations()),
    chooseFolder: (opts: { title: string; defaultPath?: string }): Promise<string | null> =>
      unwrap(commands.chooseFolder(opts.title, opts.defaultPath ?? null)),
    watchProject: (dir: string | null) => unwrap(commands.watchProject(dir)),
    watchProjectFiles: (dir: string, refs: readonly string[]) =>
      unwrap(commands.watchProjectFiles(dir, [...refs])),
    onProjectFileChanged: (listener: (change: ProjectFileChange) => void) =>
      on(events.projectFileChanged(here()), listener),
    onProjectFilesChanged: (listener: (change: ProjectFilesChange) => void) =>
      on(events.projectFilesChanged(here()), listener),
    readLastRun: (dir: string): Promise<LastRun | null> => unwrap(commands.readLastRun(dir)),
    importAssets: (dir: string, paths?: string[]): Promise<string[]> =>
      unwrap(commands.importAssets(dir, paths ?? null)),
    pathForFile: (file: File): string => droppedPaths.get(file.name) ?? "",
    revealInFolder: (dir: string, relativePath?: string) =>
      unwrap(commands.revealInFolder(dir, relativePath ?? null)),
    runWorkflow: (request: RunRequest): Promise<{ runId: string }> =>
      unwrap(commands.runWorkflow(request)),
    cancelRun: () => commands.cancelRun(),
    runStatus: (): Promise<RunStatus> => commands.runStatus(),
    onRunEvent: (listener: (event: RunEvent) => void) => on(events.runEventFired(here()), listener),
    pickElement: (request: PickRequest): Promise<PickResult | null> =>
      unwrap(commands.pickElement(request)),
    getEnvironment: (): Promise<EnvStatus> => commands.getEnvironment(),
    installEngine: () => unwrap(commands.installEngine()),
    onEngineInstall: (listener: (event: EngineInstallEvent) => void) =>
      on(events.engineInstallProgress(here()), listener),
    getSettings: (): Promise<AppSettings> => unwrap(commands.getSettings()),
    patchSettings: (patch: { [key: string]: Value }): Promise<AppSettings> =>
      unwrap(commands.patchSettings(patch)),
    onSettingsChanged: (listener: (settings: AppSettings) => void) =>
      on(events.settingsChanged(here()), listener),
    setViewPref: (dir: string, key: string, value: unknown) =>
      unwrap(commands.setViewPref(dir, key, value as Value)),
    onViewPrefChanged: (listener: (change: ViewPrefChange) => void) =>
      on(events.viewPrefChanged(here()), listener),
    openSettings: (page?: SettingsPage) => unwrap(commands.openSettings(page ?? null)),
    sendMenuToEditor: (command: MenuCommand): void =>
      void unwrap(commands.sendMenuToEditor(command)),
    appMenu: (): Promise<MenuSection[]> => commands.appMenuSections(),
    listLicences: (): Promise<LicenceEntry[]> => unwrap(commands.listLicences()),
    openLicenceFile: (file: string) => unwrap(commands.openLicenceFile(file)),
    apiStatus: (): Promise<ApiServerStatus> => commands.apiStatus(),
    onApiStatus: (listener: (status: ApiServerStatus) => void) =>
      on(events.apiStatusChanged(here()), listener),
    onApiRequest: (listener: (request: EditorRequest) => Promise<ApiEditorReply>) => {
      const off = on(events.apiRequestReceived(here()), ({ id, request }) => {
        void listener(request)
          .catch((e: unknown): ApiEditorReply => ({
            ok: false,
            status: 500,
            error: e instanceof Error ? e.message : String(e),
          }))
          .then((reply) => commands.apiReply(id, reply as Value));
      });
      void commands.apiListening(true);
      return () => {
        off();
        void commands.apiListening(false);
      };
    },
    onLive: (listener: (message: LiveMessage) => void) => {
      void commands.watchLive(
        frameChannel(({ header, jpeg }) =>
          listener({
            type: "frame",
            captureId: header.captureId ?? "",
            nodeId: header.nodeId ?? "",
            jpeg,
            width: header.width,
            height: header.height,
            t: header.t,
          }),
        ),
      );
      return on(events.liveNoted(here()), (note) => listener({ type: "event", ...note }));
    },
    readTrace: (dir: string, traceId: string): Promise<TraceModel> =>
      unwrap(commands.readTrace(dir, traceId)),
    listSecrets: (dir: string): Promise<string[]> => unwrap(commands.listSecrets(dir)),
    clearCache: (dir: string) => unwrap(commands.clearCache(dir)),
    onMenuCommand: (listener: (command: MenuCommand) => void) =>
      on(events.menuCommandFired(here()), listener),
    setTitle: (title: string): void => void unwrap(commands.setTitle(title)),
    windowRole: (role: WindowRole) => unwrap(commands.windowRole(role)),
    getWindowState: (): Promise<WindowState> => commands.getWindowState(),
    analyze: <K extends AnalysisRequest["kind"]>(
      request: Extract<AnalysisRequest, { kind: K }>,
    ): Promise<Extract<AnalysisResult, { kind: K | "skipped" | "error" }>> =>
      unwrap(commands.analyze(request)) as Promise<
        Extract<AnalysisResult, { kind: K | "skipped" | "error" }>
      >,
    openPanelWindow: (panel: DockPanelName) => unwrap(commands.openPanelWindow(panel)),
    focusPanelWindow: (panel: DockPanelName) => commands.focusPanelWindow(panel),
    closePanelWindow: (panel: DockPanelName) => unwrap(commands.closePanelWindow(panel)),
    closePanelWindows: () => unwrap(commands.closePanelWindows()),
    listPanelWindows: (): Promise<DockPanelName[]> => commands.listPanelWindows(),
    onPanelWindows: (listener: (open: DockPanelName[]) => void) =>
      on(events.panelWindowsChanged(here()), listener),
    reportUnsaved: (unsaved: boolean): void => void commands.reportUnsaved(unsaved),
    onFlushSave: (listener: () => Promise<void>) =>
      on(events.flushSave(here()), () => {
        void listener()
          .catch((e: unknown) => console.error("The project couldn't be saved before closing.", e))
          .finally(() => void commands.saveFlushed());
      }),
    onConfirmClose: (listener: (request: CloseRequest) => void) =>
      on(events.confirmClose(here()), listener),
    decideClose: (decision: CloseDecision): void => void unwrap(commands.decideClose(decision)),
    closeShown: (): void => void commands.closeShown(),
    onWindowState: (listener: (state: WindowState) => void) =>
      on(events.windowStateChanged(here()), listener),
    onSettingsPage: (listener: (page: SettingsPage) => void) =>
      on(events.settingsPageShown(here()), listener),
    listFonts: (): Promise<FontFamilyDef[]> => unwrap(commands.listFonts()),
    inspectFontFiles: (paths?: string[]): Promise<InspectedFonts | null> =>
      unwrap(commands.inspectFontFiles(paths ?? null)),
    installedFontFamilies: (): Promise<InstalledFamily[]> =>
      unwrap(commands.installedFontFamilies()),
    inspectInstalledFonts: (families: string[]): Promise<InspectedFonts> =>
      unwrap(commands.inspectInstalledFonts(families)),
    addInspectedFonts: (token: string, families: string[]): Promise<FontFamilyDef[]> =>
      unwrap(commands.addInspectedFonts(token, families)),
    releaseInspectedFonts: (token: string) => commands.releaseInspectedFonts(token),
    removeFont: (id: string) => unwrap(commands.removeFont(id)),
    fontCatalog: (): Promise<FontCatalogEntry[]> => unwrap(commands.fontCatalog()),
    downloadFont: (id: string): Promise<FontFamilyDef> => unwrap(commands.downloadFont(id)),
    useFontInProject: (dir: string, fontId: string): Promise<FontFamilyDef> =>
      unwrap(commands.useFontInProject(dir, fontId)),
    compile: (
      workflow: Workflow,
      selected: string | null,
      inspected: string | null,
    ): Promise<EditorCompile> => unwrap(commands.compile(workflow, selected, inspected)),
    sendCompiled: () => commands.sendCompiled(),
    onCompiled: (listener: (compiled: EditorCompile) => void) =>
      on(events.compiled(here()), listener),
    relay: (to: RelayTarget, message: unknown) => unwrap(commands.relay(to, message as Value)),
    onRelayed: (listener: (relayed: Relayed) => void) => on(events.relayed(here()), listener),
    attachView: (id: string, listener: (frame: StreamFrame) => void) =>
      unwrap(commands.attachView(id, frameChannel(listener))),
    onViewEvent: (id: string, listener: (event: ViewEvent) => void) =>
      on(events.viewEventFired(here()), (fired) => {
        if (fired.id === id) listener(fired.event);
      }),
    viewInput: (id: string, input: ViewInput) => unwrap(commands.viewInput(id, input)),
    viewResize: (id: string, size: ViewSize) => unwrap(commands.viewResize(id, size)),
    viewNavigate: (id: string, url: string) => unwrap(commands.viewNavigate(id, url)),
    viewGo: (id: string, steps: number) => unwrap(commands.viewGo(id, steps)),
    viewReload: (id: string) => unwrap(commands.viewReload(id)),
    viewSwitchTab: (id: string, tab: string) => unwrap(commands.viewSwitchTab(id, tab)),
    viewAnswerDialog: (id: string, accept: boolean, prompt: string) =>
      unwrap(commands.viewAnswerDialog(id, accept, prompt)),
    viewChooseFiles: (id: string) => unwrap(commands.viewChooseFiles(id)),
    viewClose: (id: string) => commands.viewClose(id),
    previewOpen: (
      projectDir: string,
      size: ViewSize,
      listener: (frame: StreamFrame) => void,
    ): Promise<string> => unwrap(commands.previewOpen(projectDir, size, frameChannel(listener))),
    previewLoad: (spec: TimelineSpec) => unwrap(commands.previewLoad(spec)),
    previewPlay: () => unwrap(commands.previewPlay()),
    previewPause: () => unwrap(commands.previewPause()),
    previewSeek: (frame: number) => unwrap(commands.previewSeek(frame)),
    previewRate: (rate: number) => unwrap(commands.previewRate(rate)),
    previewRange: (range: PlayRange) => unwrap(commands.previewRange(range)),
    previewResize: (size: ViewSize) => unwrap(commands.previewResize(size)),
    previewClose: () => unwrap(commands.previewClose()),
    onPreviewReported: (listener: (reported: PreviewReported) => void) =>
      on(events.previewReported(here()), listener),
  };
}

export type OpsisApi = ReturnType<typeof createOpsis>;

declare global {
  interface Window {
    opsis: OpsisApi;
  }
}
