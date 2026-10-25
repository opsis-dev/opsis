import { applyRunEvent, emptyRunProgress, type RunProgress } from "@opsis/core/run-progress";
import type { RunEvent } from "@opsis/core/schema/events";
import { Notification } from "electron";
import { focusMainWindow, getMainWindow } from "./window-registry";

type RunDone = Extract<RunEvent, { type: "run:done" }>;

const NODE_FINISHED = new Set<RunEvent["type"]>(["node:done", "node:error", "node:skipped"]);
const STARTED_PROGRESS = 0.01;
const BEFORE_DONE_PROGRESS = 0.99;
const NO_PROGRESS_BAR = -1;

let progress: RunProgress | null = null;
let tally = { count: 1, done: 0 };
let lastNotification: Notification | null = null;

export const runProgressFor = (runId: string | null): RunProgress =>
  runId && progress?.runId === runId ? progress : emptyRunProgress(runId);

function showProgressBar(event: RunEvent): void {
  const win = getMainWindow();
  if (event.type === "run:start") {
    tally = { count: Math.max(1, event.nodeIds.length), done: 0 };
    win?.setProgressBar(STARTED_PROGRESS);
  } else if (NODE_FINISHED.has(event.type)) {
    tally.done += 1;
    win?.setProgressBar(Math.min(BEFORE_DONE_PROGRESS, tally.done / tally.count));
  } else if (event.type === "run:done") {
    win?.setProgressBar(NO_PROGRESS_BAR);
  }
}

function finishedMessage(event: RunDone): { title: string; body: string } {
  if (event.ok)
    return {
      title: "Export finished",
      body:
        event.outputs.length === 1
          ? "Your video is ready."
          : `${event.outputs.length} files are ready.`,
    };
  if (event.cancelled)
    return { title: "Run stopped", body: "The run was stopped before it finished." };
  return { title: "Run failed", body: event.error ?? "Something went wrong." };
}

function notifyRunFinished(event: RunDone): void {
  if (!Notification.isSupported() || getMainWindow()?.isFocused()) return;
  lastNotification?.close();
  const notification = new Notification(finishedMessage(event));
  const forget = () => {
    if (lastNotification === notification) lastNotification = null;
  };
  notification.on("click", () => {
    focusMainWindow();
    forget();
  });
  notification.on("close", forget);
  lastNotification = notification;
  notification.show();
}

function nextProgress(event: RunEvent, current: RunProgress | null): RunProgress | null {
  if (event.type === "run:done") return null;
  const base = event.type === "run:start" || !current ? emptyRunProgress(event.runId) : current;
  return applyRunEvent(base, event);
}

export function reportRunEvent(event: RunEvent): void {
  progress = nextProgress(event, progress);
  showProgressBar(event);
  if (event.type === "run:done") notifyRunFinished(event);
}
