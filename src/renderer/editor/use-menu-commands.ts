import { useEffect } from "react";
import { toast } from "sonner";
import { DOCK_PANELS } from "@/shared/ipc";
import type { MenuCommand } from "@/shared/menu";
import { onMenuCommand } from "../app/menu-commands";
import { useUiStore } from "../store/ui";
import { useViewOptions } from "../store/view-options";
import { useWorkflowStore } from "../store/workflow";
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
  "edit:undo": () => useWorkflowStore.getState().undo(),
  "edit:redo": () => useWorkflowStore.getState().redo(),
  "project:save": () => void saveWithToast().then((ok) => ok && toast.success("Saved")),
  "run:start": () => void startRun(),
  "run:cancel": () => void window.opsis.cancelRun(),
  "edit:group": () => useWorkflowStore.getState().groupNodes(useUiStore.getState().selectedIds),
  "edit:layout": () => void tidyLayout(),
  "edit:add-node": () => useUiStore.getState().setPaletteOpen(true),
  "view:toggle-minimap": () => useViewOptions.getState().toggleMinimap(),
  ...actionsFor(PANEL_SIDES, (side) => PANEL_TOGGLES[side].command, togglePanel),
  ...actionsFor(DOCK_PANELS, (panel) => `view:focus-${panel}`, focusPanel),
};

export function useMenuCommands(): void {
  useEffect(() => onMenuCommand((command) => MENU_ACTIONS[command]?.()), []);
}
