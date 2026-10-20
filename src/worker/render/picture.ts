import { existsSync } from "node:fs";
import { mkdir, utimes } from "node:fs/promises";
import path from "node:path";
import { writeViaTemp } from "../../engine/atomic";
import { pruneDir } from "../../engine/dir";
import { versionedHash } from "../../engine/hash";
import { PICTURE_VERSION } from "../../engine/picture-version";
import { projectPaths } from "../../engine/project-paths";
import type { TimelineSpec } from "../../shared/schema/timeline";
import { ENGINE_VERSION } from "../engine/cache";
import { CODECS, FASTSTART, type RenderCodec, type SoundCodec } from "../media/codecs";
import type { GlRenderer } from "./render";

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

export async function cachedPicture(
  projectDir: string,
  name: string,
  render: (temp: string) => Promise<unknown>,
): Promise<{ file: string; reused: boolean }> {
  const dir = projectPaths(projectDir).pictures;
  await mkdir(dir, { recursive: true });
  const file = path.join(dir, name);
  if (existsSync(file)) {
    const now = new Date();
    await utimes(file, now, now).catch(() => undefined);
    return { file, reused: true };
  }
  await writeViaTemp(file, render);
  await pruneDir(dir, { newest: KEEP_PICTURES });
  return { file, reused: false };
}
