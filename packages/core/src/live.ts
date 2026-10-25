import { hasStringType } from "./guards";
import type { Rect } from "./schema/actions";
import type { NodeRunStatus } from "./schema/events";

export type LiveStepStatus = Extract<NodeRunStatus, "running" | "done" | "error">;

export type LiveEvent =
  | {
      kind: "start";
      title: string;
      viewport: { width: number; height: number };
      steps: Array<{ nodeId: string; label: string }>;
    }
  | { kind: "step"; nodeId: string; status: LiveStepStatus; message?: string }
  | { kind: "cursor"; x: number; y: number }
  | { kind: "target"; rect: Rect; label: string; selector?: string }
  | { kind: "click"; x: number; y: number }
  | { kind: "end"; ok: boolean; message?: string };

export type LiveMessage =
  | {
      type: "frame";
      captureId: string;
      nodeId: string;
      jpeg: Uint8Array<ArrayBuffer>;
      width: number;
      height: number;
      t: number;
    }
  | { type: "event"; captureId: string; nodeId: string; t: number; event: LiveEvent };

export const isLiveMessage = (data: unknown): data is LiveMessage =>
  hasStringType(data) && typeof data.captureId === "string";
