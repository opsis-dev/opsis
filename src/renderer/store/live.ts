import { create } from "zustand";
import type { LiveEvent, LiveMessage, LiveStepStatus } from "@/shared/live";
import type { Rect } from "@/shared/schema/actions";
import { appendCapped } from "@/shared/lists";
import { liveAutoOpen } from "./app-settings";

type LiveStep = { nodeId: string; label: string; status?: LiveStepStatus; message?: string };

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

const DEFAULT_VIEWPORT = { width: 1920, height: 1080 };
const MAX_CLICKS = 9;

const blank = (captureId: string, nodeId: string): LiveCapture => ({
  captureId,
  nodeId,
  title: nodeId,
  viewport: DEFAULT_VIEWPORT,
  steps: [],
  frameUrl: null,
  cursor: null,
  target: null,
  clicks: [],
  status: "live",
  startedAt: Date.now(),
});

const frameUrl = (jpeg: Uint8Array<ArrayBuffer>) =>
  URL.createObjectURL(new Blob([jpeg], { type: "image/jpeg" }));

function applyEvent(prev: LiveCapture, e: LiveEvent): LiveCapture {
  switch (e.kind) {
    case "start":
      return {
        ...prev,
        title: e.title,
        viewport: e.viewport,
        steps: e.steps,
        status: "live",
      };
    case "step":
      return {
        ...prev,
        steps: prev.steps.map((st) =>
          st.nodeId === e.nodeId ? { ...st, status: e.status, message: e.message } : st,
        ),
      };
    case "cursor":
      return { ...prev, cursor: { x: e.x, y: e.y } };
    case "target":
      return { ...prev, target: { rect: e.rect, label: e.label, selector: e.selector } };
    case "click":
      return {
        ...prev,
        clicks: appendCapped(prev.clicks, { x: e.x, y: e.y, at: Date.now() }, MAX_CLICKS),
      };
    case "end":
      return { ...prev, status: e.ok ? "done" : "error", message: e.message };
  }
}

function applyMessage(prev: LiveCapture, message: LiveMessage): LiveCapture {
  if (message.type === "event") return applyEvent(prev, message.event);
  if (prev.frameUrl) URL.revokeObjectURL(prev.frameUrl);
  return { ...prev, frameUrl: frameUrl(message.jpeg) };
}

export const useLiveStore = create<LiveState>()((set, get) => ({
  captures: {},
  order: [],
  pipOpen: false,
  focusId: null,
  dialogOpen: false,

  apply: (message, autoOpen) => {
    const { captureId } = message;
    const s = get();
    const prev = s.captures[captureId];
    const next = applyMessage(prev ?? blank(captureId, message.nodeId), message);
    if (prev) {
      set({ captures: { ...s.captures, [captureId]: next } });
      return;
    }
    const focusLive = s.focusId !== null && s.captures[s.focusId]?.status === "live";
    set({
      captures: { ...s.captures, [captureId]: next },
      order: [...s.order, captureId],
      focusId: focusLive ? s.focusId : captureId,
      pipOpen: autoOpen || s.pipOpen,
    });
  },
  setPipOpen: (pipOpen) => set({ pipOpen }),
  focus: (focusId) => set({ focusId }),
  setDialogOpen: (dialogOpen) => set({ dialogOpen }),
  reset: () => {
    for (const c of Object.values(get().captures)) if (c.frameUrl) URL.revokeObjectURL(c.frameUrl);
    set({ captures: {}, order: [], focusId: null });
  },
}));

export const applyLive = (message: LiveMessage): void =>
  useLiveStore.getState().apply(message, liveAutoOpen());
