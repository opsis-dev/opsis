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
  merge: (queued, incoming) => {
    if (incoming.request.kind !== "mix")
      return { ...queued, done: [...queued.done, ...incoming.done] };
    for (const done of queued.done) done({ kind: "skipped" });
    return incoming;
  },
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
  return new Promise((resolve) =>
    analysis.enqueue(
      { request, key: keyOf(request), done: [resolve] },
      { first: request.kind === "mix" },
    ),
  );
}

export const stopAnalysis = (): void => analysis.stop();
