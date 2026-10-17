import { useEffect } from "react";
import { toast } from "sonner";
import type { MenuCommand } from "@/shared/ipc";
import { useUiStore } from "../store/ui";
import { useViewOptions } from "../store/view-options";
import { useWorkflowStore } from "../store/workflow";
import { tidyLayout } from "./layout";
import { DOCK_PANELS, focusPanel, PANEL_SIDES, togglePanel } from "./panels";
import { cancelRun, startRun } from "./run";
import { saveWithToast } from "./save";

const MENU_ACTIONS: Partial<Record<MenuCommand, () => void>> = {
  "edit:undo": () => useWorkflowStore.temporal.getState().undo(),
  "edit:redo": () => useWorkflowStore.temporal.getState().redo(),
  "project:save": () => void saveWithToast().then((ok) => ok && toast.success("Saved")),
  "run:start": () => void startRun(),
  "run:cancel": () => void cancelRun(),
  "edit:group": () => useWorkflowStore.getState().groupNodes(useUiStore.getState().selectedIds),
  "edit:layout": () => void tidyLayout(),
  "edit:add-node": () => useUiStore.getState().setPaletteOpen(true),
  "view:toggle-minimap": () => {
    const view = useViewOptions.getState();
    view.setMinimap(!view.minimap);
  },
  ...Object.fromEntries(
    PANEL_SIDES.map((side) => [`view:toggle-${side}`, () => togglePanel(side)]),
  ),
  ...Object.fromEntries(
    DOCK_PANELS.map((panel) => [`view:focus-${panel}`, () => focusPanel(panel)]),
  ),
};

export function useMenuCommands(): void {
  useEffect(() => window.demoflow.onMenuCommand((command) => MENU_ACTIONS[command]?.()), []);
}
