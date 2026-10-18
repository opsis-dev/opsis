import { randomUUID } from "node:crypto";
import type { MessagePortMain, UtilityProcess } from "electron";
import { pendingReplies } from "../shared/pending-replies";
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
  const replies = pendingReplies<R>();

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
      if (reply) replies.settle(reply.id, reply.result);
    });
    spawned.child.on("exit", () => {
      if (worker?.child === spawned.child) worker = null;
      replies.settleAll(options.stopped);
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
      const reply = replies.wait(id);
      control.postMessage(message(env, id));
      return reply;
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
