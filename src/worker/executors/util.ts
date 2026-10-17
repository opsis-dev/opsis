import path from "node:path";
import { mediaInput } from "../../shared/nodes/media";
import type { CompileValue, MediaValue } from "../../shared/schema/values";
import type { ExecutorContext } from "../engine/context";
import type { WorkerEnv } from "../../shared/worker-protocol";

export function publicDir(
  env: Pick<WorkerEnv, "isPackaged" | "appRoot" | "resourcesPath">,
): string {
  return env.isPackaged
    ? path.join(env.resourcesPath, "export-src", "public")
    : path.join(env.appRoot, "public");
}

export function refToPath(ctx: Pick<ExecutorContext, "projectDir" | "env">, ref: string): string {
  if (ref.startsWith("project:")) return path.join(ctx.projectDir, ref.slice("project:".length));
  if (ref.startsWith("app:")) return path.join(publicDir(ctx.env), ref.slice("app:".length));
  return path.isAbsolute(ref) ? ref : path.join(ctx.projectDir, ref);
}

export function requireMedia(value: CompileValue | undefined): MediaValue & { src: string } {
  const media = mediaInput(value);
  if (!media?.src) throw new Error("Connect a media input (and run the node that produces it).");
  return { ...media, src: media.src };
}

export const rel = (ctx: Pick<ExecutorContext, "relDir">, file: string) => `${ctx.relDir}/${file}`;
