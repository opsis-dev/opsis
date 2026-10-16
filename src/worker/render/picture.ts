import { createHash } from "node:crypto";
import { readdir, rm, stat } from "node:fs/promises";
import path from "node:path";
import { stableJson } from "../../shared/hash";
import type { TimelineSpec } from "../../shared/schema/timeline";
import { ENGINE_VERSION } from "../engine/cache";

export const PICTURE_VERSION = "demoflow-picture/1";
export const KEEP_PICTURES = 12;

export type PictureOptions = { codec: string; crf?: number; scale?: number; gl: string };

export function pictureKey(spec: TimelineSpec, opts: PictureOptions): string {
  const { audio: _audio, mix: _mix, assets: _assets, ...picture } = spec;
  return createHash("sha256")
    .update(stableJson({ v: PICTURE_VERSION, engine: ENGINE_VERSION, picture, opts }))
    .digest("hex")
    .slice(0, 32);
}

const AUDIO_CODEC: Record<string, string[]> = {
  h264: ["-c:a", "aac", "-b:a", "320k"],
  h265: ["-c:a", "aac", "-b:a", "320k"],
  vp8: ["-c:a", "libopus", "-b:a", "192k"],
  vp9: ["-c:a", "libopus", "-b:a", "192k"],
  prores: ["-c:a", "pcm_s16le"],
};

export const hasSoundTrack = (codec: string): boolean => codec in AUDIO_CODEC;

export function muxArgs(codec: string, picture: string, sound: string, out: string): string[] {
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
    ...(AUDIO_CODEC[codec] ?? AUDIO_CODEC.h264),
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
    ...(opts.faststart ? ["-movflags", "+faststart"] : []),
    out,
  ];
}

export async function prunePictures(dir: string, keep = KEEP_PICTURES): Promise<void> {
  const names = await readdir(dir).catch(() => [] as string[]);
  const files = await Promise.all(
    names.map(async (name) => {
      const file = path.join(dir, name);
      return { file, at: (await stat(file).catch(() => null))?.mtimeMs ?? 0 };
    }),
  );
  const old = files.toSorted((a, b) => b.at - a.at).slice(keep);
  await Promise.all(old.map((f) => rm(f.file, { force: true })));
}
