import { lerp } from "../../shared/math";
import { applyMixLevels, withMasterStem } from "../../shared/mix/levels";
import { buildMixPlan, hasSound } from "../../shared/mix/plan";
import type { TimelineSpec } from "../../shared/schema/timeline";
import type { ExecutorContext } from "../engine/context";
import { renderMix } from "../mix/render";
import { refToPath } from "./util";

export type Stage = readonly [from: number, to: number];

export function specFor(ctx: ExecutorContext): TimelineSpec {
  const result = ctx.compile();
  const error = result.errors[ctx.node.id];
  if (error) throw new Error(error);
  const spec = result.specs[ctx.node.id];
  if (!spec) throw new Error("Connect a clip to this node.");
  return spec;
}

export async function mixed(ctx: ExecutorContext, spec: TimelineSpec, stage: Stage) {
  const plan = buildMixPlan(spec, ctx.workflow.mix);
  if (!hasSound(plan)) return applyMixLevels(spec, ctx.workflow.mix);
  const result = await renderMix(plan, {
    projectDir: ctx.projectDir,
    resolve: (ref) => refToPath(ctx, ref),
    ffmpeg: ctx.ffmpeg,
    signal: ctx.signal,
    onProgress: (p, message) => ctx.progress(lerp(...stage, p), message),
  });
  if (!result.master) return applyMixLevels(spec, ctx.workflow.mix);
  const lufs = result.loudness?.integrated;
  const peak = result.loudness?.truePeak;
  if (lufs != null)
    ctx.log("info", `Sound: ${lufs} LUFS${peak != null ? `, peak ${peak} dBTP` : ""}`);
  return withMasterStem(spec, result.master);
}
