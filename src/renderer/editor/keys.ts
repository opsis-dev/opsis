const MODIFIERS = ["Control", "Meta", "Alt", "Shift"];

function keyName(key: string): string {
  if (key === " ") return "Space";
  return key.length === 1 ? key.toUpperCase() : key;
}

export function chordFromEvent(
  e: Pick<KeyboardEvent, "key" | "ctrlKey" | "metaKey" | "altKey" | "shiftKey">,
): string | null {
  if (MODIFIERS.includes(e.key)) return null;
  return [
    e.ctrlKey && "Control",
    e.metaKey && "Meta",
    e.altKey && "Alt",
    e.shiftKey && "Shift",
    keyName(e.key),
  ]
    .filter(Boolean)
    .join("+");
}
