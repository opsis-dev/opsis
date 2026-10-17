import { readdir, rm, stat } from "node:fs/promises";
import path from "node:path";
import type { TimelineSpec } from "../../shared/schema/timeline";
import { ENGINE_VERSION } from "../engine/cache";
import { versionedHash } from "../../engine/hash";
import { CODECS, type SoundCodec } from "./codecs";

const PICTURE_VERSION = "demoflow-picture/2";
const KEEP_PICTURES = 12;

type PictureOptions = { codec: string; crf: number; scale: number; gl: string };

export function pictureKey(spec: TimelineSpec, opts: PictureOptions): string {
  const { audio: _audio, mix: _mix, assets: _assets, ...picture } = spec;
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
    ...(opts.faststart ? ["-movflags", "+faststart"] : []),
    out,
  ];
}

export async function prunePictures(dir: string): Promise<void> {
  const names = await readdir(dir).catch(() => [] as string[]);
  const files = await Promise.all(
    names.map(async (name) => {
      const file = path.join(dir, name);
      return { file, at: (await stat(file).catch(() => null))?.mtimeMs ?? 0 };
    }),
  );
  const old = files.toSorted((a, b) => b.at - a.at).slice(KEEP_PICTURES);
  await Promise.all(old.map((f) => rm(f.file, { force: true })));
}
