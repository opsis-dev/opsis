export type ActionKind = "click" | "type" | "hover" | "scroll" | "navigate" | "wait" | "chapter" | "key" | "focus" | "drag";

export type ActionLog = {
	viewport: Viewport,
	dpr: number,
	t0: number,
	durationMs: number,
	actions: Action[],
	cursorTrack?: CursorSample[],
	addresses?: PageAddress[],
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

export type AgentRunRunStep = {
	name: string,
	detail?: string,
	duration: number,
	icon?: AgentStepIcon,
	sources?: string[],
};

export type AgentRun = {
	theme: Theme,
	accentColor: string,
	compression: number,
	seed: number,
	runTitle: string,
	runPrompt: string,
	runPlan: string[],
	runSteps: AgentRunRunStep[],
	runAnswer: string,
	speed: number,
	thinkingLabel: string,
	thoughtLabel: string,
	showTimer: boolean,
	cornerRadius: number,
	font: string,
};

export type AgentStepIcon = "chart" | "database" | "search" | "book" | "globe" | "terminal" | "file";

export type AiPromptFlow = {
	theme: Theme,
	prompt: string,
	buttonLabel: string,
	answerLines: string[],
	toastTitle: string,
	typingSeconds: number,
	loadingSeconds: number,
	noticeSeconds: number,
	speed: number,
	fieldWidth: number,
	answerWidth: number,
	answerSize: number,
	fontFamily: string,
	buttonVariant: ButtonVariant,
	noticeCorner: Corner,
	backgroundColor: string,
	radius: number,
};

export type Anchor = {
	nodeId?: string,
	clipEnd?: boolean,
	keepEnd?: boolean,
};

export type AnimatedBarChartBar = {
	label: string,
	value: number,
};

export type AnimatedBarChart = {
	barColor: string,
	gap: number,
	staggerFrames: number,
	bars: AnimatedBarChartBar[],
	width: number,
	height: number,
	baselineColor: string,
	labelColor: string,
	speed: number,
	cornerRadius: number,
	glow: number,
	labelSize: number,
	fontFamily: string,
};

export type AnimatedLineChart = {
	strokeColor: string,
	strokeWidth: number,
	gridColor: string,
	showDot: boolean,
	data: (number)[],
	width: number,
	height: number,
	speed: number,
	startAtZero: boolean,
	finishAt: number,
	gridRows: number,
	showColumns: boolean,
	glow: number,
};

export type AnimationFit = "contain" | "cover";

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

export type AnnounceTitle = {
	eyebrow: string,
	title: string,
	tagline: string,
	symbolPath: string,
	symbolColors: Colour[],
	symbolScale: number,
	fieldColor: string,
	titleColor: string,
	paperColor: string,
	voidColor: string,
	nightColor: string,
	glowColor: string,
	glowStrength: number,
	taglineColor: string,
	inkColor: string,
	fontFamily: string,
	speed: number,
	fontWeight: number,
	textScale: number,
	symbolGlow: number,
	taglineTint: Colour[],
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

export type AuroraBg = {
	backgroundColor: string,
	colors: Colour[],
	ribbonCount: number,
	amplitude: number,
	thickness: number,
	centerY: number,
	spread: number,
	speed: number,
	blur: number,
	intensity: number,
	horizonGlow: number,
	striation: number,
	seed: number,
	folds: number,
	sidewaysDrift: number,
	raySpacing: number,
	vignette: number,
	horizonGlowColor: string,
};

export type Backdrop = "glaze" | "ink" | "cobalt" | "graphite" | "ember" | "flare" | "prism" | "bloom" | "halo" | "dusk" | "studio" | "moonrise" | "canyon" | "mist" | "chalk" | "aurora";

export type BarChartRaceSerie = {
	label: string,
	values: (number)[],
	color?: string,
};

export type BarChartRace = {
	series: BarChartRaceSerie[],
	steps: string[],
	framesPerStep: number,
	visibleRows: number,
	width: number,
	rowHeight: number,
	gap: number,
	labelWidth: number,
	showStepLabel: boolean,
	colors: Colour[],
	inkColor: string,
	barInkColor: string,
	labelColor: string,
	numberFormat: NumberFormat,
	cornerRadius: number,
	stepLabelPosition: Corner,
	stepLabelSize: number,
	stepLabelOpacity: number,
	fontFamily: string,
};

export type BlendMode = "multiply" | "screen" | "normal";

export type Bounds = {
	x: number,
	y: number,
	width: number,
	height: number,
};

export type ButtonVariant = "default" | "secondary" | "destructive" | "outline" | "ghost";

export type Callout = {
	text: string,
	x: number,
	y: number,
	side: Side,
	color: string,
	accent: string,
	fontFamily: string,
	drawTime: number,
	labelDistance: number,
	bend: number,
	arrowWidth: number,
	dotSize: number,
	labelRadius: number,
	fontSize: number,
	fontWeight: number,
	slant: number,
	fadeOut: number,
	labelBackground: string,
};

export type CameraEasing = "smooth" | "gentle" | "snappy" | "linear";

export type CameraKey = {
	frame: number,
	yaw: number,
	pitch: number,
	distance: number,
	height: number,
	ease: Ease,
};

export type CameraLens = {
	softness: number,
	bloom: number,
	aberration: number,
	vignette: number,
	distortion: number,
	sharpCentre: number,
	bloomThreshold: number,
};

export type CameraPath = {
	keys: CameraKey[],
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

export type Caption = {
	text: string,
	startMs: number,
	endMs: number,
};

export type CaptionPosition = "bottom" | "center" | "top";

export type CaptionStyle = "word-by-word" | "karaoke" | "line";

export type CaptionsValue = {
	captions: Caption[],
};

export type Captions = {
	style: CaptionStyle,
	position: CaptionPosition,
	fontSize: number,
	color: string,
	highlightColor: string,
	panel: string,
	fontWeight: number,
	fontFamily: string,
	wordsPerLine: number,
	lingerMs: number,
	edgeOffset: number,
	sideMargin: number,
	panelPadding: number,
	panelRadius: number,
	textShadow: boolean,
	unspokenOpacity: number,
};

export type CapturedVideoCamera = {
	at: number,
	duration: number,
	zoom: number,
	x: number,
	y: number,
};

export type CapturedVideoEntrance = "fade" | "none";

export type CapturedVideoFit = "cover" | "contain";

export type CapturedVideo = {
	src: string,
	cropTop: number,
	cropRight: number,
	cropBottom: number,
	cropLeft: number,
	fit: CapturedVideoFit,
	backdropColor: string,
	entrance: CapturedVideoEntrance,
	radius: number,
	trimBefore: number,
	audio: boolean,
	theme: Theme,
	camera: CapturedVideoCamera[],
	speed: number,
	sourceAspect?: number,
};

export type Card3d = {
	src: string,
	cornerRadius: number,
	depth: number,
	tilt: number,
	motion: Motion,
	camera: CameraPath,
	speed: number,
	lighting: Lighting,
	background: string,
	zoom: number,
	edgeColor: string,
	roughness: number,
	shadow: number,
	growIn: boolean,
	endFacing: boolean,
	motionAmount: number,
	reverse: boolean,
	lens: number,
};

export type ChangeKind = "added" | "fixed" | "changed" | "removed";

export type ChangelogEntryChange = {
	kind: ChangeKind,
	text: string,
};

export type ChangelogEntry = {
	changes: ChangelogEntryChange[],
	version: string,
	date: string,
	summary: string,
	startAtSeconds: number,
	staggerSeconds: number,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	kindAdded: string,
	kindFixed: string,
	kindChanged: string,
	kindRemoved: string,
	addedLabel: string,
	fixedLabel: string,
	changedLabel: string,
	removedLabel: string,
	cardWidth: number,
	fontFamily: string,
};

export type CheckListItem = {
	text: string,
	checked: boolean,
};

export type CheckList = {
	fontSize: number,
	itemGap: number,
	closeGap: number,
	perStep: number,
	strokeWidth: number,
	color: string,
	tickColor: string,
	step: number,
	items: CheckListItem[],
	width: number,
	boxColor: string,
	weight: FontWeight,
	rowGap?: number,
	seed: string,
	background: string,
	wobble: number,
	fontFamily: string,
	strikeColor: string,
};

export type ChromaticAberrationWipe = {
	intensity: number,
	axis: Orientation,
	slide: number,
	fade: boolean,
};

export type ClickButton = "left" | "right";

export type CodeDiffWipe = {
	before: string,
	after: string,
	title: string,
	wipeSeconds: number,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	fontSize: number,
	windowWidth: number,
	lineHeight: number,
	startAt: number,
	collapseRemoved: boolean,
	showCounts: boolean,
	dimUnchanged: boolean,
	strikeRemoved: boolean,
	glow: boolean,
	vignette: boolean,
	cornerRadius: number,
	windowDots: boolean,
	font: string,
};

export type CodeMorphStep = {
	code: string,
	at: number,
};

export type CodeMorph = {
	filename: string,
	theme: Theme,
	accentColor: string,
	morphDuration: number,
	fontSize: number,
	highlight: boolean,
	lineNumbers: boolean,
	windowDots: boolean,
	steps: CodeMorphStep[],
	speed: number,
	lineHeight: number,
	cornerRadius: number,
	maxSize: number,
	highlightHold: number,
	font: string,
	backgroundColor: string,
};

export type CodeReveal = {
	code: string,
	highlightedLines: (number)[],
	title: string,
	language: string,
	startLine: number,
	showLineNumbers: boolean,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	fontSize: number,
	windowWidth: number,
	lineHeight: number,
	visibleLines: number,
	typingSpeed: number,
	linePause: number,
	startAt: number,
	highlightAfter: number,
	dimOthers: number,
	glow: boolean,
	vignette: boolean,
	cornerRadius: number,
	windowDots: boolean,
	font: string,
};

export type Colour = string;

export type CommandMenuItem = {
	icon?: MenuIcon,
	label: string,
	shortcut?: string,
};

export type CommandMenu = {
	state: MenuState,
	query: string,
	revealCount?: number,
	selectedIndex: number,
	highlightedIndex: number,
	theme: Theme,
	items: CommandMenuItem[],
	autoplay: boolean,
	pressedIndex: number,
	letterSeconds: number,
	pauseSeconds: number,
	speed: number,
	pickIndex: number,
	placeholder: string,
	emptyText: string,
	backdropOpacity: number,
	panelWidth: number,
	top: number,
	fontFamily: string,
	radius: number,
};

export type CommitGraphCommit = {
	message: string,
	hash: string,
	lane?: number,
	parent?: number,
	mergeFrom?: number,
	ref?: string,
};

export type CommitGraph = {
	commits: CommitGraphCommit[],
	windowTitle: string,
	startAtSeconds: number,
	stepSeconds: number,
	accentColor: string,
	laneColors: Colour[],
	theme: Theme,
	speed: number,
	backgroundColor: string,
	codeFont: string,
	font: string,
	windowWidth: number,
	rowSpacing: number,
	laneSpacing: number,
	dotSize: number,
	lineWidth: number,
	textSize: number,
	cornerRadius: number,
	windowDots: boolean,
};

export type ComparisonBarsRow = {
	label: string,
	before: number,
	after: number,
	delta?: string,
};

export type ComparisonBars = {
	rows: ComparisonBarsRow[],
	width: number,
	rowHeight: number,
	gap: number,
	labelWidth: number,
	beforeColor: string,
	afterColor: string,
	downColor: string,
	seriesLabels: string[],
	showDelta: boolean,
	durationInFrames: number,
	staggerInFrames: number,
	pairOffsetInFrames: number,
	barInkColor: string,
	labelColor: string,
	numberFormat: NumberFormat,
	cornerRadius: number,
	showBeforeValue: boolean,
	fontFamily: string,
};

export type ComparisonTableRow = {
	label: string,
	cells: string[],
};

export type ComparisonTable = {
	columns: string[],
	rows: ComparisonTableRow[],
	title: string,
	highlight: boolean,
	highlightColumn: number,
	highlightLabel: string,
	startAtSeconds: number,
	rowStaggerSeconds: number,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	tickColor: string,
	labelWidth: number,
	cardWidth: number,
	fontFamily: string,
	uppercaseHeadings: boolean,
};

export type Confetti = {
	particleCount: number,
	power: number,
	gravity: number,
	size: number,
	seed: number,
	colors: Colour[],
	originX: number,
	originY: number,
	lifetime: number,
	spread: number,
	direction: number,
	spin: number,
	pieceShape: number,
};

export type Corner = "bottom-left" | "bottom-right" | "top-left" | "top-right";

export type CrtScreen = {
	curvature: number,
	scanlines: number,
	maskScale: number,
	vignette: number,
	scanlineCount: number,
	mask: number,
	glow: number,
	flicker: number,
	bezelColor: string,
};

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

export type CursorVariant = "arrow" | "pointer";

export type DataStoryBarData = {
	label: string,
	value: number,
	delta?: string,
	color?: string,
};

export type DataStoryMetric = {
	label: string,
	value: number,
	delta: string,
	trend: (number)[],
	suffix?: string,
	prefix?: string,
};

export type DataStoryStep = {
	title: string,
	description: string,
};

export type DataStory = {
	title: string,
	subtitle: string,
	barData: DataStoryBarData[],
	metrics: DataStoryMetric[],
	steps: DataStoryStep[],
	chartTitle: string,
	metricsTitle: string,
	timelineTitle: string,
	insight: string,
	insightEyebrow: string,
	ctaTitle: string,
	ctaLabel: string,
	theme: Theme,
	accentColor: string,
	dataColor: string,
	timelineColor: string,
	backgroundColor: string,
	showScale: boolean,
	highlightBar: string,
	chartSubtitle: string,
	numberFormat: NumberFormat,
	textSize: number,
	logoSrc: string,
	ctaUrl: string,
	ctaSubtitle: string,
	eyebrowCaps: boolean,
	glow: number,
	fontFamily: string,
};

export type Design = {
	width: number,
	height: number,
	exact?: boolean,
};

export type DeviceMockupZoom = {
	src: string,
	title: string,
	subtitle: string,
	eyebrow: string,
	device: MockupDevice,
	backgroundColor: string,
	accentColor: string,
	textColor: string,
	deviceColor: string,
	deviceEdgeColor: string,
	bezelColor: string,
	buttonColor: string,
	cameraColor: string,
	islandColor: string,
	lensColor: string,
	baseGradient: string,
	secondAccent: string,
	thirdAccent: string,
	screenColor: string,
	screenTextColor: string,
	screenMutedColor: string,
	backgroundTint: string,
	shadow: number,
	glow: number,
	showGrid: boolean,
	pullBackFrom: number,
	scale: number,
	titleSize: number,
	titleWeight: number,
	subtitleSize: number,
	fontFamily: string,
};

export type Direction = "from-left" | "from-right" | "from-top" | "from-bottom";

export type DitherDissolve = {
	shape: DitherShape,
	colorBack: string,
	colorFront: string,
	speed: number,
	dotSize: number,
	pattern: DitherPattern,
};

export type DitherPattern = "random" | "2x2" | "4x4" | "8x8";

export type DitherShape = "simplex" | "warp" | "dots" | "wave" | "ripple" | "swirl" | "sphere";

export type DonutChartSegment = {
	label: string,
	value: number,
	color?: string,
	delta?: string,
};

export type DonutChart = {
	segments: DonutChartSegment[],
	size: number,
	thickness: number,
	colors: Colour[],
	showLegend: boolean,
	showTotal: boolean,
	totalLabel: string,
	durationInFrames: number,
	staggerInFrames: number,
	trackColor: string,
	labelColor: string,
	inkColor: string,
	upColor: string,
	downColor: string,
	numberFormat: NumberFormat,
	segmentGap: number,
	roundEnds: boolean,
	startAngle: number,
	uppercaseLabel: boolean,
	legendShows: LegendValue,
	fontFamily: string,
};

export type DragDropFlow = {
	fileName: string,
	fileSize: string,
	siblings: string[],
	sourceLabel: string,
	label: string,
	hint: string,
	activeLabel: string,
	doneLabel: string,
	accentColor: string,
	theme: Theme,
	speed: number,
	uploadingLabel: string,
	backgroundColor: string,
	uploadSeconds: number,
	showPercent: boolean,
	draggedRow: number,
	fontFamily: string,
	cursorColor: string,
};

export type Duck = {
	level: number,
	fadeFrames: number,
	keys: string[],
	spans: ([number, number])[],
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

export type ElementSettings = {
	placeholder: PlaceholderProps,
	still: StillProps,
	title: Title,
	"kinetic-headline": KineticHeadline,
	"lower-third": LowerThird,
	callout: Callout,
	captions: Captions,
	image: Image,
	mosaic: Mosaic,
	lottie: Lottie,
	video: Video,
	solid: Solid,
	gradient: Gradient,
	noise: Noise,
	starburst: Starburst,
	paper: Paper,
	backdrop: StudioBackdrop,
	"video-loop": VideoLoop,
	"light-leak": LightLeak,
	"logo-3d": Logo3d,
	"card-3d": Card3d,
	"text-3d": Text3d,
	"model-3d": Model3d,
	"screen-3d": Screen3d,
	"shapes-3d": Shapes3d,
	fade: NoSettings,
	slide: Slide,
	wipe: Wipe,
	flip: Flip,
	"clock-wipe": NoSettings,
	iris: NoSettings,
	"soft-blur-in": SoftBlurIn,
	typewriter: Typewriter,
	"shimmer-sweep": ShimmerSweep,
	"matrix-decode": MatrixDecode,
	"slot-machine-roll": SlotMachineRoll,
	"staggered-fade-up": StaggeredFadeUp,
	"kinetic-center-build": KineticCenterBuild,
	"mask-reveal-up": MaskRevealUp,
	"tracking-in": TrackingIn,
	"word-push": WordPush,
	"rgb-glitch-text": RgbGlitchText,
	"gradient-scale-cut-text": GradientScaleCutText,
	"strikethrough-replace": StrikethroughReplace,
	"line-by-line-slide": LineByLineSlide,
	"marker-highlight": MarkerHighlight,
	"inline-highlight": InlineHighlight,
	"ink-underline": InkUnderline,
	"scribble-circle": ScribbleCircle,
	"ink-arrow": InkArrow,
	keystroke: Keystroke,
	confetti: Confetti,
	"radial-burst": RadialBurst,
	"whip-pan": WhipPan,
	"push-through": PushThrough,
	"zoom-blur": ZoomBlur,
	"focus-pull": FocusPull,
	"lens-zoom": LensZoom,
	"ripple-zoom": RippleZoom,
	"wave-wipe": WaveWipe,
	"smoke-dissolve": SmokeDissolve,
	"glitch-cut": GlitchCut,
	"page-turn": PageTurn,
	"grain-dissolve": GrainDissolve,
	"dither-dissolve": DitherDissolve,
	"swirl-dissolve": SwirlDissolve,
	"particle-dissolve": ParticleDissolve,
	"shader-mesh-gradient": ShaderMeshGradient,
	"shader-grain-gradient": ShaderGrainGradient,
	"shader-warp": ShaderWarp,
	"shader-swirl": ShaderSwirl,
	"shader-god-rays": ShaderGodRays,
	"shader-perlin-noise": ShaderPerlinNoise,
	"shader-dithering": ShaderDithering,
	"crt-screen": CrtScreen,
	"vhs-filter": VhsFilter,
	"halftone-print": HalftonePrint,
	"camera-lens": CameraLens,
	polaroid: Polaroid,
	"tv-power-off": TvPowerOff,
	"speed-lines": SpeedLines,
	"terminal-simulator": TerminalSimulator,
	"glass-code-block": GlassCodeBlock,
	"code-morph": CodeMorph,
	"logo-enter": LogoEnter,
	"check-list": CheckList,
	"search-reveal": SearchReveal,
	"agent-run": AgentRun,
	"animated-bar-chart": AnimatedBarChart,
	"animated-line-chart": AnimatedLineChart,
	"rolling-number": RollingNumber,
	"spring-settle": SpringSettle,
	"signup-flow": SignupFlow,
	"ai-prompt-flow": AiPromptFlow,
	"command-menu": CommandMenu,
	"release-teaser": ReleaseTeaser,
	"launch-anything": LaunchAnything,
	"laptop-frame": LaptopFrame,
	"phone-frame": PhoneFrame,
	"captured-video": CapturedVideo,
	"announce-title": AnnounceTitle,
	"punch-lines": PunchLines,
	"text-highlight": TextHighlight,
	"status-cycle": StatusCycle,
	"logo-assemble": LogoAssemble,
	"logo-drift": LogoDrift,
	"hero-launch": HeroLaunch,
	"grid-pixelate-wipe": GridPixelateWipe,
	"frosted-glass-wipe": FrostedGlassWipe,
	"chromatic-aberration-wipe": ChromaticAberrationWipe,
	"transition-blinds": TransitionBlinds,
	"transition-morph-shape": TransitionMorphShape,
	"stat-card": StatCard,
	"metric-ticker": MetricTicker,
	"funnel-chart": FunnelChart,
	"donut-chart": DonutChart,
	"comparison-bars": ComparisonBars,
	"bar-chart-race": BarChartRace,
	"gauge-dial": GaugeDial,
	"data-story": DataStory,
	"logo-reveal": LogoReveal,
	"pricing-card": PricingCard,
	"comparison-table": ComparisonTable,
	"changelog-entry": ChangelogEntry,
	"logo-wall": LogoWall,
	"end-card": EndCard,
	"code-reveal": CodeReveal,
	"code-diff-wipe": CodeDiffWipe,
	"kanban-move": KanbanMove,
	"file-tree-reveal": FileTreeReveal,
	"drag-drop-flow": DragDropFlow,
	"commit-graph": CommitGraph,
	"notification-stack": NotificationStack,
	"device-mockup-zoom": DeviceMockupZoom,
	"multi-device-lineup": MultiDeviceLineup,
	"handwriting-text": HandwritingText,
	"neon-flicker-text": NeonFlickerText,
	"text-mask-video": TextMaskVideo,
	"aurora-bg": AuroraBg,
	"particle-field": ParticleField,
};

export type EndCard = {
	title: string,
	subtitle: string,
	eyebrow: string,
	cta: string,
	url: string,
	handles: string[],
	logoSrc: string,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	logoSize?: number,
	fontFamily: string,
	glow: number,
	textSize: number,
	titleWeight: number,
	eyebrowCaps: boolean,
	buttonRadius: number,
	buttonArrow: boolean,
};

export type Entrance = "rise" | "fade" | "blur" | "scale" | "none";

export type FadeCurve = "tri" | "qsin" | "exp" | "log";

export type FieldWidget = "action" | "asset" | "backdrop" | "boxes" | "camera" | "code" | "color" | "cut" | "datetime" | "envelope" | "font" | "gradient" | "keys" | "loudness" | "marker" | "marks" | "multiselect" | "music" | "offsets" | "port" | "quality" | "sound" | "spot" | "target" | "textarea" | "theme" | "timezone" | "transition" | "variable";

export type FileTreeReveal = {
	nodes: string[],
	title: string,
	selectedPath: string,
	rowStagger: number,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	countLabel: string,
	panelWidth: number,
	fontFamily: string,
};

export type FilmFonts = {
	families: { [key in string]: FontFamilyDef },
	preload: string[],
};

export type Fit = "contain" | "cover" | "fill";

export type FlickerOrder = "start" | "end" | "center" | "edges" | "random";

export type Flip = {
	direction: Direction,
	perspective: number,
};

export type FocusPull = {
	blur: number,
	glow: number,
	zoom: number,
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

export type FontWeight = "400" | "500" | "600" | "700";

export type FootageChain = {
	grade?: StyleEntry[],
	redact?: StyleEntry[],
};

export type FrameTheme = "light" | "dark";

export type FrostedGlassWipe = {
	blur: number,
	panelWidth: number,
	direction: WipeFrom,
	frostColor: string,
	edgeColor: string,
	glow: number,
};

export type FunnelChartStage = {
	label: string,
	value: number,
	color?: string,
};

export type FunnelChart = {
	stages: FunnelChartStage[],
	width: number,
	height: number,
	gap: number,
	tailOpacity: number,
	showDropoff: boolean,
	showConversion: boolean,
	durationInFrames: number,
	staggerInFrames: number,
	color: string,
	inkColor: string,
	labelColor: string,
	dropoffColor: string,
	labelWidth?: number,
	numberFormat: NumberFormat,
	dropoffHeading: string,
	fontFamily: string,
};

export type GaugeDial = {
	value: number,
	min: number,
	max: number,
	size: number,
	thickness: number,
	sweepInDegrees: number,
	label: string,
	unit: string,
	tickCount: number,
	durationInFrames: number,
	color: string,
	trackColor: string,
	inkColor: string,
	labelColor: string,
	roundEnds: boolean,
	showNeedle: boolean,
	uppercaseLabel: boolean,
	fontFamily: string,
	decimals?: number,
};

export type GlassCodeBlock = {
	title: string,
	width: number,
	height: number,
	fontSize: number,
	glassColor: string,
	staggerFrames: number,
	showTrafficLights: boolean,
	code: string,
	aura: boolean,
	speed: number,
	frost: number,
	cornerRadius: number,
	lineHeight: number,
	font: string,
	codeColor: string,
	commentColor: string,
	stringColor: string,
	keywordColor: string,
	numberColor: string,
	titleColor: string,
	lineNumberColor: string,
	auraColors: Colour[],
};

export type GlitchCut = {
	intensity: number,
	slices: number,
	rgbSplit: number,
	blockNoise: number,
	flicker: number,
	blockColumns: number,
	tint: string,
};

export type GradientScaleCutText = {
	text: string,
	giantFontSize: number,
	compactFontSize: number,
	anchorOffsetX: number,
	giantTravel: number,
	settleTravel: number,
	cutFrame: number,
	revealSoftness: number,
	compactBlur: number,
	gradientStart: string,
	gradientEnd: string,
	ghostColor: string,
	backgroundColor: string,
	fontWeight: FontWeight,
	fontFamily: string,
	speed: number,
	revealEnd: number,
	compactWidth: number,
	gradientSpread: number,
	letterSpacing: number,
};

export type GradientShape = "linear" | "radial" | "conic";

export type Gradient = {
	from: string,
	to: string,
	angle: number,
	grain: number,
	shape: GradientShape,
	grainSize: number,
};

export type GrainDissolveShape = "wave" | "dots" | "truchet" | "corners" | "ripple" | "blob" | "sphere";

export type GrainDissolve = {
	shape: GrainDissolveShape,
	noise: number,
	zoom: number,
	colorBack: string,
	colors: Colour[],
	speed: number,
	sceneBlur: number,
	rotation: number,
};

export type GrainGradientShape = "corners" | "wave" | "dots" | "truchet" | "ripple" | "blob" | "sphere";

export type GridPixelateWipe = {
	cols: number,
	rows: number,
	order: PixelateOrder,
	shape: PixelShape,
	stagger: number,
	reverse: boolean,
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

export type GroupSeparator = "none" | "space" | "comma" | "dot";

export type HalftonePrint = {
	dotSize: number,
	angle: number,
	misregistration: number,
	paperTint: string,
	cyanInk: string,
	magentaInk: string,
	yellowInk: string,
};

export type HandwritingText = {
	text: string,
	penSize: number,
	penColor: string,
	inkSoftness: number,
	wobble: number,
	staggerInFrames: number,
	fontFamily: string,
	seed: string,
	color: string,
	fontSize?: number,
	fontWeight: FontWeight,
	durationInFrames: number,
	wordSpacing: number,
};

export type HeroLaunch = {
	image1: string,
	image2: string,
	heading: string,
	theme: Theme,
	fontFamily: string,
	cardTilt: number,
	cardGap: number,
	cardAspect: number,
	cardRadius: number,
	headlineSize: number,
	headlineWeight: number,
	edgeFade: boolean,
};

export type HighlightLook = "clean" | "box" | "circle" | "underline" | "marker";

export type HighlightPreset = "logo-wipe" | "marker" | "color" | "underline" | "strikethrough" | "shimmer";

export type Image = {
	src: string,
	fit: Fit,
	background: string,
	kenBurns: boolean,
	kenBurnsZoom: number,
};

export type InkArrow = {
	curvature: number,
	strokeWidth: number,
	pressure: number,
	release: number,
	grain: number,
	headSize: number,
	drawDur: number,
	headDur: number,
	color: string,
	seed: string,
	step: number,
	fromX: number,
	fromY: number,
	toX: number,
	toY: number,
	headAngle: number,
	wobble: number,
	opacity: number,
};

export type InkUnderline = {
	width: number,
	color: string,
	thickness: number,
	pressure: number,
	release: number,
	grain: number,
	durationSteps: number,
	seed: string,
	step: number,
	x: number,
	y: number,
	wobble: number,
	angle: number,
	opacity: number,
};

export type InlineHighlight = {
	before: string,
	highlight: string,
	after: string,
	baseColor: string,
	highlightColor: string,
	fontSize: number,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	changeStart: number,
	changeEnd: number,
	letterSpacing: number,
};

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

export type KanbanMoveCard = {
	title: string,
	meta?: string,
	column: number,
	tint?: string,
	moveTo?: number,
	moveAtSeconds?: number,
};

export type KanbanMove = {
	columns: string[],
	cards: KanbanMoveCard[],
	moveSeconds: number,
	dealStaggerSeconds: number,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	showCounts: boolean,
	fontFamily: string,
};

export type KeyNames = "symbols" | "words";

export type KeyboardPlatform = "mac" | "windows";

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

export type KeystrokeStep = {
	keys: string,
	at: number,
	hold?: number,
};

export type Keystroke = {
	platform: KeyboardPlatform,
	theme: Theme,
	accent: string,
	size: number,
	linger: number,
	x: number,
	y: number,
	steps: KeystrokeStep[],
	speed: number,
	fontFamily: string,
	fontWeight: number,
	thenText: string,
	hold: number,
	roundness: number,
};

export type KineticCenterBuild = {
	text: string,
	entryOffset: number,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	wordGap: number,
	wordTime: number,
	entryBlur: number,
	letterSpacing: number,
};

export type KineticHeadline = {
	text: string,
	color: string,
	emphasisColor: string,
	background: string,
	fontSize: number,
	wordStagger: number,
	fontFamily: string,
	fontWeight: number,
	letterSpacing: number,
	lineHeight: number,
	sideMargin: number,
	wordGap: number,
	rise: number,
	tilt: number,
	underlineEmphasis: boolean,
	fadeOutInFrames: number,
};

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

export type LaptopEntrance = "rise" | "open" | "none";

export type LaptopFinale = "none" | "zoom-to-screen";

export type LaptopFrame = {
	screenSrc: string,
	entrance: LaptopEntrance,
	finale: LaptopFinale,
	notchLabel: string,
	showNotch: boolean,
	batteryLevel: number,
	bezelColor: string,
	screenColor: string,
	restTilt: number,
	radius: number,
	scale: number,
	floatLoop: boolean,
	floatAmplitude: number,
	shadow: string,
	theme: Theme,
	indicatorColor: string,
	fontFamily: string,
	speed: number,
	notchAt: number,
	zoomAt: number,
	zoomLength: number,
	floatPeriod: number,
	notchLabelSize: number,
};

export type LaunchAnythingLogoImage = {
	image: string,
};

export type LaunchAnythingStat = {
	label: string,
	value: string,
};

export type LaunchAnything = {
	opening: string,
	subject: string,
	brandUrl: string,
	accentColor: string,
	copyReady: string,
	copyNext: string,
	copyAction: string,
	copyProof: string,
	copyIndustry: string,
	copyPromise: string,
	copyLaunch: string,
	copyShoppers: string,
	copyShopName: string,
	copyShopHeadline: string,
	copyShopDetail: string,
	copyShopAction: string,
	copyBuilder: string,
	copyBuilderPrompt: string,
	copyTrading: string,
	copyLoyalty: string,
	copyLoyaltyPayoff: string,
	copyIntegration: string,
	copyConnections: string,
	copyIntegrations: string[],
	screenDashboard: string,
	screenShoppers: string,
	screenExchange: string,
	screenBuilder: string,
	screenTrading: string,
	screenLoyalty: string,
	screenIntegrations: string,
	photoDesk: string,
	photoHorizon: string,
	photoRocket: string,
	photoEarth: string,
	logoImages: LaunchAnythingLogoImage[],
	logoSrc: string,
	music: string,
	musicVolume: number,
	fontFamily: string,
	nightColor: string,
	paperColor: string,
	lightTextColor: string,
	inkColor: string,
	sageColor: string,
	openingShimmer: number,
	openingSize: number,
	headlineWeight: number,
	textSize: number,
	photoDimming: number,
	logoSize: number,
	copyAppName: string,
	copyDashboardMenu: string[],
	copyDashboard: string,
	copyDashboardPeriod: string,
	copyDashboardNote: string,
	dashboardStats: LaunchAnythingStat[],
	copyMomentum: string,
	copyActivityTitle: string,
	dashboardActivity: string[],
	copyShopNote: string,
	copyReviewTitle: string,
	copyReviewProject: string,
	copyReviewButtons: string[],
	reviewRows: LaunchAnythingStat[],
	copyIdeasCard: string,
	copyIdeasNote: string,
};

export type LegendValue = "percent" | "value";

export type LensZoom = {
	lensSteps: number,
	blurSamples: number,
	shakeAmount: number,
	shakeTranslatePx: number,
	colorSplit: boolean,
	zoom: number,
	zoomBlur: number,
	distortion: number,
	twist: number,
	pullBack: number,
	shakeSpeed: number,
	shakeRotation: number,
	fringe: number,
	letterbox: number,
	background: string,
};

export type LightLeak = {
	seed: number,
	colors: Colour[],
	strength: number,
	blend: BlendMode,
};

export type Lighting = "studio" | "soft" | "dramatic";

export type LineByLineSlide = {
	text: string,
	distance: number,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	align: TextAlign,
	lineStagger: number,
	exit: boolean,
	letterSpacing: number,
	lineHeight: number,
};

export type LinkedParam = {
	param: string,
	sign: number,
	ifNonZero?: boolean,
};

export type Logo3d = {
	src: string,
	depth: number,
	color: string,
	metalness: number,
	motion: Motion,
	camera: CameraPath,
	speed: number,
	lighting: Lighting,
	background: string,
	zoom: number,
	roughness: number,
	bevel: number,
	cornerRadius: number,
	tilt: number,
	growIn: boolean,
	endFacing: boolean,
	motionAmount: number,
	reverse: boolean,
	lens: number,
};

export type LogoAssembleImage = {
	image: string,
};

export type LogoAssemble = {
	brandName: string,
	middleText: string,
	logoSrc: string,
	count: number,
	background: string,
	theme: Theme,
	images: LogoAssembleImage[],
	fontFamily: string,
	speed: number,
	orbitSeconds: number,
	orbitSpeed: number,
	cardRatio: number,
	cardShadow: number,
	cardRadius: number,
	ringSize: number,
	cardSize: number,
	logoSize: number,
	logoGap: number,
	nameSize: number,
	middleTextSize: number,
	fontWeight: number,
};

export type LogoDriftTile = {
	glyph: string,
	label?: string,
	background: string,
	color: string,
	x: number,
	y: number,
	vx: number,
	vy: number,
	size: number,
	at: number,
};

export type LogoDrift = {
	headline: string,
	pullback: number,
	tileSpeed: number,
	tileScale: number,
	tileFade: number,
	tileRadius: number,
	glyphScale: number,
	wordStagger: number,
	wordDuration: number,
	wordScale: number,
	wordBlur: number,
	exitAt: number,
	exitDuration: number,
	fontSize: number,
	glow: boolean,
	glowOpacity: number,
	accentColor: string,
	theme: Theme,
	fontWeight: FontWeight,
	glowColor: string,
	tiles: LogoDriftTile[],
	fontFamily: string,
	speed: number,
	wordAt: number,
	exitScale: number,
	exitBlur: number,
	glowSize: number,
	glyphWeight: number,
	labelScale: number,
	labelOpacity: number,
	headlineWidth: number,
};

export type LogoEnterLogo = {
	image: string,
	background: string,
};

export type LogoEnter = {
	diameter: number,
	overlap: number,
	ringColor: string,
	orientation: Orientation,
	stagger: number,
	logos: LogoEnterLogo[],
	speed: number,
	bounce: number,
	ringWidth: number,
	shadow: number,
	logoScale: number,
	firstOnTop: boolean,
};

export type LogoReveal = {
	pathD: string,
	wordmark: string,
	tagline: string,
	stroke: string,
	backgroundColor: string,
	textColor: string,
	viewBox: string,
	size?: number,
	strokeWidth?: number,
	fill: string,
	fontFamily: string,
	speed: number,
	drawSeconds: number,
	pathStagger: number,
	nameAt: number,
	glow: boolean,
	penTip: boolean,
	nameSize: number,
	fontWeight: number,
	taglineSize: number,
	taglineColor: string,
};

export type LogoWallLogo = {
	image?: string,
	name: string,
	color?: string,
	glyph?: string,
};

export type LogoWall = {
	logos: LogoWallLogo[],
	eyebrow: string,
	title: string,
	columns: number,
	startAtSeconds: number,
	arriveStaggerSeconds: number,
	staggerSeconds: number,
	warmSeconds: number,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	glyphColor: string,
	fontFamily: string,
	wallWidth: number,
	gap: number,
	tileHeight: number,
	tileRadius: number,
	titleSize: number,
	eyebrowUppercase: boolean,
	nameSize: number,
	imageSize: number,
	showMark: boolean,
	startBrightness: number,
};

export type Lottie = {
	src: string,
	loop: boolean,
	speed: number,
	backwards: boolean,
	fit: AnimationFit,
	background: string,
	align: TextAlign,
};

export type LowerThird = {
	title: string,
	subtitle: string,
	position: Corner,
	accent: string,
	color: string,
	panel: string,
	fontFamily: string,
	exitTime: number,
	slide: number,
	margin: number,
	barWidth: number,
	cornerRadius: number,
	panelBlur: number,
	titleSize: number,
	titleWeight: number,
	subtitleSize: number,
	subtitleOpacity: number,
};

export type MarkLook = "underline" | "circle" | "box" | "marker" | "crossed-off";

export type MarkerHighlight = {
	before: string,
	highlight: string,
	after: string,
	markerColor: string,
	baseColor: string,
	highlightedTextColor: string,
	fontSize: number,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	markerAt: number,
	markerHeight: number,
	x: number,
	y: number,
};

export type MaskReveal = "left" | "right" | "up" | "down" | "none";

export type MaskRevealUp = {
	text: string,
	distance: number,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	align: TextAlign,
	lineStagger: number,
	exitDistance: number,
	blur: number,
	lineHeight: number,
};

export type MasterStem = {
	src: string,
	hash: string,
};

export type MatrixDecode = {
	text: string,
	charset: string,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	revealDuration: number,
	background: string,
	fontFamily: string,
	speed: number,
	scrambleEvery: number,
	seed: number,
	letterSpacing: number,
};

export type MediaKind = "video" | "audio" | "image";

export type MenuIcon = "search" | "settings" | "user" | "file";

export type MenuState = "opened" | "closed";

export type MetricTickerMetric = {
	label: string,
	value: number,
	delta?: string,
	trend?: (number)[],
	suffix?: string,
	from?: number,
	prefix?: string,
	color?: string,
};

export type MetricTicker = {
	metrics: MetricTickerMetric[],
	title: string,
	eyebrow: string,
	maxCards: number,
	accentColor: string,
	backgroundColor: string,
	theme: Theme,
	showArrows: boolean,
	numberFormat: NumberFormat,
	glow: number,
	uppercaseEyebrow: boolean,
	cornerRadius: number,
	trendFill: boolean,
	fontFamily: string,
};

export type MockupDevice = "phone" | "browser" | "laptop";

export type Model3d = {
	src: string,
	motion: ModelMotion,
	camera: CameraPath,
	speed: number,
	animation: string,
	animationSpeed: number,
	loop: boolean,
	zoom: number,
	lighting: Lighting,
	shadow: boolean,
	background: string,
	shadowOpacity: number,
	growIn: boolean,
	motionAmount: number,
	tilt: number,
	endFacing: boolean,
	reverse: boolean,
	lens: number,
};

export type ModelMotion = "spin" | "orbit" | "swing" | "float" | "still" | "keyframes";

export type MorphShape = "circle" | "square" | "squircle" | "triangle" | "diamond" | "blob";

export type MosaicBox = {
	x: number,
	y: number,
	width: number,
	height: number,
};

export type Mosaic = {
	boxes: MosaicBox[],
	frameMs: number,
	blockSize: number,
};

export type Motion = "spin" | "tilt-in" | "float" | "orbit" | "swing" | "still" | "keyframes";

export type MultiDeviceLineup = {
	scale: number,
	staggerInFrames: number,
	bezelColor: string,
	screenColor: string,
	labelColor: string,
	phoneLabel: string,
	tabletLabel: string,
	laptopLabel: string,
	baseGradient: string,
	islandColor: string,
	cameraColor: string,
	edgeColor: string,
	shadow: number,
	gap: number,
	labelSize: number,
	labelUppercase: boolean,
	fontFamily: string,
};

export type NeonFlickerText = {
	text: string,
	glowColor: string,
	glowSize: number,
	offLevel: number,
	hum: number,
	buzz: number,
	flickerOnExit: boolean,
	order: FlickerOrder,
	staggerInFrames: number,
	seed: number,
	color: string,
	fontSize?: number,
	fontWeight: FontWeight,
	offColor: string,
	durationInFrames: number,
	humRate: number,
	wordSpacing: number,
};

export type NoSettings = Record<string, never>;

export type NodeLook = {
	glyph: string,
	motion?: NodeMotion,
	morph?: string,
};

export type NodeMotion = "blur-swap" | "bob" | "caret" | "clock-wipe" | "collapse" | "dissolve" | "draw" | "fade-through" | "flare" | "flicker" | "flip" | "glitch" | "grow" | "hard-cut" | "iris" | "march" | "mask-up" | "nudge-down" | "nudge-right" | "nudge-up" | "open" | "pan" | "press" | "push" | "rack-focus" | "ripple" | "rise" | "roll" | "shimmer" | "shutter" | "slide-through" | "smear" | "snap" | "stretch" | "track" | "tracking" | "turn" | "turn-over" | "twist" | "whip" | "wipe-reveal" | "zoom-blur" | "zoom-through";

export type Noise = {
	color1: string,
	color2: string,
	scale: number,
	speed: number,
	blobs: number,
	wander: number,
	blobSize: number,
	intensity: number,
	softness: number,
	seed: number,
	grain: number,
	grainSize: number,
};

export type NotificationStackToast = {
	title: string,
	body?: string,
	tone?: ToastTone,
	meta?: string,
	atSeconds?: number,
	holdFor?: number,
};

export type NotificationStack = {
	toasts: NotificationStackToast[],
	align: ToastCorner,
	startAtSeconds: number,
	staggerSeconds: number,
	lifeSeconds: number,
	showProgress: boolean,
	accentColor: string,
	theme: Theme,
	speed: number,
	fontFamily: string,
	size: number,
	toastWidth: number,
	cornerRadius: number,
	margin: number,
};

export type NumberFormat = "compact" | "full";

export type Orientation = "horizontal" | "vertical";

export type PageAddress = {
	t: number,
	url: string,
};

export type PageTurn = {
	angle: number,
	poses: number,
	hingeX: number,
};

export type Paper = {
	color: string,
	grain: number,
	grainSize: number,
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

export type ParticleDissolveDirection = "reveal" | "up" | "down" | "left" | "right";

export type ParticleDissolve = {
	particleSize: number,
	scatter: number,
	shimmer: number,
	aberration: number,
	direction: ParticleDissolveDirection,
	stagger: number,
	dustColor: number,
	raggedness: number,
};

export type ParticleField = {
	backgroundColor: string,
	color: string,
	count: number,
	angle: number,
	speed: number,
	size: number,
	minSize: number,
	depthBlur: number,
	drift: number,
	intensity: number,
	glow: number,
	seed: number,
	twinkle: number,
};

export type PhoneEntrance = "rise" | "rotate-in" | "float";

export type PhoneFrame = {
	variant: PhoneVariant,
	entrance: PhoneEntrance,
	screenSrc: string,
	bezelColor: string,
	screenColor: string,
	radius: number,
	screenRadius: number,
	scale: number,
	tiltAngle: number,
	showDynamicIsland: boolean,
	floatLoop: boolean,
	floatAmplitude: number,
	shadow: string,
	theme: Theme,
	fontFamily: string,
	speed: number,
	floatPeriod: number,
};

export type PhoneVariant = "flat" | "tilt" | "showcase";

export type PhotoShape = "16:9" | "4:3" | "1:1" | "3:4";

export type PixelShape = "square" | "dot";

export type PixelateOrder = "from-left" | "from-top" | "diagonal" | "center";

export type PlaceholderProps = {
	title: string,
	subtitle: string,
};

export type Point = {
	x: number,
	y: number,
};

export type Polaroid = {
	caption: string,
	captionAt: number,
	width: number,
	captionSize?: number,
	step: number,
	frameColor: string,
	captionColor: string,
	photoShape: PhotoShape,
	captionStrip: number,
	cornerRadius: number,
	captionWeight: number,
	writeSpeed: number,
	captionFont: string,
	photoColor: string,
};

export type PreparedTypeface = {
	typeface: string,
	typefaceItalic: boolean,
};

export type PricingCard = {
	tier: string,
	price: number,
	currency: string,
	period: string,
	badge: string,
	note: string,
	features: string[],
	ctaLabel: string,
	priceAtSeconds: number,
	rollSeconds: number,
	featuresAtSeconds: number,
	featureStaggerSeconds: number,
	accentColor: string,
	theme: Theme,
	speed: number,
	backgroundColor: string,
	ctaTextColor: string,
	wasPrice?: number,
	rollFrom: number,
	cardWidth: number,
	fontFamily: string,
};

export type PunchLines = {
	script: string,
	styles: string,
	sizes: string,
	holds: string,
	accentBeat: number,
	fontSize: number,
	fontWeight: FontWeight,
	lineHeight: number,
	letterSpacing: string,
	wordSpacing: string,
	ground: string,
	ink: string,
	accentColor: string,
	lead: number,
	slideFrom: number,
	slideKick: number,
	slideSettle: number,
	lineStagger: number,
	slideDrop: number,
	motionBlur: number,
	punchFrom: number,
	punchFrames: number,
	wordStagger: number,
	pushFrom: number,
	pushRate: number,
	rushLead: number,
	rushFrames: number,
	rushTravel: number,
	fit: number,
	theme: Theme,
	fontFamily: string,
	speed: number,
};

export type PushThrough = {
	zoom: number,
	blur: number,
	approachFrom: number,
};

export type RadialBurst = {
	segments: number,
	radius: number,
	thickness: number,
	rotation: number,
	intensity: number,
	twist: number,
	echoes: number,
	color: string,
	accentColor: string,
	backgroundColor: string,
	loop: boolean,
	speed: number,
	echoOpacity: number,
	x: number,
	y: number,
};

export type RecordingProps = {
	src: string,
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
	address?: string,
	addressMode?: AddressMode,
	backdrop: string,
	backdropChosen?: boolean,
	cursor: boolean,
	cursorScale: number,
	cursorLook: CursorLook,
	cursorFeel?: CursorFeel,
	ripple: RippleLook | null,
	extraBeats: Action[],
	keystrokes?: KeystrokeLook,
	annotations: AnnotationSpec[],
	footage?: FootageChain,
	footageUrl?: string,
	screen?: boolean,
	windowFit?: number,
	stageGround?: string,
	windowRadius?: number,
	rimGlow?: number,
	liftColours?: boolean,
	shutterAngle?: number,
	bleedBlur?: number,
	bleedDim?: number,
	frameFont?: string,
};

export type Rect = {
	x: number,
	y: number,
	w: number,
	h: number,
};

export type ReframeMode = "follow" | "center" | "fit";

export type ReleaseTeaser = {
	brandName: string,
	release: string,
	tagline: string,
	accentColor: string,
	reducedMotion: boolean,
	statements: string[],
	logoSrc: string,
	themeBackground: string,
	themeSurface: string,
	themeForeground: string,
	themeMuted: string,
	lightIntensity: number,
	music: string,
	musicVolume: number,
	fontFamily: string,
	textSize: number,
	statementWeight: number,
	logoSize: number,
};

export type RgbGlitchText = {
	text: string,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	glitchAt: number,
	glitchDuration: number,
	intensity: number,
	seed: string,
	glitchColors: Colour[],
	blend: BlendMode,
	background: string,
	fontFamily: string,
	speed: number,
	letterSpacing: number,
};

export type RippleLook = {
	color: string,
	size: number,
	width: number,
	ringOpacity?: number,
	fillOpacity?: number,
	durationS?: number,
};

export type RippleZoom = {
	zoom: number,
	intensity: number,
	softness: number,
	colorBack: string,
	colors: Colour[],
	speed: number,
	noise: number,
	originX: number,
};

export type RollingNumber = {
	from: number,
	to: number,
	fontSize: number,
	color: string,
	background: string,
	speed: number,
	finishAt: number,
	groupSeparator: GroupSeparator,
	fontWeight: number,
	prefix: string,
	suffix: string,
	fontFamily: string,
};

export type Screen3d = {
	device: ScreenDevice,
	deviceColor: string,
	move: ScreenMove,
	camera: CameraPath,
	endFaceOn: boolean,
	lighting: Lighting,
	background: string,
	bezelColor: string,
	metalness: number,
	roughness: number,
	glare: number,
	moveEndsAt: number,
};

export type ScreenDevice = "laptop" | "monitor" | "phone" | "floating";

export type ScreenMove = "push-in" | "orbit" | "tilt-reveal" | "rise" | "keyframes";

export type ScribbleCircle = {
	laps: number,
	strokeWidth: number,
	pressure: number,
	grain: number,
	durationSteps: number,
	color: string,
	seed: string,
	step: number,
	x: number,
	y: number,
	width: number,
	height: number,
	release: number,
	wobble: number,
	startAngle: number,
	clockwise: boolean,
	opacity: number,
};

export type SearchReveal = {
	text: string,
	fieldWidth: number,
	fontSize: number,
	framesPerCharacter: number,
	showPanel: boolean,
	showGuides: boolean,
	color: string,
	fieldColor: string,
	ringColor: string,
	panelColor: string,
	accentColor: string,
	backgroundColor: string,
	reducedMotion: boolean,
	fontFamily: string,
	speed: number,
	weight: FontWeight,
};

export type SequencerProps = {
	component: string,
	props: { [key in string]: unknown },
	backgrounds: (string | null)[],
};

export type ShaderDithering = {
	speed: number,
	size: number,
	colorBack: string,
	colorFront: string,
	shape: DitherShape,
	type: DitherPattern,
	scale: number,
	rotation: number,
};

export type ShaderGodRays = {
	speed: number,
	intensity: number,
	density: number,
	bloom: number,
	colorBack: string,
	colors: Colour[],
	colorBloom: string,
	offsetY: number,
	offsetX: number,
	spotty: number,
	midIntensity: number,
	midSize: number,
};

export type ShaderGrainGradient = {
	speed: number,
	softness: number,
	intensity: number,
	noise: number,
	colorBack: string,
	colors: Colour[],
	shape: GrainGradientShape,
	scale: number,
	rotation: number,
};

export type ShaderMeshGradient = {
	speed: number,
	distortion: number,
	swirl: number,
	colors: Colour[],
	grainOverlay: number,
	grainMixer: number,
	scale: number,
};

export type ShaderPerlinNoise = {
	speed: number,
	proportion: number,
	softness: number,
	colorBack: string,
	colorFront: string,
	octaveCount: number,
	persistence: number,
	lacunarity: number,
	scale: number,
};

export type ShaderSwirl = {
	speed: number,
	bandCount: number,
	twist: number,
	softness: number,
	colorBack: string,
	colors: Colour[],
	center: number,
	proportion: number,
	noise: number,
	noiseFrequency: number,
	scale: number,
};

export type ShaderWarp = {
	speed: number,
	distortion: number,
	swirl: number,
	softness: number,
	proportion: number,
	colors: Colour[],
	shape: WarpShape,
	shapeScale: number,
	swirlIterations: number,
	scale: number,
};

export type ShapeKind = "sphere" | "ring" | "box";

export type Shapes3d = {
	count: number,
	palette: Colour[],
	seed: number,
	speed: number,
	camera: CameraPath,
	lighting: Lighting,
	background: string,
	depth: number,
	size: number,
	drift: number,
	spin: number,
	roughness: number,
	metalness: number,
	kinds: ShapeKind[],
};

export type ShimmerSweep = {
	text: string,
	baseColor: string,
	shineColor: string,
	fontSize: number,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	sweeps: number,
	bandAngle: number,
	bandWidth: number,
	letterSpacing: number,
};

export type Side = "top" | "bottom" | "left" | "right";

export type SignupFlow = {
	theme: Theme,
	title: string,
	description: string,
	fullName: string,
	email: string,
	password: string,
	createLabel: string,
	toastTitle: string,
	nameLabel: string,
	emailLabel: string,
	emailHelp: string,
	passwordLabel: string,
	passwordHelp: string,
	confirmLabel: string,
	speed: number,
	loadingSeconds: number,
	noticeSeconds: number,
	cursorShape: CursorVariant,
	cursorSize: number,
	noticeCorner: Corner,
	backgroundColor: string,
	radius: number,
	fontFamily: string,
};

export type Slide = {
	direction: Direction,
};

export type Slot = {
	nodeId: string,
	port: string,
};

export type SlotMachineRoll = {
	from: string,
	to: string,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	columnStagger: number,
	direction: VerticalDirection,
	letterSpacing: number,
};

export type SmokeDissolve = {
	colorBack: string,
	colors: Colour[],
	speed: number,
	detail: number,
	originX: number,
};

export type SoftBlurIn = {
	text: string,
	blur: number,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	stagger: number,
	distance: number,
	letterSpacing: number,
};

export type Solid = {
	color: string,
};

export type SpecAssets = {
	projectBase?: string,
};

export type SpeedLinesMode = "trail" | "focus";

export type SpeedLines = {
	mode: SpeedLinesMode,
	count: number,
	weight: number,
	spread: number,
	duration: number,
	bounce: number,
	x: number,
	y: number,
	delay: number,
	color: string,
	seed: number,
	speed: number,
	fromX: number,
	fromY: number,
	hold: number,
	streakLength: number,
};

export type SpringSettle = {
	enterScale: number,
	enterStagger: number,
	exitScale: number,
	exitFrames: number,
	enterFadeFrames: number,
	springDamping: number,
	springStiffness: number,
	bgFadeFrames: number,
	loop: boolean,
	enterStaggerJitter: number,
};

export type StaggeredFadeUp = {
	text: string,
	staggerDelay: number,
	distance: number,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	wordDuration: number,
	letterSpacing: number,
};

export type Starburst = {
	color1: string,
	color2: string,
	rays: number,
	rotationSpeed: number,
	edgeFade: number,
	rayWidth: number,
	centerX: number,
};

export type StatCard = {
	value: number,
	suffix: string,
	prefix: string,
	decimals: number,
	label: string,
	caption: string,
	delta: number,
	deltaSuffix: string,
	accentColor: string,
	theme: Theme,
	speed: number,
	showDelta: boolean,
	backgroundColor: string,
	max?: number,
	glow: number,
	ringThickness: number,
	groupSeparator: GroupSeparator,
	fontFamily: string,
	downColor: string,
	ringGlow: number,
};

export type StatusCycle = {
	prefix: string,
	statuses: string,
	chips: string,
	fieldColor: string,
	pageColor: string,
	pillColor: string,
	pillLabelColor: string,
	prefixColor: string,
	chipFills: Colour[],
	chipTextColor: string,
	chipBorderColor: string,
	fontWeight: FontWeight,
	chipFontWeight: FontWeight,
	fontSize: number,
	chipFontSize: number,
	introFrames: number,
	statusHold: number,
	chipStagger: number,
	pillCenterY: number,
	startAt: number,
	theme: Theme,
	pillRadius?: number,
	chipRadius?: number,
	chipBorderWidth?: number,
	fontFamily: string,
	speed: number,
	pillHeight: number,
	pillPadding: number,
	pillGap: number,
	chipPadding: number,
	chipInset: number,
	chipGap: number,
	pillResizeTime: number,
	rollTime: number,
	fieldFadeTime: number,
	letterStagger: number,
	introZoom: number,
};

export type StillAlign = "top" | "middle";

export type StillLook = "window" | "cutout" | "bleed";

export type StillProps = {
	src: string,
	look: StillLook,
	backdrop: string,
	shot?: StillShot,
	fit: number,
	shadow: number,
	frame?: WindowStyle,
	pushIn: number,
	pushInFrames: number,
	align: StillAlign,
};

export type StillShot = {
	width: number,
	height: number,
	scale: number,
	body?: Bounds,
};

export type StrikethroughReplace = {
	from: string,
	to: string,
	lineColor: string,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	strikeEnd: number,
	swapLength: number,
	letterSpacing: number,
	thickness?: number,
	toColor: string,
};

export type StudioBackdrop = {
	name: Backdrop,
};

export type StyleEntry = {
	nodeId: string,
	type: string,
	params: { [key in string]: unknown },
};

export type SwirlDissolve = {
	bandCount: number,
	softness: number,
	colorBack: string,
	colors: Colour[],
	speed: number,
	noise: number,
};

export type TerminalLineKind = "command" | "log" | "success" | "error";

export type TerminalSimulatorLine = {
	text: string,
	type: TerminalLineKind,
	delay: number,
};

export type TerminalSimulator = {
	prompt: string,
	title: string,
	background: string,
	chromeColor: string,
	fontSize: number,
	charsPerFrame: number,
	chunkSize: number,
	lines: TerminalSimulatorLine[],
	speed: number,
	width: number,
	height: number,
	ellipsisPause: number,
	startAt: number,
	windowDots: boolean,
	cornerRadius: number,
	lineHeight: number,
	font: string,
	commandColor: string,
	logColor: string,
	successColor: string,
	errorColor: string,
	promptColor: string,
	cursorColor: string,
	titleColor: string,
};

export type Text3d = {
	text: string,
	fontFamily: string,
	fontWeight: number,
	italic: boolean,
	depth: number,
	bevel: number,
	color: string,
	metalness: number,
	align: TextAlign,
	lineWidth: number,
	motion: Motion,
	camera: CameraPath,
	speed: number,
	lighting: Lighting,
	background: string,
	size: number,
	lineHeight: number,
	roughness: number,
	lens: number,
	tilt: number,
	endFacing: boolean,
	motionAmount: number,
	growIn: boolean,
	reverse: boolean,
};

export type TextAlign = "center" | "left" | "right";

export type TextHighlight = {
	before: string,
	highlight: string,
	after: string,
	preset: HighlightPreset,
	baseColor: string,
	accentColor: string,
	highlightedTextColor: string,
	replaceWith: string,
	startAt: number,
	drawDuration: number,
	thickness: number,
	shineColor: string,
	logoSrc: string,
	logoScale: number,
	spinTurns: number,
	holdDuration: number,
	rushDuration: number,
	rushScale: number,
	rushDrift: number,
	swingOut: number,
	coverColor: string,
	fontSize: number,
	fontWeight: FontWeight,
	theme: Theme,
	background: string,
	fontFamily: string,
	speed: number,
	letterSpacing: number,
	lineHeight: number,
	textWidth: number,
	markerOpacity: number,
	shineWidth: number,
	markGap: number,
};

export type TextMaskVideo = {
	text: string,
	src: string,
	gradient: string,
	reveal: MaskReveal,
	drift: number,
	zoom: number,
	durationInFrames: number,
	fontFamily: string,
	fontWeight: number,
	letterSpacing: number,
	lineHeight: number,
	fontSize?: number,
	driftSpeed: number,
	fitMax: number,
	startFrom: number,
};

export type ThemeMode = "light" | "dark";

export type Theme = {
	mode: ThemeMode,
	colors: { [key in string]: string },
};

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
	fonts?: FilmFonts,
	mix?: MasterStem,
};

export type TimelineTrack = {
	id: string,
	items: TimelineItem[],
	layer?: boolean,
};

export type TitleAlign = "center" | "left";

export type Title = {
	eyebrow: string,
	title: string,
	subtitle: string,
	mark: string,
	markLook: MarkLook,
	markColor: string,
	align: TitleAlign,
	entrance: Entrance,
	color: string,
	accent: string,
	background: string,
	fontSize: number,
	fontWeight: number,
	fontFamily: string,
	sideMargin: number,
	lineGap: number,
	eyebrowSize: number,
	eyebrowUppercase: boolean,
	subtitleSize: number,
	letterSpacing: number,
	lineStagger: number,
	markThickness: number,
	markerOpacity: number,
	markSeed: number,
	markDrawTime: number,
	fadeOut: number,
	subtitleColor: string,
};

export type ToastCorner = "top-right" | "top-left" | "bottom-right" | "bottom-left";

export type ToastTone = "info" | "success" | "warn" | "error";

export type TrackingIn = {
	text: string,
	startTracking: number,
	startBlur: number,
	fontSize: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	endTracking: number,
	fadeIn: number,
};

export type TransitionBlinds = {
	orientation: Orientation,
	slats: number,
	stagger: number,
	alternate: boolean,
	edgeSoftness: number,
	reverse: boolean,
};

export type TransitionEase = "linear" | "in" | "out" | "in-out";

export type TransitionMorphShape = {
	from: MorphShape,
	to: MorphShape,
	originX: number,
	originY: number,
	overshoot?: number,
	warp: number,
	warpDetail: number,
	seed: number,
};

export type TransitionRef = {
	kind: string,
	durationInFrames: number,
	params?: { [key in string]: unknown },
	hold?: boolean,
	easing?: TransitionEase,
};

export type TvPowerOff = {
	durationInFrames: number,
	gain: number,
	afterglow: number,
	phosphor: string,
	screenColor: string,
};

export type Typewriter = {
	text: string,
	cursor: boolean,
	charsPerSecond: number,
	fontSize: number,
	color: string,
	cursorColor: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	cursorWidth: number,
	blinkRate: number,
	align: TextAlign,
	letterSpacing: number,
};

export type VerticalDirection = "up" | "down";

export type VhsFilter = {
	bleed: number,
	wobble: number,
	noise: number,
	intensity: number,
	headSwitch: number,
	scanlines: number,
};

export type VideoLoop = {
	src: string,
	dim: number,
	dimColor: string,
	fit: Fit,
	playbackRate: number,
	trimBeforeMs: number,
};

export type Video = {
	src: string,
	fit: Fit,
	loop: boolean,
	muted: boolean,
	volume: number,
	playbackRate: number,
	trimBeforeMs: number,
	background: string,
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

export type WarpShape = "checks" | "stripes" | "edge";

export type WaveWipe = {
	intensity: number,
	softness: number,
	noise: number,
	colorBack: string,
	colors: Colour[],
	speed: number,
	zoom: number,
	direction: VerticalDirection,
};

export type WhipDirection = "left" | "right" | "up" | "down";

export type WhipPan = {
	direction: WhipDirection,
	blur: number,
	stretch: number,
};

export type WindowStyle = "macos" | "windows" | "linux";

export type WipeDirection = "from-left" | "from-right" | "from-top" | "from-bottom" | "from-top-left" | "from-top-right" | "from-bottom-left" | "from-bottom-right";

export type WipeFrom = "from-left" | "from-right" | "from-top" | "from-bottom";

export type Wipe = {
	direction: WipeDirection,
};

export type WordPush = {
	text: string,
	fontSize: number,
	wordGap: number,
	accel: number,
	color: string,
	fontWeight: FontWeight,
	background: string,
	fontFamily: string,
	speed: number,
	pushTime: number,
	driveDistance: number,
	letterSpacing: number,
};

export type ZoomBlur = {
	blur: number,
	rise: number,
	zoom: number,
};
