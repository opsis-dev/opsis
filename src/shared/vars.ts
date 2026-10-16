export const VAR = /\{\{\s*([^{}]+?)\s*\}\}/g;

export const PAUSE_MARK = "¦";

export type TextToken =
  | { kind: "text"; text: string }
  | { kind: "var"; name: string }
  | { kind: "pause" };

export function tokenize(value: string, pauses = false): TextToken[] {
  const out: TextToken[] = [];
  const push = (text: string) => {
    if (!text) return;
    if (!pauses) {
      out.push({ kind: "text", text });
      return;
    }
    const parts = text.split(PAUSE_MARK);
    parts.forEach((p, i) => {
      if (p) out.push({ kind: "text", text: p });
      if (i < parts.length - 1) out.push({ kind: "pause" });
    });
  };
  let last = 0;
  for (const m of value.matchAll(VAR)) {
    push(value.slice(last, m.index));
    out.push({ kind: "var", name: m[1] });
    last = m.index + m[0].length;
  }
  push(value.slice(last));
  return out;
}

export const untokenize = (tokens: TextToken[]) =>
  tokens
    .map((t) => (t.kind === "text" ? t.text : t.kind === "var" ? `{{${t.name}}}` : PAUSE_MARK))
    .join("");

export function interpolate<T>(value: T, vars: Record<string, string>): T {
  if (typeof value === "string") {
    return value.replace(VAR, (whole, name: string) => (name in vars ? vars[name] : whole)) as T;
  }
  if (Array.isArray(value)) return value.map((v) => interpolate(v, vars)) as T;
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, interpolate(v, vars)]),
    ) as T;
  }
  return value;
}

export function referencedVariables(value: unknown, out = new Set<string>()): Set<string> {
  if (typeof value === "string") for (const m of value.matchAll(VAR)) out.add(m[1]);
  else if (Array.isArray(value)) for (const v of value) referencedVariables(v, out);
  else if (value && typeof value === "object")
    for (const v of Object.values(value)) referencedVariables(v, out);
  return out;
}

export function renameVariable<T>(value: T, from: string, to: string): T {
  if (typeof value === "string")
    return value.replace(VAR, (whole, name: string) => (name === from ? `{{${to}}}` : whole)) as T;
  if (Array.isArray(value)) return value.map((v) => renameVariable(v, from, to)) as T;
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, renameVariable(v, from, to)]),
    ) as T;
  return value;
}
