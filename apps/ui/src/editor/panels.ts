import { isOneOf, isString } from "@opsis/video/common/guards";
import { PanelBottom, PanelLeft, PanelRight, type LucideIcon } from "lucide-react";
import type { LayoutStorage, PanelImperativeHandle } from "react-resizable-panels";
import { create } from "zustand";
import { DOCK_PANELS } from "@/core/shell";
import type { DockPanelName, MenuCommand } from "@/generated/bindings";
import { readPref, writePref } from "../store/view-prefs";

export const PANEL_SIDES = ["left", "bottom", "right"] as const;
export type PanelSide = (typeof PANEL_SIDES)[number];

export const PANEL_TOGGLES: Record<PanelSide, { icon: LucideIcon; command: MenuCommand }> = {
  left: { icon: PanelLeft, command: "view:toggle-left" },
  bottom: { icon: PanelBottom, command: "view:toggle-bottom" },
  right: { icon: PanelRight, command: "view:toggle-right" },
};

export const isDockPanel = (v: unknown): v is DockPanelName => isOneOf(DOCK_PANELS, v);

const handles: Partial<Record<PanelSide, PanelImperativeHandle | null>> = {};

const focusTargets: Partial<Record<DockPanelName, HTMLElement | null>> = {};

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
    set((s) => (s.open[side] === open ? s : { open: { ...s.open, [side]: open } })),
  setPopped: (popped) => set({ popped }),
}));

const handleRef = (side: PanelSide) => (handle: PanelImperativeHandle | null) => {
  handles[side] = handle;
};

export const PANEL_REFS: Record<PanelSide, (handle: PanelImperativeHandle | null) => void> = {
  left: handleRef("left"),
  bottom: handleRef("bottom"),
  right: handleRef("right"),
};

export const focusTargetRef = (panel: DockPanelName) => (el: HTMLElement | null) => {
  focusTargets[panel] = el;
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

export function showPanel(panel: DockPanelName): boolean {
  if (usePanels.getState().popped.includes(panel)) {
    void window.opsis.focusPanelWindow(panel);
    return false;
  }
  openPanel("bottom");
  usePanels.getState().setActive(panel);
  return true;
}

const focusAfterPaint = (target: () => HTMLElement | null) =>
  requestAnimationFrame(() => target()?.focus());

export function focusPanel(panel: DockPanelName): void {
  if (!showPanel(panel)) return;
  focusAfterPaint(() => focusTargets[panel] ?? document.getElementById(`panel-${panel}`));
}

export const layoutStorage: LayoutStorage = {
  getItem: (key) => readPref<string | null>(key, null, isString),
  setItem: writePref,
};
