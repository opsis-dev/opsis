import { randomUUID } from "node:crypto";
import type { MessagePortMain, UtilityProcess } from "electron";
import type { MainToWorker, WorkerEnv, WorkerToMain } from "../shared/worker-protocol";
import type { WorkerKind } from "./worker-temp";
import { spawnWorker, workerEnv } from "./workers";

export type LazyWorkerOptions<R> = {
  kind: WorkerKind;
  idleMs: number;
  reply: (msg: WorkerToMain) => { id: string; result: R } | null;
  stopped: R;
};

export type LazyWorker<R> = {
  request: (message: (env: WorkerEnv, id: string) => MainToWorker) => Promise<R>;
  scheduleIdle: () => void;
  stop: () => void;
};

export function createLazyWorker<R>(options: LazyWorkerOptions<R>): LazyWorker<R> {
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
    const spawned = spawnWorker(false, options.kind);
    spawned.control.on("message", (e) => {
      const reply = options.reply(e.data as WorkerToMain);
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
