import type { AnalysisRequest, AnalysisResult } from "./analysis";
import type { EngineInstallEvent } from "./engine";
import { hasTypeIn } from "./guards";
import type { RunRequest } from "./ipc";
import type { RunEvent } from "./schema/events";
import type { TimelineSpec } from "./schema/timeline";
import type { AppSettings } from "./settings";

export type WorkerEnv = {
  appRoot: string;
  isPackaged: boolean;
  resourcesPath: string;
  userData: string;
  engineDir: string;
  settings: AppSettings;
};

export type LoginRequest = {
  url: string;
  doneWhen: "url-change" | "selector" | "window-closed";
  selector: string;
  timeoutSec: number;
};

export type WorkerReply =
  | { type: "secret:value"; id: string; value: string | null }
  | { type: "secret:written"; id: string }
  | { type: "login:state"; id: string; value: string | null }
  | { type: "request:failed"; id: string; error: string };

export type MainToWorker =
  | { type: "run"; env: WorkerEnv; request: RunRequest; runId: string }
  | { type: "install-engine"; env: WorkerEnv }
  | {
      type: "still";
      env: WorkerEnv;
      id: string;
      projectDir: string;
      spec: TimelineSpec;
      frame: number;
      output: string;
      width: number;
    }
  | { type: "analysis"; env: WorkerEnv; id: string; request: AnalysisRequest }
  | { type: "cancel" }
  | WorkerReply;

export type MainMessage<T extends MainToWorker["type"]> = Extract<MainToWorker, { type: T }>;

const MAIN_TO_WORKER_TYPES: Record<MainToWorker["type"], true> = {
  run: true,
  "install-engine": true,
  still: true,
  analysis: true,
  cancel: true,
  "secret:value": true,
  "secret:written": true,
  "login:state": true,
  "request:failed": true,
};

export const isMainToWorker = hasTypeIn<MainToWorker>(MAIN_TO_WORKER_TYPES);

export type WorkerToMain =
  | { type: "event"; event: RunEvent }
  | { type: "secret:read"; id: string; projectDir: string; name: string }
  | { type: "secret:write"; id: string; projectDir: string; name: string; value: string }
  | { type: "login:request"; id: string; request: LoginRequest }
  | { type: "install:progress"; event: EngineInstallEvent }
  | { type: "install:done"; error?: string }
  | { type: "still:done"; id: string; error?: string }
  | { type: "analysis:done"; id: string; result: AnalysisResult };

const WORKER_TO_MAIN_TYPES: Record<WorkerToMain["type"], true> = {
  event: true,
  "secret:read": true,
  "secret:write": true,
  "login:request": true,
  "install:progress": true,
  "install:done": true,
  "still:done": true,
  "analysis:done": true,
};

export const isWorkerToMain = hasTypeIn<WorkerToMain>(WORKER_TO_MAIN_TYPES);
