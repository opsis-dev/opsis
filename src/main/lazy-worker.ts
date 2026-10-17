import { randomUUID } from "node:crypto";
import type { MessagePortMain, UtilityProcess } from "electron";
import type { MainToWorker, WorkerEnv, WorkerToMain } from "../shared/worker-protocol";
import type { WorkerKind } from "./worker-temp";
import { onWorkerMessage, spawnWorker, workerEnv } from "./workers";

type LazyWorkerOptions<R> = {
  kind: WorkerKind;
  idleMs: number;
  reply: (msg: WorkerToMain) => { id: string; result: R } | null;
  stopped: R;
};

type LazyWorker<R> = {
  request: (message: (env: WorkerEnv, id: string) => MainToWorker) => Promise<R>;
  scheduleIdle: () => void;
  stop: () => void;
};

function createLazyWorker<R>(options: LazyWorkerOptions<R>): LazyWorker<R> {
  let worker: { child: UtilityProcess; control: MessagePortMain } | null = null;
  let idleTimer: NodeJS.Timeout | null = null;
  const waiting = new Map<string, (result: R) => void>();

  const cancelIdle = () => {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = null;
  };

  const ensure = () => {
    cancelIdle();
    if (worker) return worker;
    const spawned = spawnWorker(options.kind);
    onWorkerMessage(spawned.control, (msg) => {
      const reply = options.reply(msg);
      if (!reply) return;
      const done = waiting.get(reply.id);
      waiting.delete(reply.id);
      done?.(reply.result);
    });
    spawned.child.on("exit", () => {
      if (worker?.child === spawned.child) worker = null;
      for (const [id, done] of waiting) {
        waiting.delete(id);
        done(options.stopped);
      }
    });
    worker = { child: spawned.child, control: spawned.control };
    return worker;
  };

  const stop = () => {
    cancelIdle();
    worker?.child.kill();
    worker = null;
  };

  return {
    request: async (message) => {
      const env = await workerEnv();
      const { control } = ensure();
      const id = randomUUID();
      return new Promise<R>((resolve) => {
        waiting.set(id, resolve);
        control.postMessage(message(env, id));
      });
    },
    scheduleIdle: () => {
      cancelIdle();
      idleTimer = setTimeout(stop, options.idleMs);
    },
    stop,
  };
}

const HOLD_POLL_MS = 2000;

type QueuedWorkerOptions<R, J> = LazyWorkerOptions<R> & {
  keyOf: (job: J) => string;
  run: (job: J, request: LazyWorker<R>["request"]) => Promise<void>;
  failed: (job: J, error: unknown) => void;
  holdWhile?: () => boolean;
};

export function createQueuedWorker<R, J>(options: QueuedWorkerOptions<R, J>) {
  const worker = createLazyWorker(options);
  const jobs: J[] = [];
  let busy = false;
  let held: NodeJS.Timeout | null = null;

  const pump = async (): Promise<void> => {
    if (busy || held || jobs.length === 0) return;
    if (options.holdWhile?.()) {
      held = setTimeout(() => {
        held = null;
        void pump();
      }, HOLD_POLL_MS);
      return;
    }
    const job = jobs.shift();
    if (job === undefined) return;
    busy = true;
    try {
      await options.run(job, worker.request);
    } catch (error) {
      options.failed(job, error);
    } finally {
      busy = false;
      if (jobs.length) void pump();
      else worker.scheduleIdle();
    }
  };

  const queued = (key: string): J | undefined => jobs.find((j) => options.keyOf(j) === key);

  const enqueue = (job: J, { first }: { first: boolean }): void => {
    const key = options.keyOf(job);
    const existing = jobs.findIndex((j) => options.keyOf(j) === key);
    if (existing !== -1) jobs.splice(existing, 1);
    if (first) jobs.unshift(job);
    else jobs.push(job);
    void pump();
  };

  return { queued, enqueue, stop: worker.stop };
}
