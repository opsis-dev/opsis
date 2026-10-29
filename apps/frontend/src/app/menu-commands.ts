import { create } from "zustand";
import { on } from "@/app/tauri";
import { events, type MenuCommand } from "@/generated/bindings";
import { listenerSet } from "@/lib/listener-set";

type Listener = (command: MenuCommand) => void;

export type ProjectAction = "new" | "open";

const PROJECT_COMMANDS: Partial<Record<MenuCommand, ProjectAction>> = {
  "project:new": "new",
  "project:open": "open",
};

export const projectActionOf = (command: MenuCommand): ProjectAction | undefined =>
  PROJECT_COMMANDS[command];

export const PROJECT_SWITCH_COMMANDS: ReadonlySet<MenuCommand> = new Set<MenuCommand>([
  "view:projects",
  "project:new",
  "project:open",
]);

export const usePendingProjectAction = create<{ pending: ProjectAction | null }>()(() => ({
  pending: null,
}));

const commands = listenerSet<[MenuCommand]>();
let followingMenu = false;

export function dispatchMenuCommand(command: MenuCommand): void {
  commands.emit(command);
}

export function onMenuCommand(listener: Listener): () => void {
  if (!followingMenu) {
    followingMenu = true;
    on(events.menuCommandFired, dispatchMenuCommand);
  }
  return commands.add(listener);
}
