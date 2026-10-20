import type { Executor } from "../engine/context";
import { recordExecutor, signInExecutor, stillExecutor } from "./capture";
import { exportProjectExecutor } from "./export-project";
import { mediaExecutors } from "./media";
import { renderExecutor, stillOutputExecutor } from "./output";

export const EXECUTORS: Executor[] = [
  recordExecutor,
  stillExecutor,
  signInExecutor,
  ...mediaExecutors,
  renderExecutor,
  stillOutputExecutor,
  exportProjectExecutor,
];

export function executorTable(executors: readonly Executor[]): Map<string, Executor> {
  const byType = new Map<string, Executor>();
  for (const e of executors) {
    if (byType.has(e.type)) throw new Error(`Two executors for ${e.type}`);
    byType.set(e.type, e);
  }
  return byType;
}

const byType = executorTable(EXECUTORS);

export const getExecutor = (type: string): Executor | undefined => byType.get(type);
