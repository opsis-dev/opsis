import type { LayoutStorage, PanelImperativeHandle } from "react-resizable-panels";
import { create } from "zustand";

export type PanelSide = "left" | "bottom" | "right";

const handles: Partial<Record<PanelSide, PanelImperativeHandle | null>> = {};

export const usePanels = create<{
  open: Record<PanelSide, boolean>;
  setOpen: (side: PanelSide, open: boolean) => void;
}>()((set) => ({
  open: { left: true, bottom: true, right: true },
  setOpen: (side, open) =>
    set((s) => (s.open[side] === open ? {} : { open: { ...s.open, [side]: open } })),
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
