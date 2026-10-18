import path from "node:path";
import { mediaInput } from "../../shared/nodes/media";
import type { CompileValue, MediaValue } from "../../shared/schema/values";
import type { ExecutorContext } from "../engine/context";
import type { WorkerEnv } from "../../shared/worker-protocol";
import { projectRef, refTarget } from "../../shared/asset-ref";

type SourceEnv = Pick<WorkerEnv, "isPackaged" | "appRoot" | "resourcesPath">;

export const exportSourceRoot = (env: SourceEnv): string =>
  env.isPackaged ? path.join(env.resourcesPath, "export-src") : path.join(env.appRoot, "src");

export const publicDir = (env: SourceEnv): string =>
  env.isPackaged ? path.join(exportSourceRoot(env), "public") : path.join(env.appRoot, "public");

export const exportUtilsFile = (env: SourceEnv): string =>
  env.isPackaged
    ? path.join(exportSourceRoot(env), "utils.ts")
    : path.join(exportSourceRoot(env), "renderer", "lib", "utils.ts");

export type LinkedMedia = MediaValue & { src: string };

export function refToPath(ctx: Pick<ExecutorContext, "projectDir" | "env">, ref: string): string {
  const target = refTarget(ref);
  if (target.kind === "project") return path.join(ctx.projectDir, target.rel);
  if (target.kind === "app") return path.join(publicDir(ctx.env), target.rel);
  if (target.kind === "absolute") return target.path;
  throw new Error("Use a file on this computer, not a web address.");
}

const isLinked = (media: MediaValue | undefined): media is LinkedMedia => !!media?.src;

export function requireMedia(value: CompileValue | undefined): LinkedMedia {
  const media = mediaInput(value);
  if (!isLinked(media))
    throw new Error("Connect a media input (and run the node that produces it).");
  return media;
}

export const rel = (ctx: Pick<ExecutorContext, "relDir">, file: string) => `${ctx.relDir}/${file}`;

export const projectFileRef = (ctx: Pick<ExecutorContext, "relDir">, file: string) =>
  projectRef(rel(ctx, file));
