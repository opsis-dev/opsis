import { Channel } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
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
  type DiskChoice,
  type DockPanelName,
  type DocumentChanged,
  type DocumentState,
  type Edited,
  type EditorCompile,
  type EngineInstallEvent,
  type EngineStatus,
  type EngineTool,
  type EnvStatus,
  type FontCatalogEntry,
  type FontFamilyDef,
  type InspectedFonts,
  type InstalledFamily,
  type LastRun,
  type LicenceEntry,
  type LogLine,
  type LiveEvent,
  type MenuCommand,
  type MenuSection,
  type OpenProject,
  type PickRequest,
  type PickResult,
  type PlaybackCall,
  type Playhead,
  type PlayheadShared,
  type PreviewReported,
  type ProjectFilesChange,
  type ProjectLocations,
  type RecentProject,
  type ReplayRequested,
  type ReplayTab,
  type Removal,
  type RunEvent,
  type RunState,
  type RunView,
  type Selection,
  type SelectionShared,
  type SharedState,
  type SharedView,
  type RunTargets,
  type SettingsPage,
  type TimelineEdit,
  type TimelineReply,
  type TimelineSpec,
  type TraceModel,
  type Value,
  type ViewEvent,
  type ViewInput,
  type ViewPrefChange,
  type ViewShared,
  type ViewSize,
  type WindowRole,
  type WindowState,
  type WorkflowEdit,
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
    documentState: (): Promise<DocumentState | null> => commands.documentState(),
    onDocumentChanged: (listener: (changed: DocumentChanged) => void) =>
      on(events.documentChanged(here()), listener),
    editWorkflow: (edit: WorkflowEdit): Promise<Edited> => unwrap(commands.editWorkflow(edit)),
    editTimeline: (edit: TimelineEdit): Promise<TimelineReply> =>
      unwrap(commands.editTimeline(edit)),
    timelineRemoval: (selected: string[]): Promise<Removal | null> =>
      commands.timelineRemoval(selected),
    undo: () => unwrap(commands.undo()),
    redo: () => unwrap(commands.redo()),
    beginGesture: (): void => void commands.beginGesture(),
    endGesture: (): void => void commands.endGesture(),
    saveProject: (): Promise<boolean> => unwrap(commands.saveProject()),
    resolveDiskChange: (choice: DiskChoice): Promise<boolean> =>
      unwrap(commands.resolveDiskChange(choice)),
    onDiskChangeAsked: (listener: (asked: { readable: boolean }) => void) =>
      on(events.diskChangeAsked(here()), listener),
    onProjectReloaded: (listener: () => void) => on(events.projectReloaded(here()), listener),
    onOpenRequested: (listener: (request: { dir: string }) => void) =>
      on(events.openRequested(here()), listener),
    projectLocations: (): Promise<ProjectLocations> => unwrap(commands.projectLocations()),
    chooseFolder: (opts: { title: string; defaultPath?: string }): Promise<string | null> =>
      unwrap(commands.chooseFolder(opts.title, opts.defaultPath ?? null)),
    watchProjectFiles: (dir: string, refs: readonly string[]) =>
      unwrap(commands.watchProjectFiles(dir, [...refs])),
    onProjectFilesChanged: (listener: (change: ProjectFilesChange) => void) =>
      on(events.projectFilesChanged(here()), listener),
    readLastRun: (dir: string): Promise<LastRun | null> => unwrap(commands.readLastRun(dir)),
    importAssets: (dir: string, paths?: string[]): Promise<string[]> =>
      unwrap(commands.importAssets(dir, paths ?? null)),
    pathForFile: (file: File): string => droppedPaths.get(file.name) ?? "",
    revealInFolder: (dir: string, relativePath?: string) =>
      unwrap(commands.revealInFolder(dir, relativePath ?? null)),
    runWorkflow: (targets: RunTargets): Promise<{ runId: string }> =>
      unwrap(commands.runWorkflow(targets)),
    cancelRun: () => commands.cancelRun(),
    runState: (): Promise<RunState> => commands.runState(),
    clearRunLog: (): void => void commands.clearRunLog(),
    onRunViewChanged: (listener: (view: RunView) => void) =>
      on(events.runViewChanged(here()), listener),
    onRunLogged: (listener: (line: LogLine) => void) => on(events.runLogged(here()), listener),
    onRunStateReset: (listener: (state: RunState) => void) =>
      on(events.runStateReset(here()), listener),
    onRunEvent: (listener: (event: RunEvent) => void) => on(events.runEventFired(here()), listener),
    pickElement: (request: PickRequest): Promise<PickResult | null> =>
      unwrap(commands.pickElement(request)),
    getEnvironment: (): Promise<EnvStatus> => commands.getEnvironment(),
    installEngine: () => unwrap(commands.installEngine()),
    onEngineInstall: (listener: (event: EngineInstallEvent) => void) =>
      on(events.engineInstallProgress(here()), listener),
    chooseEngineTool: (tool: EngineTool): Promise<boolean> =>
      unwrap(commands.chooseEngineTool(tool)),
    onEngineChanged: (listener: (status: EngineStatus) => void) =>
      on(events.engineChanged(here()), listener),
    getSettings: (): Promise<AppSettings> => commands.getSettings(),
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
    openPreviewWindow: () => unwrap(commands.openPreviewWindow()),
    focusPanelWindow: (panel: DockPanelName) => commands.focusPanelWindow(panel),
    closePanelWindow: (panel: DockPanelName) => unwrap(commands.closePanelWindow(panel)),
    closePanelWindows: () => unwrap(commands.closePanelWindows()),
    listPanelWindows: (): Promise<DockPanelName[]> => commands.listPanelWindows(),
    onPanelWindows: (listener: (open: DockPanelName[]) => void) =>
      on(events.panelWindowsChanged(here()), listener),
    reportDrafts: (held: boolean): void => void commands.reportDrafts(held),
    onFlushSave: (listener: () => Promise<void>) =>
      on(events.flushSave(here()), () => {
        void listener()
          .catch((e: unknown) => console.error("The values being typed couldn't be kept.", e))
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
    setCompileFocus: (selected: string | null, inspected: string | null): void =>
      void commands.setCompileFocus(selected, inspected),
    latestCompile: (): Promise<EditorCompile | null> => unwrap(commands.latestCompile()),
    onCompiled: (listener: (compiled: EditorCompile) => void) =>
      on(events.compiled(here()), listener),
    windowLabel: here().label,
    sharedState: (): Promise<SharedState> => commands.sharedState(),
    shareSelection: (selection: Selection): void => void commands.shareSelection(selection),
    shareView: (view: SharedView): void => void commands.shareView(view),
    sharePlayhead: (playhead: Playhead): void => void commands.sharePlayhead(playhead),
    onSelectionShared: (listener: (shared: SelectionShared) => void) =>
      on(events.selectionShared(here()), listener),
    onViewShared: (listener: (shared: ViewShared) => void) =>
      on(events.viewShared(here()), listener),
    onPlayheadShared: (listener: (shared: PlayheadShared) => void) =>
      on(events.playheadShared(here()), listener),
    controlPlayback: (call: PlaybackCall): void => void commands.controlPlayback(call),
    onPlaybackCalled: (listener: (call: PlaybackCall) => void) =>
      on(events.playbackCalled(here()), listener),
    showReplay: (nodeId: string | null, tab: ReplayTab): void =>
      void commands.showReplay(nodeId, tab),
    onReplayRequested: (listener: (request: ReplayRequested) => void) =>
      on(events.replayRequested(here()), listener),
    viewState: (dir: string): Promise<Partial<Record<string, unknown>>> =>
      unwrap(commands.viewState(dir)),
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
