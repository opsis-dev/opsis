import { randomUUID } from "node:crypto";
import type { MessagePortMain, UtilityProcess } from "electron";
import { engineStatus } from "../engine/paths";
import type { AnalysisRequest, AnalysisResult } from "../shared/mix/result";
import type { MainToWorker, WorkerToMain } from "../shared/worker-protocol";
import { engineDir, spawnWorker, workerEnv } from "./workers";

const IDLE_MS = 90_000;

type Job = { request: AnalysisRequest; key: string; done: Array<(r: AnalysisResult) => void> };

const queue: Job[] = [];
let busy = false;
let worker: { child: UtilityProcess; control: MessagePortMain } | null = null;
let idleTimer: NodeJS.Timeout | null = null;
const waiting = new Map<string, (r: AnalysisResult) => void>();

const keyOf = (r: AnalysisRequest) =>
  r.kind === "mix" ? `mix:${r.projectDir}` : `${r.kind}:${r.projectDir}:${r.src}`;

export function analyze(request: AnalysisRequest): Promise<AnalysisResult> {
  if (!engineStatus(engineDir()).ffmpeg)
    return Promise.resolve({ kind: "error", error: "The engine isn't set up yet." });
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

function ensureWorker() {
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = null;
  if (worker) return worker;
  const spawned = spawnWorker(false, "analysis");
  spawned.control.on("message", (e) => {
    const msg = e.data as WorkerToMain;
    if (msg.type !== "analysis:done") return;
    const done = waiting.get(msg.id);
    waiting.delete(msg.id);
    done?.(msg.result);
  });
  spawned.child.on("exit", () => {
    if (worker?.child === spawned.child) worker = null;
    for (const [id, done] of waiting) {
      waiting.delete(id);
      done({ kind: "error", error: "The sound worker stopped." });
    }
  });
  worker = { child: spawned.child, control: spawned.control };
  return worker;
}

function scheduleIdle() {
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    worker?.child.kill();
    worker = null;
  }, IDLE_MS);
}

async function pump(): Promise<void> {
  if (busy || queue.length === 0) return;
  busy = true;
  const job = queue.shift()!;
  try {
    const { control } = ensureWorker();
    const env = await workerEnv();
    const id = randomUUID();
    const result = await new Promise<AnalysisResult>((resolve) => {
      waiting.set(id, resolve);
      control.postMessage({
        type: "analysis",
        env,
        id,
        request: job.request,
      } satisfies MainToWorker);
    });
    for (const done of job.done) done(result);
  } catch (error) {
    for (const done of job.done)
      done({ kind: "error", error: error instanceof Error ? error.message : String(error) });
  } finally {
    busy = false;
    if (queue.length) void pump();
    else scheduleIdle();
  }
}

export function stopAnalysis(): void {
  worker?.child.kill();
  worker = null;
}
