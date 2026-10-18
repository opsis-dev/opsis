import { useEffect } from "react";
import { isEditingElement } from "@/renderer/lib/dom";
import { isMac, platform } from "@/renderer/lib/platform";
import {
  acceleratorLabel,
  appMenu,
  matchesAccelerator,
  menuCommandEntry,
  type MenuCommand,
  type MenuEntry,
  type WindowRole,
} from "@/shared/menu";
import { dispatchMenuCommand } from "./menu-commands";

export const shortcutLabel = (accelerator: string) => acceleratorLabel(accelerator, platform);

export const commandLabel = (cmd: MenuCommand): string =>
  menuCommandEntry(platform, cmd)?.label ?? cmd;

export function commandShortcut(cmd: MenuCommand): string | null {
  const accelerator = menuCommandEntry(platform, cmd)?.accelerator;
  return accelerator ? shortcutLabel(accelerator) : null;
}

export const MENU_SECTIONS = appMenu(platform);

export type MenuItem = Exclude<MenuEntry, { kind: "separator" }>;

export const menuItemId = (entry: MenuItem): MenuCommand | WindowRole =>
  entry.kind === "command" ? entry.command : entry.role;

export function runMenuItem(entry: MenuItem): void {
  if (entry.kind === "command") dispatchMenuCommand(entry.command);
  else void window.demoflow.windowRole(entry.role);
}

const NATIVE_WHILE_EDITING = new Set<MenuCommand | WindowRole>([
  "cut",
  "copy",
  "paste",
  "selectAll",
  "edit:undo",
  "edit:redo",
]);

export function useMenuShortcuts(): void {
  useEffect(() => {
    if (isMac) return;
    const entries = MENU_SECTIONS.flatMap((s) => s.items).filter(
      (e): e is MenuItem & { accelerator: string } => e.kind !== "separator" && !!e.accelerator,
    );
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      const entry = entries.find((x) => matchesAccelerator(x.accelerator, e, platform));
      if (!entry) return;
      if (isEditingElement(document.activeElement) && NATIVE_WHILE_EDITING.has(menuItemId(entry)))
        return;
      e.preventDefault();
      runMenuItem(entry);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
