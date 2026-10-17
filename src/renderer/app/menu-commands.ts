import type { MenuCommand } from "@/shared/menu";

type Listener = (command: MenuCommand) => void;

export const PROJECT_SWITCH_COMMANDS: ReadonlySet<MenuCommand> = new Set<MenuCommand>([
  "view:projects",
  "project:new",
  "project:open",
]);

const listeners = new Set<Listener>();
let followingMenu = false;

export function dispatchMenuCommand(command: MenuCommand): void {
  for (const listener of listeners) listener(command);
}

export function onMenuCommand(listener: Listener): () => void {
  if (!followingMenu) {
    followingMenu = true;
    window.demoflow.onMenuCommand(dispatchMenuCommand);
  }
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
