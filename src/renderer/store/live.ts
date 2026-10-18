import { create } from "zustand";
import { appendCapped } from "@/shared/lists";
import type { LiveEvent, LiveMessage, LiveStepStatus } from "@/shared/live";
import type { Rect } from "@/shared/schema/actions";
import { DEFAULT_VIEWPORT } from "@/shared/schema/workflow";
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
  apply: (message: LiveMessage) => void;
  setPipOpen: (open: boolean) => void;
  focus: (captureId: string | null) => void;
  setDialogOpen: (open: boolean) => void;
  reset: () => void;
};

const MAX_CLICKS = 9;

export const CLICK_RIPPLE_MS = 600;

const recentClicks = (clicks: LiveCapture["clicks"], now: number): LiveCapture["clicks"] =>
  clicks.filter((c) => now - c.at < CLICK_RIPPLE_MS);

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

const objectUrl = (jpeg: Uint8Array<ArrayBuffer>) =>
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
    case "click": {
      const now = Date.now();
      const click = { x: e.x, y: e.y, at: now };
      return { ...prev, clicks: appendCapped(recentClicks(prev.clicks, now), click, MAX_CLICKS) };
    }
    case "end":
      return { ...prev, status: e.ok ? "done" : "error", message: e.message };
  }
}

function applyMessage(prev: LiveCapture, message: LiveMessage): LiveCapture {
  if (message.type === "event") return applyEvent(prev, message.event);
  if (prev.frameUrl) URL.revokeObjectURL(prev.frameUrl);
  return {
    ...prev,
    frameUrl: objectUrl(message.jpeg),
    clicks: recentClicks(prev.clicks, Date.now()),
  };
}

function opened(s: LiveState, captureId: string): Partial<LiveState> {
  const focusLive = s.focusId !== null && s.captures[s.focusId]?.status === "live";
  return {
    order: [...s.order, captureId],
    focusId: focusLive ? s.focusId : captureId,
    pipOpen: liveAutoOpen() || s.pipOpen,
  };
}

export const useLiveStore = create<LiveState>()((set, get) => ({
  captures: {},
  order: [],
  pipOpen: false,
  focusId: null,
  dialogOpen: false,

  apply: (message) => {
    const { captureId } = message;
    const s = get();
    const prev = s.captures[captureId];
    const next = applyMessage(prev ?? blank(captureId, message.nodeId), message);
    set({ captures: { ...s.captures, [captureId]: next }, ...(prev ? {} : opened(s, captureId)) });
  },
  setPipOpen: (pipOpen) => set({ pipOpen }),
  focus: (focusId) => set({ focusId }),
  setDialogOpen: (dialogOpen) => set({ dialogOpen }),
  reset: () => {
    for (const c of Object.values(get().captures)) if (c.frameUrl) URL.revokeObjectURL(c.frameUrl);
    set({ captures: {}, order: [], focusId: null });
  },
}));
