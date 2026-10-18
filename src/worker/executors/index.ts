import type { Executor } from "../engine/context";
import { loginOnceExecutor, recordExecutor, stillExecutor } from "./capture";
import { exportProjectExecutor } from "./export-project";
import { mediaExecutors } from "./media";
import { renderExecutor, stillOutputExecutor } from "./output";

const EXECUTORS: Executor[] = [
  recordExecutor,
  stillExecutor,
  loginOnceExecutor,
  ...mediaExecutors,
  renderExecutor,
  stillOutputExecutor,
  exportProjectExecutor,
];

const byType = new Map(EXECUTORS.map((e) => [e.type, e]));

export const getExecutor = (type: string): Executor | undefined => byType.get(type);
