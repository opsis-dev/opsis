import type { TimelineSpec } from "../../shared/schema/timeline";
import { ENGINE_VERSION } from "../engine/cache";
import { pruneDir } from "../../engine/dir";
import { versionedHash } from "../../engine/hash";
import { CODECS, FASTSTART, type RenderCodec, type SoundCodec } from "../media/codecs";
import type { GlRenderer } from "./render";

const PICTURE_VERSION = "demoflow-picture/3";
const KEEP_PICTURES = 12;

type PictureOptions = { codec: RenderCodec; crf: number; scale: number; gl: GlRenderer };

export function pictureKey(spec: TimelineSpec, opts: PictureOptions): string {
  const picture = { ...spec, audio: undefined, mix: undefined, assets: undefined };
  return versionedHash(PICTURE_VERSION, { engine: ENGINE_VERSION, picture, opts });
}

export function muxArgs(codec: SoundCodec, picture: string, sound: string, out: string): string[] {
  return [
    "-y",
    "-i",
    picture,
    "-i",
    sound,
    "-map",
    "0:v:0",
    "-map",
    "1:a:0",
    "-c:v",
    "copy",
    ...CODECS[codec].audio,
    "-shortest",
    out,
  ];
}

export function finishArgs(
  input: string,
  out: string,
  opts: { faststart: boolean; chapters: string | null },
): string[] {
  return [
    "-y",
    "-i",
    input,
    ...(opts.chapters ? ["-i", opts.chapters, "-map_metadata", "1", "-map_chapters", "1"] : []),
    "-map",
    "0",
    "-c",
    "copy",
    ...(opts.faststart ? FASTSTART : []),
    out,
  ];
}

export async function prunePictures(dir: string): Promise<void> {
  await pruneDir(dir, { newest: KEEP_PICTURES });
}
