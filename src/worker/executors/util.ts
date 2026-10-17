import path from "node:path";
import { mediaInput } from "../../shared/nodes/media";
import type { CompileValue, MediaValue } from "../../shared/schema/values";
import type { ExecutorContext } from "../engine/context";
import type { WorkerEnv } from "../../shared/worker-protocol";
import { appRelPath, isProjectRef, projectRef, projectRelPath } from "../../shared/asset-ref";

type SourceEnv = Pick<WorkerEnv, "isPackaged" | "appRoot" | "resourcesPath">;

export const exportSourceRoot = (env: SourceEnv): string =>
  env.isPackaged ? path.join(env.resourcesPath, "export-src") : path.join(env.appRoot, "src");

export const publicDir = (env: SourceEnv): string =>
  env.isPackaged ? path.join(exportSourceRoot(env), "public") : path.join(env.appRoot, "public");

export function refToPath(ctx: Pick<ExecutorContext, "projectDir" | "env">, ref: string): string {
  if (isProjectRef(ref)) return path.join(ctx.projectDir, projectRelPath(ref));
  const app = appRelPath(ref);
  if (app !== null) return path.join(publicDir(ctx.env), app);
  return path.isAbsolute(ref) ? ref : path.join(ctx.projectDir, ref);
}

type LinkedMedia = MediaValue & { src: string };

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
