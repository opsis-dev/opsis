import type { ComponentProps } from "react";
import { ResizablePanel } from "@/components/ui/resizable";
import { DOCK_TAB_BAR_PX } from "./dock";
import { PANEL_REFS, usePanels, type PanelSide } from "./panels";

const CATEGORY_RAIL_PX = 44;
const BORDER_PX = 1;
const OPEN_ABOVE_COLLAPSED_PX = 12;

const COLLAPSED_PX: Record<PanelSide, number> = {
  left: CATEGORY_RAIL_PX,
  bottom: DOCK_TAB_BAR_PX + BORDER_PX,
  right: 0,
};

export function SidePanel({
  side,
  ...props
}: { side: PanelSide } & Omit<
  ComponentProps<typeof ResizablePanel>,
  "id" | "className" | "style" | "panelRef" | "collapsible" | "collapsedSize" | "onResize"
>) {
  const collapsedPx = COLLAPSED_PX[side];
  return (
    <ResizablePanel
      id={side}
      className="bg-sidebar"
      style={{ overflow: "hidden" }}
      panelRef={PANEL_REFS[side]}
      collapsible
      collapsedSize={`${collapsedPx}px`}
      onResize={(size) =>
        usePanels.getState().setOpen(side, size.inPixels > collapsedPx + OPEN_ABOVE_COLLAPSED_PX)
      }
      {...props}
    />
  );
}
