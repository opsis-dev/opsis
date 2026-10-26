export type ActionKind = "click" | "type" | "hover" | "scroll" | "navigate" | "wait" | "chapter" | "key" | "focus" | "drag";

export type ActionLog = {
	viewport: Viewport,
	dpr: number,
	t0: number,
	durationMs: number,
	actions: Action[],
	cursorTrack?: CursorSample[],
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

export type AnimationFit = "contain" | "cover";

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

export type Backdrop = "glaze" | "ink" | "cobalt" | "graphite" | "ember" | "flare" | "prism" | "bloom" | "halo" | "dusk" | "studio" | "moonrise" | "canyon" | "mist" | "chalk" | "aurora";

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

export type Callout = {
	text: string,
	x: number,
	y: number,
	side: Side,
	color: string,
	accent: string,
	fontFamily: string,
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
};

export type Colour = string;

export type CoreElementProps = {
	"core/placeholder": PlaceholderProps,
	"core/still": StillProps,
	"core/title": Title,
	"core/kinetic-headline": KineticHeadline,
	"core/lower-third": LowerThird,
	"core/callout": Callout,
	"core/captions": Captions,
	"core/image": Image,
	"core/mosaic": Mosaic,
	"core/lottie": Lottie,
	"core/video": Video,
	"core/solid": Solid,
	"core/gradient": Gradient,
	"core/noise": Noise,
	"core/starburst": Starburst,
	"core/paper": Paper,
	"core/backdrop": StudioBackdrop,
	"core/video-loop": VideoLoop,
	"core/light-leak": LightLeak,
	"core/logo-3d": Logo3d,
	"core/card-3d": Card3d,
	"core/text-3d": Text3d,
	"core/model-3d": Model3d,
	"core/screen-3d": Screen3d,
	"core/shapes-3d": Shapes3d,
};

export type Corner = "bottom-left" | "bottom-right" | "top-left" | "top-right";

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

export type Entrance = "rise" | "fade" | "blur" | "scale" | "none";

export type FilmFonts = {
	families: { [key in string]: FontFamilyDef },
	preload: string[],
};

export type Fit = "contain" | "cover" | "fill";

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

export type FrameTheme = "light" | "dark";

export type Gradient = {
	from: string,
	to: string,
	angle: number,
	grain: number,
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

export type Image = {
	src: string,
	fit: Fit,
	background: string,
	kenBurns: boolean,
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

export type KineticHeadline = {
	text: string,
	color: string,
	emphasisColor: string,
	background: string,
	fontSize: number,
	wordStagger: number,
	fontFamily: string,
};

export type LightLeak = {
	seed: number,
	hueShift: number,
};

export type Lighting = "studio" | "soft" | "dramatic";

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
};

export type Lottie = {
	src: string,
	loop: boolean,
	speed: number,
	backwards: boolean,
	fit: AnimationFit,
	background: string,
};

export type LowerThird = {
	title: string,
	subtitle: string,
	position: Corner,
	accent: string,
	color: string,
	panel: string,
	fontFamily: string,
};

export type MarkLook = "underline" | "circle" | "box" | "marker" | "crossed-off";

export type MasterStem = {
	src: string,
	hash: string,
};

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
};

export type ModelMotion = "spin" | "orbit" | "swing" | "float" | "still" | "keyframes";

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

export type Noise = {
	color1: string,
	color2: string,
	scale: number,
	speed: number,
};

export type Paper = {
	color: string,
	grain: number,
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

export type PlaceholderProps = {
	title: string,
	subtitle: string,
};

export type Point = {
	x: number,
	y: number,
};

export type PreparedTypeface = {
	typeface: string,
	typefaceItalic: boolean,
};

export type PreviewQuality = "fast" | "exact";

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

export type Screen3d = {
	device: ScreenDevice,
	deviceColor: string,
	move: ScreenMove,
	camera: CameraPath,
	endFaceOn: boolean,
	lighting: Lighting,
	background: string,
};

export type ScreenDevice = "laptop" | "monitor" | "phone" | "floating";

export type ScreenMove = "push-in" | "orbit" | "tilt-reveal" | "rise" | "keyframes";

export type SequencerProps = {
	component: string,
	props: { [key in string]: unknown },
	backgrounds: (string | null)[],
};

export type Shapes3d = {
	count: number,
	palette: Colour[],
	seed: number,
	speed: number,
	camera: CameraPath,
	lighting: Lighting,
	background: string,
};

export type Side = "top" | "bottom" | "left" | "right";

export type Slot = {
	nodeId: string,
	port: string,
};

export type Solid = {
	color: string,
};

export type SpecAssets = {
	projectBase?: string,
	versions?: { [key in string]: string },
};

export type Starburst = {
	color1: string,
	color2: string,
	rays: number,
	rotationSpeed: number,
};

export type StillLook = "window" | "cutout" | "bleed";

export type StillProps = {
	src: string,
	look: StillLook,
	backdrop: string,
	shot?: StillShot,
};

export type StillShot = {
	width: number,
	height: number,
	scale: number,
	body?: Bounds,
};

export type StudioBackdrop = {
	name: Backdrop,
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
};

export type TextAlign = "center" | "left" | "right";

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
};

export type TransitionRef = {
	kind: string,
	durationInFrames: number,
	params?: { [key in string]: unknown },
	hold?: boolean,
};

export type VideoLoop = {
	src: string,
	dim: number,
};

export type Video = {
	src: string,
	fit: Fit,
	loop: boolean,
	muted: boolean,
	volume: number,
	playbackRate: number,
	trimBeforeMs: number,
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

export type WindowStyle = "macos" | "windows" | "linux";
