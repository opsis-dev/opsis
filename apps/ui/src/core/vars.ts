import { isRecord } from "@opsis/video/common/guards";

const VAR_SOURCE = String.raw`\{\{\s*([^{}]+?)\s*\}\}`;

const varPattern = () => new RegExp(VAR_SOURCE, "g");

export const PAUSE_MARK = "¦";

export type TextToken =
  | { kind: "text"; text: string }
  | { kind: "var"; name: string }
  | { kind: "pause" };

const plainText = (text: string): TextToken[] => (text ? [{ kind: "text", text }] : []);

function textWithPauses(text: string): TextToken[] {
  const out: TextToken[] = [];
  for (const [i, part] of text.split(PAUSE_MARK).entries()) {
    if (i > 0) out.push({ kind: "pause" });
    out.push(...plainText(part));
  }
  return out;
}

export function tokenize(value: string, pauses = false): TextToken[] {
  const text = pauses ? textWithPauses : plainText;
  const out: TextToken[] = [];
  let last = 0;
  for (const m of value.matchAll(varPattern())) {
    out.push(...text(value.slice(last, m.index)), { kind: "var", name: m[1] });
    last = m.index + m[0].length;
  }
  out.push(...text(value.slice(last)));
  return out;
}

export function stringsIn(value: unknown, test: (text: string) => boolean): string[] {
  if (typeof value === "string") return test(value) ? [value] : [];
  if (Array.isArray(value)) return value.flatMap((v) => stringsIn(v, test));
  if (isRecord(value)) return Object.values(value).flatMap((v) => stringsIn(v, test));
  return [];
}

function mapJsonStrings<T>(value: T, fn: (text: string) => string): T {
  const walk = (v: unknown): unknown => {
    if (typeof v === "string") return fn(v);
    if (Array.isArray(v)) return v.map(walk);
    if (isRecord(v)) return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, walk(x)]));
    return v;
  };
  return walk(value) as T;
}

export const varToken = (name: string): string => `{{${name}}}`;

export const renameVariable = <T>(value: T, from: string, to: string): T =>
  mapJsonStrings(value, (text) =>
    text.replace(varPattern(), (whole, name: string) => (name === from ? varToken(to) : whole)),
  );
