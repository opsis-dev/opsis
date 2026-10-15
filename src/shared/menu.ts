import type { MenuCommand } from "./ipc";

export type WindowRole =
  | "cut"
  | "copy"
  | "paste"
  | "selectAll"
  | "reload"
  | "toggleDevTools"
  | "resetZoom"
  | "zoomIn"
  | "zoomOut"
  | "togglefullscreen"
  | "quit";

export type MenuEntry =
  | { kind: "command"; label: string; command: MenuCommand; accelerator?: string }
  | { kind: "role"; label: string; role: WindowRole; accelerator?: string }
  | { kind: "separator" };

export type MenuSection = { label: string; items: MenuEntry[] };

const command = (label: string, cmd: MenuCommand, accelerator?: string): MenuEntry => ({
  kind: "command",
  label,
  command: cmd,
  ...(accelerator ? { accelerator } : {}),
});
const role = (label: string, r: WindowRole, accelerator?: string): MenuEntry => ({
  kind: "role",
  label,
  role: r,
  ...(accelerator ? { accelerator } : {}),
});
const separator: MenuEntry = { kind: "separator" };

export function appMenu(platform: string): MenuSection[] {
  const mac = platform === "darwin";
  return [
    {
      label: "File",
      items: [
        command("New project…", "project:new", "CmdOrCtrl+N"),
        command("Open project…", "project:open", "CmdOrCtrl+O"),
        command("Save", "project:save", "CmdOrCtrl+S"),
        separator,
        command("All projects", "view:projects", "CmdOrCtrl+Shift+P"),
        command("Settings", "view:settings", "CmdOrCtrl+,"),
        ...(mac ? [] : [separator, role("Quit", "quit", "Ctrl+Q")]),
      ],
    },
    {
      label: "Edit",
      items: [
        command("Undo", "edit:undo", "CmdOrCtrl+Z"),
        command("Redo", "edit:redo", mac ? "Cmd+Shift+Z" : "Ctrl+Y"),
        separator,
        role("Cut", "cut", "CmdOrCtrl+X"),
        role("Copy", "copy", "CmdOrCtrl+C"),
        role("Paste", "paste", "CmdOrCtrl+V"),
        role("Select all", "selectAll", "CmdOrCtrl+A"),
        separator,
        command("Add node…", "edit:add-node", "CmdOrCtrl+K"),
        command("Group selection", "edit:group", "CmdOrCtrl+G"),
        command("Tidy up layout", "edit:layout", "CmdOrCtrl+Shift+L"),
      ],
    },
    {
      label: "Run",
      items: [
        command("Run", "run:start", "CmdOrCtrl+Enter"),
        command("Stop run", "run:cancel", "CmdOrCtrl+."),
      ],
    },
    {
      label: "View",
      items: [
        command("Node list", "view:toggle-left", "CmdOrCtrl+B"),
        command("Preview and run", "view:toggle-bottom", "CmdOrCtrl+J"),
        command("Inspector", "view:toggle-right", "CmdOrCtrl+Alt+B"),
        separator,
        role("Reload", "reload", "CmdOrCtrl+R"),
        role("Developer tools", "toggleDevTools", mac ? "Alt+Cmd+I" : "Ctrl+Shift+I"),
        separator,
        role("Actual size", "resetZoom", "CmdOrCtrl+0"),
        role("Zoom in", "zoomIn", "CmdOrCtrl+="),
        role("Zoom out", "zoomOut", "CmdOrCtrl+-"),
        separator,
        role("Full screen", "togglefullscreen", mac ? "Ctrl+Cmd+F" : "F11"),
      ],
    },
  ];
}

type KeyLike = Pick<KeyboardEvent, "key" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey">;

export function matchesAccelerator(accelerator: string, e: KeyLike, platform: string): boolean {
  const parts = accelerator.split("+");
  const key = parts.pop()!.toLowerCase();
  const mods = new Set(parts.map((p) => p.toLowerCase()));
  const mac = platform === "darwin";
  const wantCtrl = mods.has("ctrl") || mods.has("control") || (!mac && mods.has("cmdorctrl"));
  const wantMeta = mods.has("cmd") || mods.has("command") || (mac && mods.has("cmdorctrl"));
  if (e.ctrlKey !== wantCtrl || e.metaKey !== wantMeta) return false;
  if (e.altKey !== mods.has("alt")) return false;
  const pressed = e.key.toLowerCase();
  const shiftOk = e.shiftKey === mods.has("shift") || key === "=" || key === "-";
  if (!shiftOk) return false;
  if (key === "=") return pressed === "=" || pressed === "+";
  return pressed === key;
}

export function acceleratorLabel(accelerator: string, platform: string): string {
  const mac = platform === "darwin";
  const names: Record<string, string> = mac
    ? { cmdorctrl: "⌘", cmd: "⌘", ctrl: "⌃", alt: "⌥", shift: "⇧", enter: "↩" }
    : { cmdorctrl: "Ctrl", cmd: "Ctrl", ctrl: "Ctrl", alt: "Alt", shift: "Shift", enter: "Enter" };
  const parts = accelerator.split("+").map((p) => names[p.toLowerCase()] ?? p.toUpperCase());
  return mac ? parts.join("") : parts.join("+");
}

export const TEXT_EDITING: MenuCommand[] = ["edit:undo", "edit:redo"];
