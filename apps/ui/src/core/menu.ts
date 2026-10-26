import type { KeyLike } from "@opsis/video/common/chord";
import type { MenuCommand } from "@/generated/bindings";
import type { Platform } from "@/lib/platform";
import { appMenu } from "@/store/catalog";

const isMacPlatform = (platform: Platform) => platform === "darwin";

export function menuCommandEntry(
  cmd: MenuCommand,
): { label: string; accelerator?: string } | undefined {
  for (const section of appMenu())
    for (const item of section.items)
      if (item.kind === "command" && item.command === cmd) return item;
  return undefined;
}

const ZOOM_KEYS_SHIFT_MAY_TYPE: ReadonlySet<string> = new Set(["=", "-"]);

const ZOOM_KEYS_SHIFTED: Partial<Record<string, string>> = { "=": "+" };

const primaryKey = (platform: Platform) => (isMacPlatform(platform) ? "metaKey" : "ctrlKey");

export const isPrimaryModifier = (
  e: { ctrlKey: boolean; metaKey: boolean },
  platform: Platform,
): boolean => e[primaryKey(platform)];

export function matchesAccelerator(accelerator: string, e: KeyLike, platform: Platform): boolean {
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

export function acceleratorLabel(accelerator: string, platform: Platform): string {
  const mac = isMacPlatform(platform);
  const names: Partial<Record<string, string>> = mac
    ? { cmdorctrl: "⌘", cmd: "⌘", ctrl: "⌃", alt: "⌥", shift: "⇧", enter: "↩" }
    : { cmdorctrl: "Ctrl", cmd: "Ctrl", ctrl: "Ctrl", alt: "Alt", shift: "Shift", enter: "Enter" };
  const parts = accelerator.split("+").map((p) => names[p.toLowerCase()] ?? p.toUpperCase());
  return mac ? parts.join("") : parts.join("+");
}
