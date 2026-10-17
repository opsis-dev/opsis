import type { z } from "zod";
import type { renderNode } from "../../shared/nodes/output";

export type RenderCodec = z.infer<typeof renderNode.params>["codec"];

type CodecInfo = {
  ext: string;
  audio: string[] | null;
  faststart: boolean;
  crf: boolean;
};

const AAC = ["-c:a", "aac", "-b:a", "320k"];
const OPUS = ["-c:a", "libopus", "-b:a", "192k"];

export const CODECS = {
  h264: { ext: "mp4", audio: AAC, faststart: true, crf: true },
  h265: { ext: "mp4", audio: AAC, faststart: true, crf: true },
  vp8: { ext: "webm", audio: OPUS, faststart: false, crf: true },
  vp9: { ext: "webm", audio: OPUS, faststart: false, crf: true },
  prores: { ext: "mov", audio: ["-c:a", "pcm_s16le"], faststart: false, crf: false },
  gif: { ext: "gif", audio: null, faststart: false, crf: false },
} satisfies Record<RenderCodec, CodecInfo>;

export type SoundCodec = Exclude<RenderCodec, "gif">;

export const hasSoundTrack = (codec: RenderCodec): codec is SoundCodec =>
  CODECS[codec].audio !== null;
