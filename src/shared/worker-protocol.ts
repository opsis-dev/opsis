import type { EngineInstallEvent, RunRequest } from "./ipc";
import type { AnalysisRequest, AnalysisResult } from "./analysis";
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

export type WorkerToMain =
  | { type: "event"; event: RunEvent }
  | { type: "secret:read"; id: string; projectDir: string; name: string }
  | { type: "secret:write"; id: string; projectDir: string; name: string; value: string }
  | { type: "login:request"; id: string; request: LoginRequest }
  | { type: "install:progress"; event: EngineInstallEvent }
  | { type: "install:done"; error?: string }
  | { type: "still:done"; id: string; error?: string }
  | { type: "analysis:done"; id: string; result: AnalysisResult };
