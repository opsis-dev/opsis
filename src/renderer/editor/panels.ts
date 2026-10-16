import type { LayoutStorage, PanelImperativeHandle } from "react-resizable-panels";
import { create } from "zustand";
import { readPref, writePref } from "../store/view-prefs";

export type PanelSide = "left" | "bottom" | "right";
export type DockPanel = "preview" | "timeline" | "mixer" | "run";
export type PanelId = PanelSide | DockPanel;

export const DOCK_PANELS: DockPanel[] = ["preview", "timeline", "mixer", "run"];

export const DOCK_TITLES: Record<DockPanel, string> = {
  preview: "Preview",
  timeline: "Timeline",
  mixer: "Mixer",
  run: "Run",
};

const handles: Partial<Record<PanelId, PanelImperativeHandle | null>> = {};

export const storedTab = (): DockPanel =>
  readPref<DockPanel>("dockTab", "preview", (v) => DOCK_PANELS.includes(v as DockPanel));

export const usePanels = create<{
  open: Record<PanelId, boolean>;
  popped: Record<DockPanel, boolean>;
  active: DockPanel;
  setOpen: (side: PanelId, open: boolean) => void;
  setPopped: (popped: Partial<Record<DockPanel, boolean>>) => void;
  setActive: (panel: DockPanel) => void;
}>()((set) => ({
  active: storedTab(),
  setActive: (active) => {
    set({ active });
    writePref("dockTab", active);
  },

  open: {
    left: true,
    bottom: true,
    right: true,
    preview: true,
    timeline: true,
    mixer: true,
    run: true,
  },
  popped: { preview: false, timeline: false, mixer: false, run: false },
  setOpen: (side, open) =>
    set((s) => (s.open[side] === open ? {} : { open: { ...s.open, [side]: open } })),
  setPopped: (popped) => set((s) => ({ popped: { ...s.popped, ...popped } })),
}));

export const panelRef = (side: PanelId) => (handle: PanelImperativeHandle | null) => {
  handles[side] = handle;
};

export function togglePanel(side: PanelId): void {
  const handle = handles[side];
  if (!handle) return;
  if (handle.isCollapsed()) handle.expand();
  else handle.collapse();
}

export function openPanel(side: PanelId): void {
  const handle = handles[side];
  if (handle?.isCollapsed()) handle.expand();
}

export function showPanel(panel: DockPanel): void {
  if (usePanels.getState().popped[panel]) {
    void window.demoflow.focusPanelWindow(panel);
    return;
  }
  openPanel("bottom");
  usePanels.getState().setActive(panel);
}

export function focusPanel(panel: DockPanel): void {
  showPanel(panel);
  if (usePanels.getState().popped[panel]) return;
  requestAnimationFrame(() => {
    const el = document.getElementById(`panel-${panel}`);
    (el?.querySelector<HTMLElement>("[data-timeline-root]") ?? el)?.focus();
  });
}

export const layoutStorage: LayoutStorage = {
  getItem: (key) => {
    try {
      return window.localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  setItem: (key, value) => {
    try {
      window.localStorage.setItem(key, value);
    } catch {
      return;
    }
  },
};
