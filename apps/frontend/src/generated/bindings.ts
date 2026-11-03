import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";
import * as __TAURI_EVENT from "@tauri-apps/api/event";

/** Commands */
export const commands = {
	listRecentProjects: () => __TAURI_INVOKE<RecentProject[]>("list_recent_projects"),
	projectLocations: () => __TAURI_INVOKE<ProjectLocations>("project_locations"),
	chooseFolder: (title: string, defaultPath: string | null) => __TAURI_INVOKE<string | null>("choose_folder", { title, defaultPath }),
	pickProjectDir: () => __TAURI_INVOKE<string | null>("pick_project_dir"),
	openProject: (dir: string) => __TAURI_INVOKE<OpenProject>("open_project", { dir }),
	documentState: () => __TAURI_INVOKE<DocumentState | null>("document_state"),
	editWorkflow: (edit: WorkflowEdit) => __TAURI_INVOKE<Edited>("edit_workflow", { edit }),
	undo: () => __TAURI_INVOKE<null>("undo"),
	redo: () => __TAURI_INVOKE<null>("redo"),
	beginGesture: () => __TAURI_INVOKE<void>("begin_gesture"),
	endGesture: () => __TAURI_INVOKE<void>("end_gesture"),
	editTimeline: (edit: TimelineEdit) => __TAURI_INVOKE<TimelineReply>("edit_timeline", { edit }),
	timelineRemoval: (selected: string[]) => __TAURI_INVOKE<Removal | null>("timeline_removal", { selected }),
	saveProject: () => __TAURI_INVOKE<boolean>("save_project"),
	resolveDiskChange: (choice: DiskChoice) => __TAURI_INVOKE<boolean>("resolve_disk_change", { choice }),
	projectFolder: (parent: string, name: string) => __TAURI_INVOKE<string>("project_folder", { parent, name }),
	createProject: (name: string, template: string, dir: string | null) => __TAURI_INVOKE<OpenProject | null>("create_project", { name, template, dir }),
	readLastRun: (dir: string) => __TAURI_INVOKE<LastRun | null>("read_last_run", { dir }),
	importAssets: (dir: string, paths: string[] | null) => __TAURI_INVOKE<string[]>("import_assets", { dir, paths }),
	clearCache: (dir: string) => __TAURI_INVOKE<null>("clear_cache", { dir }),
	listTemplates: () => __TAURI_INVOKE<TemplateInfo[]>("list_templates"),
	getSettings: () => __TAURI_INVOKE<AppSettings>("get_settings"),
	patchSettings: (patch: { [key in string]: Value }) => __TAURI_INVOKE<AppSettings>("patch_settings", { patch }),
	viewState: (dir: string) => __TAURI_INVOKE<{ [key in string]: Value }>("view_state", { dir }),
	setViewPref: (dir: string, key: string, value: Value) => __TAURI_INVOKE<null>("set_view_pref", { dir, key, value }),
	appMenuSections: () => __TAURI_INVOKE<MenuSection[]>("app_menu_sections"),
	getEnvironment: () => __TAURI_INVOKE<EnvStatus>("get_environment"),
	installEngine: () => __TAURI_INVOKE<null>("install_engine"),
	chooseBrowser: () => __TAURI_INVOKE<boolean>("choose_browser"),
	getEngineStatus: () => __TAURI_INVOKE<EngineStatus>("get_engine_status"),
	listLicences: () => __TAURI_INVOKE<LicenceEntry[]>("list_licences"),
	openLicenceFile: (file: string) => __TAURI_INVOKE<null>("open_licence_file", { file }),
	revealInFolder: (dir: string, file: string | null) => __TAURI_INVOKE<null>("reveal_in_folder", { dir, file }),
	readTrace: (dir: string, traceId: string) => __TAURI_INVOKE<TraceModel>("read_trace", { dir, traceId }),
	openSettings: (page: SettingsPage | null) => __TAURI_INVOKE<null>("open_settings", { page }),
	openPanelWindow: (panel: DockPanelName) => __TAURI_INVOKE<null>("open_panel_window", { panel }),
	openLiveWindow: () => __TAURI_INVOKE<null>("open_live_window"),
	focusPanelWindow: (panel: DockPanelName) => __TAURI_INVOKE<void>("focus_panel_window", { panel }),
	closePanelWindow: (panel: DockPanelName) => __TAURI_INVOKE<null>("close_panel_window", { panel }),
	closePanelWindows: () => __TAURI_INVOKE<null>("close_panel_windows"),
	listPanelWindows: () => __TAURI_INVOKE<DockPanelName[]>("list_panel_windows"),
	getWindowState: () => __TAURI_INVOKE<WindowState>("get_window_state"),
	setTitle: (title: string) => __TAURI_INVOKE<null>("set_title", { title }),
	windowRole: (role: WindowRole) => __TAURI_INVOKE<null>("window_role", { role }),
	reportDrafts: (held: boolean) => __TAURI_INVOKE<void>("report_drafts", { held }),
	closeShown: () => __TAURI_INVOKE<void>("close_shown"),
	decideClose: (decision: CloseDecision) => __TAURI_INVOKE<void>("decide_close", { decision }),
	saveFlushed: () => __TAURI_INVOKE<void>("save_flushed"),
	setCompileFocus: (selected: string | null, inspected: string | null) => __TAURI_INVOKE<void>("set_compile_focus", { selected, inspected }),
	latestCompile: () => __TAURI_INVOKE<EditorCompile | null>("latest_compile"),
	editorCatalog: () => __TAURI_INVOKE<EditorCatalog>("editor_catalog"),
	fileRoot: () => __TAURI_INVOKE<string>("file_root"),
	runWorkflow: (targets: RunTargets) => __TAURI_INVOKE<StartedRun>("run_workflow", { targets }),
	cancelRun: () => __TAURI_INVOKE<void>("cancel_run"),
	runState: () => __TAURI_INVOKE<RunState>("run_state"),
	clearRunLog: () => __TAURI_INVOKE<void>("clear_run_log"),
	watchLive: (frames: Channel<ArrayBuffer>) => __TAURI_INVOKE<void>("watch_live", { frames }),
	unwatchLive: (channel: number) => __TAURI_INVOKE<void>("unwatch_live", { channel }),
	liveWindowWatching: () => __TAURI_INVOKE<boolean>("live_window_watching"),
	gradientView: (css: string) => __TAURI_INVOKE<GradientView | null>("gradient_view", { css }),
	editGradient: (css: string, edit: GradientEdit) => __TAURI_INVOKE<GradientView | null>("edit_gradient", { css, edit }),
	listFonts: () => __TAURI_INVOKE<FontChoice[]>("list_fonts"),
	fontChoices: (value: string, project: FontFamilyDef[]) => __TAURI_INVOKE<FontChoices>("font_choices", { value, project }),
	terminalFonts: () => __TAURI_INVOKE<FontFamilyDef[]>("terminal_fonts"),
	inspectFontFiles: (paths: string[] | null) => __TAURI_INVOKE<InspectedFonts | null>("inspect_font_files", { paths }),
	installedFontFamilies: () => __TAURI_INVOKE<InstalledFamily[]>("installed_font_families"),
	inspectInstalledFonts: (families: string[]) => __TAURI_INVOKE<InspectedFonts>("inspect_installed_fonts", { families }),
	addInspectedFonts: (token: string, families: string[]) => __TAURI_INVOKE<FontFamilyDef[]>("add_inspected_fonts", { token, families }),
	releaseInspectedFonts: (token: string) => __TAURI_INVOKE<void>("release_inspected_fonts", { token }),
	removeFont: (id: string) => __TAURI_INVOKE<null>("remove_font", { id }),
	fontCatalog: () => __TAURI_INVOKE<FontCatalogEntry[]>("font_catalog"),
	downloadFont: (id: string) => __TAURI_INVOKE<FontFamilyDef>("download_font", { id }),
	useFontInProject: (dir: string, fontId: string) => __TAURI_INVOKE<FontFamilyDef>("use_font_in_project", { dir, fontId }),
	analyze: (request: AnalysisRequest) => __TAURI_INVOKE<AnalysisResult>("analyze", { request }),
	pickElement: (request: PickRequest) => __TAURI_INVOKE<PickResult | null>("pick_element", { request }),
	attachView: (id: string, frames: Channel<ArrayBuffer>) => __TAURI_INVOKE<null>("attach_view", { id, frames }),
	viewInput: (id: string, input: ViewInput) => __TAURI_INVOKE<null>("view_input", { id, input }),
	viewResize: (id: string, size: ViewSize) => __TAURI_INVOKE<null>("view_resize", { id, size }),
	viewNavigate: (id: string, url: string) => __TAURI_INVOKE<null>("view_navigate", { id, url }),
	viewGo: (id: string, steps: number) => __TAURI_INVOKE<null>("view_go", { id, steps }),
	viewReload: (id: string) => __TAURI_INVOKE<null>("view_reload", { id }),
	viewSwitchTab: (id: string, tab: string) => __TAURI_INVOKE<null>("view_switch_tab", { id, tab }),
	viewAnswerDialog: (id: string, accept: boolean, prompt: string) => __TAURI_INVOKE<null>("view_answer_dialog", { id, accept, prompt }),
	viewChooseFiles: (id: string) => __TAURI_INVOKE<null>("view_choose_files", { id }),
	viewClose: (id: string) => __TAURI_INVOKE<void>("view_close", { id }),
	terminalFrames: (projectDir: string, cast: string, fps: number, theme: TerminalTheme | null) => __TAURI_INVOKE<ArrayBuffer>("terminal_frames", { projectDir, cast, fps, theme }),
	attachTerminal: (id: string, frames: Channel<ArrayBuffer>) => __TAURI_INVOKE<null>("attach_terminal", { id, frames }),
	terminalInput: (id: string, input: TerminalInput) => __TAURI_INVOKE<null>("terminal_input", { id, input }),
	finishTerminal: (id: string, save: boolean) => __TAURI_INVOKE<null>("finish_terminal", { id, save }),
	pageLog: (level: PageLogLevel, message: string) => __TAURI_INVOKE<void>("page_log", { level, message }),
	sendMenuToEditor: (command: MenuCommand) => __TAURI_INVOKE<null>("send_menu_to_editor", { command }),
	sharedState: () => __TAURI_INVOKE<SharedState>("shared_state"),
	shareSelection: (selection: Selection) => __TAURI_INVOKE<null>("share_selection", { selection }),
	sharePlayhead: (playhead: Playhead) => __TAURI_INVOKE<null>("share_playhead", { playhead }),
	controlPlayback: (call: PlaybackCall) => __TAURI_INVOKE<null>("control_playback", { call }),
	showReplay: (nodeId: string | null, tab: ReplayTab) => __TAURI_INVOKE<null>("show_replay", { nodeId, tab }),
	apiStatus: () => __TAURI_INVOKE<ApiServerStatus>("api_status"),
};

/** Events */
export const events = {
	apiStatusChanged: makeEvent<ApiStatusChanged, ApiStatusChanged>("api-status-changed"),
	compiled: makeEvent<Compiled, Compiled>("compiled"),
	confirmClose: makeEvent<ConfirmClose, ConfirmClose>("confirm-close"),
	diskChangeAsked: makeEvent<DiskChangeAsked, DiskChangeAsked>("disk-change-asked"),
	documentChanged: makeEvent<DocumentChanged, DocumentChanged>("document-changed"),
	engineChanged: makeEvent<EngineChanged, EngineChanged>("engine-changed"),
	engineInstallProgress: makeEvent<EngineInstallProgress, EngineInstallProgress>("engine-install-progress"),
	flushSave: makeEvent<FlushSave, FlushSave>("flush-save"),
	liveNoted: makeEvent<LiveNoted, LiveNoted>("live-noted"),
	liveWindowWatching: makeEvent<LiveWindowWatching, LiveWindowWatching>("live-window-watching"),
	menuCommandFired: makeEvent<MenuCommandFired, MenuCommandFired>("menu-command-fired"),
	openRequested: makeEvent<OpenRequested, OpenRequested>("open-requested"),
	panelWindowsChanged: makeEvent<PanelWindowsChanged, PanelWindowsChanged>("panel-windows-changed"),
	playbackCalled: makeEvent<PlaybackCalled, PlaybackCalled>("playback-called"),
	playheadShared: makeEvent<PlayheadShared, PlayheadShared>("playhead-shared"),
	projectFilesChanged: makeEvent<ProjectFilesChanged, ProjectFilesChanged>("project-files-changed"),
	projectReloaded: makeEvent<ProjectReloaded, ProjectReloaded>("project-reloaded"),
	replayRequested: makeEvent<ReplayRequested, ReplayRequested>("replay-requested"),
	runEventFired: makeEvent<RunEventFired, RunEventFired>("run-event-fired"),
	runLogged: makeEvent<RunLogged, RunLogged>("run-logged"),
	runStateReset: makeEvent<RunStateReset, RunStateReset>("run-state-reset"),
	runViewChanged: makeEvent<RunViewChanged, RunViewChanged>("run-view-changed"),
	selectionShared: makeEvent<SelectionShared, SelectionShared>("selection-shared"),
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
	addresses?: PageAddress[],
	caretTrack?: CaretSample[],
	titles?: TitleChange[],
	cells?: CellGrid,
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
	tSecondDown?: number,
	button?: ClickButton,
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
	keyTimes?: (number)[],
	silent?: boolean,
};

export type AddressMode = "fixed" | "host";

export type AnalysisRequest = { kind: "waveform"; projectDir: string; src: string } | { kind: "filmstrip"; projectDir: string; src: string } | { kind: "mix"; projectDir: string; plan: MixPlan };

export type AnalysisResult = { kind: "waveform"; waveform: Waveform } | { kind: "filmstrip"; filmstrip: Filmstrip } | { kind: "mix"; mix: MixResult } | { kind: "skipped" } | { kind: "error"; error: string };

export type Anchor = {
	nodeId?: string,
	clipEnd?: boolean,
	keepEnd?: boolean,
};

export type AnnotationKind = "highlight" | "spotlight" | "tooltip" | "callout";

export type AnnotationLook = {
	fadeS: number,
	radius: number,
	softness?: number,
	lineWidth?: number,
	fillOpacity?: number,
	markerOpacity?: number,
	drawS?: number,
	strokes?: number,
};

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
	labelLook?: LabelLook,
	markLook?: AnnotationLook,
};

export type ApiProblem = {
	path?: string,
} & GraphIssue;

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
	projects: ProjectsSection,
	editor: EditorSection,
	api: ApiSection,
	engine: EngineSection,
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
	hits?: (number)[],
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
	pitch?: number,
	hit?: boolean,
	fadeInFrames?: number,
	fadeOutFrames?: number,
	fadeCurve?: FadeCurve,
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
	hits?: Hits,
};

export type BandBounds = {
	min: number,
	max: number,
};

export type BarKind = "recording" | "element";

export type BarThumbnail = { kind: "filmstrip"; src: string; speed: number; trimMs: number } | { kind: "still"; src: string } | { kind: "terminal"; cast: string; theme: TerminalTheme; speed: number };

export type Bounds = {
	x: number,
	y: number,
	width: number,
	height: number,
};

export type BrowserAction = "browser.startRecording" | "browser.navigate" | "browser.click" | "browser.type" | "browser.hover" | "browser.drag" | "browser.scroll" | "browser.select" | "browser.upload" | "browser.press" | "browser.wait" | "browser.assert" | "browser.hoist" | "browser.setState" | "browser.mockRoute" | "browser.freezeClock" | "browser.hideElements" | "browser.chapter" | "browser.overlayHtml" | "browser.signIn";

export type BrowserSessionValue = {
	browser: SessionBrowser,
	steps: SessionStep[],
	address?: string,
};

export type BrowserSource = {
	video: string | null,
	viewport: Viewport,
};

export type BusDuck = {
	keys: string[],
	amount: number,
	attack: number,
	release: number,
};

export type BusPatch = {
	name?: string,
	highpass?: Highpass,
	eq?: Eq,
	comp?: Comp,
	fader?: number,
	mute?: boolean,
	solo?: boolean,
	duck?: DuckSettings,
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

export type CameraDial = {
	key: CameraDialKey,
	label: string,
	min: number,
	max: number,
	step: number,
	unit: string,
};

export type CameraDialKey = "yaw" | "pitch" | "distance" | "height";

export type CameraEasing = "smooth" | "gentle" | "snappy" | "linear";

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

export type CameraProps = {
	zoom: boolean,
	tuning: CameraTuning,
};

export type CameraStyle = "classic" | "proof" | "ledger" | "stage";

export type CameraTuning = {
	sMax?: number,
	holdMinS?: number,
	minScale?: number,
	firstMaxScale?: number,
	roomPx?: number,
	targetShare?: number,
	offCentre?: number,
	joinWithinS?: number,
	stayZoomedWithinS?: number,
	holdAfterTypingS?: number,
	restAtEndS?: number,
	longestPanPx?: number,
	pace?: number,
	easing?: CameraEasing,
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

export type CaretSample = {
	t: number,
	col: number,
	row: number,
};

export type CatalogCategory = "sans-serif" | "serif" | "monospace" | "display" | "handwriting" | "icons" | "other";

export type CatalogEntry = {
	params: { [key in string]: unknown },
} & CatalogNode;

export type CatalogFamily = {
	prefix: string,
	type: PortType,
	required: number,
};

export type CatalogNode = {
	type: string,
	title: string,
	category: string,
	description: string,
	runs: boolean,
	inputs: { [key in string]: CatalogPort },
	family?: CatalogFamily,
	outputs: { [key in string]: CatalogPort },
};

export type CatalogPort = {
	type: PortType,
	label: string,
	optional: boolean,
	many: boolean,
};

export type Category = "Browser" | "Terminal" | "Capture" | "Camera" | "Cursor" | "Text & overlays" | "Transitions" | "Backgrounds" | "Effects" | "Grade" | "Elements" | "Audio" | "Timeline" | "Media" | "Image" | "Output" | "Utility";

export type CellGrid = {
	cols: number,
	rows: number,
	width: number,
	height: number,
	padding: number,
};

export type CellTarget = { kind: "cell"; col: number; row: number } | { kind: "text"; text: string };

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

export type CheckedTool = {
	path: string,
	version: string,
	sha256: string,
	size: number,
	modifiedMs: number,
};

export type ClickButton = "left" | "right";

export type ClipValue = {
	durationInFrames: number,
	tracks: TimelineTrack[],
	audio: AudioItem[],
	recording?: TaggedRecording,
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

export type CompRanges = {
	threshold: NumberRange,
	ratio: NumberRange,
	attack: NumberRange,
	release: NumberRange,
	makeup: NumberRange,
};

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
	order: string[],
};

export type CompileValue = {
	type: "browser-session",
} & BrowserSessionValue | {
	type: "terminal-session",
} & TerminalSessionValue | {
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

export type CursorFeel = {
	growWithZoom: number,
	outlineWidth: number,
	shadow: number,
	pressSquash: number,
	hideWhileTyping: boolean,
};

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

export type DiskChangeAsked = {
	readable: boolean,
};

export type DiskChoice = "load" | "keepMine" | "wait";

export type DockPanelName = "timeline" | "mixer" | "run";

export type DocumentChanged = {
	document: DocumentState,
	edited: string[],
};

export type DocumentState = {
	session: string,
	dir: string,
	workflow: Workflow,
	revision: number,
	savedRevision: number,
	saving: boolean,
	canUndo: boolean,
	canRedo: boolean,
};

export type Duck = {
	level: number,
	fadeFrames: number,
	keys: string[],
	spans: ([number, number])[],
};

export type DuckRanges = {
	amount: NumberRange,
	attack: NumberRange,
	release: NumberRange,
	ignoreShorterMs: NumberRange,
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

export type EdgeEnds = {
	source: string,
	sourceHandle: string,
	target: string,
	targetHandle: string,
};

export type EditHandles = {
	move?: ParamRef,
	trimStart?: ParamRef,
	trimEnd?: ParamRef,
	speed?: ParamRef,
	volume?: ParamRef,
};

export type Edited = {
	revision: number,
	ids: string[],
};

export type EditorCatalog = {
	nodes: EditorNode[],
	transitions: { [key in string]: string },
	connects: PortLink[],
	text: TextRules,
	mixer: MixerBook,
	bundledFonts: FontFamilyDef[],
	settings: SettingsBook,
	themeBackgrounds: Partial<{ [key in Theme]: string }>,
	shapes: ShapeChoice[],
	qualityPresets: ([string, number])[],
	loudnessPresets: ([string, number])[],
	timeline: TimelineBook,
	sounds: SoundChoice[],
	music: SoundChoice[],
	terminalThemes: TerminalTheme[],
};

export type EditorCompile = {
	seq: number,
	result: CompileResult,
	issues: GraphIssue[],
	film: FilmTarget | null,
	flat: FlatTimeline | null,
	plan: MixPlan | null,
	levelled: TimelineSpec | null,
	cards: { [key in string]: CardRow[] },
	inputs: { [key in string]: { [key in string]: CatalogPort } },
	inspector: InspectorContext | null,
	localFrames: { [key in string]: LocalFrames },
	mix: Mix,
	solo: boolean,
	outdated: string[],
	converts: { [key in string]: { [key in string]: PortType[] } },
};

export type EditorNode = {
	type: string,
	title: string,
	category: Category,
	categoryLabel: string,
	description?: string,
	phase: Phase,
	inputs: { [key in string]: CatalogPort },
	family?: CatalogFamily,
	outputs: { [key in string]: CatalogPort },
	params: { [key in string]: unknown },
	defaults: { [key in string]: unknown },
	hostDefaults: { [key in string]: unknown },
	virtual: boolean,
	action: boolean,
	executes: boolean,
	hidden: boolean,
	usesProjectSize: boolean,
	element?: string,
	look?: NodeLook,
	cardHeight: number,
	hasCard: boolean,
	compiles: boolean,
};

export type EditorSection = {
	canvasMode: CanvasMode,
};

export type ElementRef = {
	component: string,
	props: { [key in string]: unknown },
};

export type EngineChanged = EngineStatus;

export type EngineInstallEvent = {
	done?: boolean,
	error?: string,
} & EngineInstallUpdate;

export type EngineInstallProgress = EngineInstallEvent;

export type EngineInstallUpdate = {
	step: string,
	progress: number,
	message: string,
};

export type EngineSection = {
	browser: CheckedTool | null,
};

export type EngineStatus = {
	downloadDir: string,
	browser: ToolStatus,
	complete: boolean,
};

export type EntriesValue = {
	entries: StyleEntry[],
};

export type EnvStatus = {
	platform: string,
	arch: string,
	isPackaged: boolean,
	userData: string,
	versions: AppVersions,
	engine: EngineStatus,
};

export type EnvVar = {
	name: string,
	value: string,
};

export type EnvelopeKey = "opacity" | "x" | "y" | "scale" | "volume" | "pan";

export type EnvelopeTrack = {
	key: EnvelopeKey,
	label: string,
	min: number,
	max: number,
	rest: number,
	audio: boolean,
};

export type EqBand = {
	type: EqType,
	freq: number,
	gain: number,
	q: number,
};

export type EqBandBounds = {
	freq: BandBounds,
	gain: BandBounds,
	q: BandBounds,
};

export type EqType = "lowshelf" | "peak" | "highshelf";

export type Eq = {
	on: boolean,
	bands: EqBand[],
};

export type FadeCurve = "tri" | "qsin" | "exp" | "log";

export type FilmChapter = {
	title: string,
	startMs: number,
	endMs: number,
};

export type FilmFonts = {
	families: { [key in string]: FontFamilyDef },
	preload: string[],
};

export type FilmTarget = {
	spec: TimelineSpec,
	spans: { [key in string]: FrameRange },
	sourceNodeId: string,
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
	added: boolean,
};

export type FontCategory = "sans-serif" | "serif" | "monospace" | "display" | "handwriting";

export type FontChoice = {
	def: FontFamilyDef,
	preview: string,
	stack: string,
	selected: boolean,
};

export type FontChoices = {
	project: FontChoice[],
	builtIn: FontChoice[],
	library: FontChoice[],
	current?: FontChoice,
};

export type FontFaceDef = {
	file: string,
	weight: string,
	style: FontStyle,
	format: FontFormat,
	unicodeRange?: string,
};

export type FontFamilyDef = {
	id: string,
	family: string,
	category: FontCategory,
	source: FontSource,
	license?: string,
	licenseFile?: string,
	faces: FontFaceDef[],
	hidden?: boolean,
};

export type FontFormat = "woff2" | "woff" | "truetype" | "opentype";

export type FontRole = "main" | "heading" | "code";

export type FontSource = "bundled" | "online" | "file" | "installed";

export type FontStyle = "normal" | "italic";

export type FootageChain = {
	grade?: StyleEntry[],
	redact?: StyleEntry[],
};

export type FrameRange = {
	from: number,
	to: number,
};

export type FrameTheme = "light" | "dark";

export type Gradient = {
	kind: GradientKind,
	angle: number,
	stops: GradientStop[],
};

export type GradientEdit = { op: "kind"; kind: GradientKind } | { op: "angle"; angle: number } | { op: "stopColor"; index: number; color: string } | { op: "stopAt"; index: number; at: number } | { op: "addStop"; at: number } | { op: "removeStop"; index: number } | { op: "sort"; index: number };

export type GradientKind = "linear" | "radial";

export type GradientStop = {
	color: string,
	at: number,
};

export type GradientView = {
	gradient: Gradient,
	css: string,
	strip: string,
	canAdd: boolean,
	canRemove: boolean,
	selected?: number,
};

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
	shiftPitch?: boolean,
	loopEvery?: number,
	crop?: Rect,
	cropFill?: boolean,
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

export type Hits = {
	clicks: number,
	keys: number,
	transitions: number,
};

export type Icon = "app-window" | "arrow-right-left" | "audio-lines" | "blend" | "bookmark" | "captions" | "check" | "circle-check" | "circle-dot" | "clapperboard" | "clock" | "code" | "code-xml" | "columns-2" | "combine" | "command" | "crop" | "crosshair" | "eye" | "eye-off" | "fast-forward" | "file" | "file-play" | "file-video" | "film" | "flashlight" | "gauge" | "git-branch" | "globe" | "grab" | "hard-drive" | "hourglass" | "image" | "image-play" | "info" | "key-round" | "keyboard" | "languages" | "laptop" | "layers" | "link" | "list" | "list-checks" | "list-video" | "locate-fixed" | "map" | "maximize" | "message-square" | "monitor" | "moon" | "mouse-pointer" | "mouse-pointer-2" | "mouse-pointer-click" | "move" | "move-up-right" | "music" | "palette" | "presentation" | "ratio" | "refresh-cw" | "repeat" | "reply" | "rocket" | "ruler" | "scan" | "scan-search" | "scissors" | "scroll" | "scroll-text" | "server" | "settings-2" | "shapes" | "shield-alert" | "shield-check" | "smartphone" | "sparkles" | "spline" | "square-dashed" | "square-terminal" | "sticky-note" | "sun" | "timer" | "timer-off" | "triangle-alert" | "type" | "upload" | "video" | "volume-2" | "volume-x" | "wallpaper" | "wifi" | "wind" | "zoom-in" | "zoom-out";

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
	ports: PortChoice[],
	fileKinds: { [key in string]: MediaKind },
	takeKind: TakeKind | null,
};

export type InstalledFamily = {
	family: string,
	faces: number,
	stack: string,
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

export type KeyNames = "symbols" | "words";

export type KeyframeEdit = { op: "addKey"; nodeId: string; track: EnvelopeKey; frame: number } | { op: "appendKey"; nodeId: string; track: EnvelopeKey } | { op: "setKey"; nodeId: string; track: EnvelopeKey; index: number; frame?: number; value?: number; ease?: Ease } | { op: "removeKey"; nodeId: string; track: EnvelopeKey; index: number } | { op: "addCameraKey"; nodeId: string; frame: number } | { op: "setCameraKey"; nodeId: string; index: number; frame?: number; yaw?: number; pitch?: number; distance?: number; height?: number; ease?: Ease } | { op: "removeCameraKey"; nodeId: string; index: number };

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
	chordHoldS?: number,
	typedHoldS?: number,
	edge?: number,
	radius?: number,
	keyNames?: KeyNames,
	font?: string,
	shadow?: number,
	caret?: boolean,
};

export type KeystrokePosition = "bottom" | "top";

export type LabelLook = {
	fontWeight: number,
	boxPadding: number,
	radius: number,
	pointerSize: number,
	shadow: boolean,
	arrowSize: number,
	lineWidth: number,
	tilt: number,
};

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

export type LiveWindowWatching = boolean;

export type LocalFrames = {
	first: number,
	values: (number)[],
};

export type LogLevel = "debug" | "info" | "warn" | "error";

export type LogLine = {
	id: number,
	at: number,
	level: LogLevel,
	nodeId?: string,
	message: string,
};

export type LookIn = "line" | "screen";

export type LoudnessChoice = {
	target: LoudnessTarget,
	label: string,
	goal: LoudnessGoal | null,
};

export type LoudnessGoal = {
	lufs: number,
	truePeak: number,
};

export type LoudnessTarget = "off" | "web" | "podcast" | "broadcast";

export type Loudness = {
	fps: number,
	momentary: (number)[],
	shortTerm: (number)[],
	integrated: number,
	range: number,
	truePeak: number,
};

export type MarkerEdit = { op: "add"; atMs: number } | { op: "update"; id: string; atMs?: number; label?: string } | { op: "remove"; id: string };

export type Marker = {
	id: string,
	atMs: number,
	label: string,
};

export type MasterPatch = {
	fader?: number,
	eq?: Eq,
	comp?: Comp,
	limiter?: Limiter,
	loudness?: LoudnessTarget,
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

export type Measured = { type: "noiseLevel"; beforeDb: number; afterDb: number } | { type: "file"; bytes: number; fps?: number } | { type: "changes"; count: number } | { type: "fitted"; tempo?: number; loops: number } | { type: "tape"; steps: number; ignored: string[]; warnings: string[] } | { type: "cast"; events: number };

export type MediaKind = "video" | "audio" | "image";

export type MediaValue = {
	src: string | null,
	kind: MediaKind,
	durationMs?: number,
	width?: number,
	height?: number,
	hasAudio?: boolean,
};

export type MenuCommand = "project:new" | "project:open" | "project:save" | "edit:undo" | "edit:redo" | "edit:group" | "edit:select-all" | "edit:layout" | "edit:add-node" | "run:start" | "run:cancel" | "view:settings" | "view:projects" | "view:toggle-left" | "view:toggle-bottom" | "view:toggle-right" | "view:toggle-minimap" | "view:focus-timeline" | "view:focus-mixer" | "view:focus-run";

export type MenuCommandFired = MenuCommand;

export type MenuEntry = { kind: "command"; label: string; command: MenuCommand; accelerator?: string } | { kind: "role"; label: string; role: WindowRole; accelerator?: string } | { kind: "separator" };

export type MenuSection = {
	label: string,
	items: MenuEntry[],
};

export type MixEdit = { op: "channels"; ids: string[]; patch: StripPatch } | { op: "resetChannels"; ids: string[] } | { op: "bus"; id: string; patch: BusPatch } | { op: "resetBus"; id: string } | { op: "addBus" } | { op: "removeBus"; id: string } | { op: "master"; patch: MasterPatch };

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

export type MixerBook = {
	sampleRate: number,
	minDb: number,
	silenceDb: number,
	maxDb: number,
	fader: NumberRange,
	trim: NumberRange,
	pan: NumberRange,
	highpass: NumberRange,
	limiterCeiling: NumberRange,
	comp: CompRanges,
	duck: DuckRanges,
	eqBand: EqBandBounds,
	eqTypes: EqType[],
	defaultStrip: Strip,
	defaultMaster: Master,
	builtInBuses: string[],
	loudness: LoudnessChoice[],
	presets: StripPreset[],
};

export type MouseButton = "none" | "left" | "middle" | "right" | "back" | "forward";

export type NodeFailure = {
	message: string,
	at: number,
	traceId?: string,
};

export type NodeLook = {
	glyph: string,
	motion?: NodeMotion,
	morph?: string,
};

export type NodeMotion = "blur-swap" | "bob" | "caret" | "clock-wipe" | "collapse" | "dissolve" | "draw" | "fade-through" | "flare" | "flicker" | "flip" | "glitch" | "grow" | "hard-cut" | "iris" | "march" | "mask-up" | "nudge-down" | "nudge-right" | "nudge-up" | "open" | "pan" | "press" | "push" | "rack-focus" | "ripple" | "rise" | "roll" | "shimmer" | "shutter" | "slide-through" | "smear" | "snap" | "stretch" | "track" | "tracking" | "turn" | "turn-over" | "twist" | "whip" | "wipe-reveal" | "zoom-blur" | "zoom-through";

export type NodePatch = {
	label?: string,
	disabled?: boolean,
	position?: Position,
	size?: Viewport,
	params?: { [key in string]: unknown },
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
	measured?: Measured[],
};

export type NodeRunStatus = "idle" | "queued" | "running" | "cached" | "done" | "error" | "skipped";

export type NumberRange = {
	min: number,
	max: number,
	default: number,
};

export type OpenProject = {
	dir: string,
	document: DocumentState,
	lastRun: LastRun | null,
	view: { [key in string]: unknown },
};

export type OpenRequested = {
	dir: string,
};

export type PageAddress = {
	t: number,
	url: string,
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

export type PlaybackCall = { call: "seek"; frame: number } | { call: "play" } | { call: "pause" } | { call: "rate"; rate: number };

export type PlaybackCalled = PlaybackCall;

export type PlayheadShared = {
	from: string,
	playhead: Playhead,
};

export type Playhead = {
	frame: number,
	playing: boolean,
	rate: number,
	fps: number,
	durationInFrames: number,
	inPoint?: number,
	outPoint?: number,
};

export type Point = {
	x: number,
	y: number,
};

export type PointerButton = "left" | "right" | "middle";

export type PointerMove = "press" | "release" | "move";

export type PointerProps = {
	shown: boolean,
	scale: number,
	look: CursorLook,
	feel?: CursorFeel,
	ripple: RippleLook | null,
};

export type PortChoice = {
	port: string,
	label: string,
};

export type PortLink = {
	from: PortType,
	to: PortType,
};

export type PortType = "browser-session" | "terminal-session" | "recording" | "recording-style" | "grade" | "clip" | "audio" | "captions" | "image" | "media";

export type Position = {
	x: number,
	y: number,
};

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

export type ProjectReloaded = null;

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
	log: ActionLog,
	name: string,
	speed: number,
	camera: CameraProps,
	shot: ShotProps,
	window: WindowProps,
	pointer: PointerProps,
	keystrokes?: KeystrokeLook,
	annotations: AnnotationSpec[],
	extraBeats: Action[],
	screen: ScreenProps,
};

export type RecordingValue = {
	log: ActionLog | null,
	durationMs: number,
	nodeId: string,
	edits: StyleEntry[],
	source: TakeSource,
};

export type Rect = {
	x: number,
	y: number,
	w: number,
	h: number,
};

export type ReframeMode = "follow" | "center" | "fit";

export type Removal = {
	nodeIds: string[],
	edgeIds: string[],
};

export type ReplayRequested = {
	nodeId: string | null,
	tab: ReplayTab,
};

export type ReplayTab = "replay" | "trace";

export type RippleLook = {
	color: string,
	size: number,
	width: number,
	ringOpacity?: number,
	fillOpacity?: number,
	durationS?: number,
};

export type RunEventFired = RunEvent;

export type RunEvent = { type: "run:start"; runId: string; nodeIds: string[]; at: number } | { type: "node:start"; runId: string; nodeId: string; at: number } | { type: "node:progress"; runId: string; nodeId: string; progress: number; message?: string } | { type: "node:log"; runId: string; nodeId?: string; level: LogLevel; message: string; at: number } | { type: "node:done"; runId: string; nodeId: string; cached: boolean; result?: NodeResult } | { type: "node:error"; runId: string; nodeId: string; message: string; stack?: string; traceId?: string } | { type: "node:skipped"; runId: string; nodeId: string; reason: string } | { type: "run:done"; runId: string; ok: boolean; cancelled: boolean; error?: string; failedNodeIds?: string[]; skippedNodeIds?: string[]; outputs: string[]; at: number };

export type RunLogged = LogLine;

export type RunProgress = {
	runId: string | null,
	statuses: { [key in string]: NodeRunStatus },
	progress: { [key in string]: NodeProgress },
	cached: { [key in string]: boolean },
	errors: { [key in string]: string },
	traces: { [key in string]: string },
	results: { [key in string]: NodeResult },
};

export type RunStateReset = RunState;

export type RunState = {
	view: RunView,
	logs: LogLine[],
};

export type RunTargets = {
	targetNodeIds?: string[],
	fromNodeId?: string,
};

export type RunViewChanged = RunView;

export type RunView = {
	seq: number,
	progress: RunProgress,
	running: boolean,
	outputs: string[],
	lastError: string | null,
	failedNodeIds: string[],
	skippedNodeIds: string[],
	exports: Artifact[],
};

export type ScreenProps = { kind: "footage"; src: string; chain?: FootageChain; footageUrl?: string } | { kind: "terminal"; cast: string; cols: number; rows: number; look: TerminalLook; chain?: FootageChain; framesUrl?: string; effects?: TerminalEffects };

export type SelectionShared = {
	from: string,
	selection: Selection,
};

export type Selection = {
	selectedIds: string[],
	activeIds: string[],
	pinnedSource: string | null,
};

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
	type: BrowserAction,
	label?: string,
	params: { [key in string]: unknown },
};

export type SettingsBook = {
	defaults: AppSettings,
	startOptions: ([StartOption, string])[],
	traceRetention: ([TraceRetention, string])[],
	canvasModes: CanvasMode[],
	keepLastTraces: WholeRange,
	liveFps: WholeRange,
};

export type SettingsChanged = AppSettings;

export type SettingsPage = "general" | "projects" | "engine" | "recording" | "fonts" | "api" | "licences" | "about";

export type SettingsPageShown = SettingsPage;

export type SettingsPatch = {
	width?: number,
	height?: number,
	fps?: number,
	background?: string,
	theme?: Theme,
	fonts?: ProjectFonts,
};

export type ShapeChoice = {
	shape: Variant,
	canvas: Viewport,
};

export type SharedState = {
	selection: Selection,
	playhead: Playhead | null,
};

export type ShellChoice = "clean" | "own";

export type ShotProps = {
	style: CameraStyle,
	backdrop: string,
	backdropChosen?: boolean,
	inDevice?: boolean,
	windowFit?: number,
	ground?: string,
	radius?: number,
	rimGlow?: number,
	liftColours?: boolean,
	bleedBlur?: number,
	bleedDim?: number,
	motionBlur: boolean,
	shutterAngle?: number,
};

export type Slot = {
	nodeId: string,
	port: string,
};

export type SoundChoice = {
	reference: string,
	label: string,
	hint?: string,
	durationMs: number,
};

export type SoundPlacement = {
	src: string,
	at: number,
	offset: number,
	length: number,
	loop: boolean,
	rate: number,
	pitch?: number,
	volume: number,
	fadeIn: number,
	fadeOut: number,
	fadeCurve?: FadeCurve,
	envelope?: AudioEnvelope,
};

export type SoundWindow = {
	from: number,
	to: number,
	contentStart: number,
};

export type SpecAssets = {
	projectBase?: string,
};

export type SpotBackground = {
	src: string,
	timeMs: number,
	durationMs: number,
	still: boolean,
	terminal: TerminalTheme | null,
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

export type StripPatch = {
	trim?: number,
	highpass?: Highpass,
	eq?: Eq,
	comp?: Comp,
	fader?: number,
	pan?: number,
	mute?: boolean,
	solo?: boolean,
	bus?: string,
};

export type StripPreset = {
	id: string,
	label: string,
	strip: StripPatch,
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

export type TakeKind = "browser" | "terminal";

export type TakeSource = {
	kind: "browser",
} & BrowserSource | {
	kind: "terminal",
} & TerminalSource;

export type TemplateInfo = {
	id: string,
	title: string,
	description: string,
	icon: Icon,
};

export type TerminalAction = { type: "run"; command: string; wait: WaitFor; limitMs: number; zoomOutput: boolean } | { type: "type"; text: string; speedMs?: number } | { type: "keys"; keys: string; repeat: number } | { type: "click"; target: CellTarget; button: PointerButton; double: boolean } | { type: "drag"; from: CellTarget; to: CellTarget } | { type: "wheel"; target: CellTarget; notches: number } | { type: "wait-text"; pattern: string; scope: LookIn; limitMs: number } | { type: "wait-prompt"; limitMs: number } | { type: "sleep"; ms: number } | { type: "start-recording" } | { type: "hide" } | { type: "show" } | { type: "chapter"; title: string } | { type: "clear" } | { type: "scroll"; lines: number } | { type: "copy"; text: string } | { type: "paste" } | { type: "env"; name: string; value: string } | { type: "require"; program: string } | { type: "screenshot" };

export type TerminalCursor = "program" | "block" | "bar" | "underline";

export type TerminalEffects = {
	soften: number,
	vignette: number,
	vignetteX: number,
	vignetteY: number,
	vignetteFollowsFrame: boolean,
	grain: number,
	colourGrain: boolean,
	fringe: number,
	fringeAngle: number,
	bulge: number,
	bulgeX: number,
	bulgeY: number,
	trails: number,
};

export type TerminalInput = { type: "key"; key: string; ctrl: boolean; alt: boolean; shift: boolean } | { type: "text"; text: string } | { type: "paste"; text: string } | { type: "pointer"; action: PointerMove; button: PointerButton | null; col: number; row: number } | { type: "wheel"; col: number; row: number; notches: number } | { type: "focus"; gained: boolean };

export type TerminalLook = {
	theme: TerminalTheme,
	fontSize: number,
	lineHeight: number,
	letterSpacing: number,
	padding: number,
	cursor: TerminalCursor,
	cursorBlink: boolean,
};

export type TerminalSessionValue = {
	shell: ShellChoice,
	folder: string,
	cols: number,
	rows: number,
	env: EnvVar[],
	typingMs: number,
	pointerMs: number,
	steps: TerminalStep[],
};

export type TerminalSource = {
	cast: string | null,
	cols: number,
	rows: number,
	theme?: TerminalTheme,
};

export type TerminalStep = {
	id: string,
	nodeId: string,
	label: string,
	action: TerminalAction,
	waitAfterMs: number,
};

export type TerminalTheme = {
	name: string,
	fg: string,
	bg: string,
	cursor: string,
	palette: [string, string, string, string, string, string, string, string, string, string, string, string, string, string, string, string],
};

export type TextRules = {
	variablePattern: string,
	pauseMark: string,
};

export type Theme = "light" | "dark";

export type TimelineBook = {
	eases: ([Ease, string])[],
	envelopes: EnvelopeTrack[],
	cameraDials: CameraDial[],
};

export type TimelineEdit = { op: "move"; bar: string; delta: number; ripple: boolean } | { op: "trimStart"; bar: string; delta: number } | { op: "trimEnd"; bar: string; delta: number; ripple: boolean } | { op: "slip"; bar: string; delta: number } | { op: "speed"; bar: string; delta: number } | { op: "rollStart"; bar: string; delta: number } | { op: "rollEnd"; bar: string; delta: number } | { op: "audioMove"; bar: string; delta: number; ripple: boolean } | { op: "audioTrimStart"; bar: string; delta: number } | { op: "audioTrimEnd"; bar: string; delta: number } | { op: "volume"; bar: string; volume: number } | { op: "split"; at: number; selected: string[] } | { op: "splitBar"; bar: string; at: number } | { op: "cameraKey"; key: string; delta: number } | { op: "keyframes"; edit: KeyframeEdit };

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

export type TimelineReply = {
	error?: string,
	message?: string,
};

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
	fonts?: FilmFonts,
	mix?: MasterStem,
};

export type TimelineTrack = {
	id: string,
	items: TimelineItem[],
	layer?: boolean,
};

export type TitleChange = {
	t: number,
	title: string,
};

export type Tone = "muted" | "warn";

export type ToolStatus = {
	path?: string,
	version?: string,
	problem?: string,
	warning?: string,
	checking: boolean,
};

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

export type TransitionEase = "linear" | "in" | "out" | "in-out";

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
	easing?: TransitionEase,
};

export type Value = null | boolean | number | string | Value[] | { [key in string]: Value };

export type VarInfo = {
	name: string,
	value: string,
};

export type Variant = "16:9" | "1:1" | "4:5" | "9:16";

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

export type WaitFor = "prompt" | "quiet";

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
	bpm?: number,
};

export type WholeRange = {
	min: number,
	max: number,
};

export type WindowProps = {
	chrome: boolean,
	style?: WindowStyle,
	theme?: FrameTheme,
	font?: string,
	bar?: boolean,
	text?: string,
	mode?: AddressMode,
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

export type WorkflowEdit = { op: "addNode"; type: string; id?: string; label?: string; position?: Position; params?: { [key in string]: unknown } } | { op: "updateNode"; id: string; patch: NodePatch } | { op: "moveNodes"; positions: { [key in string]: Position } } | { op: "removeNodes"; ids: string[] } | { op: "addEdge"; ends: EdgeEnds } | { op: "removeEdges"; ids: string[] } | { op: "removeParts"; nodeIds: string[]; edgeIds: string[] } | { op: "setSettings"; patch: SettingsPatch } | { op: "addProjectFont"; font: FontFamilyDef } | { op: "setFontRole"; role: FontRole; family: string } | { op: "setVariables"; variables: { [key in string]: string } } | { op: "renameVariable"; from: string; to: string } | { op: "rename"; name: string } | { op: "paste"; nodes: WorkflowNode[]; edges: WorkflowEdge[]; offset: Position } | { op: "group"; ids: string[]; title: string; sizes: { [key in string]: Viewport } } | { op: "ungroup"; id: string } | { op: "mix"; edit: MixEdit } | { op: "markers"; edit: MarkerEdit } | { op: "keyframes"; edit: KeyframeEdit } | { op: "replace"; workflow: Workflow };

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
