import { baseName } from "../asset-ref";
import { formatDuration, percentText } from "../format";
import { nonBlankString } from "../guards";
import type { PickedRole } from "../interactive";
import type { NodeResult } from "../schema/values";
import type { WorkflowSettings } from "../schema/workflow";
import { framesToMs } from "../timeline/time";
import type { NodeIconName } from "./icon-name";

export type CardRow = {
  icon: NodeIconName;
  text: string;
  title?: string;
  tone?: "muted" | "warn";
};

export type CardContext<P> = {
  params: P;
  settings: WorkflowSettings;
  result?: NodeResult;
  secrets?: string[] | null;
  connected: string[];
  clipFrames?: number;
  inputFrames?: Partial<Record<string, number>>;
  inputItems?: Partial<Record<string, number>>;
};

export const MAX_CARD_ROWS = 4;

const TRANSITION_WARN_RATIO = 0.4;

const CARD_TEXT_MAX = 42;
const QUOTED_TEXT_MAX = CARD_TEXT_MAX - 2;
export const CODE_TEXT_MAX = 36;
export const LABELLED_TEXT_MAX = 24;
export const TAG_TEXT_MAX = 20;
const NAME_WITH_ROLE_MAX = 34;

type ClipLengths = ReadonlyArray<number | undefined>;

function shortestClip(clipFrames: ClipLengths): number | null {
  const known = clipFrames.filter((f): f is number => f !== undefined && f > 0);
  return known.length ? Math.min(...known) : null;
}

export type Cover = { ratio: number; shortest: number };

export function cover(transitionFrames: number, clipFrames: ClipLengths): Cover | null {
  const shortest = shortestClip(clipFrames);
  return shortest === null ? null : { ratio: transitionFrames / shortest, shortest };
}

export const coversTooMuch = (ratio: number): boolean => ratio > TRANSITION_WARN_RATIO;

export const squeezedTooMuch = (length: number, designed: number): boolean =>
  length < TRANSITION_WARN_RATIO * designed;

export function coverRow(covered: Cover | null, fps: number): CardRow | null {
  if (!covered || !coversTooMuch(covered.ratio)) return null;
  const share = percentText(covered.ratio, { max: 100 });
  return row("triangle-alert", `Covers ${share} of a ${frameSecs(covered.shortest, fps)} clip`, {
    tone: "warn",
    title: "Turn on Keep every clip whole, or make the transition shorter.",
  });
}

export const row = (icon: NodeIconName, text: string, extra: Partial<CardRow> = {}): CardRow => ({
  icon,
  text,
  ...extra,
});

export const missing = (icon: NodeIconName, text: string): CardRow => ({
  icon,
  text,
  tone: "warn",
});

export const mutedIf = (when: boolean): Partial<CardRow> => (when ? { tone: "muted" } : {});

export const warnIf = (when: boolean): Partial<CardRow> => (when ? { tone: "warn" } : {});

export const toggleRow = (icon: NodeIconName, on: boolean): CardRow =>
  row(icon, on ? "On" : "Off", mutedIf(!on));

export const fileRow = (icon: NodeIconName, path: string, name = baseName(path)): CardRow =>
  row(icon, name, { title: path });

export const frameSecs = (frames: number, fps: number) => formatDuration(framesToMs(frames, fps));

export function truncate(text: string, max = CARD_TEXT_MAX): string {
  const one = text.replace(/\s+/g, " ").trim();
  return one.length > max ? `${one.slice(0, max - 1)}…` : one;
}

export const quote = (text: string, max = QUOTED_TEXT_MAX) => `“${truncate(text, max)}”`;

export const quotedRow = (
  icon: NodeIconName,
  text: string,
  empty: string,
  { suffix = "", ...extra }: Partial<CardRow> & { suffix?: string } = {},
): CardRow =>
  text.trim()
    ? row(icon, `${quote(text, QUOTED_TEXT_MAX - suffix.length)}${suffix}`, extra)
    : missing(icon, empty);

export const connectedPorts = <P extends string>(
  ports: readonly P[],
  connected: readonly string[],
): P[] => ports.filter((p) => connected.includes(p));

const ROLE_WORDS: Partial<Record<string, string>> = {
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
} satisfies Record<PickedRole, string>;

function elementName(target: string | undefined, role: string | undefined): string | null {
  const t = nonBlankString(target);
  if (!t) return null;
  if (t.startsWith("css=")) return "Custom element";
  const point = /^(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)$/.exec(t);
  if (point) return `Point ${Math.round(Number(point[1]))}, ${Math.round(Number(point[2]))}`;
  const word = role ? ROLE_WORDS[role] : undefined;
  return word ? `${quote(t, NAME_WITH_ROLE_MAX)} ${word}` : quote(t);
}

export function elementRow(
  icon: NodeIconName,
  target: string | undefined,
  role: string | undefined,
  empty = "Pick an element",
): CardRow {
  const name = elementName(target, role);
  return name ? row(icon, name, { title: target }) : missing(icon, empty);
}

const keepsCase = (word: string): boolean => /[A-Z].*[A-Z]|\d/.test(word);

export function sentence(title: string): string {
  return title
    .split(" ")
    .map((w, i) => (i === 0 || keepsCase(w) ? w : w.toLowerCase()))
    .join(" ");
}
