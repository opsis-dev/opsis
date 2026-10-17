import { engineStatus } from "../engine/paths";
import { ENGINE_MISSING_MESSAGE } from "../shared/engine-message";
import { errorMessage } from "../shared/errors";
import type { AnalysisRequest, AnalysisResult } from "../shared/analysis";
import { createQueuedWorker } from "./lazy-worker";
import { engineDir } from "./workers";

type Job = { request: AnalysisRequest; key: string; done: Array<(r: AnalysisResult) => void> };

const analysis = createQueuedWorker<AnalysisResult, Job>({
  kind: "analysis",
  idleMs: 90_000,
  reply: (msg) => (msg.type === "analysis:done" ? { id: msg.id, result: msg.result } : null),
  stopped: { kind: "error", error: "The sound worker stopped." },
  run: async (job, request) => {
    const result = await request((env, id) => ({
      type: "analysis",
      env,
      id,
      request: job.request,
    }));
    for (const done of job.done) done(result);
  },
  failed: (job, error) => {
    for (const done of job.done) done({ kind: "error", error: errorMessage(error) });
  },
});

const keyOf = (r: AnalysisRequest) =>
  r.kind === "mix" ? `mix:${r.projectDir}` : `${r.kind}:${r.projectDir}:${r.src}`;

export function analyze(request: AnalysisRequest): Promise<AnalysisResult> {
  if (!engineStatus(engineDir()).ffmpeg)
    return Promise.resolve({ kind: "error", error: ENGINE_MISSING_MESSAGE, engineMissing: true });
  const key = keyOf(request);
  const { jobs } = analysis;
  return new Promise((resolve) => {
    const existing = jobs.find((j) => j.key === key);
    if (existing && request.kind === "mix") {
      for (const done of existing.done) done({ kind: "skipped" });
      existing.request = request;
      existing.done = [resolve];
      return;
    }
    if (existing) {
      existing.done.push(resolve);
      return;
    }
    const job: Job = { request, key, done: [resolve] };
    if (request.kind === "mix") jobs.unshift(job);
    else jobs.push(job);
    analysis.pump();
  });
}

export const stopAnalysis = (): void => analysis.stop();
