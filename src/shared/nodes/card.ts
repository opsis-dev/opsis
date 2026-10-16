import type { NodeResult } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { formatDuration } from "../format";

export type CardRow = { icon: string; text: string; title?: string; tone?: "muted" | "warn" };

export type CardContext<P> = {
  params: P;
  raw: Record<string, unknown>;
  settings: WorkflowSettings;
  result?: NodeResult;
  secrets?: string[];
  connected: string[];
  clipFrames?: number;
  inputFrames?: Record<string, number>;
};

export function coverRow(
  transitionFrames: number,
  clipFrames: number[],
  fps: number,
): CardRow | null {
  const shortest = Math.min(...clipFrames.filter((f) => f > 0));
  if (!Number.isFinite(shortest) || transitionFrames <= 0.4 * shortest) return null;
  const pct = Math.min(100, Math.round((transitionFrames / shortest) * 100));
  return row("triangle-alert", `Covers ${pct}% of a ${frameSecs(shortest, fps)} clip`, {
    tone: "warn",
    title: "Turn on Keep every clip whole, or make the transition shorter.",
  });
}

export const row = (icon: string, text: string, extra: Partial<CardRow> = {}): CardRow => ({
  icon,
  text,
  ...extra,
});

export const missing = (icon: string, text: string): CardRow => ({ icon, text, tone: "warn" });

export const frameSecs = (frames: number, fps: number) =>
  formatDuration((frames / Math.max(1, fps)) * 1000);

export function clip(text: string, max = 42): string {
  const one = text.replace(/\s+/g, " ").trim();
  return one.length > max ? `${one.slice(0, max - 1)}…` : one;
}

export const quote = (text: string, max = 40) => `“${clip(text, max)}”`;

const ROLE_WORDS: Record<string, string> = {
  button: "button",
  link: "link",
  textbox: "field",
  searchbox: "search field",
  checkbox: "checkbox",
  radio: "option",
  combobox: "dropdown",
  listbox: "list",
  option: "option",
  tab: "tab",
  menuitem: "menu item",
  heading: "heading",
  img: "image",
  switch: "switch",
  slider: "slider",
};

export function elementName(target: unknown, role?: unknown): string | null {
  const t = typeof target === "string" ? target.trim() : "";
  if (!t) return null;
  if (t.startsWith("css=")) return "Custom element";
  const point = /^(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)$/.exec(t);
  if (point) return `Point ${Math.round(Number(point[1]))}, ${Math.round(Number(point[2]))}`;
  const word = typeof role === "string" ? ROLE_WORDS[role] : undefined;
  return word ? `${quote(t, 34)} ${word}` : quote(t, 40);
}

export function elementRow(
  icon: string,
  target: unknown,
  raw: Record<string, unknown>,
  key = "target",
  empty = "Pick an element",
): CardRow {
  const name = elementName(target, raw[`${key}Role`]);
  return name ? row(icon, name, { title: String(target) }) : missing(icon, empty);
}

export function sentence(title: string): string {
  const words = title.split(" ");
  return words
    .map((w, i) => (i === 0 || /[A-Z].*[A-Z]|\d/.test(w) ? w : w.toLowerCase()))
    .join(" ");
}
