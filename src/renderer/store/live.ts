import { create } from "zustand";
import type { LiveMessage, LiveStepStatus } from "@/shared/live";
import type { Rect } from "@/shared/schema/actions";

export type LiveStep = { nodeId: string; label: string; status?: LiveStepStatus; message?: string };

export type LiveCapture = {
  captureId: string;
  nodeId: string;
  title: string;
  viewport: { width: number; height: number };
  steps: LiveStep[];
  frameUrl: string | null;
  cursor: { x: number; y: number } | null;
  target: { rect: Rect; label: string; selector?: string } | null;
  clicks: Array<{ x: number; y: number; at: number }>;
  status: "live" | "done" | "error";
  message?: string;
  startedAt: number;
};

type LiveState = {
  captures: Record<string, LiveCapture>;
  order: string[];
  pipOpen: boolean;
  focusId: string | null;
  dialogOpen: boolean;
  apply: (message: LiveMessage, autoOpen: boolean) => void;
  setPipOpen: (open: boolean) => void;
  focus: (captureId: string | null) => void;
  setDialogOpen: (open: boolean) => void;
  reset: () => void;
};

const blank = (captureId: string, nodeId: string): LiveCapture => ({
  captureId,
  nodeId,
  title: nodeId,
  viewport: { width: 1920, height: 1080 },
  steps: [],
  frameUrl: null,
  cursor: null,
  target: null,
  clicks: [],
  status: "live",
  startedAt: Date.now(),
});

export const useLiveStore = create<LiveState>()((set) => ({
  captures: {},
  order: [],
  pipOpen: false,
  focusId: null,
  dialogOpen: false,

  apply: (message, autoOpen) =>
    set((s) => {
      const prev = s.captures[message.captureId] ?? blank(message.captureId, message.nodeId);
      const isNew = !s.captures[message.captureId];
      let next: LiveCapture = prev;
      if (message.type === "frame") {
        if (prev.frameUrl) URL.revokeObjectURL(prev.frameUrl);
        next = {
          ...prev,
          frameUrl: URL.createObjectURL(
            new Blob([new Uint8Array(message.jpeg)], { type: "image/jpeg" }),
          ),
        };
      } else {
        const e = message.event;
        if (e.kind === "start")
          next = {
            ...prev,
            title: e.title,
            viewport: e.viewport,
            steps: e.steps.map((st) => ({ ...st })),
            status: "live",
          };
        else if (e.kind === "step")
          next = {
            ...prev,
            steps: prev.steps.map((st) =>
              st.nodeId === e.nodeId ? { ...st, status: e.status, message: e.message } : st,
            ),
          };
        else if (e.kind === "cursor") next = { ...prev, cursor: { x: e.x, y: e.y } };
        else if (e.kind === "target")
          next = { ...prev, target: { rect: e.rect, label: e.label, selector: e.selector } };
        else if (e.kind === "click")
          next = {
            ...prev,
            clicks: [...prev.clicks.slice(-8), { x: e.x, y: e.y, at: Date.now() }],
          };
        else if (e.kind === "end")
          next = { ...prev, status: e.ok ? "done" : "error", message: e.message };
      }
      return {
        captures: { ...s.captures, [message.captureId]: next },
        order: isNew ? [...s.order, message.captureId] : s.order,
        focusId:
          isNew && (!s.focusId || s.captures[s.focusId]?.status !== "live")
            ? message.captureId
            : s.focusId,
        pipOpen: isNew && autoOpen ? true : s.pipOpen,
      };
    }),
  setPipOpen: (pipOpen) => set({ pipOpen }),
  focus: (focusId) => set({ focusId }),
  setDialogOpen: (dialogOpen) => set({ dialogOpen }),
  reset: () =>
    set((s) => {
      for (const c of Object.values(s.captures)) if (c.frameUrl) URL.revokeObjectURL(c.frameUrl);
      return { captures: {}, order: [], focusId: null };
    }),
}));
