import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";
import * as __TAURI_EVENT from "@tauri-apps/api/event";

/** Commands */
export const commands = {
	listRecentProjects: () => typedError<RecentProject[], CommandError>(__TAURI_INVOKE("list_recent_projects")),
	projectLocations: () => typedError<ProjectLocations, CommandError>(__TAURI_INVOKE("project_locations")),
	chooseFolder: (title: string, defaultPath: string | null) => typedError<string | null, CommandError>(__TAURI_INVOKE("choose_folder", { title, defaultPath })),
	pickProjectDir: () => typedError<string | null, CommandError>(__TAURI_INVOKE("pick_project_dir")),
	openProject: (dir: string) => typedError<OpenProject, CommandError>(__TAURI_INVOKE("open_project", { dir })),
	projectFolder: (parent: string, name: string) => typedError<string, CommandError>(__TAURI_INVOKE("project_folder", { parent, name })),
	createProject: (name: string, template: string, dir: string | null) => typedError<OpenProject | null, CommandError>(__TAURI_INVOKE("create_project", { name, template, dir })),
	saveWorkflow: (dir: string, workflow: Workflow, base: string | null) => typedError<SaveResult, CommandError>(__TAURI_INVOKE("save_workflow", { dir, workflow, base })),
	watchProject: (dir: string | null) => typedError<null, CommandError>(__TAURI_INVOKE("watch_project", { dir })),
	watchProjectFiles: (dir: string, refs: string[]) => typedError<null, CommandError>(__TAURI_INVOKE("watch_project_files", { dir, refs })),
	readLastRun: (dir: string) => typedError<LastRun | null, CommandError>(__TAURI_INVOKE("read_last_run", { dir })),
	importAssets: (dir: string, paths: string[] | null) => typedError<string[], CommandError>(__TAURI_INVOKE("import_assets", { dir, paths })),
	clearCache: (dir: string) => typedError<null, CommandError>(__TAURI_INVOKE("clear_cache", { dir })),
	listTemplates: () => __TAURI_INVOKE<TemplateInfo[]>("list_templates"),
	getSettings: () => typedError<AppSettings, CommandError>(__TAURI_INVOKE("get_settings")),
	patchSettings: (patch: { [key in string]: Value }) => typedError<AppSettings, CommandError>(__TAURI_INVOKE("patch_settings", { patch })),
	setViewPref: (dir: string, key: string, value: Value) => typedError<null, CommandError>(__TAURI_INVOKE("set_view_pref", { dir, key, value })),
	appMenuSections: () => __TAURI_INVOKE<MenuSection[]>("app_menu_sections"),
	getEnvironment: () => __TAURI_INVOKE<EnvStatus>("get_environment"),
	installEngine: () => typedError<null, CommandError>(__TAURI_INVOKE("install_engine")),
	listLicences: () => typedError<LicenceEntry[], CommandError>(__TAURI_INVOKE("list_licences")),
	openLicenceFile: (file: string) => typedError<null, CommandError>(__TAURI_INVOKE("open_licence_file", { file })),
	revealInFolder: (dir: string, relativePath: string | null) => typedError<null, CommandError>(__TAURI_INVOKE("reveal_in_folder", { dir, relativePath })),
	readTrace: (dir: string, traceId: string) => typedError<TraceModel, CommandError>(__TAURI_INVOKE("read_trace", { dir, traceId })),
	listSecrets: (dir: string) => typedError<string[], CommandError>(__TAURI_INVOKE("list_secrets", { dir })),
	openSettings: (page: SettingsPage | null) => typedError<null, CommandError>(__TAURI_INVOKE("open_settings", { page })),
	openPanelWindow: (panel: DockPanelName) => typedError<null, CommandError>(__TAURI_INVOKE("open_panel_window", { panel })),
	focusPanelWindow: (panel: DockPanelName) => __TAURI_INVOKE<void>("focus_panel_window", { panel }),
	closePanelWindow: (panel: DockPanelName) => typedError<null, CommandError>(__TAURI_INVOKE("close_panel_window", { panel })),
	closePanelWindows: () => typedError<null, CommandError>(__TAURI_INVOKE("close_panel_windows")),
	listPanelWindows: () => __TAURI_INVOKE<DockPanelName[]>("list_panel_windows"),
	getWindowState: () => __TAURI_INVOKE<WindowState>("get_window_state"),
	setTitle: (title: string) => typedError<null, CommandError>(__TAURI_INVOKE("set_title", { title })),
	windowRole: (role: WindowRole) => typedError<null, CommandError>(__TAURI_INVOKE("window_role", { role })),
	reportUnsaved: (unsaved: boolean) => __TAURI_INVOKE<void>("report_unsaved", { unsaved }),
	closeShown: () => __TAURI_INVOKE<void>("close_shown"),
	decideClose: (decision: CloseDecision) => typedError<null, CommandError>(__TAURI_INVOKE("decide_close", { decision })),
	saveFlushed: () => __TAURI_INVOKE<void>("save_flushed"),
	compile: (workflow: Workflow, selected: string | null, inspected: string | null) => typedError<EditorCompile, CommandError>(__TAURI_INVOKE("compile", { workflow, selected, inspected })),
	sendCompiled: () => __TAURI_INVOKE<void>("send_compiled"),
	editorCatalog: () => __TAURI_INVOKE<EditorCatalog>("editor_catalog"),
	fileRoot: () => __TAURI_INVOKE<string>("file_root"),
	runWorkflow: (request: RunRequest) => typedError<StartedRun, CommandError>(__TAURI_INVOKE("run_workflow", { request })),
	cancelRun: () => __TAURI_INVOKE<void>("cancel_run"),
	runStatus: () => __TAURI_INVOKE<RunStatus>("run_status"),
	watchLive: (frames: Channel<ArrayBuffer>) => __TAURI_INVOKE<void>("watch_live", { frames }),
	listFonts: () => typedError<FontFamilyDef[], CommandError>(__TAURI_INVOKE("list_fonts")),
	inspectFontFiles: (paths: string[] | null) => typedError<InspectedFonts | null, CommandError>(__TAURI_INVOKE("inspect_font_files", { paths })),
	installedFontFamilies: () => typedError<InstalledFamily[], CommandError>(__TAURI_INVOKE("installed_font_families")),
	inspectInstalledFonts: (families: string[]) => typedError<InspectedFonts, CommandError>(__TAURI_INVOKE("inspect_installed_fonts", { families })),
	addInspectedFonts: (token: string, families: string[]) => typedError<FontFamilyDef[], CommandError>(__TAURI_INVOKE("add_inspected_fonts", { token, families })),
	releaseInspectedFonts: (token: string) => __TAURI_INVOKE<void>("release_inspected_fonts", { token }),
	removeFont: (id: string) => typedError<null, CommandError>(__TAURI_INVOKE("remove_font", { id })),
	fontCatalog: () => typedError<FontCatalogEntry[], CommandError>(__TAURI_INVOKE("font_catalog")),
	downloadFont: (id: string) => typedError<FontFamilyDef, CommandError>(__TAURI_INVOKE("download_font", { id })),
	useFontInProject: (dir: string, fontId: string) => typedError<FontFamilyDef, CommandError>(__TAURI_INVOKE("use_font_in_project", { dir, fontId })),
	analyze: (request: AnalysisRequest) => typedError<AnalysisResult, CommandError>(__TAURI_INVOKE("analyze", { request })),
	pickElement: (request: PickRequest) => typedError<PickResult | null, CommandError>(__TAURI_INVOKE("pick_element", { request })),
	attachView: (id: string, frames: Channel<ArrayBuffer>) => typedError<null, CommandError>(__TAURI_INVOKE("attach_view", { id, frames })),
	viewInput: (id: string, input: ViewInput) => typedError<null, CommandError>(__TAURI_INVOKE("view_input", { id, input })),
	viewResize: (id: string, size: ViewSize) => typedError<null, CommandError>(__TAURI_INVOKE("view_resize", { id, size })),
	viewNavigate: (id: string, url: string) => typedError<null, CommandError>(__TAURI_INVOKE("view_navigate", { id, url })),
	viewGo: (id: string, steps: number) => typedError<null, CommandError>(__TAURI_INVOKE("view_go", { id, steps })),
	viewReload: (id: string) => typedError<null, CommandError>(__TAURI_INVOKE("view_reload", { id })),
	viewSwitchTab: (id: string, tab: string) => typedError<null, CommandError>(__TAURI_INVOKE("view_switch_tab", { id, tab })),
	viewAnswerDialog: (id: string, accept: boolean, prompt: string) => typedError<null, CommandError>(__TAURI_INVOKE("view_answer_dialog", { id, accept, prompt })),
	viewChooseFiles: (id: string) => typedError<null, CommandError>(__TAURI_INVOKE("view_choose_files", { id })),
	viewClose: (id: string) => __TAURI_INVOKE<void>("view_close", { id }),
	previewOpen: (projectDir: string, size: ViewSize, frames: Channel<ArrayBuffer>) => typedError<string, CommandError>(__TAURI_INVOKE("preview_open", { projectDir, size, frames })),
	previewLoad: (spec: TimelineSpec) => typedError<null, CommandError>(__TAURI_INVOKE("preview_load", { spec })),
	previewPlay: () => typedError<null, CommandError>(__TAURI_INVOKE("preview_play")),
	previewPause: () => typedError<null, CommandError>(__TAURI_INVOKE("preview_pause")),
	previewSeek: (frame: number) => typedError<null, CommandError>(__TAURI_INVOKE("preview_seek", { frame })),
	previewRate: (rate: number) => typedError<null, CommandError>(__TAURI_INVOKE("preview_rate", { rate })),
	previewRange: (range: PlayRange) => typedError<null, CommandError>(__TAURI_INVOKE("preview_range", { range })),
	previewResize: (size: ViewSize) => typedError<null, CommandError>(__TAURI_INVOKE("preview_resize", { size })),
	previewClose: () => typedError<null, CommandError>(__TAURI_INVOKE("preview_close")),
	relay: (to: RelayTarget, message: Value) => typedError<null, CommandError>(__TAURI_INVOKE("relay", { to, message })),
	pageLog: (level: PageLogLevel, message: string) => __TAURI_INVOKE<void>("page_log", { level, message }),
	sendMenuToEditor: (command: MenuCommand) => typedError<null, CommandError>(__TAURI_INVOKE("send_menu_to_editor", { command })),
	apiStatus: () => __TAURI_INVOKE<ApiServerStatus>("api_status"),
	apiListening: (listening: boolean) => __TAURI_INVOKE<void>("api_listening", { listening }),
	apiReply: (id: string, reply: Value) => __TAURI_INVOKE<void>("api_reply", { id, reply }),
};

/** Events */
export const events = {
	apiRequestReceived: makeEvent<ApiRequestReceived, ApiRequestReceived>("api-request-received"),
	apiStatusChanged: makeEvent<ApiStatusChanged, ApiStatusChanged>("api-status-changed"),
	compiled: makeEvent<Compiled, Compiled>("compiled"),
	confirmClose: makeEvent<ConfirmClose, ConfirmClose>("confirm-close"),
	engineInstallProgress: makeEvent<EngineInstallProgress, EngineInstallProgress>("engine-install-progress"),
	flushSave: makeEvent<FlushSave, FlushSave>("flush-save"),
	liveNoted: makeEvent<LiveNoted, LiveNoted>("live-noted"),
	menuCommandFired: makeEvent<MenuCommandFired, MenuCommandFired>("menu-command-fired"),
	panelWindowsChanged: makeEvent<PanelWindowsChanged, PanelWindowsChanged>("panel-windows-changed"),
	previewReported: makeEvent<PreviewReported, PreviewReported>("preview-reported"),
	projectFileChanged: makeEvent<ProjectFileChanged, ProjectFileChanged>("project-file-changed"),
	projectFilesChanged: makeEvent<ProjectFilesChanged, ProjectFilesChanged>("project-files-changed"),
	relayed: makeEvent<Relayed, Relayed>("relayed"),
	runEventFired: makeEvent<RunEventFired, RunEventFired>("run-event-fired"),
	settingsChanged: makeEvent<SettingsChanged, SettingsChanged>("settings-changed"),
	settingsPageShown: makeEvent<SettingsPageShown, SettingsPageShown>("settings-page-shown"),
	viewEventFired: makeEvent<ViewEventFired, ViewEventFired>("view-event-fired"),
	viewPrefChanged: makeEvent<ViewPrefChanged, ViewPrefChanged>("view-pref-changed"),
	windowStateChanged: makeEvent<WindowStateChanged, WindowStateChanged>("window-state-changed"),
};

/* Types */

export type ActionKind = "click" | "type" | "hover" | "scroll" | "navigate" | "wait" | "chapter" | "key" | "focus" | "drag";

export type ActionLog = {
	viewport: Viewport,
	dpr: number,
	t0: number,
	durationMs: number,
	actions: Action[],
	cursorTrack?: CursorSample[],
};

export type ActionMark = {
	key: string,
	frame: number,
	kind: ActionKind,
	label: string,
	nodeId?: string,
	barKey: string,
};

export type ActionRef = {
	id: string,
	label: string,
};

export type Action = {
	id: string,
	kind: ActionKind,
	t: number,
	tEnd?: number,
	tDepart?: number,
	tDown?: number,
	point?: Point,
	rect?: Rect,
	cluster?: string,
	label?: string,
	selectorUsed?: string,
	zoom?: boolean,
	zoomScale?: number,
	text?: string,
	nodeId?: string,
	path?: Point[],
	drop?: Rect,
};

export type AddEdgeBody = {
	source: string,
	sourceHandle: string,
	target: string,
	targetHandle: string,
};

export type AddNodeBody = {
	type: string,
	id?: string,
	label?: string,
	position?: Position,
	params?: { [key in string]: Value },
};

export type AnalysisRequest = { kind: "waveform"; projectDir: string; src: string } | { kind: "filmstrip"; projectDir: string; src: string } | { kind: "mix"; projectDir: string; plan: MixPlan };

export type AnalysisResult = { kind: "waveform"; waveform: Waveform } | { kind: "filmstrip"; filmstrip: Filmstrip } | { kind: "mix"; mix: MixResult } | { kind: "skipped" } | { kind: "error"; error: string; engineMissing?: boolean };

export type Anchor = {
	nodeId: string,
};

export type AnnotationKind = "highlight" | "spotlight" | "tooltip" | "callout";

export type AnnotationSpec = {
	kind: AnnotationKind,
	look?: HighlightLook,
	roughness?: number,
	match: string,
	text: string,
	color: string,
	padding: number,
	leadMs: number,
	holdMs: number,
	textColor?: string,
	fontSize?: number,
	font?: string,
};

export type ApiProblem = {
	path?: string,
} & GraphIssue;

export type ApiRequestReceived = {
	id: string,
	request: EditorRequest,
};

export type ApiSection = {
	enabled: boolean,
};

export type ApiServerStatus = {
	enabled: boolean,
	listening: boolean,
	address: string,
	error?: string,
};

export type ApiStatusChanged = ApiServerStatus;

export type AppSection = {
	onStart: StartOption,
	confirmExit: boolean,
	setupSkipped: boolean,
};

export type AppSettings = {
	app: AppSection,
	traces: TracesSection,
	live: LiveSection,
	render: RenderSection,
	projects: ProjectsSection,
	editor: EditorSection,
	api: ApiSection,
};

export type AppVersions = {
	app: string,
	tauri: string,
	webview: string,
};

export type ArtifactKind = "video" | "image" | "audio" | "json" | "file";

export type Artifact = {
	kind: ArtifactKind,
	path: string,
	label?: string,
	bytes?: number,
	stamp?: string,
};

export type AudioBar = {
	type: AudioTag,
	key: string,
	id: string,
	nodeId?: string,
	channel: string,
	src: string,
	role: ChannelKind,
	from: number,
	to: number,
	lane: number,
	trimBefore: number,
	loop: boolean,
	volume: number,
	handles?: EditHandles,
	item?: AudioItem,
	bar?: VideoBar,
};

export type AudioEnvelope = {
	volume?: Keyframe[],
	pan?: Keyframe[],
};

export type AudioItem = {
	id: string,
	src: string,
	from: number,
	durationInFrames?: number,
	volume?: number,
	trimBefore?: number,
	loop?: boolean,
	rate?: number,
	fadeInFrames?: number,
	fadeOutFrames?: number,
	nodeId?: string,
	channel?: string,
	layerTrack?: string,
	role?: AudioRole,
	lengthFrames?: number,
	keepMusic?: boolean,
	gain?: number,
	duck?: Duck,
	envelope?: AudioEnvelope,
	handles?: EditHandles,
	anchor?: Anchor,
};

export type AudioRole = "music" | "effect";

export type AudioTag = "audio";

export type AudioValue = {
	items: AudioItem[],
	durationInFrames?: number,
};

export type BarClock = {
	steps: ClockStep[],
	itemFrom: number,
	contentEnd: number,
};

export type BarKind = "recording" | "element";

export type BarThumbnail = { kind: "filmstrip"; src: string; speed: number; trimMs: number } | { kind: "still"; src: string };

export type BlurArea = {
	x: number,
	y: number,
	width: number,
	height: number,
	strength: number,
	softness: number,
	roundness: number,
	fromMs?: number,
	untilMs?: number,
};

export type Bounds = {
	x: number,
	y: number,
	width: number,
	height: number,
};

export type BrowserSessionValue = {
	browser: SessionBrowser,
	steps: SessionStep[],
};

export type BusDuck = {
	keys: string[],
	amount: number,
	attack: number,
	release: number,
};

export type BusPlan = {
	id: string,
	name: string,
	processing: BusProcessing,
	fader: number,
	channels: string[],
	duck: BusDuck | null,
};

export type BusProcessing = {
	highpass: Highpass,
	eq: Eq,
	comp: Comp,
};

export type Bus = {
	id: string,
	name: string,
	highpass: Highpass,
	eq: Eq,
	comp: Comp,
	fader: number,
	mute: boolean,
	solo: boolean,
	duck: DuckSettings,
};

export type CameraKeyMark = {
	key: string,
	itemId: string,
	nodeId: string,
	index: number,
	frame: number,
};

export type CameraKey = {
	frame: number,
	yaw: number,
	pitch: number,
	distance: number,
	height: number,
	ease: Ease,
};

export type CameraPath = {
	keys: CameraKey[],
};

export type CameraStyle = "classic" | "proof" | "ledger" | "stage";

export type CameraTuning = {
	sMax?: number,
	holdMinS?: number,
};

export type CanvasMode = "mouse" | "touchpad";

export type Caption = {
	text: string,
	startMs: number,
	endMs: number,
};

export type CaptionsValue = {
	captions: Caption[],
};

export type CardRow = {
	icon: Icon,
	text: string,
	title?: string,
	tone?: Tone,
	swatches?: string[],
};

export type CatalogCategory = "sans-serif" | "serif" | "monospace" | "display" | "handwriting" | "icons" | "other";

export type CatalogEntry = {
	params: { [key in string]: unknown },
} & CatalogNode;

export type CatalogNode = {
	type: string,
	title: string,
	category: string,
	description: string,
	runs: boolean,
	inputs: { [key in string]: CatalogPort },
	outputs: { [key in string]: CatalogPort },
};

export type CatalogPort = {
	type: PortType,
	label: string,
	optional: boolean,
	many: boolean,
};

export type Category = "Browser" | "Capture" | "Camera" | "Cursor" | "Text & overlays" | "Transitions" | "Backgrounds" | "Effects" | "Grade" | "Elements" | "Audio" | "Timeline" | "Media" | "Image" | "Output" | "Utility";

export type ChainLink = {
	itemId: string,
	nodeId?: string,
	handles?: EditHandles,
	slot?: Slot,
	rate: number,
	rateEnd: number,
	end: number,
	from: number,
	to: number,
};

export type ChannelKind = "music" | "effect" | "clip";

export type ChannelPlan = {
	id: string,
	kind: ChannelKind,
	bus: string,
	strip: Strip,
	sounds: SoundPlacement[],
	audible: boolean,
};

export type CheckResult = {
	ok: boolean,
	problems: ApiProblem[],
	workflow: Workflow | null,
};

export type ClipValue = {
	durationInFrames: number,
	tracks: TimelineTrack[],
	audio: AudioItem[],
	recording?: TaggedRecording,
};

export type ClockStep = {
	from: number,
	startFrom: number,
	loopOffset: number,
	remap: ([number, number])[] | null,
};

export type CloseDecision = {
	close: boolean,
	dontAsk: boolean,
};

export type CloseRequest = {
	ask: boolean,
	busy: string[],
};

export type CommandError = string;

export type Comp = {
	on: boolean,
	threshold: number,
	ratio: number,
	attack: number,
	release: number,
	makeup: number,
};

export type CompileResult = {
	values: { [key in string]: { [key in string]: CompileValue } },
	errors: { [key in string]: string },
	specs: { [key in string]: TimelineSpec },
	order: string[],
};

export type CompileValue = {
	type: "browser-session",
} & BrowserSessionValue | {
	type: "recording",
} & RecordingValue | {
	type: "recording-style",
} & EntriesValue | {
	type: "grade",
} & EntriesValue | {
	type: "image",
} & ImageValue | {
	type: "media",
} & MediaValue | {
	type: "file-info",
} & SrcValue | {
	type: "folder",
} & SrcValue | {
	type: "clip",
} & ClipValue | {
	type: "audio",
} & AudioValue | {
	type: "captions",
} & CaptionsValue;

export type Compiled = EditorCompile;

export type ConfirmClose = CloseRequest;

export type ConnectionProblem = {
	notFound: boolean,
	message: string,
};

export type ConsoleLevel = "log" | "info" | "warning" | "error" | "debug";

export type CursorLook = { shape: "arrow"; fill: string; edge: string } | { shape: "mac" };

export type CursorSample = {
	t: number,
	x: number,
	y: number,
	c?: string,
};

export type Design = {
	width: number,
	height: number,
	exact?: boolean,
};

export type DockPanelName = "preview" | "timeline" | "mixer" | "run";

export type Duck = {
	level: number,
	fadeFrames: number,
	keys: string[],
};

export type DuckSettings = {
	on: boolean,
	keys: string[],
	amount: number,
	attack: number,
	release: number,
	ignoreShorterMs: number,
};

export type Ease = "linear" | "in" | "out" | "in-out" | "hold";

export type EditHandles = {
	move?: ParamRef,
	trimStart?: ParamRef,
	trimEnd?: ParamRef,
	speed?: ParamRef,
	volume?: ParamRef,
};

export type Edition = "full" | "lite" | "dev";

export type EditorCatalog = {
	nodes: EditorNode[],
	transitions: { [key in string]: string },
	connects: PortLink[],
};

export type EditorCompile = {
	seq: number,
	result: CompileResult,
	issues: GraphIssue[],
	preview: PreviewTarget | null,
	flat: FlatTimeline | null,
	plan: MixPlan | null,
	levelled: TimelineSpec | null,
	cards: { [key in string]: CardRow[] },
	inspector: InspectorContext | null,
	clocks: { [key in string]: BarClock },
};

export type EditorNode = {
	type: string,
	title: string,
	category: Category,
	categoryLabel: string,
	description?: string,
	phase: Phase,
	inputs: { [key in string]: CatalogPort },
	outputs: { [key in string]: CatalogPort },
	params: { [key in string]: unknown },
	defaults: { [key in string]: unknown },
	hostDefaults: { [key in string]: unknown },
	virtual: boolean,
	action: boolean,
	executes: boolean,
	hidden: boolean,
	secretParams: string[],
	usesProjectSize: boolean,
	element?: string,
	cardHeight: number,
	hasCard: boolean,
	compiles: boolean,
};

export type EditorRequest = ({ op: "open"; dir: string }) & { ids?: never; settings?: never; variables?: never; workflow?: never } | ({ op: "read" }) & { dir?: never; ids?: never; settings?: never; variables?: never; workflow?: never } | ({ op: "replace"; workflow: Workflow }) & { dir?: never; ids?: never; settings?: never; variables?: never } | {
	op: "addNode",
} & AddNodeBody | {
	op: "updateNode",
} & {
	id: string,
} & UpdateNodeBody | ({ op: "removeNodes"; ids: string[] }) & { dir?: never; settings?: never; variables?: never; workflow?: never } | {
	op: "addEdge",
} & AddEdgeBody | ({ op: "removeEdges"; ids: string[] }) & { dir?: never; settings?: never; variables?: never; workflow?: never } | ({ op: "setSettings"; settings: SettingsPatch }) & { dir?: never; ids?: never; variables?: never; workflow?: never } | ({ op: "setVariables"; variables: { [key in string]: string } }) & { dir?: never; ids?: never; settings?: never; workflow?: never } | ({ op: "problems" }) & { dir?: never; ids?: never; settings?: never; variables?: never; workflow?: never } | ({ op: "undo" }) & { dir?: never; ids?: never; settings?: never; variables?: never; workflow?: never } | ({ op: "redo" }) & { dir?: never; ids?: never; settings?: never; variables?: never; workflow?: never } | ({ op: "save" }) & { dir?: never; ids?: never; settings?: never; variables?: never; workflow?: never } | {
	op: "run",
} & RunBody | ({ op: "stop" }) & { dir?: never; ids?: never; settings?: never; variables?: never; workflow?: never };

export type EditorSection = {
	canvasMode: CanvasMode,
};

export type ElementRef = {
	component: string,
	props: { [key in string]: unknown },
};

export type EngineInstallEvent = {
	done?: boolean,
	error?: string,
} & EngineInstallUpdate;

export type EngineInstallProgress = EngineInstallEvent;

export type EngineInstallUpdate = {
	part: EnginePart,
	step: string,
	progress: number,
	message: string,
};

export type EnginePart = "ffmpeg" | "headless-shell";

export type EngineStatus = {
	dir: string,
	headlessShell: string | null,
	ffmpeg: string | null,
	ffprobe: string | null,
	complete: boolean,
};

export type EntriesValue = {
	entries: StyleEntry[],
};

export type EnvStatus = {
	platform: string,
	arch: string,
	isPackaged: boolean,
	edition: Edition,
	userData: string,
	versions: AppVersions,
	engine: EngineStatus,
};

export type EqBand = {
	type: EqType,
	freq: number,
	gain: number,
	q: number,
};

export type EqType = "lowshelf" | "peak" | "highshelf";

export type Eq = {
	on: boolean,
	bands: EqBand[],
};

export type FilmChapter = {
	title: string,
	startMs: number,
	endMs: number,
};

export type Filmstrip = {
	src: string,
	image: string,
	count: number,
	intervalMs: number,
	tileWidth: number,
	tileHeight: number,
	durationMs: number,
};

export type FlatTimeline = {
	fps: number,
	durationInFrames: number,
	bars: VideoBar[],
	lanes: number,
	audio: AudioBar[],
	audioLanes: number,
	transitions: TransitionMark[],
	marks: ActionMark[],
	cameraKeys: CameraKeyMark[],
};

export type FlushSave = null;

export type FontCandidate = {
	name: string,
	family: string,
	weight: string,
	style: FontStyle,
	variable: boolean,
	format: FontFormat,
	size: number,
};

export type FontCatalogEntry = {
	id: string,
	family: string,
	category: CatalogCategory,
	variable: boolean,
	styles: string[],
	weights: number[],
	license: string,
	type: string,
};

export type FontCategory = "sans-serif" | "serif" | "monospace" | "display" | "handwriting";

export type FontFaceDef = {
	file: string,
	weight: string,
	style: FontStyle,
	format: FontFormat,
};

export type FontFamilyDef = {
	id: string,
	family: string,
	category: FontCategory,
	source: FontSource,
	license?: string,
	licenseFile?: string,
	faces: FontFaceDef[],
};

export type FontFormat = "woff2" | "woff" | "truetype" | "opentype";

export type FontSource = "bundled" | "online" | "file" | "installed";

export type FontStyle = "normal" | "italic";

export type FrameRange = {
	from: number,
	to: number,
};

export type FrameTheme = "light" | "dark";

export type GraphIssue = {
	level: IssueLevel,
	message: string,
	nodeId?: string,
	edgeId?: string,
};

export type GroupProps = {
	tracks: TimelineTrack[],
	startFrom?: number,
	remap?: ([number, number])[],
	loopEvery?: number,
	crop?: Rect,
	wrapper?: ElementRef,
	background?: string,
	design?: Viewport,
	sequencer?: SequencerProps,
};

export type HighlightLook = "clean" | "box" | "circle" | "underline" | "marker";

export type Highpass = {
	on: boolean,
	freq: number,
};

export type Icon = "app-window" | "arrow-right-left" | "audio-lines" | "blend" | "bookmark" | "captions" | "check" | "circle-check" | "circle-dot" | "clapperboard" | "clock" | "code" | "code-xml" | "columns-2" | "combine" | "command" | "crop" | "crosshair" | "eye" | "eye-off" | "fast-forward" | "file" | "file-play" | "file-video" | "film" | "flashlight" | "folder-output" | "gauge" | "git-branch" | "globe" | "grab" | "hard-drive" | "hourglass" | "image" | "image-play" | "info" | "key-round" | "keyboard" | "languages" | "laptop" | "layers" | "link" | "list" | "list-checks" | "list-video" | "locate-fixed" | "map" | "maximize" | "message-square" | "monitor" | "moon" | "mouse-pointer" | "mouse-pointer-2" | "mouse-pointer-click" | "move" | "move-up-right" | "music" | "palette" | "presentation" | "ratio" | "refresh-cw" | "repeat" | "reply" | "rocket" | "ruler" | "scan" | "scan-search" | "scissors" | "scroll" | "scroll-text" | "server" | "settings-2" | "shapes" | "shield-alert" | "shield-check" | "smartphone" | "sparkles" | "spline" | "square-dashed" | "sticky-note" | "sun" | "timer" | "timer-off" | "triangle-alert" | "type" | "upload" | "video" | "volume-2" | "volume-x" | "wallpaper" | "wifi" | "wind" | "zoom-in" | "zoom-out";

export type ImageValue = {
	src: string | null,
	width?: number,
	height?: number,
	scale?: number,
	body?: Bounds,
};

export type InspectedFonts = {
	token: string,
	families: PlannedFamily[],
	license: string | null,
	skipped: number,
	unreadable: number,
};

export type InspectorContext = {
	nodeId: string,
	vars: VarInfo[],
	layers: number,
	actions: ActionRef[],
	spotBackground: SpotBackground | null,
	picker: PickerTarget,
	swatches: string[],
	connectedInputs: string[],
};

export type InstalledFamily = {
	family: string,
	faces: number,
};

export type IssueLevel = "error" | "warning";

export type ItemBase = {
	id: string,
	from: number,
	durationInFrames: number,
	enter?: TransitionRef,
	exit?: TransitionRef,
	nodeId?: string,
	handles?: EditHandles,
	slot?: Slot,
	envelope?: VisualEnvelope,
	camera?: CameraPath,
	design?: Design,
};

export type Keyframe = {
	frame: number,
	value: number,
	ease: Ease,
};

export type KeystrokeLook = {
	position: KeystrokePosition,
	typed: boolean,
	chords: boolean,
	keycap: string,
	ink: string,
	size: number,
};

export type KeystrokePosition = "bottom" | "top";

export type LastRun = {
	updatedAt: number,
	nodes: { [key in string]: NodeResult },
	failures?: { [key in string]: NodeFailure },
};

export type LicenceEntry = {
	id: string,
	title: string,
	summary: string,
	text?: string,
	file?: string,
};

export type Limiter = {
	on: boolean,
	ceiling: number,
};

export type LinkedParam = {
	param: string,
	sign: 1 | -1,
	ifNonZero?: boolean,
};

export type LiveEvent = { kind: "start"; title: string; viewport: Viewport; steps: LiveStep[] } | { kind: "step"; nodeId: string; status: LiveStepStatus; message?: string } | { kind: "cursor"; x: number; y: number } | { kind: "target"; rect: Rect; label: string; selector?: string } | { kind: "click"; x: number; y: number } | { kind: "end"; ok: boolean; message?: string };

export type LiveNoted = {
	captureId: string,
	nodeId: string,
	t: number,
	event: LiveEvent,
};

export type LiveSection = {
	fps: number,
	autoOpen: boolean,
};

export type LiveStep = {
	nodeId: string,
	label: string,
};

export type LiveStepStatus = "running" | "done" | "error";

export type LogLevel = "debug" | "info" | "warn" | "error";

export type LoudnessTarget = "off" | "web" | "podcast" | "broadcast";

export type Loudness = {
	fps: number,
	momentary: (number)[],
	shortTerm: (number)[],
	integrated: number,
	range: number,
	truePeak: number,
};

export type Marker = {
	id: string,
	atMs: number,
	label: string,
};

export type MasterStem = {
	src: string,
	hash: string,
};

export type Master = {
	fader: number,
	eq: Eq,
	comp: Comp,
	limiter: Limiter,
	loudness: LoudnessTarget,
};

export type MediaKind = "video" | "audio" | "image";

export type MediaValue = {
	src: string | null,
	kind: MediaKind,
	durationMs?: number,
	width?: number,
	height?: number,
	hasAudio?: boolean,
};

export type MenuCommand = "project:new" | "project:open" | "project:save" | "edit:undo" | "edit:redo" | "edit:group" | "edit:select-all" | "edit:layout" | "edit:add-node" | "run:start" | "run:cancel" | "view:settings" | "view:projects" | "view:toggle-left" | "view:toggle-bottom" | "view:toggle-right" | "view:toggle-minimap" | "view:focus-preview" | "view:focus-timeline" | "view:focus-mixer" | "view:focus-run";

export type MenuCommandFired = MenuCommand;

export type MenuEntry = { kind: "command"; label: string; command: MenuCommand; accelerator?: string } | { kind: "role"; label: string; role: WindowRole; accelerator?: string } | { kind: "separator" };

export type MenuSection = {
	label: string,
	items: MenuEntry[],
};

export type MixLevels = {
	channels: { [key in string]: StemLevels },
	buses: { [key in string]: StemLevels },
	master: StemLevels | null,
};

export type MixPlan = {
	fps: number,
	durationInFrames: number,
	channels: ChannelPlan[],
	buses: BusPlan[],
	master: Master,
	key: string,
};

export type MixReduction = {
	channels: { [key in string]: (number)[] },
	master: (number)[] | null,
};

export type MixResult = {
	key: string,
	master: MasterStem | null,
	levels: MixLevels,
	reduction: MixReduction,
	loudness: Loudness | null,
};

export type Mix = {
	channels: { [key in string]: Strip },
	buses: Bus[],
	master: Master,
};

export type MouseButton = "none" | "left" | "middle" | "right" | "back" | "forward";

export type NodeFailure = {
	message: string,
	at: number,
	traceId?: string,
};

export type NodeProgress = {
	value: number,
	message?: string,
};

export type NodeResult = {
	hash: string,
	outputs: { [key in string]: CompileValue },
	artifacts: Artifact[],
	finishedAt: number,
	durationMs: number,
	traceId?: string,
};

export type NodeRunStatus = "idle" | "queued" | "running" | "cached" | "done" | "error" | "skipped";

export type OpenProject = {
	dir: string,
	workflow: Workflow,
	lastRun: LastRun | null,
	version: string,
	view: { [key in string]: unknown },
};

export type PageLogLevel = "error" | "warn";

export type PanelWindowsChanged = DockPanelName[];

export type ParamRef = {
	nodeId: string,
	param: string,
	unit: ParamUnit,
	scale?: number,
	index?: number,
	linked?: LinkedParam[],
};

export type ParamUnit = "ms" | "frames" | "rate" | "number";

export type Phase = "capture" | "media" | "compile" | "render" | "post";

export type PickRequest = {
	url: string,
	viewport: Viewport | null,
};

export type PickResult = {
	target: string,
	css: string,
	role?: string,
	selectorUsed: PickedBy,
};

export type PickedBy = "accessible name" | "css" | "spot";

export type PickerPage = {
	url: string,
	viewport: Viewport | null,
};

export type PickerTarget = {
	kind: "page",
} & PickerPage | { kind: "problem"; message: string };

export type PictureLook = {
	brightness?: number,
	contrast?: number,
	saturation?: number,
	warmth?: number,
	vignette?: number,
	grain?: number,
};

export type PlannedFamily<C = {
	name: string,
	family: string,
	weight: string,
	style: FontStyle,
	variable: boolean,
	format: FontFormat,
	size: number,
}> = {
	family: string,
	faces: C[],
};

export type PlayRange = {
	inFrame?: number,
	outFrame?: number,
};

export type Point = {
	x: number,
	y: number,
};

export type PortLink = {
	from: PortType,
	to: PortType,
};

export type PortType = "browser-session" | "recording" | "recording-style" | "grade" | "clip" | "audio" | "captions" | "image" | "media" | "file-info" | "folder";

export type Position = {
	x: number,
	y: number,
};

export type PreviewQuality = "fast" | "exact";

export type PreviewReported = {
	frame: number,
	playing: boolean,
};

export type PreviewTarget = {
	spec: TimelineSpec,
	spans: { [key in string]: FrameRange },
	sourceNodeId: string,
};

export type ProjectFileChange = {
	dir: string,
	workflow: Workflow | null,
	version: string,
};

export type ProjectFileChanged = ProjectFileChange;

export type ProjectFilesChange = {
	dir: string,
	files: { [key in string]: string },
};

export type ProjectFilesChanged = ProjectFilesChange;

export type ProjectFonts = {
	main: string,
	heading: string,
	code: string,
	families: FontFamilyDef[],
};

export type ProjectLocations = {
	defaultDir: string,
	suggestedDir: string,
};

export type ProjectsSection = {
	defaultDir: string,
	lastDir: string,
};

export type RecentProject = {
	dir: string,
	name: string,
	openedAt: number,
};

export type RecordingProps = {
	src: string,
	proxySrc?: string,
	log: ActionLog,
	name: string,
	speed: number,
	zoom: boolean,
	tuning: CameraTuning,
	motionBlur: boolean,
	style: CameraStyle,
	chrome: boolean,
	windowStyle?: WindowStyle,
	frameTheme?: FrameTheme,
	addressBar?: boolean,
	backdrop: string,
	backdropChosen?: boolean,
	cursor: boolean,
	cursorScale: number,
	cursorLook: CursorLook,
	ripple: RippleLook | null,
	extraBeats: Action[],
	keystrokes?: KeystrokeLook,
	annotations: AnnotationSpec[],
	blurAreas?: BlurArea[],
	look?: PictureLook,
	screen?: boolean,
};

export type RecordingValue = {
	src: string | null,
	proxySrc?: string,
	log: ActionLog | null,
	durationMs: number,
	viewport: Viewport,
	nodeId: string,
	style: StyleEntry[],
};

export type Rect = {
	x: number,
	y: number,
	w: number,
	h: number,
};

export type ReframeMode = "follow" | "center" | "fit";

export type RelayTarget = "editor" | "others";

export type Relayed = {
	from: string,
	message: unknown,
};

export type RenderSection = {
	licenseKey: string,
};

export type RippleLook = {
	color: string,
	size: number,
	width: number,
};

export type RunBody = {
	targetNodeIds?: string[],
	fromNodeId?: string,
};

export type RunEventFired = RunEvent;

export type RunEvent = { type: "run:start"; runId: string; nodeIds: string[]; at: number } | { type: "node:start"; runId: string; nodeId: string; at: number } | { type: "node:progress"; runId: string; nodeId: string; progress: number; message?: string } | { type: "node:log"; runId: string; nodeId?: string; level: LogLevel; message: string; at: number } | { type: "node:done"; runId: string; nodeId: string; cached: boolean; result?: NodeResult } | { type: "node:error"; runId: string; nodeId: string; message: string; stack?: string; traceId?: string } | { type: "node:skipped"; runId: string; nodeId: string; reason: string } | { type: "run:done"; runId: string; ok: boolean; cancelled: boolean; error?: string; failedNodeIds?: string[]; skippedNodeIds?: string[]; outputs: string[]; at: number };

export type RunProgress = {
	runId: string | null,
	statuses: { [key in string]: NodeRunStatus },
	progress: { [key in string]: NodeProgress },
	cached: { [key in string]: boolean },
	errors: { [key in string]: string },
	traces: { [key in string]: string },
	results: { [key in string]: NodeResult },
};

export type RunRequest = {
	projectDir: string,
	workflow: Workflow,
	targetNodeIds?: string[],
	fromNodeId?: string,
};

export type RunStatus = {
	runId: string | null,
	projectDir: string | null,
	progress: RunProgress,
};

export type SaveResult = { outcome: "saved"; version: string } | { outcome: "changed"; version: string; workflow: Workflow | null };

export type SequencerProps = {
	component: string,
	props: { [key in string]: unknown },
	backgrounds: (string | null)[],
};

export type SessionBrowser = {
	nodeId: string,
	params: { [key in string]: unknown },
};

export type SessionStep = {
	id: string,
	type: string,
	label?: string,
	params: { [key in string]: unknown },
};

export type SettingsChanged = AppSettings;

export type SettingsPage = "general" | "projects" | "engine" | "recording" | "fonts" | "export" | "api" | "licences" | "about";

export type SettingsPageShown = SettingsPage;

export type SettingsPatch = {
	width?: number,
	height?: number,
	fps?: number,
	background?: string,
	theme?: Theme,
	fonts?: ProjectFonts,
};

export type Slot = {
	nodeId: string,
	port: string,
};

export type SoundPlacement = {
	src: string,
	at: number,
	offset: number,
	length: number,
	loop: boolean,
	rate: number,
	volume: number,
	fadeIn: number,
	fadeOut: number,
	envelope?: AudioEnvelope,
};

export type SoundWindow = {
	from: number,
	to: number,
	contentStart: number,
};

export type SpecAssets = {
	projectBase?: string,
	versions?: { [key in string]: string },
};

export type SpotBackground = {
	src: string,
	timeMs: number,
	durationMs: number,
	still: boolean,
};

export type SrcValue = {
	src: string | null,
};

export type StartOption = "start-screen" | "last-project" | "new-project";

export type StartedRun = {
	runId: string,
};

export type StemLevels = {
	fps: number,
	peak: (number)[],
	rms: (number)[],
};

export type Strip = {
	trim: number,
	highpass: Highpass,
	eq: Eq,
	comp: Comp,
	fader: number,
	pan: number,
	mute: boolean,
	solo: boolean,
	bus?: string,
};

export type StyleEntry = {
	nodeId: string,
	type: string,
	params: { [key in string]: unknown },
};

export type TabInfo = {
	id: string,
	url: string,
};

export type TaggedRecording = {
	type: "recording",
} & RecordingValue;

export type TemplateInfo = {
	id: string,
	title: string,
	description: string,
	icon: Icon,
};

export type Theme = "light" | "dark";

export type TimelineItem = {
	kind: "group",
} & {
	props: GroupProps,
} & ItemBase | {
	kind: "recording",
} & {
	props: RecordingProps,
} & ItemBase | {
	kind: "element",
} & {
	component: string,
	props: { [key in string]: unknown },
} & ItemBase;

export type TimelineSpec = {
	width: number,
	height: number,
	fps: number,
	durationInFrames: number,
	background?: string,
	tracks: TimelineTrack[],
	audio: AudioItem[],
	assets?: SpecAssets,
	reframe?: ReframeMode,
	quality?: PreviewQuality,
	fonts?: ProjectFonts,
	mix?: MasterStem,
};

export type TimelineTrack = {
	id: string,
	items: TimelineItem[],
	layer?: boolean,
};

export type Tone = "muted" | "warn";

export type TraceAction = {
	id: string,
	parentId?: string,
	group: boolean,
	nodeId?: string,
	title: string,
	target?: string,
	start: number,
	end: number,
	error?: string,
	logs: string[],
	point?: Point,
	box?: Bounds,
};

export type TraceConsole = {
	level: ConsoleLevel,
	text: string,
	time: number,
	url?: string,
};

export type TraceError = {
	message: string,
	stack?: string,
	time: number,
	actionId?: string,
};

export type TraceFrame = {
	src: string,
	time: number,
	width: number,
	height: number,
};

export type TraceHeader = {
	id: string,
	nodeId: string,
	label: string,
	ok: boolean,
	createdAt: number,
};

export type TraceModel = {
	start: number,
	end: number,
	viewport: Viewport | null,
	actions: TraceAction[],
	frames: TraceFrame[],
	console: TraceConsole[],
	errors: TraceError[],
	network: TraceRequest[],
} & TraceHeader;

export type TraceRequest = {
	method: string,
	url: string,
	status: number,
	mimeType: string,
	resourceType: string,
	size: number,
	start: number,
	duration: number,
	failure?: string,
};

export type TraceRetention = "always" | "failures" | "last-n" | "off";

export type TracesSection = {
	retention: TraceRetention,
	keepLast: number,
};

export type TransitionMark = {
	key: string,
	itemKey: string,
	from: number,
	to: number,
	kind: string,
	lanes: number[],
	covers: number,
	slot?: Slot,
};

export type TransitionRef = {
	kind: string,
	durationInFrames: number,
	params?: { [key in string]: unknown },
	hold?: boolean,
};

export type UpdateNodeBody = {
	label?: string,
	position?: Position,
	disabled?: boolean,
	params?: { [key in string]: Value },
};

export type Value = null | boolean | number | string | Value[] | { [key in string]: Value };

export type VarInfo = {
	name: string,
	value: string,
};

export type VideoBar = {
	type: VideoTag,
	key: string,
	itemId: string,
	nodeId: string,
	kind: BarKind,
	component?: string,
	props: { [key in string]: unknown },
	thumbnail: BarThumbnail | null,
	from: number,
	to: number,
	lane: number,
	order: number,
	chain: ChainLink[],
	rate: number,
	contentStart: number,
	contentEnd: number,
	loop: number,
	hasSound: boolean,
	sound: SoundWindow | null,
};

export type VideoTag = "video";

export type ViewEvent = { kind: "tabs"; tabs: TabInfo[]; active: number } | { kind: "dialog"; dialog: string; message: string; defaultPrompt: string } | { kind: "fileChooser"; multiple: boolean };

export type ViewEventFired = {
	id: string,
	event: ViewEvent,
};

export type ViewInput = { kind: "mouseDown"; x: number; y: number; button: MouseButton; buttons: number; clickCount: number; modifiers: number } | { kind: "mouseUp"; x: number; y: number; button: MouseButton; buttons: number; clickCount: number; modifiers: number } | { kind: "mouseMove"; x: number; y: number; buttons: number; modifiers: number } | { kind: "wheel"; x: number; y: number; deltaX: number; deltaY: number; modifiers: number } | { kind: "keyDown"; key: string; code: string; text: string; keyCode: number; location: number; modifiers: number; repeat: boolean } | { kind: "keyUp"; key: string; code: string; keyCode: number; location: number; modifiers: number } | { kind: "text"; text: string } | { kind: "composition"; text: string; selectionStart: number; selectionEnd: number };

export type ViewPrefChange = {
	dir: string,
	key: string,
	value: unknown,
};

export type ViewPrefChanged = ViewPrefChange;

export type ViewSize = {
	width: number,
	height: number,
	dpr: number,
};

export type Viewport = {
	width: number,
	height: number,
};

export type VisualEnvelope = {
	opacity?: Keyframe[],
	x?: Keyframe[],
	y?: Keyframe[],
	scale?: Keyframe[],
};

export type WaveformLevel = {
	rate: number,
	min: (number)[],
	max: (number)[],
};

export type Waveform = {
	src: string,
	durationMs: number,
	levels: WaveformLevel[],
	beats: (number)[],
	bpm: number,
};

export type WindowRole = "cut" | "copy" | "paste" | "selectAll" | "reload" | "toggleDevTools" | "resetZoom" | "zoomIn" | "zoomOut" | "togglefullscreen" | "minimize" | "toggleMaximize" | "close" | "quit";

export type WindowState = {
	maximized: boolean,
	fullscreen: boolean,
	focused: boolean,
};

export type WindowStateChanged = WindowState;

export type WindowStyle = "macos" | "windows" | "linux";

export type WorkflowEdge = {
	id: string,
	source: string,
	sourceHandle: string,
	target: string,
	targetHandle: string,
};

export type WorkflowNode = {
	id: string,
	type: string,
	position: Position,
	params: { [key in string]: unknown },
	label?: string,
	disabled?: boolean,
	parentId?: string,
	size?: Viewport,
};

export type WorkflowSettings = {
	width: number,
	height: number,
	fps: 30 | 60,
	background?: string,
	theme: Theme,
	fonts: ProjectFonts,
};

export type Workflow = {
	version: 1,
	id: string,
	name: string,
	settings: WorkflowSettings,
	variables: { [key in string]: string },
	nodes: WorkflowNode[],
	edges: WorkflowEdge[],
	mix?: Mix,
	markers: Marker[],
};

/* Tauri Specta runtime */
async function typedError<T, E>(result: Promise<T>): Promise<{ status: "ok"; data: T } | { status: "error"; error: E }> {
    try {
        return { status: "ok", data: await result };
    } catch (e) {
        if (e instanceof Error) throw e;
        return { status: "error", error: e as any };
    }
}

type EventEmit<T> = [T] extends [null] ? () => Promise<void> : (payload: T) => Promise<void>;

type EventEmitTo<T> = [T] extends [null] ? (target: __TAURI_EVENT.EventTarget | string) => Promise<void> : (target: __TAURI_EVENT.EventTarget | string, payload: T) => Promise<void>;

function makeEvent<TListen, TEmit = TListen>(name: string, serialize?: (payload: TEmit) => unknown, deserialize?: (payload: any) => TListen) {
    const mapEvent = (cb: __TAURI_EVENT.EventCallback<TListen>) => (event: __TAURI_EVENT.Event<any>) => cb({ ...event, payload: deserialize ? deserialize(event.payload) : event.payload });
    const mapPayload = (payload: TEmit) => serialize ? serialize(payload) : payload;

    const base = {
        listen: (cb: __TAURI_EVENT.EventCallback<TListen>) => __TAURI_EVENT.listen(name, mapEvent(cb)),
        once: (cb: __TAURI_EVENT.EventCallback<TListen>) => __TAURI_EVENT.once(name, mapEvent(cb)),
        emit: ((payload: TEmit) => __TAURI_EVENT.emit(name, mapPayload(payload)) as unknown) as EventEmit<TEmit>,
        emitTo: ((to: __TAURI_EVENT.EventTarget | string, payload: TEmit) => __TAURI_EVENT.emitTo(to, name, mapPayload(payload)) as unknown) as EventEmitTo<TEmit>
    };

    const fn = (target: import("@tauri-apps/api/webview").Webview | import("@tauri-apps/api/window").Window) => ({
        listen: (cb: __TAURI_EVENT.EventCallback<TListen>) => target.listen(name, mapEvent(cb)),
        once: (cb: __TAURI_EVENT.EventCallback<TListen>) => target.once(name, mapEvent(cb)),
        emit: ((payload: TEmit) => target.emit(name, mapPayload(payload)) as unknown) as EventEmit<TEmit>,
        emitTo: ((to: __TAURI_EVENT.EventTarget | string, payload: TEmit) => target.emitTo(to, name, mapPayload(payload)) as unknown) as EventEmitTo<TEmit>
    });

    return Object.assign(fn, base);
}
