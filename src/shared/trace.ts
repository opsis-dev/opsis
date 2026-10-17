import { partitionPoint } from "./math";

export type TraceFrame = { src: string; time: number; width: number; height: number };

export type TraceAction = {
  id: string;
  parentId?: string;
  group: boolean;
  nodeId?: string;
  title: string;
  target?: string;
  start: number;
  end: number;
  count: number;
  error?: string;
  logs: string[];
  point?: { x: number; y: number };
  box?: { x: number; y: number; width: number; height: number };
};

export type TraceConsole = {
  level: "log" | "info" | "warning" | "error" | "debug";
  text: string;
  time: number;
  url?: string;
};

export type TraceError = { message: string; stack?: string; time: number; actionId?: string };

export type TraceRequest = {
  method: string;
  url: string;
  status: number;
  mimeType: string;
  resourceType: string;
  size: number;
  start: number;
  duration: number;
  failure?: string;
};

export type TraceHeader = {
  id: string;
  nodeId: string;
  label: string;
  ok: boolean;
  createdAt: number;
};

export type TraceModel = TraceHeader & {
  start: number;
  end: number;
  viewport: { width: number; height: number } | null;
  actions: TraceAction[];
  frames: TraceFrame[];
  console: TraceConsole[];
  errors: TraceError[];
  network: TraceRequest[];
};

export function frameAt(frames: TraceFrame[], time: number): TraceFrame | undefined {
  const after = partitionPoint(frames, (f) => f.time <= time);
  return frames[Math.max(0, after - 1)];
}
