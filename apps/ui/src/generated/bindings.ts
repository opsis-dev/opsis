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

export type ConnectionProblem = {
	notFound: boolean,
	message: string,
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

export type ElementRef = {
	component: string,
	props: { [key in string]: unknown },
};

export type EntriesValue = {
	entries: StyleEntry[],
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

export type Limiter = {
	on: boolean,
	ceiling: number,
};

export type LinkedParam = {
	param: string,
	sign: 1 | -1,
	ifNonZero?: boolean,
};

export type LogLevel = "debug" | "info" | "warn" | "error";

export type LoudnessTarget = "off" | "web" | "podcast" | "broadcast";

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

export type MixPlan = {
	fps: number,
	durationInFrames: number,
	channels: ChannelPlan[],
	buses: BusPlan[],
	master: Master,
	key: string,
};

export type Mix = {
	channels: { [key in string]: Strip },
	buses: Bus[],
	master: Master,
};

export type NodeFailure = {
	message: string,
	at: number,
	traceId?: string,
};

export type NodeResult = {
	hash: string,
	outputs: { [key in string]: CompileValue },
	artifacts: Artifact[],
	finishedAt: number,
	durationMs: number,
	traceId?: string,
};

export type ParamRef = {
	nodeId: string,
	param: string,
	unit: ParamUnit,
	scale?: number,
	index?: number,
	linked?: LinkedParam[],
};

export type ParamUnit = "ms" | "frames" | "rate" | "number";

export type PictureLook = {
	brightness?: number,
	contrast?: number,
	saturation?: number,
	warmth?: number,
	vignette?: number,
	grain?: number,
};

export type Point = {
	x: number,
	y: number,
};

export type PortType = "browser-session" | "recording" | "recording-style" | "grade" | "clip" | "audio" | "captions" | "image" | "media" | "file-info" | "folder";

export type Position = {
	x: number,
	y: number,
};

export type PreviewQuality = "fast" | "exact";

export type PreviewTarget = {
	spec: TimelineSpec,
	spans: { [key in string]: FrameRange },
	sourceNodeId: string,
};

export type ProjectFonts = {
	main: string,
	heading: string,
	code: string,
	families: FontFamilyDef[],
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

export type RippleLook = {
	color: string,
	size: number,
	width: number,
};

export type RunEvent = { type: "run:start"; runId: string; nodeIds: string[]; at: number } | { type: "node:start"; runId: string; nodeId: string; at: number } | { type: "node:progress"; runId: string; nodeId: string; progress: number; message?: string } | { type: "node:log"; runId: string; nodeId?: string; level: LogLevel; message: string; at: number } | { type: "node:done"; runId: string; nodeId: string; cached: boolean; result?: NodeResult } | { type: "node:error"; runId: string; nodeId: string; message: string; stack?: string; traceId?: string } | { type: "node:skipped"; runId: string; nodeId: string; reason: string } | { type: "run:done"; runId: string; ok: boolean; cancelled: boolean; error?: string; failedNodeIds?: string[]; skippedNodeIds?: string[]; outputs: string[]; at: number };

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

export type SrcValue = {
	src: string | null,
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

export type TaggedRecording = {
	type: "recording",
} & RecordingValue;

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
