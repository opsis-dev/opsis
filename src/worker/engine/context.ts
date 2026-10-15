import type { EnginePaths } from "../../engine/paths";
import type { CompileResult } from "../../shared/compile/compile";
import type { NodeDef } from "../../shared/nodes/define";
import type { LogLevel } from "../../shared/schema/events";
import type { Artifact, CompileValue, NodeResult } from "../../shared/schema/values";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import type { AppSettings } from "../../shared/settings";
import type { LoginRequest, WorkerEnv } from "../../shared/worker-protocol";
import type { LiveSink } from "../capture/driver";
import type { FfmpegTools } from "../media/ffmpeg";

export type ExecutorContext = {
  runId: string;
  node: WorkflowNode;
  workflow: Workflow;
  env: WorkerEnv;
  projectDir: string;
  dir: string;
  relDir: string;
  outDir: string;
  hash: string;
  vars: Record<string, string>;
  signal: AbortSignal;
  ffmpeg: FfmpegTools;
  engine: EnginePaths;
  settings: AppSettings;
  live: (title: string) => LiveSink | null;
  login: (request: LoginRequest) => Promise<string>;
  log: (level: LogLevel, message: string) => void;
  progress: (progress: number, message?: string) => void;
  step: (nodeId: string, status: "start" | "done" | "error", message?: string) => void;
  compile: () => CompileResult;
  results: Record<string, NodeResult>;
  secrets: {
    read: (name: string) => Promise<string | null>;
    write: (name: string, value: string) => Promise<void>;
  };
  ref: (file: string) => string;
};

export type ExecutorOutput = {
  outputs: Record<string, CompileValue>;
  artifacts?: Artifact[];
  traceId?: string;
};

export class TracedError extends Error {
  constructor(
    message: string,
    readonly traceId: string | undefined,
    readonly signInExpired = false,
  ) {
    super(message);
    this.name = "TracedError";
  }
}

// oxlint-disable-next-line typescript/no-explicit-any
export type Executor<P = any> = {
  type: string;
  run: (
    ctx: ExecutorContext,
    inputs: Partial<Record<string, CompileValue>>,
    params: P,
  ) => Promise<ExecutorOutput>;
};

// oxlint-disable-next-line typescript/no-explicit-any
export function defineExecutor<D extends NodeDef<any>>(
  def: D,
  run: (
    ctx: ExecutorContext,
    inputs: Partial<Record<string, CompileValue>>,
    params: ReturnType<D["params"]["parse"]>,
  ) => Promise<ExecutorOutput>,
): Executor<ReturnType<D["params"]["parse"]>> {
  return { type: def.type, run };
}
