import type { LayoutStorage, PanelImperativeHandle } from "react-resizable-panels";
import { create } from "zustand";
import { isOneOf } from "@/renderer/lib/one-of";
import { readStored, writeStored } from "@/renderer/lib/local-storage";
import { readPref, writePref } from "../store/view-prefs";

export const PANEL_SIDES = ["left", "bottom", "right"] as const;
export type PanelSide = (typeof PANEL_SIDES)[number];

export const DOCK_PANELS = ["preview", "timeline", "mixer", "run"] as const;
export type DockPanel = (typeof DOCK_PANELS)[number];

export const isDockPanel = (v: unknown): v is DockPanel => isOneOf(DOCK_PANELS, v);

export const DOCK_TITLES: Record<DockPanel, string> = {
  preview: "Preview",
  timeline: "Timeline",
  mixer: "Mixer",
  run: "Run",
};

const handles: Partial<Record<PanelSide, PanelImperativeHandle | null>> = {};

export const storedTab = (): DockPanel => readPref("dockTab", "preview", isDockPanel);

export const usePanels = create<{
  open: Record<PanelSide, boolean>;
  popped: readonly DockPanel[];
  active: DockPanel;
  setOpen: (side: PanelSide, open: boolean) => void;
  setPopped: (popped: readonly DockPanel[]) => void;
  setActive: (panel: DockPanel) => void;
}>()((set) => ({
  active: storedTab(),
  setActive: (active) => {
    set({ active });
    writePref("dockTab", active);
  },

  open: { left: true, bottom: true, right: true },
  popped: [],
  setOpen: (side, open) =>
    set((s) => (s.open[side] === open ? {} : { open: { ...s.open, [side]: open } })),
  setPopped: (popped) => set({ popped }),
}));

export const panelRef = (side: PanelSide) => (handle: PanelImperativeHandle | null) => {
  handles[side] = handle;
};

export function togglePanel(side: PanelSide): void {
  const handle = handles[side];
  if (!handle) return;
  if (handle.isCollapsed()) handle.expand();
  else handle.collapse();
}

export function openPanel(side: PanelSide): void {
  const handle = handles[side];
  if (handle?.isCollapsed()) handle.expand();
}

export function showPanel(panel: DockPanel): void {
  if (usePanels.getState().popped.includes(panel)) {
    void window.demoflow.focusPanelWindow(panel);
    return;
  }
  openPanel("bottom");
  usePanels.getState().setActive(panel);
}

export function focusPanel(panel: DockPanel): void {
  showPanel(panel);
  if (usePanels.getState().popped.includes(panel)) return;
  requestAnimationFrame(() => {
    const el = document.getElementById(`panel-${panel}`);
    (el?.querySelector<HTMLElement>("[data-timeline-root]") ?? el)?.focus();
  });
}

export const layoutStorage: LayoutStorage = { getItem: readStored, setItem: writeStored };
