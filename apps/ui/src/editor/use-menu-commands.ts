import { useEffect } from "react";
import { toast } from "sonner";
import { DOCK_PANELS } from "@/core/shell";
import type { MenuCommand } from "@/generated/bindings";
import { isEditingElement } from "@/lib/dom";
import { onMenuCommand } from "../app/menu-commands";
import { useUiStore } from "../store/ui";
import { useViewOptions } from "../store/view-options";
import { useWorkflowStore, workflowEdits } from "../store/workflow";
import { tidyLayout } from "./layout";
import { focusPanel, PANEL_SIDES, PANEL_TOGGLES, togglePanel } from "./panels";
import { startRun } from "./run";
import { saveWithToast } from "./save";

type MenuActions = Partial<Record<MenuCommand, () => void>>;

const actionsFor = <T>(
  items: readonly T[],
  command: (item: T) => MenuCommand,
  run: (item: T) => void,
): MenuActions => Object.fromEntries(items.map((item) => [command(item), () => run(item)]));

const MENU_ACTIONS: MenuActions = {
  "edit:undo": () => workflowEdits.undo(),
  "edit:redo": () => workflowEdits.redo(),
  "project:save": () => void saveWithToast().then((ok) => ok && toast.success("Saved")),
  "run:start": () => void startRun(),
  "run:cancel": () => void window.opsis.cancelRun(),
  "edit:group": () => workflowEdits.groupNodes(useUiStore.getState().selectedIds),
  "edit:select-all": () => {
    if (isEditingElement(document.activeElement)) {
      void window.opsis.windowRole("selectAll");
      return;
    }
    const nodes = useWorkflowStore.getState().workflow?.nodes ?? [];
    useUiStore.getState().setSelected(nodes.map((n) => n.id));
  },
  "edit:layout": () => void tidyLayout(),
  "edit:add-node": () => useUiStore.getState().setPaletteOpen(true),
  "view:toggle-minimap": () => useViewOptions.getState().toggleMinimap(),
  "view:preview": () => void window.opsis.openPreviewWindow(),
  ...actionsFor(PANEL_SIDES, (side) => PANEL_TOGGLES[side].command, togglePanel),
  ...actionsFor(DOCK_PANELS, (panel) => `view:focus-${panel}`, focusPanel),
};

export function useMenuCommands(): void {
  useEffect(() => onMenuCommand((command) => MENU_ACTIONS[command]?.()), []);
}
