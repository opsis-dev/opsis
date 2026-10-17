import type { LayoutStorage, PanelImperativeHandle } from "react-resizable-panels";
import { create } from "zustand";
import { isOneOf } from "@/renderer/lib/one-of";
import { DOCK_PANELS, type DockPanelName } from "@/shared/ipc";
import { readStored, writeStored } from "@/renderer/lib/local-storage";
import { readPref, writePref } from "../store/view-prefs";

export const PANEL_SIDES = ["left", "bottom", "right"] as const;
export type PanelSide = (typeof PANEL_SIDES)[number];

export const isDockPanel = (v: unknown): v is DockPanelName => isOneOf(DOCK_PANELS, v);

const handles: Partial<Record<PanelSide, PanelImperativeHandle | null>> = {};

export const storedTab = (): DockPanelName => readPref("dockTab", "preview", isDockPanel);

export const usePanels = create<{
  open: Record<PanelSide, boolean>;
  popped: readonly DockPanelName[];
  active: DockPanelName;
  setOpen: (side: PanelSide, open: boolean) => void;
  setPopped: (popped: readonly DockPanelName[]) => void;
  setActive: (panel: DockPanelName) => void;
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

export function showPanel(panel: DockPanelName): void {
  if (usePanels.getState().popped.includes(panel)) {
    void window.demoflow.focusPanelWindow(panel);
    return;
  }
  openPanel("bottom");
  usePanels.getState().setActive(panel);
}

export function focusPanel(panel: DockPanelName): void {
  showPanel(panel);
  if (usePanels.getState().popped.includes(panel)) return;
  requestAnimationFrame(() => {
    const el = document.getElementById(`panel-${panel}`);
    (el?.querySelector<HTMLElement>("[data-timeline-root]") ?? el)?.focus();
  });
}

export const layoutStorage: LayoutStorage = { getItem: readStored, setItem: writeStored };
