import type { NodeParams } from "../../shared/nodes/define";
import type { convertNode, extractAudioNode } from "../../shared/nodes/media";
import type { renderNode } from "../../shared/nodes/output";

export type RenderCodec = NodeParams<typeof renderNode>["codec"];

type ConvertParams = NodeParams<typeof convertNode>;

type AudioFormat = NodeParams<typeof extractAudioNode>["format"];

export const FASTSTART = ["-movflags", "+faststart"];

type H264 = { crf: number; preset?: string; keyframeEvery?: number; faststart?: boolean };

export const h264Args = ({ crf, preset, keyframeEvery, faststart = false }: H264): string[] => [
  "-c:v",
  "libx264",
  ...(preset ? ["-preset", preset] : []),
  "-crf",
  String(crf),
  ...(keyframeEvery ? ["-g", String(keyframeEvery)] : []),
  "-pix_fmt",
  "yuv420p",
  ...(faststart ? FASTSTART : []),
];

const AAC = ["-c:a", "aac"];
const PCM_16 = ["-c:a", "pcm_s16le"];

type CodecInfo = {
  ext: string;
  audio: string[] | null;
  faststart: boolean;
  crf: boolean;
};

const AAC_EXPORT = [...AAC, "-b:a", "320k"];
const OPUS_EXPORT = ["-c:a", "libopus", "-b:a", "192k"];

export const CODECS = {
  h264: { ext: "mp4", audio: AAC_EXPORT, faststart: true, crf: true },
  h265: { ext: "mp4", audio: AAC_EXPORT, faststart: true, crf: true },
  vp8: { ext: "webm", audio: OPUS_EXPORT, faststart: false, crf: true },
  vp9: { ext: "webm", audio: OPUS_EXPORT, faststart: false, crf: true },
  prores: { ext: "mov", audio: PCM_16, faststart: false, crf: false },
  gif: { ext: "gif", audio: null, faststart: false, crf: false },
} satisfies Record<RenderCodec, CodecInfo>;

export type SoundCodec = Exclude<RenderCodec, "gif">;

export const hasSoundTrack = (codec: RenderCodec): codec is SoundCodec =>
  CODECS[codec].audio !== null;

export const EDIT_VIDEO = h264Args({ crf: 18 });

export const EDIT_AUDIO = AAC;

export const CONVERT: Record<
  ConvertParams["codec"],
  { ext: string; args: (crf: number, preset: ConvertParams["preset"]) => string[] }
> = {
  h264: {
    ext: "mp4",
    args: (crf, preset) => [...h264Args({ crf, preset, faststart: true }), ...AAC],
  },
  h265: {
    ext: "mp4",
    args: (crf, preset) => [
      "-c:v",
      "libx265",
      "-crf",
      String(crf),
      "-preset",
      preset,
      "-tag:v",
      "hvc1",
      "-pix_fmt",
      "yuv420p",
      ...AAC,
    ],
  },
  vp9: {
    ext: "webm",
    args: (crf) => ["-c:v", "libvpx-vp9", "-crf", String(crf), "-b:v", "0", "-c:a", "libopus"],
  },
  prores: {
    ext: "mov",
    args: () => ["-c:v", "prores_ks", "-profile:v", "3", "-pix_fmt", "yuv422p10le", ...PCM_16],
  },
};

export const AUDIO_FORMATS: Record<AudioFormat, string[]> = {
  m4a: [...AAC, "-b:a", "192k"],
  mp3: ["-c:a", "libmp3lame", "-q:a", "2"],
  wav: PCM_16,
};

type FrameGrab = { atMs: number; width: number; evenHeight?: boolean; jpegQuality?: number };

export const frameGrabArgs = (
  input: string,
  output: string,
  { atMs, width, evenHeight = false, jpegQuality }: FrameGrab,
): string[] => [
  "-ss",
  (atMs / 1000).toFixed(3),
  "-i",
  input,
  "-frames:v",
  "1",
  "-vf",
  `scale=${width}:${evenHeight ? -2 : -1}`,
  ...(jpegQuality ? ["-q:v", String(jpegQuality)] : []),
  output,
];
