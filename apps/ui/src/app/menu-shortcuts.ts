import { useEffect } from "react";
import { acceleratorLabel, matchesAccelerator, menuCommandEntry } from "@/core/menu";
import type { MenuCommand, MenuEntry, WindowRole } from "@/generated/bindings";
import { isEditingElement } from "@/lib/dom";
import { isMac, platform } from "@/lib/platform";
import { appMenu } from "@/store/catalog";
import { dispatchMenuCommand } from "./menu-commands";

export const shortcutLabel = (accelerator: string) => acceleratorLabel(accelerator, platform);

export const commandLabel = (cmd: MenuCommand): string => menuCommandEntry(cmd)?.label ?? cmd;

export function commandShortcut(cmd: MenuCommand): string | null {
  const accelerator = menuCommandEntry(cmd)?.accelerator;
  return accelerator ? shortcutLabel(accelerator) : null;
}

export const menuSections = appMenu;

export type MenuItem = Exclude<MenuEntry, { kind: "separator" }>;

export const menuItemId = (entry: MenuItem): MenuCommand | WindowRole =>
  entry.kind === "command" ? entry.command : entry.role;

export function runMenuItem(entry: MenuItem): void {
  if (entry.kind === "command") dispatchMenuCommand(entry.command);
  else void window.opsis.windowRole(entry.role);
}

const NATIVE_WHILE_EDITING = new Set<MenuCommand | WindowRole>([
  "cut",
  "copy",
  "paste",
  "selectAll",
  "edit:select-all",
  "edit:undo",
  "edit:redo",
]);

export function useMenuShortcuts(): void {
  useEffect(() => {
    if (isMac) return;
    const entries = menuSections()
      .flatMap((s) => s.items)
      .filter(
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
