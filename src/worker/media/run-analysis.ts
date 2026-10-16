import type { AnalysisRequest, AnalysisResult } from "../../shared/mix/result";
import type { WorkerEnv } from "../../shared/worker-protocol";
import { refToPath } from "../executors/util";
import { renderMix } from "../mix/render";
import { analyzeFilmstrip, analyzeWaveform } from "./analysis";
import { createFfmpegTools } from "./ffmpeg";

export async function runAnalysis(
  env: WorkerEnv,
  request: AnalysisRequest,
): Promise<AnalysisResult> {
  const ffmpeg = createFfmpegTools(env.engineDir);
  const deps = {
    projectDir: request.projectDir,
    ffmpeg,
    resolve: (ref: string) => refToPath({ projectDir: request.projectDir, env }, ref),
  };
  try {
    switch (request.kind) {
      case "waveform":
        return { kind: "waveform", waveform: await analyzeWaveform(request.src, deps) };
      case "filmstrip":
        return { kind: "filmstrip", filmstrip: await analyzeFilmstrip(request.src, deps) };
      case "mix":
        return { kind: "mix", mix: await renderMix(request.plan, deps) };
    }
  } catch (error) {
    return { kind: "error", error: error instanceof Error ? error.message : String(error) };
  }
}
