import type { KeyLike } from "#core/chord";

export type MenuCommand =
  | "project:new"
  | "project:open"
  | "project:save"
  | "edit:undo"
  | "edit:redo"
  | "edit:group"
  | "edit:select-all"
  | "edit:layout"
  | "run:start"
  | "run:cancel"
  | "view:settings"
  | "view:projects"
  | "view:toggle-left"
  | "view:toggle-bottom"
  | "view:toggle-right"
  | "view:toggle-minimap"
  | "view:focus-preview"
  | "view:focus-timeline"
  | "view:focus-mixer"
  | "view:focus-run"
  | "edit:add-node";

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
  | "minimize"
  | "toggleMaximize"
  | "close"
  | "quit";

export type MenuEntry =
  | { kind: "command"; label: string; command: MenuCommand; accelerator?: string }
  | { kind: "role"; label: string; role: WindowRole; accelerator?: string }
  | { kind: "separator" };

type MenuSection = { label: string; items: MenuEntry[] };

const shortcut = (accelerator?: string) => (accelerator ? { accelerator } : {});
const command = (label: string, cmd: MenuCommand, accelerator?: string): MenuEntry => ({
  kind: "command",
  label,
  command: cmd,
  ...shortcut(accelerator),
});
const role = (label: string, r: WindowRole, accelerator?: string): MenuEntry => ({
  kind: "role",
  label,
  role: r,
  ...shortcut(accelerator),
});
const separator: MenuEntry = { kind: "separator" };

const isMacPlatform = (platform: NodeJS.Platform) => platform === "darwin";

export function appMenu(platform: NodeJS.Platform): MenuSection[] {
  const mac = isMacPlatform(platform);
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
        command("Select all", "edit:select-all", "CmdOrCtrl+A"),
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
        command("Preview, timeline, mixer and run", "view:toggle-bottom", "CmdOrCtrl+J"),
        command("Inspector", "view:toggle-right", "CmdOrCtrl+Alt+B"),
        command("Minimap", "view:toggle-minimap", "CmdOrCtrl+Shift+M"),
        separator,
        command("Preview", "view:focus-preview", "CmdOrCtrl+1"),
        command("Timeline", "view:focus-timeline", "CmdOrCtrl+2"),
        command("Mixer", "view:focus-mixer", "CmdOrCtrl+3"),
        command("Run", "view:focus-run", "CmdOrCtrl+4"),
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

export function menuCommandEntry(
  platform: NodeJS.Platform,
  cmd: MenuCommand,
): { label: string; accelerator?: string } | undefined {
  for (const section of appMenu(platform))
    for (const item of section.items)
      if (item.kind === "command" && item.command === cmd) return item;
  return undefined;
}

const ZOOM_KEYS_SHIFT_MAY_TYPE: ReadonlySet<string> = new Set(["=", "-"]);

const ZOOM_KEYS_SHIFTED: Partial<Record<string, string>> = { "=": "+" };

const primaryKey = (platform: NodeJS.Platform) => (isMacPlatform(platform) ? "metaKey" : "ctrlKey");

export const isPrimaryModifier = (
  e: { ctrlKey: boolean; metaKey: boolean },
  platform: NodeJS.Platform,
): boolean => e[primaryKey(platform)];

export function matchesAccelerator(
  accelerator: string,
  e: KeyLike,
  platform: NodeJS.Platform,
): boolean {
  const key = accelerator.slice(accelerator.lastIndexOf("+") + 1).toLowerCase();
  const mods = new Set(
    accelerator
      .split("+")
      .slice(0, -1)
      .map((p) => p.toLowerCase()),
  );
  const primary = mods.has("cmdorctrl") ? primaryKey(platform) : null;
  const wantCtrl = mods.has("ctrl") || mods.has("control") || primary === "ctrlKey";
  const wantMeta = mods.has("cmd") || mods.has("command") || primary === "metaKey";
  if (e.ctrlKey !== wantCtrl || e.metaKey !== wantMeta) return false;
  if (e.altKey !== mods.has("alt")) return false;
  const pressed = e.key.toLowerCase();
  if (!ZOOM_KEYS_SHIFT_MAY_TYPE.has(key) && e.shiftKey !== mods.has("shift")) return false;
  return pressed === key || pressed === ZOOM_KEYS_SHIFTED[key];
}

export function acceleratorLabel(accelerator: string, platform: NodeJS.Platform): string {
  const mac = isMacPlatform(platform);
  const names: Partial<Record<string, string>> = mac
    ? { cmdorctrl: "⌘", cmd: "⌘", ctrl: "⌃", alt: "⌥", shift: "⇧", enter: "↩" }
    : { cmdorctrl: "Ctrl", cmd: "Ctrl", ctrl: "Ctrl", alt: "Alt", shift: "Shift", enter: "Enter" };
  const parts = accelerator.split("+").map((p) => names[p.toLowerCase()] ?? p.toUpperCase());
  return mac ? parts.join("") : parts.join("+");
}
