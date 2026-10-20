import { isKeyOf } from "../../shared/guards";
import type { NodeParams } from "../../shared/nodes/define";
import type { convertNode, extractAudioNode } from "../../shared/nodes/media";
import type { renderNode } from "../../shared/nodes/output";

export type RenderCodec = NodeParams<typeof renderNode>["codec"];

type ConvertParams = NodeParams<typeof convertNode>;

type AudioFormat = NodeParams<typeof extractAudioNode>["format"];

export const FASTSTART = ["-movflags", "+faststart"];

export const EVEN_FRAME = "crop=trunc(iw/2)*2:trunc(ih/2)*2";

export const evenDown = (size: number): number => size - (size % 2);

export const withEvenFrame = (filter: string): string => `${filter},${EVEN_FRAME}`;

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
export const PCM_16 = ["-c:a", "pcm_s16le"];

type Preset = ConvertParams["preset"];

type CodecInfo = {
  ext: string;
  audio: string[] | null;
  faststart: boolean;
  crf: boolean;
  video: ((crf: number, preset: Preset) => string[]) | null;
};

const AAC_EXPORT = [...AAC, "-b:a", "320k"];
const OPUS_EXPORT = ["-c:a", "libopus", "-b:a", "192k"];

export const CODECS = {
  h264: {
    ext: "mp4",
    audio: AAC_EXPORT,
    faststart: true,
    crf: true,
    video: (crf, preset) => ["-vf", EVEN_FRAME, ...h264Args({ crf, preset })],
  },
  h265: {
    ext: "mp4",
    audio: AAC_EXPORT,
    faststart: true,
    crf: true,
    video: (crf, preset) => [
      "-vf",
      EVEN_FRAME,
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
    ],
  },
  vp8: { ext: "webm", audio: OPUS_EXPORT, faststart: false, crf: true, video: null },
  vp9: {
    ext: "webm",
    audio: OPUS_EXPORT,
    faststart: false,
    crf: true,
    video: (crf) => ["-c:v", "libvpx-vp9", "-crf", String(crf), "-b:v", "0"],
  },
  prores: {
    ext: "mov",
    audio: PCM_16,
    faststart: false,
    crf: false,
    video: () => ["-c:v", "prores_ks", "-profile:v", "3", "-pix_fmt", "yuv422p10le"],
  },
  gif: { ext: "gif", audio: null, faststart: false, crf: false, video: null },
} satisfies Record<RenderCodec, CodecInfo>;

export type SoundCodec = Exclude<RenderCodec, "gif">;

export const hasSoundTrack = (codec: RenderCodec): codec is SoundCodec =>
  CODECS[codec].audio !== null;

export const kindOfCodec = (codec: RenderCodec): "image" | "video" =>
  codec === "gif" ? "image" : "video";

export const EDIT_VIDEO = h264Args({ crf: 18 });

export const EDIT_AUDIO = AAC;

export function convertArgs(codec: ConvertParams["codec"], crf: number, preset: Preset): string[] {
  const { video, audio, faststart } = CODECS[codec];
  return [...video(crf, preset), ...(faststart ? FASTSTART : []), ...audio];
}

export const AUDIO_FORMATS: Record<AudioFormat, string[]> = {
  m4a: [...AAC, "-b:a", "192k"],
  mp3: ["-c:a", "libmp3lame", "-q:a", "2"],
  wav: PCM_16,
};

const isAudioFormat = (ext: string): ext is AudioFormat => isKeyOf(AUDIO_FORMATS, ext);

export const audioFormatFor = (ext: string): AudioFormat => (isAudioFormat(ext) ? ext : "m4a");

export const ffmpegSeconds = (ms: number): string => (ms / 1000).toFixed(3);

type FrameGrab = { atMs: number; width: number; evenHeight?: boolean; jpegQuality?: number };

export const frameGrabArgs = (
  input: string,
  output: string,
  { atMs, width, evenHeight = false, jpegQuality }: FrameGrab,
): string[] => [
  "-ss",
  ffmpegSeconds(atMs),
  "-i",
  input,
  "-frames:v",
  "1",
  "-vf",
  `scale=${width}:${evenHeight ? -2 : -1}`,
  ...(jpegQuality ? ["-q:v", String(jpegQuality)] : []),
  output,
];

const SOUND_FILES: Record<string, readonly string[]> = {
  ".mp3": AUDIO_FORMATS.mp3,
  ".wav": AUDIO_FORMATS.wav,
  ".m4a": AUDIO_FORMATS.m4a,
};

export function editedFile(
  ext: string,
  streams: { video: boolean; audio: boolean },
): { ext: string; codecs: string[] } | null {
  if (streams.video)
    return {
      ext: ".mp4",
      codecs: ["-vf", EVEN_FRAME, ...EDIT_VIDEO, ...(streams.audio ? EDIT_AUDIO : ["-an"])],
    };
  if (!streams.audio) return null;
  const sound = SOUND_FILES[ext];
  return sound
    ? { ext, codecs: ["-vn", ...sound] }
    : { ext: ".m4a", codecs: ["-vn", ...AUDIO_FORMATS.m4a] };
}
