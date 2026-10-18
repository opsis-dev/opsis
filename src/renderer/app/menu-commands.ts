import type { MenuCommand } from "@/shared/menu";

type Listener = (command: MenuCommand) => void;

export const PROJECT_ACTIONS = ["new", "open"] as const;

export type ProjectAction = (typeof PROJECT_ACTIONS)[number];

const actionCommand = (action: ProjectAction): MenuCommand => `project:${action}`;

export const projectActionOf = (command: MenuCommand): ProjectAction | undefined =>
  PROJECT_ACTIONS.find((action) => actionCommand(action) === command);

export const PROJECT_SWITCH_COMMANDS: ReadonlySet<MenuCommand> = new Set([
  "view:projects",
  ...PROJECT_ACTIONS.map(actionCommand),
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
