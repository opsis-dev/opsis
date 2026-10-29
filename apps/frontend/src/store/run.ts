import { create } from "zustand";
import {
  commands,
  type LogLine,
  type RunProgress,
  type RunState,
  type RunView,
} from "@/generated/bindings";

type RunMirror = RunProgress & Omit<RunView, "progress"> & { logs: LogLine[] };

const MAX_LOG_LINES = 2000;

const idle: RunMirror = {
  seq: 0,
  runId: null,
  statuses: {},
  progress: {},
  cached: {},
  errors: {},
  traces: {},
  results: {},
  running: false,
  outputs: [],
  lastError: null,
  failedNodeIds: [],
  skippedNodeIds: [],
  exports: [],
  logs: [],
};

export const useRunStore = create<RunMirror>()(() => idle);

const flat = ({ progress, ...rest }: RunView) => ({ ...progress, ...rest });

export function showRunView(view: RunView): void {
  if (view.seq >= useRunStore.getState().seq) useRunStore.setState(flat(view));
}

export function showRunState({ view, logs }: RunState): void {
  if (view.seq >= useRunStore.getState().seq) useRunStore.setState({ ...flat(view), logs });
}

export const addLogLine = (line: LogLine): void =>
  useRunStore.setState((s) =>
    line.id > (s.logs.at(-1)?.id ?? 0) ? { logs: [...s.logs, line].slice(-MAX_LOG_LINES) } : {},
  );

export const clearLogs = (): void => void commands.clearRunLog();
