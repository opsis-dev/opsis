import type { ComponentProps } from "react";
import { ResizablePanel } from "@/renderer/components/ui/resizable";
import { panelRef, usePanels, type PanelSide } from "./panels";

const PANEL_LAYOUT: Record<PanelSide, { collapsedPx: number; openAbovePx: number }> = {
  left: { collapsedPx: 44, openAbovePx: 60 },
  bottom: { collapsedPx: 37, openAbovePx: 48 },
  right: { collapsedPx: 0, openAbovePx: 8 },
};

export function SidePanel({
  side,
  ...props
}: { side: PanelSide } & Omit<
  ComponentProps<typeof ResizablePanel>,
  "id" | "className" | "style" | "panelRef" | "collapsible" | "collapsedSize" | "onResize"
>) {
  const { collapsedPx, openAbovePx } = PANEL_LAYOUT[side];
  return (
    <ResizablePanel
      id={side}
      className="bg-sidebar"
      style={{ overflow: "hidden" }}
      panelRef={panelRef(side)}
      collapsible
      collapsedSize={`${collapsedPx}px`}
      onResize={(size) => usePanels.getState().setOpen(side, size.inPixels > openAbovePx)}
      {...props}
    />
  );
}
