import path from "node:path";
import { projectRef, refTarget } from "@opsis/core/asset-ref";
import { asValue, type CompileValue, type MediaValue } from "@opsis/core/schema/values";
import type { WorkerEnv } from "@opsis/core/worker-protocol";
import type { ExportedSource } from "@opsis/sidecar/source-layout";
import type { ExecutorContext } from "../engine/context";
import type { FfmpegTools } from "../media/ffmpeg";

type SourceEnv = Pick<WorkerEnv, "isPackaged" | "appRoot" | "resourcesPath">;

const exportSourceRoot = (env: SourceEnv): string => path.join(env.resourcesPath, "export-src");

export const exportedSource = (env: SourceEnv, source: ExportedSource): string =>
  env.isPackaged
    ? path.join(exportSourceRoot(env), source.to)
    : path.join(env.appRoot, ...source.from);

export const publicDir = (env: SourceEnv): string =>
  env.isPackaged ? path.join(exportSourceRoot(env), "public") : path.join(env.appRoot, "public");

export type LinkedMedia = MediaValue & { src: string };

export function refToPath(ctx: { projectDir: string; env: SourceEnv }, ref: string): string {
  const target = refTarget(ref);
  if (target.kind === "project") return path.join(ctx.projectDir, target.rel);
  if (target.kind === "app") return path.join(publicDir(ctx.env), target.rel);
  if (target.kind === "absolute") return target.path;
  throw new Error("Use a file on this computer, not a web address.");
}

const isLinked = (media: MediaValue | undefined): media is LinkedMedia => !!media?.src;

export function requireMedia(value: CompileValue | undefined): LinkedMedia {
  const media = asValue(value, "media");
  if (!isLinked(media))
    throw new Error("Connect a media input (and run the node that produces it).");
  return media;
}

export async function mediaHasSound(
  ctx: { projectDir: string; env: SourceEnv; ffmpeg: Pick<FfmpegTools, "probe"> },
  media: LinkedMedia,
): Promise<boolean> {
  return media.hasAudio ?? !!(await ctx.ffmpeg.probe(refToPath(ctx, media.src))).audio;
}

export const rel = (ctx: Pick<ExecutorContext, "relDir">, file: string) => `${ctx.relDir}/${file}`;

export const projectFileRef = (ctx: Pick<ExecutorContext, "relDir">, file: string) =>
  projectRef(rel(ctx, file));
