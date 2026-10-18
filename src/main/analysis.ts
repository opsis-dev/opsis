import { engineStatus } from "../engine/paths";
import type { AnalysisRequest, AnalysisResult } from "../shared/analysis";
import { ENGINE_MISSING_MESSAGE } from "../shared/engine";
import { errorMessage } from "../shared/errors";
import { createQueuedWorker } from "./lazy-worker";
import { engineDir } from "./workers";

type Job = { request: AnalysisRequest; key: string; done: Array<(r: AnalysisResult) => void> };

const analysis = createQueuedWorker<AnalysisResult, Job>({
  kind: "analysis",
  idleMs: 90_000,
  keyOf: (job) => job.key,
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
  return new Promise((resolve) => {
    const existing = analysis.queued(key);
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
    analysis.enqueue({ request, key, done: [resolve] }, { first: request.kind === "mix" });
  });
}

export const stopAnalysis = (): void => analysis.stop();
