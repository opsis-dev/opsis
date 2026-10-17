import type { z } from "zod";
import type { EnginePaths } from "../../engine/paths";
import type { CompileResult } from "../../shared/compile/compile";
import type { NodeDef } from "../../shared/nodes/define";
import type { LogLevel } from "../../shared/schema/events";
import type { Artifact, CompileValue, NodeResult } from "../../shared/schema/values";
import type { Workflow, WorkflowNode } from "../../shared/schema/workflow";
import type { LoginRequest, WorkerEnv } from "../../shared/worker-protocol";
import type { LiveSink } from "../capture/take-run";
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
  live: (title: string) => LiveSink | null;
  login: (request: LoginRequest) => Promise<string>;
  log: (level: LogLevel, message: string) => void;
  progress: (progress: number, message?: string) => void;
  step: (nodeId: string, status: "start" | "done" | "error", message?: string) => void;
  compile: () => CompileResult;
  results: Partial<Record<string, NodeResult>>;
  secrets: {
    read: (name: string) => Promise<string | null>;
    write: (name: string, value: string) => Promise<void>;
  };
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

type ExecutorInputs = Partial<Record<string, CompileValue>>;

export type Executor<P = unknown> = {
  type: string;
  run(ctx: ExecutorContext, inputs: ExecutorInputs, params: P): Promise<ExecutorOutput>;
};

export function defineExecutor<S extends z.ZodObject>(
  def: NodeDef<S>,
  run: (
    ctx: ExecutorContext,
    inputs: ExecutorInputs,
    params: z.infer<S>,
  ) => Promise<ExecutorOutput>,
): Executor<z.infer<S>> {
  return { type: def.type, run };
}
