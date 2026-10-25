export type CancelJob = {
	target: number,
};

export type Codec = "h264" | "h265" | "vp8" | "vp9" | "prores" | "gif";

export type Gl = "angle" | "swangle" | "egl" | "vulkan" | "swiftshader";

export type JobProgress = {
	id: number,
	progress: number,
};

export type MediaJob = {
	output: string,
	codec: Codec,
	crf?: number,
	scale: number,
	muted: boolean,
} & RenderTarget;

export type ProgressMethod = "progress";

export type RenderTarget = {
	serveUrl: string,
	browserExecutable: string,
	binariesDirectory: string | null,
	licenseKey: string,
	composition: string,
	spec: { [key in string]: unknown },
	gl: Gl,
};

export type SidecarCall = { method: "render.media"; params: MediaJob } | { method: "render.still"; params: StillJob } | { method: "cancel"; params: CancelJob } | { method: "ping" };

export type SidecarFailure = {
	message: string,
};

export type SidecarMessage = ({ method: ProgressMethod; params: JobProgress }) & { error?: never; id?: never; result?: never } | ({ id: number; error: SidecarFailure }) & { method?: never; params?: never; result?: never } | ({ id: number; result: null }) & { error?: never; method?: never; params?: never };

export type SidecarRequest = {
	id: number,
} & SidecarCall;

export type StillFormat = "png" | "jpeg";

export type StillJob = {
	frame: number,
	output: string,
	format: StillFormat,
	scale?: number,
	jpegQuality?: number,
} & RenderTarget;
