import type { AnalysisRequest, AnalysisResult } from "#core/analysis";
import type { EngineInstallEvent } from "#core/engine";
import { hasTypeIn } from "#core/guards";
import type { RunRequest } from "#core/ipc";
import type { Viewport } from "#core/schema/actions";
import type { RunEvent } from "#core/schema/events";
import type { AppSettings } from "#core/settings";

export type WorkerEnv = {
  appRoot: string;
  appVersion: string;
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
  viewport?: Viewport;
};

export type WorkerReply =
  | { type: "secret:value"; id: string; value: string | null }
  | { type: "secret:written"; id: string }
  | { type: "login:state"; id: string; value: string | null }
  | { type: "request:failed"; id: string; error: string };

export type MainToWorker =
  | { type: "run"; env: WorkerEnv; request: RunRequest; runId: string }
  | { type: "install-engine"; env: WorkerEnv }
  | { type: "analysis"; env: WorkerEnv; id: string; request: AnalysisRequest }
  | { type: "cancel" }
  | WorkerReply;

export type MainMessage<T extends MainToWorker["type"]> = Extract<MainToWorker, { type: T }>;

const MAIN_TO_WORKER_TYPES: Record<MainToWorker["type"], true> = {
  run: true,
  "install-engine": true,
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
  | { type: "analysis:done"; id: string; result: AnalysisResult };

const WORKER_TO_MAIN_TYPES: Record<WorkerToMain["type"], true> = {
  event: true,
  "secret:read": true,
  "secret:write": true,
  "login:request": true,
  "install:progress": true,
  "install:done": true,
  "analysis:done": true,
};

export const isWorkerToMain = hasTypeIn<WorkerToMain>(WORKER_TO_MAIN_TYPES);
