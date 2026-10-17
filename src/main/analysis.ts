import { engineStatus } from "../engine/paths";
import { ENGINE_MISSING_MESSAGE } from "../shared/engine-message";
import { errorMessage } from "../shared/errors";
import type { AnalysisRequest, AnalysisResult } from "../shared/analysis";
import { createLazyWorker } from "./lazy-worker";
import { engineDir } from "./workers";

type Job = { request: AnalysisRequest; key: string; done: Array<(r: AnalysisResult) => void> };

const queue: Job[] = [];
let busy = false;

const worker = createLazyWorker<AnalysisResult>({
  kind: "analysis",
  idleMs: 90_000,
  reply: (msg) => (msg.type === "analysis:done" ? { id: msg.id, result: msg.result } : null),
  stopped: { kind: "error", error: "The sound worker stopped." },
});

const keyOf = (r: AnalysisRequest) =>
  r.kind === "mix" ? `mix:${r.projectDir}` : `${r.kind}:${r.projectDir}:${r.src}`;

export function analyze(request: AnalysisRequest): Promise<AnalysisResult> {
  if (!engineStatus(engineDir()).ffmpeg)
    return Promise.resolve({ kind: "error", error: ENGINE_MISSING_MESSAGE, engineMissing: true });
  const key = keyOf(request);
  return new Promise((resolve) => {
    const existing = queue.find((j) => j.key === key);
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
    if (request.kind === "mix") queue.unshift(job);
    else queue.push(job);
    void pump();
  });
}

async function pump(): Promise<void> {
  if (busy) return;
  const job = queue.shift();
  if (!job) return;
  busy = true;
  try {
    const result = await worker.request((env, id) => ({
      type: "analysis",
      env,
      id,
      request: job.request,
    }));
    for (const done of job.done) done(result);
  } catch (error) {
    for (const done of job.done) done({ kind: "error", error: errorMessage(error) });
  } finally {
    busy = false;
    if (queue.length) void pump();
    else worker.scheduleIdle();
  }
}

export const stopAnalysis = (): void => worker.stop();
